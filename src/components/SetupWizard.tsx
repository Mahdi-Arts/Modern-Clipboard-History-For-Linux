import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect, useCallback } from 'react'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { changeLanguage, type LangCode } from '../i18n/config'
import { useLanguageEffect } from '../i18n/useLanguage'
import { useAutostart } from '../hooks/useAutostart'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'
import { useSystemThemePreference } from '../utils/systemTheme'
import {
  CheckCircle,
  AlertTriangle,
  Shield,
  Rocket,
  Keyboard,
  Settings,
  Copy,
  AlertCircle,
  Zap,
} from 'lucide-react'

interface PermissionStatus {
  uinput_accessible: boolean
  uinput_path: string
  user_in_input_group: boolean
  status_code: 'permissions_ok' | 'relogin_required' | 'permissions_missing'
}

interface ShortcutToolsStatus {
  desktop_environment: string
  gsettings_available: boolean
  kde_tools_available: boolean
  xfce_tools_available: boolean
  can_register_automatically: boolean
  has_conflicts: boolean
  conflict_count: number
  can_auto_resolve_conflicts: boolean
}

interface ShortcutConflict {
  binding: string
  current_action: string
  owner: string
  resolution_command: string | null
  resolution_steps: string
}

interface ConflictDetectionResult {
  desktop_environment: string
  conflicts: ShortcutConflict[]
  can_auto_resolve: boolean
  message: string
}

interface SetupWizardProps {
  readonly onComplete: () => void
}

interface WizardButtonProps {
  id: string
  onClick: () => void
  children: React.ReactNode
  disabled?: boolean
  primary?: boolean
  hoveredButton: string | null
  setHoveredButton: (id: string | null) => void
  isDark: boolean
  tertiaryOpacity: number
}

const WizardButton = ({
  id,
  onClick,
  children,
  disabled = false,
  primary = false,
  hoveredButton,
  setHoveredButton,
  isDark,
  tertiaryOpacity,
}: WizardButtonProps) => {
  const isHovered = hoveredButton === id

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      onMouseEnter={() => setHoveredButton(id)}
      onMouseLeave={() => setHoveredButton(null)}
      className={clsx(
        'px-5 py-2.5 rounded-win11 font-medium transition-all duration-150',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'active:scale-[0.98]',
        primary
          ? 'text-win11-bg-accent'
          : isDark
            ? 'text-win11-text-secondary'
            : 'text-win11Light-text-secondary'
      )}
      style={
        isHovered && !disabled ? getTertiaryBackgroundStyle(isDark, tertiaryOpacity) : undefined
      }
    >
      {children}
    </button>
  )
}

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const { t, i18n } = useTranslation()
  useLanguageEffect(i18n)
  const [step, setStep] = useState(0)
  const [permissions, setPermissions] = useState<PermissionStatus | null>(null)
  const [shortcutTools, setShortcutTools] = useState<ShortcutToolsStatus | null>(null)
  const [conflicts, setConflicts] = useState<ConflictDetectionResult | null>(null)
  const [fixing, setFixing] = useState(false)
  const [fixError, setFixError] = useState<string | null>(null)
  const [registeringShortcut, setRegisteringShortcut] = useState(false)
  const [shortcutRegistered, setShortcutRegistered] = useState(false)
  const [showManualInstructions, setShowManualInstructions] = useState(false)
  const [resolvingConflicts, setResolvingConflicts] = useState(false)
  const [conflictsResolved, setConflictsResolved] = useState(false)
  const [conflictError, setConflictError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)
  const [hoveredButton, setHoveredButton] = useState<string | null>(null)
  const { enableAutostart } = useAutostart()
  const isDark = useSystemThemePreference()

  // Fixed opacity for the wizard (similar to main app default)
  const tertiaryOpacity = 0.85

  const buttonProps = { hoveredButton, setHoveredButton, isDark, tertiaryOpacity }

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDark])

  const checkPermissions = useCallback(async () => {
    try {
      const status = await invoke<PermissionStatus>('check_permissions')
      setPermissions(status)
    } catch (e) {
      console.error('Failed to check permissions:', e)
    }
  }, [])

  const checkShortcutTools = useCallback(async () => {
    try {
      const status = await invoke<ShortcutToolsStatus>('check_shortcut_tools')
      setShortcutTools(status)
    } catch (e) {
      console.error('Failed to check shortcut tools:', e)
    }
  }, [])

  const checkConflicts = useCallback(async () => {
    try {
      const result = await invoke<ConflictDetectionResult>('detect_conflicts')
      setConflicts(result)
    } catch (e) {
      console.error('Failed to check conflicts:', e)
    }
  }, [])

  useEffect(() => {
    const initialChecksTimer = globalThis.setTimeout(() => {
      void checkPermissions()
      void checkShortcutTools()
      void checkConflicts()
    }, 0)

    return () => globalThis.clearTimeout(initialChecksTimer)
  }, [checkPermissions, checkShortcutTools, checkConflicts])

  const localizedPermissionError = (error: unknown) => {
    const raw = String(error)
    const code = ['pkexec_missing', 'setfacl_missing', 'permission_fix_failed'].find((candidate) =>
      raw.includes(candidate)
    )
    return code ? t(`setup.${code}`) : t('setup.permission_fix_failed')
  }

  const handleResolveConflicts = async () => {
    setResolvingConflicts(true)
    setConflictError(null)
    try {
      await invoke<string[]>('resolve_conflicts')
      setConflictsResolved(true)
      // Refresh conflict status
      await checkConflicts()
      await checkShortcutTools()
    } catch (e) {
      console.error('Failed to resolve conflicts:', e)
      setConflictError(t('setup.conflicts_failed_detail'))
    } finally {
      setResolvingConflicts(false)
    }
  }

  const handleFixPermissions = async () => {
    setFixing(true)
    setFixError(null)
    try {
      await invoke<string>('fix_permissions_now')
      await checkPermissions()
    } catch (e) {
      console.error('Failed to fix permissions:', e)
      setFixError(localizedPermissionError(e))
    } finally {
      setFixing(false)
    }
  }

  const handleRegisterShortcut = async () => {
    setRegisteringShortcut(true)
    try {
      await invoke<string>('register_de_shortcut')
      setShortcutRegistered(true)
      setTimeout(() => setStep(3), 1500)
    } catch (e) {
      console.error('Failed to register shortcut:', e)
      setShowManualInstructions(true)
    } finally {
      setRegisteringShortcut(false)
    }
  }

  const handleEnableAutostart = async () => {
    await enableAutostart()
    setStep(4)
  }

  const handleComplete = () => {
    // The setup-only `finish_setup` command atomically persists completion.
    // فرمان setup-only با نام `finish_setup` تکمیل را اتمیک ذخیره می‌کند.
    onComplete()
  }

  const handleLanguageChange = async (language: LangCode) => {
    await changeLanguage(language)
    await invoke('set_app_language', { lang: language })
  }

  const copyToClipboard = (text: string) => {
    void navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  // Status message styles
  const statusCardClass = (type: 'success' | 'warning' | 'error') =>
    clsx(
      'p-4 rounded-win11 flex items-start gap-3 text-sm',
      type === 'success' &&
        (isDark
          ? 'bg-win11-success/15 text-win11-success border border-win11-success/20'
          : 'bg-green-50 text-green-700 border border-green-200'),
      type === 'warning' &&
        (isDark
          ? 'bg-win11-warning/15 text-win11-warning border border-win11-warning/20'
          : 'bg-amber-50 text-amber-700 border border-amber-200'),
      type === 'error' &&
        (isDark
          ? 'bg-win11-error/15 text-win11-error border border-win11-error/20'
          : 'bg-red-50 text-red-700 border border-red-200')
    )

  const infoCardClass = clsx(
    'p-3 rounded-win11',
    isDark
      ? 'bg-win11-bg-tertiary/50 border border-win11-border-subtle'
      : 'bg-win11Light-bg-tertiary/50 border border-win11Light-border'
  )

  const steps = [
    // Step 0: Welcome
    <div key="welcome" className="text-center">
      <div className="mb-6">
        <div
          className={clsx(
            'w-16 h-16 mx-auto rounded-full flex items-center justify-center',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Rocket
            className={clsx(
              'w-8 h-8',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
      </div>
      <h2
        className={clsx(
          'text-xl font-semibold mb-2',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        {t('setup.welcome')}
      </h2>
      <p
        className={clsx(
          'text-sm mb-8',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        {t('setup.welcome_desc')}
        <br />
        {t('setup.welcome_next')}
      </p>
      <WizardButton {...buttonProps} id="start" onClick={() => setStep(1)} primary>
        {t('setup.get_started')}
      </WizardButton>
    </div>,

    // Step 1: Permissions
    <div key="permissions">
      <div className="text-center mb-6">
        <div
          className={clsx(
            'w-14 h-14 mx-auto rounded-full flex items-center justify-center mb-4',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Shield
            className={clsx(
              'w-7 h-7',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
        <h2
          className={clsx(
            'text-lg font-semibold mb-1',
            isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
          )}
        >
          {t('setup.step_permissions')}
        </h2>
        <p
          className={clsx(
            'text-sm',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        >
          {t('setup.permission_required')}
        </p>
      </div>

      {permissions && (
        <div
          className={clsx(
            'mb-4',
            statusCardClass(permissions.uinput_accessible ? 'success' : 'warning')
          )}
        >
          {permissions.uinput_accessible ? (
            <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          ) : (
            <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          )}
          <span>{t(`setup.${permissions.status_code}`)}</span>
        </div>
      )}

      {fixError && <div className={clsx('mb-4', statusCardClass('error'))}>{fixError}</div>}

      <div className="flex gap-3 justify-center">
        {!permissions?.uinput_accessible && (
          <WizardButton
            {...buttonProps}
            id="fix"
            onClick={() => void handleFixPermissions()}
            disabled={fixing}
          >
            {fixing ? t('setup.fixing') : t('setup.fix_now')}
          </WizardButton>
        )}
        <WizardButton {...buttonProps} id="perm-continue" onClick={() => setStep(2)} primary>
          {permissions?.uinput_accessible ? t('common.continue') : t('common.skip')}
        </WizardButton>
      </div>
    </div>,

    // Step 2: Shortcut Configuration
    <div key="shortcut">
      <div className="text-center mb-6">
        <div
          className={clsx(
            'w-14 h-14 mx-auto rounded-full flex items-center justify-center mb-4',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Keyboard
            className={clsx(
              'w-7 h-7',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
        <h2
          className={clsx(
            'text-lg font-semibold mb-1',
            isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
          )}
        >
          {t('setup.step_shortcut')}
        </h2>
        <p
          className={clsx(
            'text-sm',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        >
          {t('setup.shortcut_intro_before')}{' '}
          <kbd
            className={clsx(
              'px-2 py-0.5 rounded text-xs font-mono',
              isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
            )}
          >
            Super + V
          </kbd>{' '}
          {t('setup.shortcut_intro_after')}
        </p>
      </div>

      {shortcutTools && (
        <div className={clsx('mb-4', infoCardClass)}>
          <div
            className={clsx(
              'flex items-center gap-2 text-sm',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          >
            <Settings className="w-4 h-4" />
            <span>
              {t('setup.detected')}{' '}
              <strong
                className={isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'}
              >
                {shortcutTools.desktop_environment}
              </strong>
            </span>
          </div>
        </div>
      )}

      {/* Conflict Warning */}
      {conflicts && conflicts.conflicts.length > 0 && !conflictsResolved && (
        <div className={clsx('mb-4', statusCardClass('warning'))}>
          <AlertCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="font-medium mb-1">
              {t('setup.conflicts_detected', { count: conflicts.conflicts.length })}
            </p>
            <p className="text-xs opacity-90 mb-2">
              {t('setup.conflict_detail', {
                owner: conflicts.conflicts[0].owner,
                action: conflicts.conflicts[0].current_action,
              })}
            </p>
            {conflicts.can_auto_resolve && (
              <div className="space-y-1">
                <WizardButton
                  {...buttonProps}
                  id="resolve-conflicts"
                  onClick={() => void handleResolveConflicts()}
                  disabled={resolvingConflicts}
                >
                  <span className="flex items-center gap-2">
                    <Zap className="w-4 h-4" />
                    {resolvingConflicts ? t('setup.resolving') : t('setup.auto_fix')}
                  </span>
                </WizardButton>
                <p className="text-xs opacity-60">{t('setup.auto_fix_note')}</p>
              </div>
            )}
            {!conflicts.can_auto_resolve && (
              <p className="text-xs opacity-75 mt-1">{t('setup.manual_resolution')}</p>
            )}
          </div>
        </div>
      )}

      {conflictsResolved && (
        <div className={clsx('mb-4', statusCardClass('success'))}>
          <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <span>{t('setup.conflicts_resolved')}</span>
        </div>
      )}

      {conflictError && (
        <div className={clsx('mb-4', statusCardClass('error'))}>
          <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <div>
            <p className="font-medium">{t('setup.conflicts_failed')}</p>
            <p className="text-xs opacity-90">{conflictError}</p>
          </div>
        </div>
      )}

      {shortcutRegistered && (
        <div className={clsx('mb-4', statusCardClass('success'))}>
          <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <span>{t('setup.shortcut_success')}</span>
        </div>
      )}

      {showManualInstructions && shortcutTools && (
        <div className="mb-4 space-y-3">
          <div className={statusCardClass('warning')}>
            <div>
              <p className="font-medium mb-2">{t('setup.manual_required')}</p>
              <p className="whitespace-pre-line opacity-90 text-xs">
                {t('setup.manual_instructions', {
                  desktop: shortcutTools.desktop_environment,
                  command: 'modern-clipboard-history-for-linux',
                })}
              </p>
            </div>
          </div>
          <WizardButton
            {...buttonProps}
            id="copy-path"
            onClick={() => copyToClipboard('/usr/bin/modern-clipboard-history-for-linux')}
          >
            <span className="flex items-center justify-center gap-2">
              <Copy className="w-4 h-4" />
              {copied ? t('clipboard.copied') : t('setup.copy_command')}
            </span>
          </WizardButton>
        </div>
      )}

      <div className="flex flex-col gap-2 items-center">
        {shortcutTools?.can_register_automatically &&
          !shortcutRegistered &&
          !showManualInstructions && (
            <WizardButton
              {...buttonProps}
              id="register"
              onClick={() => void handleRegisterShortcut()}
              disabled={registeringShortcut}
              primary
            >
              {registeringShortcut ? t('setup.registering') : t('setup.register_auto')}
            </WizardButton>
          )}

        {!shortcutTools?.can_register_automatically && !showManualInstructions && (
          <WizardButton
            {...buttonProps}
            id="show-manual"
            onClick={() => setShowManualInstructions(true)}
          >
            {t('setup.show_manual')}
          </WizardButton>
        )}

        <WizardButton
          {...buttonProps}
          id="shortcut-continue"
          onClick={() => setStep(3)}
          primary={shortcutRegistered || showManualInstructions}
        >
          {shortcutRegistered || showManualInstructions ? t('common.continue') : t('common.skip')}
        </WizardButton>
      </div>
    </div>,

    // Step 3: Autostart
    <div key="autostart">
      <div className="text-center mb-6">
        <div
          className={clsx(
            'w-14 h-14 mx-auto rounded-full flex items-center justify-center mb-4',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Rocket
            className={clsx(
              'w-7 h-7',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
        <h2
          className={clsx(
            'text-lg font-semibold mb-1',
            isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
          )}
        >
          {t('setup.autostart_title')}
        </h2>
        <p
          className={clsx(
            'text-sm',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        >
          {t('setup.autostart_desc')}
        </p>
      </div>

      <div className="flex gap-3 justify-center">
        <WizardButton
          {...buttonProps}
          id="enable-autostart"
          onClick={() => void handleEnableAutostart()}
          primary
        >
          {t('setup.yes_enable')}
        </WizardButton>
        <WizardButton {...buttonProps} id="skip-autostart" onClick={() => setStep(4)}>
          {t('setup.no_thanks')}
        </WizardButton>
      </div>
    </div>,

    // Step 4: Done
    <div key="done" className="text-center">
      <div className="mb-6">
        <div
          className={clsx(
            'w-16 h-16 mx-auto rounded-full flex items-center justify-center',
            isDark ? 'bg-win11-success/20' : 'bg-green-100'
          )}
        >
          <CheckCircle className="w-8 h-8 text-win11-success" />
        </div>
      </div>
      <h2
        className={clsx(
          'text-xl font-semibold mb-2',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        {t('setup.done_title')}
      </h2>
      <p
        className={clsx(
          'text-sm mb-4',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        {t('setup.done_desc')}
      </p>
      <div className="flex items-center justify-center gap-2 mb-6">
        <Keyboard
          className={clsx(
            'w-4 h-4',
            isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
          )}
        />
        <kbd
          className={clsx(
            'px-3 py-1.5 rounded-win11 font-mono text-sm',
            isDark
              ? 'bg-win11-bg-tertiary text-win11-text-primary border border-win11-border-subtle'
              : 'bg-win11Light-bg-tertiary text-win11Light-text-primary border border-win11Light-border'
          )}
        >
          Super + V
        </kbd>
      </div>
      <WizardButton {...buttonProps} id="finish" onClick={() => void handleComplete()} primary>
        {t('setup.start_using')}
      </WizardButton>
    </div>,
  ]

  const getProgressDotClass = (i: number) => {
    if (i === step) return 'bg-win11-bg-accent w-5'
    if (i < step)
      return clsx(
        'cursor-pointer',
        isDark
          ? 'bg-win11-text-tertiary hover:bg-win11-text-secondary'
          : 'bg-win11Light-text-secondary hover:bg-win11Light-text-primary'
      )
    return isDark ? 'bg-win11-border' : 'bg-win11Light-border'
  }

  return (
    <div
      className={clsx(
        'relative h-full w-full flex flex-col items-center justify-center p-6',
        isDark
          ? 'bg-win11-bg-primary text-win11-text-primary'
          : 'bg-win11Light-bg-primary text-win11Light-text-primary'
      )}
    >
      <div
        className="absolute top-4 end-4 z-10 flex rounded-lg border p-1 shadow-sm backdrop-blur-md"
        role="group"
        aria-label={t('setup.language_selector')}
      >
        {(['en', 'fa'] as const).map((language) => {
          const selected = i18n.language === language
          return (
            <button
              key={language}
              type="button"
              aria-pressed={selected}
              onClick={() => void handleLanguageChange(language)}
              className={clsx(
                'min-w-16 rounded-md px-3 py-1.5 text-xs font-semibold transition-all',
                selected
                  ? 'bg-win11-bg-accent text-white shadow-sm'
                  : isDark
                    ? 'text-win11-text-secondary hover:bg-white/10'
                    : 'text-win11Light-text-secondary hover:bg-black/5'
              )}
            >
              {language === 'fa' ? 'فارسی' : 'English'}
            </button>
          )
        })}
      </div>

      <div className="w-full max-w-sm">
        {steps[step]}

        {/* Progress dots */}
        <div className="flex justify-center gap-2 mt-8">
          {steps.map((_, i) => (
            <button
              key={`dot-${i}`}
              onClick={() => i < step && setStep(i)}
              disabled={i >= step}
              aria-label={t('setup.step_label', { step: i + 1 })}
              className={clsx(
                'h-1.5 w-1.5 rounded-full transition-all duration-200',
                getProgressDotClass(i)
              )}
            />
          ))}
        </div>
      </div>
    </div>
  )
}

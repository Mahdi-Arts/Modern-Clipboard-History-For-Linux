import { useEffect, useState } from 'react'
import { clsx } from 'clsx'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import { Switch } from '../Switch'
import type { UserSettings } from '../../types/clipboard'

export function PrivacySection({
  settings,
  isDark,
  onToggle,
}: {
  settings: UserSettings
  isDark: boolean
  onToggle: (
    key: 'filter_secrets' | 'save_images' | 'exclude_sensitive_apps' | 'allow_wm_config_rewrite'
  ) => void
}) {
  const { t } = useTranslation()
  const [waylandLimited, setWaylandLimited] = useState(false)

  useEffect(() => {
    invoke<{ is_wayland: boolean; app_identity_available: boolean }>('get_session_info')
      .then((info) => setWaylandLimited(info.is_wayland && !info.app_identity_available))
      .catch(() => setWaylandLimited(false))
  }, [])

  const rows: {
    key: 'filter_secrets' | 'save_images' | 'exclude_sensitive_apps' | 'allow_wm_config_rewrite'
    label: string
    desc: string
    danger?: boolean
  }[] = [
    {
      key: 'filter_secrets',
      label: t('settings_page.privacy.filter_secrets'),
      desc: t('settings_page.privacy.filter_secrets_desc'),
    },
    {
      key: 'save_images',
      label: t('settings_page.privacy.save_images'),
      desc: t('settings_page.privacy.save_images_desc'),
    },
    {
      key: 'exclude_sensitive_apps',
      label: t('settings_page.privacy.exclude_apps'),
      desc: t('settings_page.privacy.exclude_apps_desc'),
    },
    {
      key: 'allow_wm_config_rewrite',
      label: t('settings_page.privacy.wm_rewrite'),
      desc: t('settings_page.privacy.wm_rewrite_desc'),
      danger: true,
    },
  ]

  return (
    <section
      className={clsx(
        'rounded-xl border shadow-sm overflow-hidden',
        isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60'
      )}
    >
      <div className="p-6 border-b border-inherit">
        <div className="flex items-center gap-3 mb-1">
          <div className={clsx('p-2 rounded-lg', isDark ? 'bg-white/5' : 'bg-gray-100')}>
            <svg
              width="22"
              height="22"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
            </svg>
          </div>
          <div>
            <h2 className="text-base font-semibold">{t('settings_page.privacy.label')}</h2>
            <p className={clsx('text-xs mt-0.5', isDark ? 'text-gray-400' : 'text-gray-500')}>
              {t('settings_page.privacy.desc')}
            </p>
          </div>
        </div>
      </div>
      <div className="p-6 space-y-6">
        {waylandLimited && (
          <div
            className={clsx(
              'text-xs leading-relaxed rounded-lg p-3 border',
              isDark
                ? 'bg-amber-500/10 text-amber-200 border-amber-500/20'
                : 'bg-amber-50 text-amber-800 border-amber-200'
            )}
          >
            {t('settings_page.privacy.wayland_note')}
          </div>
        )}
        {rows.map((row) => (
          <div key={row.key} className="flex items-start justify-between gap-4">
            <div className="min-w-0">
              <div
                className={clsx(
                  'text-sm font-medium',
                  row.danger && (isDark ? 'text-amber-300' : 'text-amber-700')
                )}
              >
                {row.label}
              </div>
              <div
                className={clsx(
                  'text-xs mt-0.5 leading-relaxed',
                  isDark ? 'text-gray-400' : 'text-gray-500'
                )}
              >
                {row.desc}
              </div>
            </div>
            <Switch
              checked={settings[row.key]}
              onChange={() => onToggle(row.key)}
              isDark={isDark}
            />
          </div>
        ))}
        <div
          className={clsx(
            'text-[11px] leading-relaxed rounded-lg p-3',
            isDark ? 'bg-white/5 text-gray-400' : 'bg-gray-50 text-gray-500'
          )}
        >
          {t('settings_page.privacy.storage_note')}
        </div>
      </div>
    </section>
  )
}

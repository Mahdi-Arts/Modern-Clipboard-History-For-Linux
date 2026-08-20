import { ClipboardList } from 'lucide-react'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'

interface EmptyStateProps {
  isDark: boolean
}

/**
 * Empty state shown before the first clipboard capture.
 */
export function EmptyState({ isDark }: EmptyStateProps) {
  const { t } = useTranslation()

  return (
    <div
      className="flex flex-col items-center justify-center h-full py-10 px-5 text-center select-none"
      data-tauri-drag-region
      role="status"
      aria-live="polite"
    >
      <div
        className={clsx(
          'relative w-20 h-20 rounded-2xl flex items-center justify-center mb-5',
          'shadow-sm',
          isDark ? 'bg-white/8 ring-1 ring-white/10' : 'bg-black/[0.04] ring-1 ring-black/5'
        )}
      >
        <ClipboardList
          className={clsx(
            'w-9 h-9',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
          aria-hidden
        />
      </div>

      <h3
        className={clsx(
          'text-base font-semibold mb-1.5 tracking-tight',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        {t('clipboard.empty_state')}
      </h3>

      <p
        className={clsx(
          'text-sm max-w-[240px] leading-relaxed',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        {t('clipboard.empty_state_desc')}
      </p>

      <div
        className={clsx(
          'mt-5 inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium',
          isDark ? 'bg-white/6 text-win11-text-secondary' : 'bg-black/[0.04] text-win11Light-text-secondary'
        )}
      >
        <kbd className="font-semibold tracking-wide">{t('clipboard.empty_shortcut')}</kbd>
      </div>
    </div>
  )
}

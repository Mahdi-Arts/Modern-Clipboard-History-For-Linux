import { clsx } from 'clsx'
import type { UserSettings } from '../../types/clipboard'
import { SectionCard } from './SectionCard'

interface UiScaleSectionProps {
  settings: UserSettings
  isDark: boolean
  onScaleChange: (value: number) => void
  onCommit: () => void
}

/** Clipboard popup scale control. */
export function UiScaleSection({ settings, isDark, onScaleChange, onCommit }: UiScaleSectionProps) {
  return (
    <SectionCard
      title="UI Scale"
      subtitle="Adjust the clipboard window size for your display"
      isDark={isDark}
    >
      <div className="space-y-4">
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <label htmlFor="ui-scale" className="text-sm font-medium">
              Clipboard Window Scale
            </label>
            <div
              className={clsx(
                'px-2 py-1 rounded text-xs font-mono font-medium',
                isDark ? 'bg-black/20' : 'bg-gray-100'
              )}
            >
              {Math.round(settings.ui_scale * 100)}%
            </div>
          </div>
          <input
            id="ui-scale"
            type="range"
            min="0.5"
            max="2"
            step="0.1"
            value={settings.ui_scale}
            onChange={(e) => onScaleChange(Number.parseFloat(e.target.value))}
            onMouseUp={onCommit}
            onTouchEnd={onCommit}
            onKeyUp={onCommit}
            className="w-full h-1.5 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700 accent-win11-bg-accent"
          />
          <p className={clsx('text-xs', isDark ? 'text-gray-500' : 'text-gray-400')}>
            This setting only affects the clipboard popup, not this settings window
          </p>
        </div>
      </div>
    </SectionCard>
  )
}

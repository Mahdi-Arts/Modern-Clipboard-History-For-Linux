import { clsx } from 'clsx'
import type { RenderingEnv, UserSettings } from '../../types/clipboard'
import { AlertTriangleIcon } from './icons'
import { SectionCard } from './SectionCard'

interface TransparencySectionProps {
  settings: UserSettings
  isDark: boolean
  renderingEnv: RenderingEnv
  onDarkOpacityChange: (value: number) => void
  onLightOpacityChange: (value: number) => void
  onCommit: () => void
}

function OpacitySlider({
  id,
  label,
  value,
  disabled,
  isDark,
  onChange,
  onCommit,
}: {
  id: string
  label: string
  value: number
  disabled: boolean
  isDark: boolean
  onChange: (v: number) => void
  onCommit: () => void
}) {
  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <label htmlFor={id} className="text-sm font-medium">
          {label}
        </label>
        <div
          className={clsx(
            'px-2 py-1 rounded text-xs font-mono font-medium',
            isDark ? 'bg-black/20' : 'bg-gray-100'
          )}
        >
          {disabled ? '100%' : `${Math.round(value * 100)}%`}
        </div>
      </div>
      <input
        id={id}
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={disabled ? 1 : value}
        onChange={(e) => onChange(Number.parseFloat(e.target.value))}
        onMouseUp={onCommit}
        onTouchEnd={onCommit}
        onKeyUp={onCommit}
        disabled={disabled}
        className={clsx(
          'w-full h-1.5 bg-gray-200 rounded-lg appearance-none dark:bg-gray-700 accent-win11-bg-accent',
          disabled ? 'cursor-not-allowed' : 'cursor-pointer'
        )}
      />
    </div>
  )
}

/** Backdrop opacity controls (auto-disabled on NVIDIA / AppImage). */
export function TransparencySection({
  settings,
  isDark,
  renderingEnv,
  onDarkOpacityChange,
  onLightOpacityChange,
  onCommit,
}: TransparencySectionProps) {
  const disabled = renderingEnv.transparency_disabled

  return (
    <SectionCard
      title="Window Transparency"
      subtitle="Control the backdrop opacity intensity"
      isDark={isDark}
      dimmed={disabled}
    >
      {disabled && (
        <div
          className={clsx(
            'mb-6 p-3 rounded-lg flex items-start gap-3 text-sm',
            isDark ? 'bg-yellow-500/10 text-yellow-300' : 'bg-yellow-50 text-yellow-800'
          )}
        >
          <AlertTriangleIcon className="flex-shrink-0 mt-0.5" />
          <div>
            <p className="font-medium text-xs">{renderingEnv.reason}</p>
            <p
              className={clsx(
                'text-[11px] mt-1',
                isDark ? 'text-yellow-400/70' : 'text-yellow-700'
              )}
            >
              Transparency and rounded window corners have been automatically disabled to prevent
              rendering artefacts.
            </p>
          </div>
        </div>
      )}

      <div className="space-y-8">
        <OpacitySlider
          id="dark-opacity"
          label="Dark Mode Opacity"
          value={settings.dark_background_opacity}
          disabled={disabled}
          isDark={isDark}
          onChange={onDarkOpacityChange}
          onCommit={onCommit}
        />
        <OpacitySlider
          id="light-opacity"
          label="Light Mode Opacity"
          value={settings.light_background_opacity}
          disabled={disabled}
          isDark={isDark}
          onChange={onLightOpacityChange}
          onCommit={onCommit}
        />
      </div>
    </SectionCard>
  )
}

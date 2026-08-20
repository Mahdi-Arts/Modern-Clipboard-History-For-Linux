import { useState, useCallback, useEffect, useRef, Suspense, lazy } from 'react'
import { clsx } from 'clsx'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import { useClipboardHistory } from './hooks/useClipboardHistory'
import { TabBar, TabBarRef } from './components/TabBar'
import { DragHandle } from './components/DragHandle'
import { calculateSecondaryOpacity, calculateTertiaryOpacity } from './utils/themeUtils'
import { useSystemThemePreference } from './utils/systemTheme'
import { useRenderingEnv } from './hooks/useRenderingEnv'
import { useLanguageEffect } from './i18n/useLanguage'
import { changeLanguage } from './i18n/config'
import type { ActiveTab, UserSettings } from './types/clipboard'
import { ClipboardTab } from './components/ClipboardTab'
import { DEFAULT_SETTINGS } from './utils/defaultSettings'

// Lazy-loaded tabs for code splitting
const EmojiPicker = lazy(() =>
  import('./components/EmojiPicker').then((m) => ({ default: m.EmojiPicker }))
)
const KaomojiPicker = lazy(() =>
  import('./components/KaomojiPicker').then((m) => ({ default: m.KaomojiPicker }))
)
const SymbolPicker = lazy(() =>
  import('./components/SymbolPicker').then((m) => ({ default: m.SymbolPicker }))
)

/**
 * Maps theme mode setting to actual dark mode state.
 * For 'system' mode, this hook delegates system theme detection
 * to useSystemThemePreference().
 */
function useThemeMode(themeMode: 'system' | 'dark' | 'light'): boolean {
  const systemPrefersDark = useSystemThemePreference()

  if (themeMode === 'dark') return true
  if (themeMode === 'light') return false
  return systemPrefersDark // 'system' mode
}

/**
 * Applies background opacity CSS variables based on user settings
 * Opacity: 0.0 = fully transparent, 1.0 = fully opaque
 */
function applyBackgroundOpacity(settings: UserSettings) {
  const root = document.documentElement

  // The gradient end is slightly less opaque than start for a subtle effect
  // Using a small offset (0.03) to create a gentle gradient
  const darkStart = settings.dark_background_opacity
  const darkEnd = darkStart >= 1 ? 1 : Math.max(0, darkStart - 0.03)
  const lightStart = settings.light_background_opacity
  const lightEnd = lightStart >= 1 ? 1 : Math.max(0, lightStart - 0.05)

  root.style.setProperty('--win11-dark-bg-alpha-start', darkStart.toString())
  root.style.setProperty('--win11-dark-bg-alpha-end', darkEnd.toString())
  root.style.setProperty('--win11-light-bg-alpha-start', lightStart.toString())
  root.style.setProperty('--win11-light-bg-alpha-end', lightEnd.toString())
}

/**
 * Updates the document's dark class based on theme
 */
function applyThemeClass(isDark: boolean) {
  if (isDark) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

/**
 * Applies the UI scale/zoom level to the webview
 */
async function applyUIScale(scale: number) {
  try {
    await getCurrentWebview().setZoom(scale)
  } catch (err) {
    console.error('Failed to apply UI scale:', err)
  }
}

/**
 * Main Clipboard App Component
 */
function ClipboardApp() {
  // i18n: language change listener updates all translations reactively
  const { i18n } = useTranslation()
  useLanguageEffect(i18n)

  const [activeTab, setActiveTab] = useState<ActiveTab>('clipboard')
  const [settings, setSettings] = useState<UserSettings>(DEFAULT_SETTINGS)
  const [settingsLoaded, setSettingsLoaded] = useState(false)

  const renderingEnv = useRenderingEnv()
  const isDark = useThemeMode(settings.theme_mode)

  // When transparency is disabled (NVIDIA / AppImage) force opacity to fully opaque
  const effectiveDarkOpacity = renderingEnv.transparency_disabled
    ? 1
    : settings.dark_background_opacity
  const effectiveLightOpacity = renderingEnv.transparency_disabled
    ? 1
    : settings.light_background_opacity

  const opacity = isDark ? effectiveDarkOpacity : effectiveLightOpacity
  const secondaryOpacity = calculateSecondaryOpacity(opacity)
  const tertiaryOpacity = calculateTertiaryOpacity(opacity)

  const { history, isLoading, clearHistory, deleteItem, togglePin, pasteItem } =
    useClipboardHistory()

  // Refs for focus management
  const tabBarRef = useRef<TabBarRef>(null)
  const contentContainerRef = useRef<HTMLDivElement>(null)

  // Load initial settings and set up listener for changes
  useEffect(() => {
    // Load initial settings
    invoke<UserSettings>('get_user_settings')
      .then((loadedSettings) => {
        const merged = { ...DEFAULT_SETTINGS, ...loadedSettings }
        setSettings(merged)
        applyBackgroundOpacity(merged)
        applyUIScale(merged.ui_scale)
        setSettingsLoaded(true)
      })
      .catch((err) => {
        console.error('Failed to load user settings:', err)
        applyBackgroundOpacity(DEFAULT_SETTINGS)
        applyUIScale(DEFAULT_SETTINGS.ui_scale)
        setSettingsLoaded(true)
      })

    // Listen for settings changes from the settings window
    const unlistenPromise = listen<UserSettings>('app-settings-changed', (event) => {
      const newSettings = event.payload
      setSettings(newSettings)
      applyBackgroundOpacity(newSettings)
      applyUIScale(newSettings.ui_scale)
    })

    // Listen for switch-tab events from Rust (e.g., when Super+. is pressed)
    const unlistenSwitchTab = listen<string>('switch-tab', (event) => {
      const tabName = event.payload as ActiveTab
      if (['clipboard', 'emoji', 'kaomoji', 'symbols'].includes(tabName)) {
        setActiveTab(tabName)
      }
    })

    return () => {
      unlistenPromise.then((unlisten) => unlisten())
      unlistenSwitchTab.then((unlisten) => unlisten())
    }
  }, [])

  // Listen for language change events from settings/backend
  useEffect(() => {
    const unlistenLang = listen<string>('app-language-changed', (event) => {
      const lang = event.payload
      if (lang === 'fa' || lang === 'en') {
        changeLanguage(lang)
      }
    })
    return () => {
      unlistenLang.then((u) => u())
    }
  }, [])

  // Re-apply CSS opacity variables whenever renderingEnv or settings change
  useEffect(() => {
    if (renderingEnv.transparency_disabled) {
      // Force fully opaque CSS variables
      const opaque: UserSettings = {
        ...settings,
        dark_background_opacity: 1,
        light_background_opacity: 1,
      }
      applyBackgroundOpacity(opaque)
    } else {
      applyBackgroundOpacity(settings)
    }
  }, [renderingEnv.transparency_disabled, settings])

  // Apply theme class when isDark changes
  useEffect(() => {
    applyThemeClass(isDark)
  }, [isDark])

  // Handle ESC key to close/hide window
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        try {
          await getCurrentWindow().hide()
        } catch (err) {
          console.error('Failed to hide window:', err)
        }
      }
    }

    globalThis.addEventListener('keydown', handleKeyDown)
    return () => globalThis.removeEventListener('keydown', handleKeyDown)
  }, [])

  // Use refs to store current values for the focus handler (to avoid re-registering listener)
  const activeTabRef = useRef(activeTab)

  // Keep refs in sync
  useEffect(() => {
    activeTabRef.current = activeTab
  }, [activeTab])

  // Handle window-shown event for focus management (registered once)
  useEffect(() => {
    const focusFirstItem = () => {
      // Small delay to ensure the window is fully rendered and focused
      setTimeout(() => {
        const currentTab = activeTabRef.current

        if (currentTab !== 'clipboard') {
          // Focus the first tab button if on other tabs
          // Clipboard tab focus is handled inside ClipboardTab component
          tabBarRef.current?.focusFirstTab()
        }
      }, 100)
    }

    // Listen to window-shown event (emitted from Rust when window is toggled visible)
    const unlistenWindowShown = listen('window-shown', focusFirstItem)

    return () => {
      unlistenWindowShown.then((unlisten) => unlisten())
    }
  }, []) // Empty dependency array - listener is registered once

  // Handle tab change
  const handleTabChange = useCallback((tab: ActiveTab) => {
    setActiveTab(tab)
  }, [])

  const handleMouseEnter = () => {
    invoke('set_mouse_state', { inside: true }).catch(console.error)
  }

  const handleMouseLeave = () => {
    invoke('set_mouse_state', { inside: false }).catch(console.error)
  }

  // Render content based on active tab (with lazy loading via Suspense)
  const renderContent = () => {
    const spinner = (
      <div className="flex items-center justify-center h-full p-8">
        <div className="w-5 h-5 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )

    switch (activeTab) {
      case 'clipboard':
        return (
          <ClipboardTab
            history={history}
            isLoading={isLoading}
            isDark={isDark}
            tertiaryOpacity={tertiaryOpacity}
            secondaryOpacity={secondaryOpacity}
            clearHistory={clearHistory}
            deleteItem={deleteItem}
            togglePin={togglePin}
            onPaste={pasteItem}
            settings={settings}
            tabBarRef={tabBarRef}
          />
        )

      case 'emoji':
        return (
          <Suspense fallback={spinner}>
            <EmojiPicker isDark={isDark} opacity={secondaryOpacity} />
          </Suspense>
        )

      // case 'gifs':
      //   return <GifPicker isDark={isDark} opacity={secondaryOpacity} />

      case 'kaomoji':
        return (
          <Suspense fallback={spinner}>
            <KaomojiPicker
              isDark={isDark}
              opacity={secondaryOpacity}
              customKaomojis={settings.custom_kaomojis}
            />
          </Suspense>
        )

      case 'symbols':
        return (
          <Suspense fallback={spinner}>
            <SymbolPicker isDark={isDark} opacity={secondaryOpacity} />
          </Suspense>
        )

      default:
        return null
    }
  }

  if (!settingsLoaded) {
    return (
      <div
        className="h-screen w-screen flex items-center justify-center bg-transparent"
        role="status"
        aria-label="Loading"
      >
        <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  return (
    <div
      className={clsx(
        'h-screen w-screen overflow-hidden flex flex-col select-none',
        renderingEnv.transparency_disabled ? 'rounded-none' : 'rounded-win11-lg',
        isDark ? 'glass-effect' : 'glass-effect-light',
        isDark ? 'bg-win11-acrylic-bg' : 'bg-win11Light-acrylic-bg',
        isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
      )}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Drag Handle */}
      <DragHandle isDark={isDark} />

      {/* Tab bar */}
      <TabBar
        ref={tabBarRef}
        activeTab={activeTab}
        onTabChange={handleTabChange}
        isDark={isDark}
        tertiaryOpacity={tertiaryOpacity}
      />

      {/* Scrollable content area */}
      <div
        ref={contentContainerRef}
        className={clsx(
          'flex-1',
          // Only use scrollbar for non-emoji/gif/kaomoji tabs, they have their own virtualized scrolling or containers
          activeTab === 'emoji' ||
            // activeTab === 'gifs' ||
            activeTab === 'kaomoji' ||
            activeTab === 'symbols'
            ? 'overflow-hidden'
            : 'overflow-y-auto scrollbar-win11'
        )}
      >
        {renderContent()}
      </div>
    </div>
  )
}

export default ClipboardApp

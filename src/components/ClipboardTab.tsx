import { useState, useMemo, useRef, useEffect, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'
import { clsx } from 'clsx'
import { Pin, History, ChevronDown } from 'lucide-react'
import { List, ListImperativeAPI } from 'react-window'

import type { ClipboardItem, UserSettings } from '../types/clipboard'
import type { TabBarRef } from './TabBar'
import { Header } from './Header'
import { SearchBar } from './common/SearchBar'
import { EmptyState } from './EmptyState'
import { HistoryItem } from './HistoryItem'
import { useHistoryKeyboardNavigation } from '../hooks/useHistoryKeyboardNavigation'

// --- Virtualized List Row Component ---

interface RowData {
  items: ClipboardItem[]
  onPaste: (id: string) => void
  onDelete: (id: string) => void
  onTogglePin: (id: string) => void
  onFocus: (idx: number) => void
  focusedIndex: number
  isDark: boolean
  isCompact: boolean
  secondaryOpacity: number
  enableSmartActions: boolean
  enableUiPolish: boolean
  itemRefs: React.MutableRefObject<(HTMLDivElement | null)[]>
}

function HistoryRow({ index, style, data }: { index: number; style: React.CSSProperties; data: RowData }) {
  const {
    items, itemRefs,
    onPaste, onDelete, onTogglePin, onFocus,
    focusedIndex, isDark, isCompact, secondaryOpacity,
    enableSmartActions, enableUiPolish
  } = data

  if (index >= items.length) return null

  const item = items[index]
  return (
    <div style={style}>
      <HistoryItem
        ref={(el) => { itemRefs.current[index] = el }}
        item={item}
        index={index}
        isFocused={index === focusedIndex}
        onPaste={onPaste}
        onDelete={onDelete}
        onTogglePin={onTogglePin}
        onFocus={() => onFocus(index)}
        isDark={isDark}
        secondaryOpacity={secondaryOpacity}
        isCompact={isCompact}
        enableSmartActions={enableSmartActions}
        enableUiPolish={enableUiPolish}
      />
    </div>
  )
}

// --- Main ClipboardTab Component ---

export function ClipboardTab(props: {
  history: ClipboardItem[]
  isLoading: boolean
  isDark: boolean
  tertiaryOpacity: number
  secondaryOpacity: number
  clearHistory: () => void
  deleteItem: (id: string) => void
  togglePin: (id: string) => void
  onPaste: (id: string) => void
  settings: UserSettings
  tabBarRef: React.RefObject<TabBarRef | null>
}) {
  const {
    history,
    isLoading,
    isDark,
    tertiaryOpacity,
    secondaryOpacity,
    clearHistory,
    deleteItem,
    togglePin,
    onPaste,
    settings,
    tabBarRef,
  } = props

  const [searchQuery, setSearchQuery] = useState('')
  const [isRegexMode, setIsRegexMode] = useState(false)

  const [isCompact, setIsCompact] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('clipboard-history-compact-mode') === 'true'
    }
    return false
  })

  useEffect(() => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('clipboard-history-compact-mode', String(isCompact))
    }
  }, [isCompact])

  const [isSearchVisible, setIsSearchVisible] = useState(false)
  const searchInputRef = useRef<HTMLInputElement>(null)
  const [focusedIndex, setFocusedIndex] = useState(0)
  const historyItemRefs = useRef<(HTMLDivElement | null)[]>([])
  const listRef = useRef<ListImperativeAPI | null>(null);
  const containerRef = useRef<HTMLDivElement>(null)
const [containerHeight, setContainerHeight] = useState(300)

// Measure container height for virtualized list
useEffect(() => {
  const el = containerRef.current
  if (!el) return
  const observer = new ResizeObserver((entries) => {
    for (const entry of entries) {
      setContainerHeight(entry.contentRect.height)
    }
  })
  observer.observe(el)
  return () => observer.disconnect()
}, [])

  // Pinned section collapsible state (persisted)
  const [pinnedExpanded, setPinnedExpanded] = useState(() => {
    if (typeof window !== 'undefined') {
      const stored = localStorage.getItem('clipboard-pinned-expanded')
      return stored !== null ? stored === 'true' : true
    }
    return true
  })

  useEffect(() => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('clipboard-pinned-expanded', String(pinnedExpanded))
    }
  }, [pinnedExpanded])

  // Check if a key is a printable character that should trigger search
  const isPrintableKey = useCallback((e: KeyboardEvent): boolean => {
    if (e.ctrlKey || e.altKey || e.metaKey) return false
    const specialKeys = [
      'Tab', 'Enter', 'Escape', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
      'Home', 'End', 'PageUp', 'PageDown', 'Delete', 'Backspace',
      'F1','F2','F3','F4','F5','F6','F7','F8','F9','F10','F11','F12',
      'CapsLock', 'NumLock', 'ScrollLock', 'Pause', 'Insert', 'PrintScreen',
      'ContextMenu', 'Shift', 'Control', 'Alt', 'Meta'
    ]
    if (specialKeys.includes(e.key)) return false
    return e.key.length === 1
  }, [])

  // Toggle search visibility with Ctrl+F or start typing to filter
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    const activeElement = document.activeElement
    if (e.ctrlKey && e.key.toLowerCase() === 'f') {
      e.preventDefault()
      setIsSearchVisible((prev) => {
        if (!prev) return true
        setSearchQuery('')
        return false
      })
      return
    }
    if (e.key.toLowerCase() === 'escape' && isSearchVisible) {
      e.preventDefault()
      setIsSearchVisible(false)
      setSearchQuery('')
      return
    }
    if (activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA') return
    if (activeElement?.getAttribute('role') === 'tab') return
    if (isPrintableKey(e)) {
      e.preventDefault()
      if (!isSearchVisible) {
        setIsSearchVisible(true)
        setSearchQuery(e.key)
      } else {
        setSearchQuery((prev) => prev + e.key)
        searchInputRef.current?.focus()
      }
    }
  }, [isSearchVisible, isPrintableKey])

  useEffect(() => {
    globalThis.addEventListener('keydown', handleKeyDown)
    return () => globalThis.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  useEffect(() => {
    if (isSearchVisible && searchInputRef.current) {
      searchInputRef.current.focus()
    }
  }, [isSearchVisible])

  useEffect(() => {
    const resetSearch = () => {
      setIsSearchVisible(false)
      setSearchQuery('')
    }
    const unlistenWindowShown = listen('window-shown', resetSearch)
    return () => { unlistenWindowShown.then((u) => u()) }
  }, [])

  // Filter history by search query
  const filteredHistory = useMemo(() => {
    if (!searchQuery) return history
    let regex: RegExp | null = null
    if (isRegexMode) {
      try { regex = new RegExp(searchQuery, 'i') }
      catch { return [] }
    }
    return history.filter((item) => {
      let searchableText = ''
      if (item.content.type === 'Text') searchableText = item.content.data
      else if (item.content.type === 'RichText') searchableText = item.content.data.plain
      else return false
      if (isRegexMode && regex) return regex.test(searchableText)
      return searchableText.toLowerCase().includes(searchQuery.toLowerCase())
    })
  }, [history, searchQuery, isRegexMode])

  const pinnedItems = useMemo(() => filteredHistory.filter((i) => i.pinned), [filteredHistory])
  const unpinnedItems = useMemo(() => filteredHistory.filter((i) => !i.pinned), [filteredHistory])
  const showSections = !searchQuery && pinnedItems.length > 0

  // Visible items (flat list for virtualizer)
  const visibleItems = useMemo(() => {
    if (showSections && !pinnedExpanded) return unpinnedItems
    return filteredHistory
  }, [filteredHistory, showSections, pinnedExpanded, pinnedItems, unpinnedItems])

  const ITEM_HEIGHT = isCompact ? 44 : 64
  const GAP_HEIGHT = 8 // gap-2 between items

  // Keyboard navigation
  const onUpFromFirstItem = useCallback(() => {
    if (showSections && !pinnedExpanded) {
      setPinnedExpanded(true)
      const lastIdx = pinnedItems.length - 1
      setFocusedIndex(lastIdx)
      listRef.current?.scrollToRow({ index: lastIdx, align: 'smart' })
      setTimeout(() => historyItemRefs.current[lastIdx]?.focus(), 50)
      return true
    }
    return false
  }, [showSections, pinnedExpanded, pinnedItems.length, listRef])

  const onLeftArrow = useCallback(() => {
    if (showSections && pinnedExpanded && focusedIndex < pinnedItems.length) {
      setPinnedExpanded(false)
      setFocusedIndex(0)
listRef.current?.scrollToRow({ index: 0, align: 'smart' })
      setTimeout(() => historyItemRefs.current[0]?.focus(), 50)
    }
  }, [showSections, pinnedExpanded, focusedIndex, pinnedItems.length, listRef])

  useHistoryKeyboardNavigation({
    activeTab: 'clipboard',
    itemsLength: visibleItems.length,
    focusedIndex,
    setFocusedIndex,
    historyItemRefs: historyItemRefs,
    tabBarRef,
    searchInputRef,
    onUpFromFirstItem,
    onLeftArrow,
  })

  useEffect(() => {
    setFocusedIndex(0)
    listRef.current?.scrollToRow({ index: 0, align: 'smart' })
  }, [filteredHistory, listRef])

  const filteredHistoryRef = useRef(filteredHistory)
  useEffect(() => { filteredHistoryRef.current = filteredHistory }, [filteredHistory])

  // Focus first item on window shown
  useEffect(() => {
    const focusFirstItem = () => {
      setTimeout(() => {
        if (filteredHistoryRef.current.length > 0) {
          setFocusedIndex(0)
listRef.current?.scrollToRow({ index: 0, align: 'smart' })
          setTimeout(() => historyItemRefs.current[0]?.focus(), 100)
        }
      }, 100)
    }
    const unlistenWindowShown = listen('window-shown', focusFirstItem)
    return () => { unlistenWindowShown.then((u) => u()) }
  }, [listRef])

  // Track which ref slot is the actual focused item for the virtualizer
  const handleItemFocus = useCallback((idx: number) => {
    setFocusedIndex(idx)
  }, [])

  // Row data for react-window
  const rowData: RowData = useMemo(() => ({
    items: visibleItems,
    onPaste,
    onDelete: deleteItem,
    onTogglePin: togglePin,
    onFocus: handleItemFocus,
    focusedIndex,
    isDark,
    isCompact,
    secondaryOpacity,
    enableSmartActions: settings.enable_smart_actions,
    enableUiPolish: settings.enable_ui_polish,
    itemRefs: historyItemRefs,
  }), [visibleItems, onPaste, deleteItem, togglePin,
      handleItemFocus, focusedIndex, isDark, isCompact, secondaryOpacity, settings])

  // --- Render ---
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full select-none">
        <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  if (history.length === 0) {
    return <EmptyState isDark={isDark} />
  }

  return (
    <>
      <Header
        onClearHistory={clearHistory}
        itemCount={filteredHistory.length}
        isDark={isDark}
        tertiaryOpacity={tertiaryOpacity}
        isCompact={isCompact}
        onToggleCompact={() => setIsCompact(!isCompact)}
      />
      {isSearchVisible && (
        <div className="px-3 pb-2 pt-1">
          <SearchBar
            ref={searchInputRef}
            value={searchQuery}
            onChange={setSearchQuery}
            isDark={isDark}
            opacity={secondaryOpacity}
            placeholder="Search history..."
            isRegex={isRegexMode}
            onToggleRegex={() => setIsRegexMode(!isRegexMode)}
            onClear={() => {
              setSearchQuery('')
              setIsSearchVisible(false)
            }}
          />
        </div>
      )}

      {filteredHistory.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-8 text-center opacity-60">
          <p className={clsx('text-sm', isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary')}>
            {searchQuery ? 'No items found' : 'No clipboard history yet'}
          </p>
        </div>
      ) : (
        <div className="flex flex-col flex-1 min-h-0">
          {/* Pinned section header (only when sections shown) */}
          {showSections && (
            <div className="px-3 pt-2 pb-1 flex-shrink-0">
              <button
                onClick={() => {
                  const willCollapse = pinnedExpanded
                  setPinnedExpanded(!pinnedExpanded)
                  if (willCollapse) {
                    setFocusedIndex(0)
                    setTimeout(() => historyItemRefs.current[0]?.focus(), 50)
                  }
                }}
                className={clsx(
                  'flex items-center gap-1.5 px-1 py-1 text-xs font-medium w-full',
                  'dark:text-win11-text-tertiary text-win11Light-text-tertiary',
                  'hover:dark:text-win11-text-secondary hover:text-win11Light-text-secondary',
                  'rounded transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
                )}
                aria-expanded={pinnedExpanded}
              >
                <Pin size={12} />
                <span>Pinned</span>
                <span className="ml-auto opacity-60">{pinnedItems.length}</span>
                <ChevronDown
                  size={12}
                  className={clsx('transition-transform duration-150', !pinnedExpanded && '-rotate-90')}
                />
              </button>
            </div>
          )}

          {/* Pinned items (always small, rendered inline) */}
          {showSections && pinnedExpanded && (
            <div className="px-3 pb-1 flex-shrink-0 space-y-2">
              {pinnedItems.map((item, offset) => (
                <HistoryItem
                  key={item.id}
                  ref={(el) => { historyItemRefs.current[offset] = el }}
                  item={item}
                  index={offset}
                  isFocused={offset === focusedIndex}
                  onPaste={onPaste}
                  onDelete={deleteItem}
                  onTogglePin={togglePin}
                  onFocus={() => setFocusedIndex(offset)}
                  isDark={isDark}
                  secondaryOpacity={secondaryOpacity}
                  isCompact={isCompact}
                  enableSmartActions={settings.enable_smart_actions}
                  enableUiPolish={settings.enable_ui_polish}
                />
              ))}
            </div>
          )}

          {/* Recent section label */}
          {showSections && unpinnedItems.length > 0 && (
            <div className="px-3 py-1 flex items-center gap-1.5 text-xs dark:text-win11-text-tertiary text-win11Light-text-tertiary flex-shrink-0">
              <History size={12} />
              <span>Recent</span>
              <span className="ml-auto opacity-60">{unpinnedItems.length}</span>
            </div>
          )}

          {/* Virtualized list */}
          <div ref={containerRef} className="flex-1 min-h-0 px-3 pb-3">
            {visibleItems.length > 0 && (
              <List
                listRef={listRef}
                height={containerHeight}
                width="100%"
                itemCount={visibleItems.length}
                itemSize={ITEM_HEIGHT + GAP_HEIGHT}
                itemData={rowData}
                overscanCount={5}
                className="scrollbar-win11"
                style={{ overflowX: 'hidden', overflowY: 'auto' }}
              >
                {({ index, style, data }: { index: number; style: React.CSSProperties; data: RowData }) => HistoryRow({ index, style, data })}
              </List>
            )}
          </div>
        </div>
      )}
    </>
  )
}
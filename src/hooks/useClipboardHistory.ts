import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import type { ClipboardItem, HistoryPage } from '../types/clipboard'
import { clampOffset, clampPageSize, hasNextPage, mergePageById } from '../utils/pagination'

/** Options for windowed history loading (ADR-0007).
 *  گزینه‌های بارگذاری پنجره‌ای تاریخچه (ADR-0007). */
export interface UseClipboardHistoryOptions {
  /**
   * When set, the initial load and `loadMore` fetch bounded pages via
   * `get_history_page` instead of one full `get_history` payload.
   * وقتی تنظیم شود، بارگذاری اولیه و `loadMore` به‌جای یک بار کامل،
   * پنجره‌های محدود از `get_history_page` می‌گیرند.
   */
  pageSize?: number
}

/**
 * Hook for managing clipboard history.
 * هوک مدیریت تاریخچهٔ کلیپ‌بورد.
 */
export function useClipboardHistory(options: UseClipboardHistoryOptions = {}) {
  const [history, setHistory] = useState<ClipboardItem[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [total, setTotal] = useState<number | null>(null)

  const pageSize = options.pageSize
  const nextOffsetRef = useRef(0)

  // Fetch initial history
  const fetchHistory = useCallback(async () => {
    try {
      setIsLoading(true)
      if (pageSize != null) {
        const page = await invoke<HistoryPage>('get_history_page', {
          limit: clampPageSize(pageSize),
          offset: 0,
        })
        nextOffsetRef.current = page.items.length
        setTotal(page.total)
        setHistory(page.items)
      } else {
        const items = await invoke<ClipboardItem[]>('get_history')
        setHistory(items)
      }
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch history')
    } finally {
      setIsLoading(false)
    }
  }, [pageSize])

  /** Fetch the next bounded window and merge it into the list.
   *  دریافت پنجرهٔ بعدی و ادغام آن در فهرست. */
  const loadMore = useCallback(async () => {
    if (pageSize == null) return
    const limit = clampPageSize(pageSize)
    const offset = clampOffset(nextOffsetRef.current)
    const currentTotal = total
    if (currentTotal != null && !hasNextPage(currentTotal, offset, limit)) return
    try {
      const page = await invoke<HistoryPage>('get_history_page', { limit, offset })
      nextOffsetRef.current = offset + page.items.length
      setTotal(page.total)
      setHistory((prev) => mergePageById(prev, page.items))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load more history')
    }
  }, [pageSize, total])

  // Clear all history
  const clearHistory = useCallback(async () => {
    try {
      await invoke('clear_history')
      setHistory((prev) => prev.filter((item) => item.pinned))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to clear history')
    }
  }, [])

  // Delete a specific item
  const deleteItem = useCallback(async (id: string) => {
    try {
      await invoke('delete_item', { id })
      setHistory((prev) => prev.filter((item) => item.id !== id))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete item')
    }
  }, [])

  // Toggle pin status
  const togglePin = useCallback(
    async (id: string) => {
      try {
        const updatedItem = await invoke<ClipboardItem>('toggle_pin', { id })
        if (updatedItem) {
          setHistory((prev) => {
            // Remove the item from its current position
            const otherItems = prev.filter((item) => item.id !== id)
            const pinnedItems = otherItems.filter((item) => item.pinned)
            const unpinnedItems = otherItems.filter((item) => !item.pinned)

            if (updatedItem.pinned) {
              // Item was pinned - add to the end of pinned items (top of list)
              return [...pinnedItems, updatedItem, ...unpinnedItems]
            } else {
              // Item was unpinned - insert in correct position by timestamp
              const allUnpinned = [updatedItem, ...unpinnedItems]
              allUnpinned.sort(
                (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
              )
              return [...pinnedItems, ...allUnpinned]
            }
          })
        } else {
          // Item not found - refresh history
          console.warn('[useClipboardHistory] Toggle pin returned null, refreshing history')
          await fetchHistory()
        }
      } catch (err) {
        console.warn('[useClipboardHistory] Toggle pin failed, refreshing history')
        await fetchHistory()
        setError(err instanceof Error ? err.message : 'Failed to toggle pin')
      }
    },
    [fetchHistory]
  )

  // Paste an item
  const pasteItem = useCallback(
    async (id: string) => {
      try {
        await invoke('paste_item', { id })
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err)
        console.warn('[useClipboardHistory] Paste failed, refreshing history:', errorMessage)
        // If paste failed due to item not found, refresh history
        // The backend already emits history-sync event, but we fetch as backup
        await fetchHistory()
        setError(errorMessage)
      }
    },
    [fetchHistory]
  )

  // Listen for clipboard changes
  useEffect(() => {
    const initialFetchTimer = globalThis.setTimeout(() => {
      fetchHistory()
    }, 0)

    let isMounted = true
    let unlistenChanged: UnlistenFn | undefined
    let unlistenCleared: UnlistenFn | undefined
    let unlistenSync: UnlistenFn | undefined

    const setupListeners = async () => {
      const uChanged = await listen<ClipboardItem>('clipboard-changed', (event) => {
        const incoming = event.payload
        setHistory((prev) => {
          const without = prev.filter((item) => item.id !== incoming.id)
          const pinned = without.filter((item) => item.pinned)
          const unpinned = without.filter((item) => !item.pinned)
          if (incoming.pinned) {
            return [incoming, ...pinned, ...unpinned]
          }
          return [...pinned, incoming, ...unpinned]
        })
      })
      if (!isMounted) {
        uChanged()
      } else {
        unlistenChanged = uChanged
      }

      const uCleared = await listen('history-cleared', async () => {
        console.log('[useClipboardHistory] history-cleared event received')
        try {
          await fetchHistory()
        } catch (e) {
          console.warn('[useClipboardHistory] Failed to refresh history on history-cleared', e)
        }
      })
      if (!isMounted) {
        uCleared()
      } else {
        unlistenCleared = uCleared
      }

      const uSync = await listen<ClipboardItem[]>('history-sync', async (event) => {
        console.log('[useClipboardHistory] history-sync event received')
        setHistory(event.payload)
      })
      if (!isMounted) {
        uSync()
      } else {
        unlistenSync = uSync
      }
    }

    setupListeners()

    return () => {
      globalThis.clearTimeout(initialFetchTimer)
      isMounted = false
      unlistenChanged?.()
      unlistenCleared?.()
      unlistenSync?.()
    }
  }, [fetchHistory])

  return {
    history,
    isLoading,
    error,
    total,
    /** Windowed loading; only meaningful with `pageSize`. / فقط با `pageSize` معنا دارد. */
    loadMore,
    fetchHistory,
    clearHistory,
    deleteItem,
    togglePin,
    pasteItem,
  }
}

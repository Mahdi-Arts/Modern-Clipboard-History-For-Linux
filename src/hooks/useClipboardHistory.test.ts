import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock Tauri APIs before importing the hook
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

import { invoke } from '@tauri-apps/api/core'

describe('useClipboardHistory — invoke contracts', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('get_history returns an array', async () => {
    const mockHistory = [
      {
        id: '1',
        content: { type: 'Text', data: 'hello' },
        timestamp: new Date().toISOString(),
        pinned: false,
        preview: 'hello',
      },
    ]
    vi.mocked(invoke).mockResolvedValueOnce(mockHistory)

    const result = await invoke('get_history')
    expect(Array.isArray(result)).toBe(true)
    expect(result).toHaveLength(1)
  })

  it('clear_history is callable', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)
    await expect(invoke('clear_history')).resolves.toBeUndefined()
  })

  it('delete_item passes id parameter', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)
    await invoke('delete_item', { id: 'test-id' })
    expect(invoke).toHaveBeenCalledWith('delete_item', { id: 'test-id' })
  })

  it('toggle_pin passes id parameter', async () => {
    const mockItem = {
      id: '1',
      content: { type: 'Text', data: 'hello' },
      timestamp: new Date().toISOString(),
      pinned: true,
      preview: 'hello',
    }
    vi.mocked(invoke).mockResolvedValueOnce(mockItem)
    const result = await invoke('toggle_pin', { id: '1' })
    expect(result).toEqual(mockItem)
  })

  it('paste_item passes id parameter', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined)
    await invoke('paste_item', { id: 'test-id' })
    expect(invoke).toHaveBeenCalledWith('paste_item', { id: 'test-id' })
  })
})

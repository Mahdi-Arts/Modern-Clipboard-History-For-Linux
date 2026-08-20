import { describe, expect, it, vi, beforeEach } from 'vitest'
import { smartActionService } from './smartActionService'
import { sanitizeOpenUrl } from '../utils/urlSafety'

const openMock = vi.fn()

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: (...args: unknown[]) => openMock(...args),
}))

describe('smartActionService', () => {
  beforeEach(() => {
    openMock.mockReset()
  })

  it('detects https URLs', () => {
    const actions = smartActionService.detectActions('https://example.com/docs')
    expect(actions.some((a) => a.id === 'open-link')).toBe(true)
  })

  it('detects emails', () => {
    const actions = smartActionService.detectActions('user@example.com')
    expect(actions.some((a) => a.id === 'compose-email')).toBe(true)
  })

  it('detects hex and rgb colors', () => {
    expect(smartActionService.detectActions('#0A84FF').some((a) => a.id === 'color-preview')).toBe(
      true
    )
    expect(
      smartActionService.detectActions('rgb(10, 132, 255)').some((a) => a.id === 'color-preview')
    ).toBe(true)
  })

  it('ignores plain text and unsafe URLs', () => {
    expect(smartActionService.detectActions('hello world')).toEqual([])
    expect(smartActionService.detectActions('javascript:alert(1)')).toEqual([])
    expect(sanitizeOpenUrl('javascript:alert(1)')).toBeNull()
  })

  it('ignores overlong text and malformed emails', () => {
    expect(smartActionService.detectActions('a'.repeat(3000))).toEqual([])
    expect(smartActionService.detectActions('not-an-email@')).toEqual([])
  })

  it('executes open-link through the scoped shell plugin', async () => {
    const actions = smartActionService.detectActions('https://example.com/docs')
    const action = actions.find((a) => a.id === 'open-link')
    expect(action).toBeDefined()

    openMock.mockResolvedValue(undefined)
    await smartActionService.execute(action!)
    expect(openMock).toHaveBeenCalledWith('https://example.com/docs')
  })

  it('executes compose-email through a mailto: URL', async () => {
    const actions = smartActionService.detectActions('user@example.com')
    const action = actions.find((a) => a.id === 'compose-email')
    expect(action).toBeDefined()

    openMock.mockResolvedValue(undefined)
    await smartActionService.execute(action!)
    expect(openMock).toHaveBeenCalledWith('mailto:user@example.com')
  })

  it('throws when asked to open a blocked URL', async () => {
    await expect(
      smartActionService.execute({ id: 'open-link', label: 'Open Link', data: 'http://127.0.0.1/' })
    ).rejects.toThrow('Blocked unsafe URL')
    expect(openMock).not.toHaveBeenCalled()
  })
})

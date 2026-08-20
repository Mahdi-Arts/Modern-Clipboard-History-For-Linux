import { describe, expect, it } from 'vitest'
import { smartActionService } from './smartActionService'
import { sanitizeOpenUrl } from '../utils/urlSafety'

describe('smartActionService', () => {
  it('detects https URLs', () => {
    const actions = smartActionService.detectActions('https://example.com/docs')
    expect(actions.some((a) => a.id === 'open-link')).toBe(true)
  })

  it('detects emails', () => {
    const actions = smartActionService.detectActions('user@example.com')
    expect(actions.some((a) => a.id === 'compose-email')).toBe(true)
  })

  it('detects hex colors', () => {
    const actions = smartActionService.detectActions('#0A84FF')
    expect(actions.some((a) => a.id === 'color-preview')).toBe(true)
  })

  it('blocks javascript URLs', () => {
    expect(sanitizeOpenUrl('javascript:alert(1)')).toBeNull()
    expect(smartActionService.detectActions('javascript:alert(1)')).toEqual([])
  })
})

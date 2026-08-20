import { invoke } from '@tauri-apps/api/core'
import { normalizeHttpUrl, sanitizeOpenUrl } from '../utils/urlSafety'

export type SmartActionType = 'open-link' | 'compose-email' | 'color-preview'

export interface SmartAction {
  id: SmartActionType
  label: string
  data?: string
}

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
const HEX_COLOR_REGEX = /^#([0-9A-F]{3}){1,2}$/i
const RGB_COLOR_REGEX =
  /^rgb\(\s*(25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(25[0-5]|2[0-4]\d|1?\d?\d)\s*\)$/i

function looksLikeHttpUrl(value: string): boolean {
  if (value.length > 2048) return false
  try {
    const url = new URL(normalizeHttpUrl(value))
    return url.protocol === 'http:' || url.protocol === 'https:'
  } catch {
    return false
  }
}

export const smartActionService = {
  detectActions(content: string): SmartAction[] {
    const actions: SmartAction[] = []
    if (!content) return actions

    const trimmed = content.trim()

    if (looksLikeHttpUrl(trimmed) && !EMAIL_REGEX.test(trimmed)) {
      const normalizedUrl = normalizeHttpUrl(trimmed)
      const safe = sanitizeOpenUrl(normalizedUrl)
      if (safe) {
        actions.push({
          id: 'open-link',
          label: 'Open Link',
          data: safe,
        })
      }
    }

    if (EMAIL_REGEX.test(trimmed)) {
      const mailto = sanitizeOpenUrl(`mailto:${trimmed}`)
      if (mailto) {
        actions.push({ id: 'compose-email', label: 'Compose Email', data: mailto })
      }
    }

    if (HEX_COLOR_REGEX.test(trimmed) || RGB_COLOR_REGEX.test(trimmed)) {
      actions.push({ id: 'color-preview', label: 'Color', data: trimmed })
    }

    return actions
  },

  async execute(action: SmartAction) {
    switch (action.id) {
      case 'open-link':
      case 'compose-email': {
        if (!action.data) return
        const safe = sanitizeOpenUrl(action.data)
        if (!safe) {
          throw new Error('Blocked unsafe URL')
        }
        // Validated again in Rust before xdg-open (no shell plugin).
        await invoke('open_safe_url', { url: safe })
        break
      }
      default:
        break
    }
  },
}

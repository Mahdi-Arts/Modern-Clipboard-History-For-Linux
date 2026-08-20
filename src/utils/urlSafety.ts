const ALLOWED_PROTOCOLS = new Set(['http:', 'https:', 'mailto:'])

export function sanitizeOpenUrl(raw: string): string | null {
  const trimmed = raw.trim()
  if (!trimmed || trimmed.length > 2048) return null

  try {
    const url = new URL(trimmed)
    if (!ALLOWED_PROTOCOLS.has(url.protocol)) return null
    if (url.username || url.password) return null
    if (url.protocol === 'mailto:') return url.toString()
    if (url.hostname === 'localhost' || url.hostname.endsWith('.localhost')) return null
    if (url.hostname === '127.0.0.1' || url.hostname === '0.0.0.0' || url.hostname === '::1') {
      return null
    }
    if (url.hostname === '169.254.169.254' || url.hostname.endsWith('.internal')) return null
    return url.toString()
  } catch {
    return null
  }
}

export function normalizeHttpUrl(raw: string): string {
  const trimmed = raw.trim()
  if (/^https?:\/\//i.test(trimmed)) return trimmed
  return `https://${trimmed}`
}

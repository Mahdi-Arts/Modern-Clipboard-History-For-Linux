import { describe, it, expect } from 'vitest'
import { sanitizeOpenUrl, normalizeHttpUrl } from './urlSafety'

describe('sanitizeOpenUrl', () => {
  it('accepts valid https URLs', () => {
    expect(sanitizeOpenUrl('https://example.com')).toBe('https://example.com/')
  })

  it('accepts valid http URLs', () => {
    expect(sanitizeOpenUrl('http://example.com')).toBe('http://example.com/')
  })

  it('accepts mailto URLs', () => {
    expect(sanitizeOpenUrl('mailto:user@example.com')).toBe(
      'mailto:user@example.com'
    )
  })

  it('rejects javascript: URLs', () => {
    expect(sanitizeOpenUrl('javascript:alert(1)')).toBeNull()
  })

  it('rejects file: URLs', () => {
    expect(sanitizeOpenUrl('file:///etc/passwd')).toBeNull()
  })

  it('rejects data: URLs', () => {
    expect(sanitizeOpenUrl('data:text/html,<h1>hi</h1>')).toBeNull()
  })

  it('rejects URLs with credentials', () => {
    expect(sanitizeOpenUrl('https://user:pass@example.com')).toBeNull()
  })

  it('rejects localhost URLs', () => {
    expect(sanitizeOpenUrl('http://localhost:3000')).toBeNull()
  })

  it('rejects loopback and metadata IPs', () => {
    expect(sanitizeOpenUrl('http://127.0.0.1/')).toBeNull()
    expect(sanitizeOpenUrl('http://169.254.169.254/latest')).toBeNull()
  })

  it('rejects *.localhost URLs', () => {
    expect(sanitizeOpenUrl('http://api.localhost:8080')).toBeNull()
  })

  it('rejects empty strings', () => {
    expect(sanitizeOpenUrl('')).toBeNull()
  })

  it('rejects whitespace-only strings', () => {
    expect(sanitizeOpenUrl('   ')).toBeNull()
  })

  it('rejects URLs exceeding 2048 characters', () => {
    const longUrl = 'https://example.com/' + 'a'.repeat(2050)
    expect(sanitizeOpenUrl(longUrl)).toBeNull()
  })

  it('trims whitespace', () => {
    expect(sanitizeOpenUrl('  https://example.com  ')).toBe(
      'https://example.com/'
    )
  })
})

describe('normalizeHttpUrl', () => {
  it('returns https URLs unchanged', () => {
    expect(normalizeHttpUrl('https://example.com')).toBe(
      'https://example.com'
    )
  })

  it('returns http URLs unchanged', () => {
    expect(normalizeHttpUrl('http://example.com')).toBe('http://example.com')
  })

  it('prepends https:// to bare domains', () => {
    expect(normalizeHttpUrl('example.com')).toBe('https://example.com')
  })

  it('trims whitespace before normalizing', () => {
    expect(normalizeHttpUrl('  example.com  ')).toBe('https://example.com')
  })
})

import { describe, expect, it } from 'vitest'
import { filterHistory, isSafeRegexPattern } from './historySearch'
import type { ClipboardItem } from '../types/clipboard'

function textItem(id: string, text: string): ClipboardItem {
  return {
    id,
    content: { type: 'Text', data: text },
    timestamp: new Date().toISOString(),
    pinned: false,
    preview: text,
  }
}

describe('historySearch', () => {
  const items = [textItem('1', 'Hello World'), textItem('2', 'Super+V shortcut')]

  it('returns all items when query is empty', () => {
    expect(filterHistory(items, '', false)).toHaveLength(2)
  })

  it('filters with case-insensitive substring', () => {
    expect(filterHistory(items, 'hello', false).map((i) => i.id)).toEqual(['1'])
  })

  it('filters with a simple regex', () => {
    expect(filterHistory(items, 'super\\+v', true).map((i) => i.id)).toEqual(['2'])
  })

  it('rejects nested quantifiers that can ReDoS', () => {
    expect(isSafeRegexPattern('(a+)+')).toBe(false)
    expect(filterHistory(items, '(a+)+', true)).toEqual([])
  })

  it('rejects overly long regex patterns', () => {
    expect(isSafeRegexPattern('a'.repeat(200))).toBe(false)
  })
})

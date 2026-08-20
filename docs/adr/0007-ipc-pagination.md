# ADR-0007: Bounded history reads over IPC (`get_history_page`)

- **Status:** Accepted (v2.3.0)
- **Date:** 2026-08-21

## Context / زمینه

`get_history` ships the entire history to the webview in one IPC payload.
`ClipboardItem::for_ipc` already caps text at 2048 chars and drops HTML, but
with the 2000-item cap the single payload can still reach several megabytes
— paid on every popup open, multiplied by `history-sync` broadcasts.

`get_history` کل تاریخچه را در یک بار IPC می‌فرستد. `for_ipc` متن را به
۲۰۴۸ کاراکتر محدود می‌کند، اما با سقف ۲۰۰۰ آیتم، هر بار چند مگابایت
هزینه می‌شود — در هر باز شدن پنجره و هر رویداد `history-sync`.

## Decision / تصمیم

1. New command `get_history_page(limit, offset)` returns a `HistoryPage`
   (`items`, `total`, `limit`, `offset`).
2. The backend clamps `limit` to `1..=MAX_PAGE_SIZE (200)` and `offset` to
   the collection length — the webview cannot request unbounded payloads.
   بک‌اند مقدار `limit` را به `1..=200` و `offset` را به طول مجموعه
   محدود می‌کند؛ وب‌ویو نمی‌تواند بار نامحدود بخواهد.
3. Items are `for_ipc()` projections, identical to the full read.
4. `useClipboardHistory({ pageSize })` opts into windowed loading
   (`loadMore()` merges pages by id); the default stays a full read, and
   push events (`clipboard-changed`, `history-sync`) keep their exact
   semantics. `src/utils/pagination.ts` mirrors the clamps on the UI side
   and is unit-tested.
5. Full `get_history` remains for small histories and tests — it is cheap
   under ~500 items and keeps old frontends working.

## Consequences / پیامدها

- Large histories render with a bounded, predictable IPC budget
  (200 × ≤2 KB ≈ 400 KB worst case per window).
- UI lists stay virtualized with `react-window`; paging composes cleanly
  with it.
- If push events ever paginate too, this ADR should be revisited.

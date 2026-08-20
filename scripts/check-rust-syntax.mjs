#!/usr/bin/env node
/**
 * Syntax-level validation for all Rust sources using tree-sitter.
 *
 * This is a CI-friendly lightweight gate that catches syntax errors in the
 * Rust codebase without a full toolchain (useful in sandboxes and as a fast
 * pre-`cargo check` step). It does NOT replace `cargo check` / `clippy`,
 * which remain the authoritative gates in CI.
 */
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'
import Parser from 'tree-sitter'
import Rust from 'tree-sitter-rust'

const root = process.argv[2] ?? 'src-tauri/src'
const parser = new Parser()
parser.setLanguage(Rust)

const files = []
function walk(dir) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    if (statSync(full).isDirectory()) {
      walk(full)
    } else if (full.endsWith('.rs')) {
      files.push(full)
    }
  }
}
walk(root)

let failures = 0
for (const file of files) {
  const source = readFileSync(file, 'utf8')
  const tree = parser.parse(source)
  const errors = []
  function visit(node) {
    if (node.type === 'ERROR' || node.isMissing) {
      const pos = node.startPosition
      errors.push(`  ${node.type} at ${pos.row + 1}:${pos.column + 1} (${node.text.slice(0, 60).replace(/\n/g, '\\n')})`)
    }
    for (const child of node.children) visit(child)
  }
  visit(tree.rootNode)
  if (errors.length > 0) {
    failures += 1
    console.error(`❌ ${file}`)
    console.error(errors.slice(0, 8).join('\n'))
  }
}

if (failures > 0) {
  console.error(`\n${failures} file(s) with syntax errors`)
  process.exit(1)
}
console.log(`✅ Rust syntax OK (${files.length} files parsed)`)

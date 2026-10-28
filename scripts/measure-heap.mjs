#!/usr/bin/env node
/**
 * Where does the heap of a screen actually go?
 *
 * The heap reading says a screen costs ten megabytes; it does not say what
 * those megabytes are. This takes a heap snapshot after a collection and
 * aggregates the retained bytes by what they are: arrays, objects, strings,
 * and — the interesting one — the DOM nodes a chart's data is mirrored on.
 *
 *   node scripts/measure-heap.mjs /physiology
 */
import { chromium } from '/home/sumit/node_modules/playwright/index.mjs'
import { writeFileSync } from 'node:fs'

const BASE = process.env.STRIDE_BASE || 'http://127.0.0.1:8000'
const ROUTE = process.argv[2] || '/physiology'

const browser = await chromium.launch()
const context = await browser.newContext({ viewport: { width: 390, height: 844 } })
const page = await context.newPage()
await page.goto(BASE + ROUTE, { waitUntil: 'networkidle', timeout: 45_000 })
await page.waitForTimeout(4000)

const cdp = await context.newCDPSession(page)
await cdp.send('HeapProfiler.enable')
await cdp.send('HeapProfiler.collectGarbage')

const chunks = []
cdp.on('HeapProfiler.addHeapSnapshotChunk', (event) => chunks.push(event.chunk))
await cdp.send('HeapProfiler.takeHeapSnapshot', { reportProgress: false, treatGlobalObjectsAsRoots: true })
const snapshot = JSON.parse(chunks.join(''))

// The snapshot is a flat node table: every node has a name, a type and a
// self size, and the edges say who points at whom.
const meta = snapshot.snapshot.meta
const nodeFields = meta.node_fields
const nodeFieldCount = nodeFields.length
const nameIndex = nodeFields.indexOf('name')
const typeIndex = nodeFields.indexOf('type')
const sizeIndex = nodeFields.indexOf('self_size')
const types = meta.node_types[typeIndex]
const nodes = snapshot.nodes
const strings = snapshot.strings

const byType = new Map()
const byName = new Map()
for (let i = 0; i < nodes.length; i += nodeFieldCount) {
  const type = types[nodes[i + typeIndex]]
  const name = strings[nodes[i + nameIndex]]
  const size = nodes[i + sizeIndex]
  byType.set(type, (byType.get(type) || 0) + size)
  const key = `${type}:${name}`
  byName.set(key, (byName.get(key) || 0) + size)
}

const mb = (n) => (n / 1024 / 1024).toFixed(2)
const top = [...byType.entries()].sort((a, b) => b[1] - a[1])
console.log('')
console.log(`  heap snapshot — ${ROUTE}`)
console.log('  ' + '-'.repeat(64))
let total = 0
for (const [type, size] of top) total += size
for (const [type, size] of top) {
  console.log(`  ${type.padEnd(12)} ${mb(size).padStart(7)} MB  ${((size / total) * 100).toFixed(1)}%`)
}
console.log('  ' + '-'.repeat(64))
console.log(`  ${'total'.padEnd(12)} ${mb(total).padStart(7)} MB`)
console.log('')
console.log('  largest single types:')
for (const [key, size] of [...byName.entries()].sort((a, b) => b[1] - a[1]).slice(0, 15)) {
  console.log(`  ${mb(size).padStart(7)} MB  ${key}`)
}
console.log('')

writeFileSync('/tmp/heap-snapshot.json', JSON.stringify(snapshot))
await browser.close()

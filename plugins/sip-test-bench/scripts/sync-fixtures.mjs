#!/usr/bin/env node
/**
 * sync-fixtures.mjs — runs in `prebuild` (and `predev`).
 *
 * Copies the prepared SMRT canonical fixture set from the repo's data directory
 * into this plugin's public/fixtures/ so Vite ships it verbatim into dist/.
 * The plugin then fetches it at runtime relative to its own document:
 *   new URL('fixtures/index.json', document.baseURI)
 *
 * Dependency order is load-bearing: SIP resolves assets to asset types and
 * locations by external id, parts to nothing, and work orders to assets and
 * parts — so the objects must be imported in this exact order.
 *
 * If the data directory is absent (fresh clone without the fixture, CI, or a
 * plugin checked out on its own) this exits 0 with a warning: the build must
 * still succeed, and the UI reports the missing fixture honestly at runtime.
 */
import { existsSync, mkdirSync, copyFileSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pluginRoot = resolve(here, '..');

// plugins/sip-test-bench -> plugins -> repo root
const repoRoot = resolve(pluginRoot, '..', '..');
const sourceDir = join(repoRoot, 'data', 'smrt-maintenance-logs', 'fixture', 'canonical');
const outDir = join(pluginRoot, 'public', 'fixtures');

const SOURCE_SYSTEM = 'ASNIJKDNI/SMRT-Maintenance-Logs';

/**
 * The load plan: dependency order, object type, and file for each dataset.
 * `file` names match the fixture directory exactly.
 */
const PLAN = [
  { order: 1, object_type: 'location', file: '01-locations.json' },
  { order: 2, object_type: 'asset_type', file: '02-asset-types.json' },
  { order: 3, object_type: 'asset', file: '03-assets.json' },
  { order: 4, object_type: 'part', file: '04-parts.json' },
  { order: 5, object_type: 'work_order', file: '05-work-orders.json' },
];

function warn(msg) {
  console.warn(`[sync-fixtures] WARNING: ${msg}`);
}

if (!existsSync(sourceDir)) {
  warn(`fixture source directory not found: ${sourceDir}`);
  warn('skipping fixture sync — the build continues, and the Test Bench UI will report the missing load plan at runtime.');
  process.exit(0);
}

mkdirSync(outDir, { recursive: true });

const datasets = [];
let totalRecords = 0;

for (const entry of PLAN) {
  const src = join(sourceDir, entry.file);
  if (!existsSync(src)) {
    warn(`missing fixture file: ${src} — dataset '${entry.object_type}' will be marked unavailable.`);
    datasets.push({
      object_type: entry.object_type,
      file: entry.file,
      count: 0,
      order: entry.order,
      available: false,
      note: 'fixture file not found at sync time',
    });
    continue;
  }

  let parsed;
  try {
    parsed = JSON.parse(readFileSync(src, 'utf8'));
  } catch (err) {
    warn(`could not parse ${entry.file}: ${err.message} — marked unavailable.`);
    datasets.push({
      object_type: entry.object_type,
      file: entry.file,
      count: 0,
      order: entry.order,
      available: false,
      note: `parse error: ${err.message}`,
    });
    continue;
  }

  const records = Array.isArray(parsed?.records) ? parsed.records : [];
  const count = records.length;
  totalRecords += count;

  copyFileSync(src, join(outDir, entry.file));

  datasets.push({
    object_type: entry.object_type,
    // The fixture's own declared object type wins when present; the plan entry
    // is the contract the loader depends on.
    declared_object_type: parsed?.object_type ?? null,
    file: entry.file,
    count,
    order: entry.order,
    available: true,
    source_system: parsed?.source_system ?? null,
    canonical_version: parsed?.canonical_version ?? null,
  });

  console.log(`[sync-fixtures] ${entry.file.padEnd(22)} ${String(count).padStart(6)} records  ->  ${entry.object_type}`);
}

const manifest = {
  source: SOURCE_SYSTEM,
  canonical_version: '1.0',
  generated_at: new Date().toISOString(),
  source_dir: sourceDir,
  total_records: totalRecords,
  datasets,
};

writeFileSync(join(outDir, 'index.json'), JSON.stringify(manifest, null, 2) + '\n', 'utf8');

console.log(`[sync-fixtures] wrote index.json (${datasets.length} datasets, ${totalRecords} records) -> ${outDir}`);

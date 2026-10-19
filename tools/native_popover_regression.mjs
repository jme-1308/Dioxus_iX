// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

// Run with node tools/native_popover_regression.mjs. Execute the native binding
// itself; Dioxus component behavior is verified by the Rust and rendered probes.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/native_popover.rs', import.meta.url), 'utf8');
const script = source.split('const PRESENT: &str = r#"')[1].split('"#;')[0];
const execute = new (async () => {}).constructor('dioxus', 'document', script);

class Panel {
  isConnected = true;
  inert = false;
  open = false;
  marker = 'current';
  calls = [];
  fail = false;
  getAttribute() { return this.marker; }
  matches() { return this.open; }
  showPopover(options) {
    this.calls.push({ shown: true, inert: this.inert, options });
    if (this.fail) throw new Error('native rejection');
    this.open = true;
  }
  hidePopover() {
    this.calls.push({ shown: false });
    this.open = false;
  }
}

const input = { id: 'combobox' };
const run = (panel, shown, guard = null, invoker = null) => execute(
  { recv: async () => ['panel', shown, guard, invoker] },
  { getElementById: id => id === 'panel' ? panel : id === input.id ? input : null },
);

// Select keeps its explicit native source. Repeated presentation is idempotent.
{
  const panel = new Panel();
  assert.equal(await run(panel, true, null, input.id), null);
  assert.deepEqual(panel.calls, [{ shown: true, inert: false, options: { source: input } }]);
  assert.equal(await run(panel, true, null, input.id), null);
  assert.equal(panel.calls.length, 1);
}

// Filter, Tooltip and overlay surfaces cannot steal focus during promotion.
// Inert is restored even when the native operation throws.
{
  const panel = new Panel();
  assert.equal(await run(panel, true), null);
  assert.equal(panel.calls[0].inert, true);
  assert.equal(panel.inert, false);
  assert.equal(await run(panel, false), null);
  assert.deepEqual(panel.calls.at(-1), { shown: false });
  panel.inert = true;
  panel.fail = true;
  assert.match(await run(panel, true), /native rejection/);
  assert.equal(panel.inert, true);
}

// A stale rendered intent cannot open or close the current surface.
{
  const panel = new Panel();
  assert.equal(await run(panel, true, ['data-current', 'old']), null);
  assert.equal(panel.calls.length, 0);
  panel.open = true;
  assert.equal(await run(panel, false, ['data-current', 'old']), null);
  assert.equal(panel.open, true);
  panel.isConnected = false;
  assert.equal(await run(panel, true, ['data-current', 'current']), null);
}

assert.match(await run(null, true), /not mounted/);
assert.match(await run(new Panel(), true, null, 'removed-invoker'), /Invoker is not mounted/);
const unsupported = new Panel();
unsupported.showPopover = undefined;
assert.match(await run(unsupported, true), /unavailable/);
console.log('PASS: native invoker, inert promotion, guarded visibility, idempotence and explicit errors');

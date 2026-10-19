// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

// Run with node tools/native_popover_regression.mjs. Execute the native binding
// itself; Dioxus component behavior is verified by the Rust and rendered probes.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/native_surface.rs', import.meta.url), 'utf8');
const script = source.split('const PRESENT: &str = r#"')[1].split('"#;')[0];
const execute = new (async () => {}).constructor('dioxus', 'document', script);

class Panel {
  isConnected = true;
  inert = false;
  open = false;
  popoverOpen = false;
  marker = 'current';
  calls = [];
  fail = false;
  prevent = false;
  getAttribute() { return this.marker; }
  matches() { return this.popoverOpen; }
  showPopover(options) {
    this.calls.push({ shown: true, inert: this.inert, options });
    if (this.fail) throw new Error('native rejection');
    if (!this.prevent) this.popoverOpen = true;
  }
  showModal() {
    this.calls.push({ modal: true, inert: this.inert });
    if (this.fail) throw new Error('native rejection');
    if (!this.prevent) this.open = true;
  }
  hidePopover() {
    this.calls.push({ shown: false });
    this.popoverOpen = false;
  }
}

const input = { id: 'combobox' };
const run = (panel, shown, guard = null, invoker = null, mode = 'popover') => execute(
  { recv: async () => ['panel', shown, guard, invoker, mode] },
  { getElementById: id => id === 'panel' ? panel : id === input.id ? input : null },
);

// Select keeps its explicit native source. Repeated presentation is idempotent.
{
  const panel = new Panel();
  assert.equal(await run(panel, true, null, input.id), true);
  assert.deepEqual(panel.calls, [{ shown: true, inert: false, options: { source: input } }]);
  assert.equal(await run(panel, true, null, input.id), true);
  assert.equal(panel.calls.length, 1);
}

// Filter, Tooltip and overlay surfaces cannot steal focus during promotion.
// Inert is restored even when the native operation throws.
{
  const panel = new Panel();
  assert.equal(await run(panel, true), true);
  assert.equal(panel.calls[0].inert, true);
  assert.equal(panel.inert, false);
  assert.equal(await run(panel, false), true);
  assert.deepEqual(panel.calls.at(-1), { shown: false });
  panel.inert = true;
  panel.fail = true;
  assert.match(await run(panel, true), /native rejection/);
  assert.equal(panel.inert, true);
}

// A stale rendered intent cannot open or close the current surface.
{
  const panel = new Panel();
  assert.equal(await run(panel, true, ['data-current', 'old']), false);
  assert.equal(panel.calls.length, 0);
  panel.popoverOpen = true;
  assert.equal(await run(panel, false, ['data-current', 'old']), false);
  assert.equal(panel.popoverOpen, true);
  panel.isConnected = false;
  assert.equal(await run(panel, true, ['data-current', 'current']), false);
  assert.equal(panel.calls.length, 0);
  panel.isConnected = true;
  assert.equal(await run(panel, false, ['data-current', 'current']), true);
  assert.equal(panel.popoverOpen, false);
  assert.deepEqual(panel.calls, [{ shown: false }]);
}

// A native opening canceled without throwing must not be reported as applied.
{
  const panel = new Panel();
  panel.prevent = true;
  assert.equal(await run(panel, true), null);
  assert.equal(panel.popoverOpen, false);
  assert.equal(panel.inert, false);
}

// Modal presentation permits native autofocus instead of promoting inertly.
// Native dialog and Popover visibility are distinct; repeat opens are idempotent.
for (const mode of ['modal', 'nonblocking-modal']) {
  const panel = new Panel();
  const guard = ['data-current', 'current'];
  assert.equal(await run(panel, true, ['data-current', 'old'], null, mode), false);
  assert.deepEqual(panel.calls, []);
  panel.isConnected = false;
  assert.equal(await run(panel, true, guard, null, mode), false);
  panel.isConnected = true;
  assert.equal(await run(panel, true, guard, null, mode), true);
  assert.deepEqual(panel.calls, mode === 'modal'
    ? [{ modal: true, inert: false }]
    : [{ shown: true, inert: false, options: undefined }]);
  assert.equal(panel.open, mode === 'modal');
  assert.equal(panel.popoverOpen, mode === 'nonblocking-modal');
  assert.equal(await run(panel, true, guard, null, mode), true);
  assert.equal(panel.calls.length, 1);

  const prevented = new Panel();
  prevented.prevent = true;
  assert.equal(await run(prevented, true, guard, null, mode), null);
  assert.equal(prevented.inert, false);
  const rejected = new Panel();
  rejected.fail = true;
  assert.match(await run(rejected, true, guard, null, mode), /native rejection/);
  assert.equal(rejected.inert, false);
  assert.match(await run(null, true, null, null, mode), /not mounted/);
}

assert.match(await run(null, true), /not mounted/);
assert.match(await run(new Panel(), true, null, 'removed-invoker'), /Invoker is not mounted/);
const unsupported = new Panel();
unsupported.showPopover = undefined;
assert.match(await run(unsupported, true), /unavailable/);
const unsupportedModal = new Panel();
unsupportedModal.showModal = undefined;
assert.match(await run(unsupportedModal, true, null, null, 'modal'), /unavailable/);
console.log('PASS: native invoker, inert promotion, Modal autofocus eligibility, guarded visibility, idempotence and explicit errors');

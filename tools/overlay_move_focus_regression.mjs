// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

// Run with node tools/overlay_move_focus_regression.mjs. Exercise the actual
// native observation code; mounted focus execution remains covered in Rust.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const raw = readFileSync(new URL('../src/overlay_registry_dom.rs', import.meta.url), 'utf8');
const script = raw.split('r#"')[1].split('"#;')[0];
const section = (start, end) => script.slice(script.indexOf(`function ${start}(`), script.indexOf(`function ${end}(`));

class Element {
  constructor(id, key = null) {
    this.id = id;
    this.key = key;
    this.isConnected = true;
  }
  closest() { return this.key ? this : null; }
  getAttribute() { return this.key; }
  matches() { return false; }
  contains(node) { return node === this; }
}

function fixture() {
  const first = new Element('inspect', 'inspect-key');
  const moving = new Element('moving', 'moving-key');
  const body = new Element('');
  const document = {
    body, documentElement: new Element(''), activeElement: first,
    hasFocus: () => true,
    getElementById: id => [first, moving].find(element => element.id === id),
  };
  const reports = [];
  const observe = new Function('HTMLElement', 'Element', 'document', 'send', `
    let alive=true,sequence=0,lastSignature='',documentEvents=true,lastFocus=null,movedSequence=0;
    const observer={takeRecords:()=>[]};
    const nativeId=element=>element?.id||'';
    const nearestHost=()=>null;
    const deepActive=()=>document.activeElement;
    const collectFacts=()=>({sequence:++sequence,active_id:nativeId(deepActive()),active_default:deepActive()===document.body});
    ${section('registeredFocus', 'collectFacts')}
    ${section('reportObserve', 'sendDocumentEvent')}
    return reportObserve;
  `)(Element, Element, document, report => reports.push(report));
  return { first, moving, document, reports, observe };
}

// Inspect blurs, then Move receives focus. The queued Inspect focusout arrives
// after that focusin; Enter then moves the same connected control in the DOM.
{
  const { moving, document, reports, observe } = fixture();
  observe([], true);
  document.activeElement = moving;
  observe([], true);
  observe([], false, 0);
  assert.equal(reports.at(-1).active_id, moving.id);
  document.activeElement = document.body;
  observe([{ type: 'childList', removedNodes: [moving], addedNodes: [moving] }], false, 0);
  assert.deepEqual(reports.at(-1).moved_focus, { key: 'moving-key', focus_id: moving.id });
  assert.equal(reports.at(-1).focus_changed, false);
}

// An actual explicit blur still clears the remembered registration. A later
// move of that now-unfocused item must not request a repair.
{
  const { first, document, reports, observe } = fixture();
  observe([], true);
  document.activeElement = document.body;
  observe([], false, 0);
  assert.equal(reports.at(-1).focus_changed, true);
  observe([{ type: 'childList', removedNodes: [first], addedNodes: [first] }]);
  assert.equal(reports.at(-1).moved_focus, undefined);
}

console.log('PASS: focus transition followed by keyed move; explicit blur does not revive focus');

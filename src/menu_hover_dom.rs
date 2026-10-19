//! Native pointer facts for the private Rust Menu category hover policy.
//! Rust owns modality, timers and independently authorized close-return jobs.
//! These helpers do not route observations through keyboard acknowledgements.
//! Rust designates the current popup host with an explicit marker; the
//! arbitrary child tree keeps its permanent DOM parent.

// Reuse the same raw observation in the long-lived listener and the separate
// timer-capture eval without depending on another eval's lexical bindings.
macro_rules! category_active_facts {
    () => {
        r#"
function menuCategoryActiveFacts() {
  const categories = [];
  for (let node = document.activeElement; node;) {
    if (node instanceof Element && node.hasAttribute('data-menu-category-root')) {
      const id = Number(node.getAttribute('data-menu-category-root'));
      if (Number.isSafeInteger(id) && id > 0 && !categories.includes(id)) categories.push(id);
    }
    node = node.parentNode || (node instanceof ShadowRoot ? node.host : null);
  }
  return categories;
}
"#
    };
}

pub(crate) const MENU_HOVER_NATIVE_FACTS: &str = concat!(
    category_active_facts!(),
    r#"
function installMenuCategoryHover(root, sendHover, directory) {
  const owner = root.id;
  const entries = new Map();
  let alive = true;
  let sequence = 0;

  function currentRoot() {
    return alive && root.isConnected && root.id === owner &&
      root.matches('[data-menu-root]') && directory.sameNodeById(root, owner);
  }

  function currentPanel(category) {
    return category.querySelector(':scope > [data-menu-category-popup]');
  }

  function currentEntry(entry) {
    return currentRoot() && entries.get(entry.category) === entry &&
      entry.category.isConnected && entry.category.dataset.menuOwner === owner &&
      entry.category.getAttribute('data-menu-category-root') === entry.identity &&
      currentPanel(entry.category) === entry.panel;
  }

  function report(entry, surface, kind, event) {
    const target = surface === 'host' ? entry.category : entry.panel;
    // Direct listeners receive only that surface's own enter/leave. In
    // particular, a synthetic bubbling descendant event is not a host entry.
    if (!currentEntry(entry) || event.currentTarget !== target || event.target !== target) return;
    if (surface === 'flyout' && !entry.panel?.isConnected) return;
    const related = event.relatedTarget;
    const relatedIsNode = related instanceof Node;
    sendHover({
      owner,
      category: Number(entry.identity),
      surface,
      kind,
      pointer_type: event.pointerType,
      related_is_node: relatedIsNode,
      related_host: relatedIsNode && entry.category.contains(related),
      related_host_shadow: relatedIsNode && !!entry.category.shadowRoot?.contains(related),
      related_flyout: relatedIsNode && !!entry.panel &&
        (entry.panel === related || entry.panel.contains(related)),
      related_flyout_shadow: relatedIsNode && !!entry.panel?.shadowRoot?.contains(related),
      active_categories: menuCategoryActiveFacts(),
      sequence: ++sequence
    });
  }

  function detach(category) {
    const entry = entries.get(category);
    if (!entry) return;
    entry.listeners.abort();
    entries.delete(category);
  }

  function sync() {
    if (!currentRoot()) {
      for (const category of entries.keys()) detach(category);
      return;
    }
    const current = new Set();
    for (const category of directory.categories(owner)) {
      current.add(category);
      const identity = category.getAttribute('data-menu-category-root');
      const panel = currentPanel(category);
      const previous = entries.get(category);
      if (previous?.identity === identity && previous.panel === panel) continue;
      detach(category);
      const listeners = new AbortController();
      const entry = { category, identity, panel, listeners };
      entries.set(category, entry);
      const options = { passive: true, signal: listeners.signal };
      category.addEventListener('pointerenter', event => report(entry, 'host', 'enter', event), options);
      category.addEventListener('pointerleave', event => report(entry, 'host', 'leave', event), options);
      if (panel) panel.addEventListener('pointerenter', event => report(entry, 'flyout', 'enter', event), options);
    }
    for (const category of entries.keys()) if (!current.has(category)) detach(category);
  }

  function dispose() {
    alive = false;
    for (const category of entries.keys()) detach(category);
    entries.clear();
  }

  sync();
  return { sync, dispose };
}
"#
);

/// Root-lifetime raw document input journal. Rust owns key membership and mode;
/// the synchronous native input flush precedes each Menu's readiness marker.
#[cfg(any(feature = "web", feature = "desktop", test))]
pub(crate) const MENU_DOCUMENT_INPUT: &str = concat!(
    crate::menu_scope_dom::menu_scope_directory!(),
    r#"
const owner = __DOCUMENT_OWNER__;
const registryKey = Symbol.for('dioxus-ix-document-input');
const registry = document[registryKey] ?? (document[registryKey] = new Map());
if (!(registry instanceof Map)) throw new Error('Invalid Menu document input registry');
if (registry.has(owner)) throw new Error('Duplicate Menu document input owner');
const listeners = new AbortController();
let alive = true;
let sequence = 0;
let pending = [];
let syncing = false;
let resync = false;
const directory = menuScopeDirectory();
const directoryLease = directory.acquire(sync);

function batch() {
  return { owner, entries: pending.map(entry => ({ ...entry })) };
}

function sync() {
  if (!alive) return;
  if (syncing) {
    resync = true;
    return;
  }
  syncing = true;
  try {
    do {
      resync = false;
      // One immutable snapshot per flush keeps all connected input recipients
      // coherent if an input handler synchronously dispatches another fact.
      const value = JSON.stringify(batch());
      for (const input of directory.transports('data-menu-document-owner', owner)) {
        if (!input.isConnected || input.dataset.menuDocumentOwner !== owner) continue;
        input.value = value;
        input.dispatchEvent(new Event('input', { bubbles: true, cancelable: false }));
        if (input.isConnected && input.dataset.menuDocumentOwner === owner) {
          input.dataset.menuDocumentReady = 'true';
        }
      }
    } while (alive && resync);
  } finally {
    syncing = false;
  }
}

const entry = { sync };
registry.set(owner, entry);
try {
  for (const kind of ['keydown', 'mousedown', 'touchstart']) {
    document.addEventListener(kind, event => {
      directory.discoverFromEvent(event);
      pending.push({ kind, key: event.key ?? '', sequence: ++sequence });
      sync();
      // Keep all unacknowledged facts in both delivery paths. A new Menu can
      // therefore consume an earlier gap input before a later key or effect.
      dioxus.send(batch());
    }, { passive: true, signal: listeners.signal });
  }
  sync();
  for (;;) {
    const message = await dioxus.recv();
    if (message === 'finish') break;
    if (Number.isSafeInteger(message?.ack) && message.ack >= 0) {
      pending = pending.filter(entry => entry.sequence > message.ack);
    }
  }
} finally {
  alive = false;
  listeners.abort();
  directory.release(directoryLease);
  if (registry.get(owner) === entry) registry.delete(owner);
}
"#
);

/// Fresh native focus capture before Rust mutates controller-dismissal state.
#[cfg(any(feature = "web", feature = "desktop"))]
pub(crate) const CATEGORY_CLOSE_CAPTURE: &str = concat!(
    crate::menu_scope_dom::menu_scope_directory!(),
    category_active_facts!(),
    r#"
const [owner, id] = await dioxus.recv();
const directory = menuScopeDirectory(), lease = directory.acquire();
try {
const active_categories = menuCategoryActiveFacts();
const root = directory.elementById(owner);
const category = directory.categories(owner, id)[0];
const connected = !!root?.isConnected && root.matches('[data-menu-root]') &&
  !!category?.isConnected && category.dataset.menuOwner === owner;
return { connected, active_categories };
} finally { directory.release(lease); }
"#
);

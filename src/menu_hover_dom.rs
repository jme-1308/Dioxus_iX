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
function installMenuCategoryHover(root, sendHover) {
  const owner = root.id;
  const entries = new Map();
  const documentListeners = new AbortController();
  let alive = true;
  let sequence = 0;

  function currentRoot() {
    return alive && root.isConnected && root.id === owner &&
      root.matches('[data-menu-root]') && document.getElementById(owner) === root;
  }

  function currentPanel(category) {
    return category.querySelector(':scope > [data-menu-category-popup]');
  }

  function currentEntry(entry) {
    return currentRoot() && entries.get(entry.category) === entry &&
      entry.category.isConnected && entry.category.closest('[data-menu-root]') === root &&
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
    for (const category of root.querySelectorAll('[data-menu-category-root]')) {
      if (!category.isConnected || category.closest('[data-menu-root]') !== root) continue;
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
    documentListeners.abort();
    for (const category of entries.keys()) detach(category);
    entries.clear();
  }

  for (const kind of ['keydown', 'mousedown', 'touchstart']) {
    document.addEventListener(kind, event => {
      if (!currentRoot()) return;
      sendHover({ owner, category: 0, surface: 'document', kind,
        key: event.key ?? '', sequence: ++sequence });
    }, { passive: true, signal: documentListeners.signal });
  }

  sync();
  return { sync, dispose };
}
"#
);

/// Fresh native focus capture before Rust mutates controller-dismissal state.
#[cfg(any(feature = "web", feature = "desktop"))]
pub(crate) const CATEGORY_CLOSE_CAPTURE: &str = concat!(
    category_active_facts!(),
    r#"
const [owner, id] = await dioxus.recv();
const active_categories = menuCategoryActiveFacts();
const root = document.getElementById(owner);
const category = root?.querySelector('[data-menu-category-root="' + id + '"]');
const connected = !!root?.isConnected && root.matches('[data-menu-root]') &&
  !!category?.isConnected && category.closest('[data-menu-root]') === root;
return { connected, active_categories };
"#
);

/// Execute one independently authorized return; no late modality/focus guard.
/// Rust's job authorization and connected-node checks are lifetime adapters,
/// not guards claimed to exist in the source's queued focus callback.
#[cfg(any(feature = "web", feature = "desktop"))]
pub(crate) const CATEGORY_DISMISS_RETURN: &str = r#"
const [owner, id, job] = await dioxus.recv();
const root = document.getElementById(owner);
const selector = '[data-menu-category-root="' + id + '"]';
const category = root?.querySelector(selector);
const targetId = 'ix-menu-category-parent-' + id;
const target = document.getElementById(targetId);
await new Promise(requestAnimationFrame);
await new Promise(requestAnimationFrame);
if (!root?.isConnected || document.getElementById(owner) !== root ||
    !root.matches('[data-menu-root]') || !category?.isConnected ||
    root.querySelector(selector) !== category || category.closest('[data-menu-root]') !== root ||
    !(target instanceof HTMLElement) || !target.isConnected || document.getElementById(targetId) !== target ||
    target.closest('[data-menu-category-root]') !== category || target.closest('[data-menu-root]') !== root) return false;
let jobs;
try { jobs = JSON.parse(category.getAttribute('data-menu-category-return-jobs') ?? '[]'); }
catch { return false; }
if (typeof job !== 'string' || !Array.isArray(jobs) || !jobs.includes(job)) return false;
target.focus();
return document.activeElement === target;
"#;

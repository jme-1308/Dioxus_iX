//! Shared private native-DOM open-scope directory for Menu and MenuCategory.
//! Maintains reachable `Document` and open `ShadowRoot` scopes on `document`
//! without overlay-runtime policy coupling or per-query full-tree scans.

macro_rules! menu_scope_directory {
    () => {
        r#"
function menuScopeDirectory() {
  const slot = Symbol.for('dioxus-ix.menu-scope-directory.v1');
  let directory = document[slot];
  if (directory) return directory;
  const scopes = new Set([document]);
  const subscribers = new Map();
  let active = false;
  let initialized = false;
  let nextHandle = 0;

  function notify(changes = []) {
    for (const callback of Array.from(subscribers.values())) {
      if (typeof callback === 'function') callback(changes);
    }
  }

  function onSlotChange(event) {
    discoverFromEvent(event);
    notify();
  }

  function observeScope(scope) {
    if (!active) return;
    const target = scope === document ? document.documentElement : scope;
    if (!target) return;
    observer.observe(target, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ['slot', 'name']
    });
    if (scope !== document) {
      scope.addEventListener('slotchange', onSlotChange);
    }
  }

  function addScope(scope) {
    if (!(scope instanceof Document || scope instanceof ShadowRoot)) return false;
    if (scope !== document && (scope.mode !== 'open' || !scope.host.isConnected)) return false;
    if (scopes.has(scope)) return false;
    scopes.add(scope);
    if (active) observeScope(scope);
    discoverSubtree(scope);
    return true;
  }

  function discoverSubtree(node) {
    if (!(node instanceof Element || node instanceof Document || node instanceof ShadowRoot)) return false;
    let added = false;
    const root = node.getRootNode();
    if (root instanceof ShadowRoot && root.host.isConnected) {
      if (addScope(root)) added = true;
    }
    if (node instanceof Element && node.shadowRoot && node.isConnected) {
      if (addScope(node.shadowRoot)) added = true;
    }
    const queryRoot = node === document ? document.documentElement : node;
    if (queryRoot) {
      for (const element of queryRoot.querySelectorAll('*')) {
        if (element.shadowRoot && element.isConnected) {
          if (addScope(element.shadowRoot)) added = true;
        }
      }
    }
    return added;
  }

  function pruneDetachedScopes() {
    let pruned = false;
    for (const scope of Array.from(scopes)) {
      if (scope !== document && !scope.host.isConnected) {
        scope.removeEventListener('slotchange', onSlotChange);
        scopes.delete(scope);
        pruned = true;
      }
    }
    if (pruned && active) {
      observer.disconnect();
      for (const scope of scopes) observeScope(scope);
    }
    return pruned;
  }

  function discoverFromEvent(event) {
    if (!event || typeof event.composedPath !== 'function') return false;
    pruneDetachedScopes();
    let added = false;
    for (const target of event.composedPath()) {
      if (target instanceof ShadowRoot && target.host.isConnected) {
        if (addScope(target)) added = true;
      } else if (target instanceof Node) {
        const root = target.getRootNode();
        if (root instanceof ShadowRoot && root.host.isConnected) {
          if (addScope(root)) added = true;
        }
        if (target instanceof Element && target.shadowRoot && target.isConnected) {
          if (addScope(target.shadowRoot)) added = true;
        }
      }
    }
    return added;
  }

  const observer = new MutationObserver(changes => {
    let sawChildList = false;
    for (const change of changes) {
      if (change.type === 'childList') {
        sawChildList = true;
        for (const node of [...change.addedNodes, ...change.removedNodes]) {
          discoverSubtree(node);
        }
      }
    }
    if (sawChildList) pruneDetachedScopes();
    notify(changes);
  });

  function start() {
    if (active) return;
    active = true;
    initialized = true;
    scopes.clear();
    scopes.add(document);
    observeScope(document);
    discoverSubtree(document);
  }

  function stop() {
    if (!active) return;
    active = false;
    initialized = false;
    observer.disconnect();
    for (const scope of Array.from(scopes)) {
      if (scope !== document) scope.removeEventListener('slotchange', onSlotChange);
    }
    scopes.clear();
    scopes.add(document);
  }

  function ensureInitialized() {
    if (!active) throw new Error('Menu scope lookup requires an active lease');
    pruneDetachedScopes();
    if (!initialized) {
      initialized = true;
      discoverSubtree(document);
    }
  }

  function acquire(callback = null) {
    const wasEmpty = subscribers.size === 0;
    const handle = ++nextHandle;
    subscribers.set(handle, callback);
    if (wasEmpty) start();
    else pruneDetachedScopes();
    return handle;
  }

  function release(handle) {
    if (!subscribers.delete(handle)) return false;
    if (subscribers.size === 0) stop();
    return true;
  }

  function subscribe(handle, callback) {
    if (!subscribers.has(handle)) throw new Error('Unknown Menu scope lease');
    subscribers.set(handle, callback);
  }

  function elementById(id, event = null) {
    if (typeof id !== 'string' || !id) return null;
    if (event) discoverFromEvent(event);
    ensureInitialized();
    for (const scope of scopes) {
      if (scope !== document && !scope.host.isConnected) continue;
      const element = scope.getElementById(id);
      if (element instanceof HTMLElement && element.isConnected) return element;
    }
    return null;
  }

  function sameNodeById(node, id, event = null) {
    // A retained owner may move synchronously before observer delivery.
    // Its own root is an exact discovery fact, not a whole-document rescan.
    if (node instanceof HTMLElement && node.isConnected) {
      const root = node.getRootNode();
      if (root instanceof ShadowRoot && !scopes.has(root)) addScope(root);
    }
    return node instanceof HTMLElement &&
      node.isConnected &&
      node.id === id &&
      elementById(id, event) === node;
  }

  function elements(selector, predicate = null, event = null) {
    if (typeof selector !== 'string' || !selector) return [];
    if (event) discoverFromEvent(event);
    ensureInitialized();
    const result = [];
    for (const scope of scopes) {
      if (scope !== document && !scope.host.isConnected) continue;
      for (const element of scope.querySelectorAll(selector)) {
        if (!element.isConnected) continue;
        if (!predicate || predicate(element)) result.push(element);
      }
    }
    return result;
  }

  function transports(attribute, owner, event = null) {
    if (typeof attribute !== 'string' || !attribute || typeof owner !== 'string' || !owner) return [];
    return elements(
      'input[' + attribute + ']',
      node => node instanceof HTMLInputElement && node.getAttribute(attribute) === owner,
      event
    );
  }

  function categories(owner, id = null, event = null) {
    if (typeof owner !== 'string' || !owner) return [];
    const selector = id === null || id === undefined
      ? '[data-menu-category-root]'
      : '[data-menu-category-root="' + id + '"]';
    return elements(
      selector,
      node => node instanceof HTMLElement &&
        node.getAttribute('data-menu-owner') === owner &&
        (id === null || id === undefined || node.getAttribute('data-menu-category-root') === String(id)),
      event
    );
  }

  function overlays(owner, event = null) {
    if (typeof owner !== 'string' || !owner) return [];
    return elements(
      '.ix-menu-overlay[data-menu-owner]',
      node => node instanceof HTMLElement && node.getAttribute('data-menu-owner') === owner,
      event
    );
  }

  directory = {
    acquire,
    release,
    subscribe,
    discoverSubtree,
    discoverFromEvent,
    pruneDetachedScopes,
    elementById,
    sameNodeById,
    elements,
    transports,
    categories,
    overlays,
    scopes: () => {
      ensureInitialized();
      return Array.from(scopes).filter(scope => scope === document || scope.host.isConnected);
    }
  };
  document[slot] = directory;
  return directory;
}
"#
    };
}
pub(crate) use menu_scope_directory;

pub(crate) const MENU_SCOPE_DIRECTORY: &str = menu_scope_directory!();

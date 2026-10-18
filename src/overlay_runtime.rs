//! Owner-scoped dispatch with document-scoped Rust overlay coordination. Native code only reports current DOM
//! facts and executes an acknowledged default/stop/activation or focus intent.
use crate::overlay_domain::{Domain, DomainCallbacks};
use crate::overlay_focus_pending::{PendingFocus, PendingFocusQueue};
use crate::overlay_registry::{CloseBehavior, NativeFact, OverlayKind, Registry};
use dioxus::prelude::*;
use serde::Deserialize;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

thread_local! {
    // Native focus facts scoped to the synchronous Window dispatch. A nested
    // event restores the outer capture rather than replacing its authorization.
    static WINDOW_CAPTURE: RefCell<Option<Vec<usize>>> = const { RefCell::new(None) };
}
struct WindowCapture(Option<Vec<usize>>);
impl WindowCapture {
    fn enter(categories: &[usize]) -> Self {
        Self(WINDOW_CAPTURE.with(|slot| slot.replace(Some(categories.to_vec()))))
    }
}
impl Drop for WindowCapture {
    fn drop(&mut self) {
        WINDOW_CAPTURE.with(|slot| slot.replace(self.0.take()));
    }
}

static NEXT_RUNTIME: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Trigger,
    Keyboard,
    Escape,
    Left,
    Inside,
    Outside,
    AnchorVisibility,
    Parent,
    Configuration,
    Drop,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Change {
    pub(crate) open: bool,
    pub(crate) reason: Reason,
    pub(crate) restore_focus: bool,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct KeyFacts {
    pub(crate) key: String,
    pub(crate) shift: bool,
    pub(crate) alt: bool,
    pub(crate) ctrl: bool,
    pub(crate) meta: bool,
    pub(crate) default_prevented: bool,
    pub(crate) focused_item: Option<usize>,
    pub(crate) items: Vec<usize>,
    pub(crate) focus_count: usize,
    pub(crate) focused_index: Option<usize>,
    pub(crate) on_trigger: bool,
    pub(crate) inside: bool,
    pub(crate) popover_ancestor: bool,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct KeyDecision {
    pub(crate) prevent: bool,
    pub(crate) stop: bool,
    pub(crate) immediate_stop: bool,
    pub(crate) release_focus: bool,
    pub(crate) activate: Option<String>,
    pub(crate) focus: Option<String>,
}
type KeyCallback = Rc<dyn Fn(&KeyFacts) -> KeyDecision>;

#[derive(Clone)]
pub(crate) struct Callbacks {
    pub(crate) read: Rc<dyn Fn() -> Option<bool>>,
    pub(crate) before: Rc<dyn Fn(Change) -> bool>,
    pub(crate) apply: Rc<dyn Fn(Change)>,
    pub(crate) key: Option<KeyCallback>,
}
#[derive(Clone)]
struct Entry {
    generation: u64,
    revision: u64,
    before_operation: Option<u64>,
    kind: OverlayKind,
    behavior: CloseBehavior,
    trigger_visibility_check: bool,
    callbacks: Callbacks,
    scope: Option<(Weak<dioxus::core::Runtime>, ScopeId)>,
}
#[derive(Default)]
struct Ack {
    sequence: u64,
    stop: bool,
    immediate_stop: bool,
    activation: Option<(String, String, u64, u64)>,
    focus: Option<(String, String, u64, u64, bool)>,
}
struct Runtime {
    owner: String,
    registry: Registry,
    nodes: BTreeMap<String, Entry>,
    order: Vec<String>,
    sequence: u64,
    keyboard: bool,
    capture_block: Vec<(String, u64)>,
    ack: Ack,
    pending_focus: PendingFocusQueue,
    bridge: Option<dioxus::document::Eval>,
    domain: Option<Domain>,
    document: String,
    categories: BTreeMap<usize, CategoryCallback>,
}
type CategoryCallback = Rc<dyn Fn(usize, &[usize])>;

impl Drop for Runtime {
    fn drop(&mut self) {
        self.pending_focus.clear();
        if let Some(domain) = &self.domain {
            domain.remove_owner(&self.owner);
        }
        if let Some(bridge) = self.bridge.take() {
            let _ = bridge.send(serde_json::json!({"finish": true}));
        }
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeHandle(Rc<RefCell<Runtime>>);
impl PartialEq for RuntimeHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Clone)]
pub(crate) struct NodeHandle {
    runtime: Weak<RefCell<Runtime>>,
    id: String,
    generation: u64,
}
impl RuntimeHandle {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Runtime {
            owner: format!(
                "ix-overlay-runtime-{}",
                NEXT_RUNTIME.fetch_add(1, Ordering::Relaxed)
            ),
            registry: Registry::new(),
            nodes: BTreeMap::new(),
            order: Vec::new(),
            sequence: 0,
            keyboard: false,
            capture_block: Vec::new(),
            ack: Ack::default(),
            pending_focus: PendingFocusQueue::default(),
            bridge: None,
            domain: None,
            document: String::new(),
            categories: BTreeMap::new(),
        })))
    }
    pub(crate) fn owner_id(&self) -> String {
        self.0.borrow().owner.clone()
    }
    pub(crate) fn keyboard_mode(&self) -> bool {
        self.document_keyboard_mode()
            .unwrap_or_else(|| self.0.borrow().keyboard)
    }
    pub(crate) fn register(
        &self,
        id: String,
        generation: u64,
        kind: OverlayKind,
        behavior: CloseBehavior,
        callbacks: Callbacks,
    ) -> NodeHandle {
        let scope = dioxus::core::Runtime::try_current().and_then(|runtime| {
            runtime
                .try_current_scope_id()
                .map(|scope| (Rc::downgrade(&runtime), scope))
        });
        let mut runtime = self.0.borrow_mut();
        if !runtime
            .nodes
            .get(&id)
            .is_some_and(|entry| entry.generation > generation)
        {
            if let Some(old_generation) = runtime.nodes.get(&id).map(|entry| entry.generation)
                && old_generation != generation
            {
                runtime.pending_focus.remove_root(&id, old_generation);
            }
            let revision = runtime
                .nodes
                .get(&id)
                .map_or(0, |entry| entry.revision.wrapping_add(1));
            runtime
                .registry
                .register(id.clone(), generation, kind, behavior);
            if !runtime.order.contains(&id) {
                runtime.order.push(id.clone());
            }
            runtime.nodes.insert(
                id.clone(),
                Entry {
                    generation,
                    revision,
                    before_operation: None,
                    kind,
                    behavior,
                    trigger_visibility_check: kind.is_dropdown(),
                    callbacks,
                    scope,
                },
            );
        }
        drop(runtime);
        self.join_node(&id);
        NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id,
            generation,
        }
    }
    fn domain(&self) -> Option<Domain> {
        self.0.borrow().domain.clone()
    }
    fn with_owner<T>(&self, id: &str, generation: u64, f: impl FnOnce() -> T) -> Option<T> {
        let entry = self
            .entry(id)
            .filter(|entry| entry.generation == generation)?;
        match entry.scope {
            Some((owner, scope)) => owner.upgrade().map(|owner| owner.in_scope(scope, f)),
            None => Some(f()),
        }
    }
    fn join_node(&self, id: &str) {
        let Some(domain) = self.domain() else { return };
        let Some(entry) = self.entry(id) else { return };
        let generation = entry.generation;
        let owner = self.owner_id();
        let read_runtime = Rc::downgrade(&self.0);
        let request_runtime = read_runtime.clone();
        let read_id = id.to_string();
        let request_id = read_id.clone();
        domain.register(
            owner,
            id.to_string(),
            generation,
            entry.kind,
            entry.behavior,
            DomainCallbacks {
                read: Rc::new(move || {
                    let runtime = RuntimeHandle(read_runtime.upgrade()?);
                    runtime
                        .with_owner(&read_id, generation, || {
                            let entry = runtime.entry(&read_id)?;
                            runtime.read_current(&read_id, &entry)
                        })
                        .flatten()
                }),
                request: Rc::new(move |change| {
                    let Some(runtime) = request_runtime.upgrade().map(RuntimeHandle) else {
                        return false;
                    };
                    runtime
                        .with_owner(&request_id, generation, || {
                            runtime.request(&request_id, generation, change)
                        })
                        .unwrap_or(false)
                }),
            },
        );
        if let Some(open) = self.read_current(id, &entry) {
            domain.sync_open(id, generation, open);
        }
    }
    fn attach_document(&self, document: &str) {
        if document.is_empty() || self.0.borrow().document == document {
            return;
        }
        let old = self.domain();
        if let Some(old) = old {
            old.remove_owner(&self.owner_id());
        }
        let domain = Domain::join(document);
        let (ids, categories) = {
            let mut state = self.0.borrow_mut();
            state.pending_focus.clear();
            state.document = document.to_string();
            state.domain = Some(domain);
            (state.order.clone(), state.categories.clone())
        };
        for id in ids {
            self.join_node(&id);
        }
        for (id, callback) in categories {
            if let Some(domain) = self.domain() {
                domain.register_category(self.owner_id(), id, callback);
            }
        }
    }
    pub(crate) fn category_close_capture(&self) -> (bool, Vec<usize>) {
        WINDOW_CAPTURE.with(|slot| {
            slot.borrow()
                .clone()
                .map_or_else(|| (false, Vec::new()), |active| (true, active))
        })
    }
    pub(crate) fn register_category(&self, id: usize, callback: CategoryCallback) {
        let owner = dioxus::core::Runtime::try_current().and_then(|runtime| {
            runtime
                .try_current_scope_id()
                .map(|scope| (Rc::downgrade(&runtime), scope))
        });
        let state = Rc::downgrade(&self.0);
        // Preserve the original category's runtime/scope across later native
        // attachment, which can occur in another transport component's scope.
        let guarded: CategoryCallback = Rc::new(move |source, active| {
            let Some(state) = state.upgrade() else { return };
            if !state.borrow().categories.contains_key(&id) {
                return;
            }
            if let Some((runtime, scope)) = &owner {
                if let Some(runtime) = runtime.upgrade() {
                    runtime.in_scope(*scope, || callback(source, active));
                }
            } else {
                callback(source, active);
            }
        });
        self.0.borrow_mut().categories.insert(id, guarded.clone());
        if let Some(domain) = self.domain() {
            domain.register_category(self.owner_id(), id, guarded);
        }
    }
    pub(crate) fn window_keyboard_mode(&self) -> Option<bool> {
        WINDOW_CAPTURE
            .with(|slot| slot.borrow().is_some())
            .then(|| self.document_keyboard_mode())
            .flatten()
    }
    pub(crate) fn document_keyboard_mode(&self) -> Option<bool> {
        self.domain().and_then(|domain| domain.keyboard_mode())
    }
    pub(crate) fn unregister_category(&self, id: usize) {
        self.0.borrow_mut().categories.remove(&id);
        if let Some(domain) = self.domain() {
            domain.unregister_category(&self.owner_id(), id);
        }
    }
    pub(crate) fn broadcast_category(&self, source: usize, active: &[usize]) -> bool {
        if let Some(domain) = self.domain() {
            domain.broadcast_category(source, active);
            true
        } else {
            false
        }
    }
    fn sync_domain_open(&self, id: &str, generation: u64, open: bool) {
        if let Some(domain) = self.domain() {
            domain.sync_open(id, generation, open);
        }
    }
    fn entry(&self, id: &str) -> Option<Entry> {
        self.0.borrow().nodes.get(id).cloned()
    }
    fn current(&self, id: &str, generation: u64, revision: u64) -> bool {
        self.0
            .borrow()
            .nodes
            .get(id)
            .is_some_and(|entry| entry.generation == generation && entry.revision == revision)
    }
    fn read_current(&self, id: &str, snapshot: &Entry) -> Option<bool> {
        let value = (snapshot.callbacks.read)();
        self.current(id, snapshot.generation, snapshot.revision)
            .then_some(value)
            .flatten()
    }
    fn request(&self, id: &str, generation: u64, change: Change) -> bool {
        let forced = change.reason == Reason::Configuration && !change.open;
        let Some(snapshot) = self.entry(id).filter(|entry| {
            entry.generation == generation && (entry.before_operation.is_none() || forced)
        }) else {
            return false;
        };
        let Some(previous) = self.read_current(id, &snapshot) else {
            return false;
        };
        let closed_popover = !previous && !change.open && snapshot.kind == OverlayKind::Popover;
        if previous == change.open && !closed_popover {
            let mut runtime = self.0.borrow_mut();
            // Even an unchanged command supersedes an older request still
            // between its before callback and apply (e.g. a child reopens its
            // still-open parent during the parent's pending close).
            if let Some(entry) = runtime.nodes.get_mut(id) {
                entry.revision = entry.revision.wrapping_add(1);
                if forced {
                    entry.before_operation = None;
                }
            }
            runtime.registry.set_presence(id, previous);
            drop(runtime);
            self.sync_domain_open(id, generation, previous);
            return true;
        }
        let operation = {
            let mut runtime = self.0.borrow_mut();
            let Some(entry) = runtime.nodes.get_mut(id).filter(|entry| {
                entry.generation == generation
                    && entry.revision == snapshot.revision
                    && (entry.before_operation.is_none() || forced)
            }) else {
                return false;
            };
            entry.revision = entry.revision.wrapping_add(1);
            entry.before_operation = Some(entry.revision);
            entry.revision
        };
        // No RefCell borrow crosses a user callback. A recursive request on the
        // same node during before is rejected, rather than recursing forever.
        let allowed = (change.reason == Reason::Configuration && !change.open)
            || (snapshot.callbacks.before)(change);
        {
            let mut runtime = self.0.borrow_mut();
            let Some(entry) = runtime
                .nodes
                .get_mut(id)
                .filter(|entry| entry.generation == generation)
            else {
                return false;
            };
            if entry.before_operation == Some(operation) {
                entry.before_operation = None;
            }
            if entry.revision != operation {
                return false;
            }
        }
        let current = Entry {
            revision: operation,
            ..snapshot.clone()
        };
        if !allowed || self.read_current(id, &current) != Some(previous) {
            return false;
        }
        if closed_popover {
            // Source Popover always emits willDismiss, even when closed, but
            // performs no child cascade, native command or completion then.
            return false;
        }
        if !change.open {
            self.close_children_with_focus(
                id,
                if change.reason == Reason::Drop {
                    Reason::Drop
                } else {
                    Reason::Parent
                },
                change.restore_focus,
            );
        }
        if !self.current(id, generation, operation)
            || self.read_current(id, &current) != Some(previous)
        {
            return false;
        }
        // Commit graph state before apply so a nested command in an accepted
        // callback sees the current hierarchy, not the superseded transaction.
        self.0.borrow_mut().registry.set_open(id, change.open);
        if change.open {
            if let Some(domain) = self.domain() {
                domain.presented(id, generation);
            }
        }
        self.sync_domain_open(id, generation, change.open);
        if !change.open {
            if let Some(domain) = self.domain() {
                domain.forget_children(id, generation);
            }
        }
        (snapshot.callbacks.apply)(change);
        if !self.current(id, generation, operation) {
            return false;
        }
        let actual = self.read_current(id, &current);
        if let Some(open) = actual {
            self.0.borrow_mut().registry.set_presence(id, open);
            self.sync_domain_open(id, generation, open);
        }
        actual == Some(change.open)
    }
    fn request_id(&self, id: &str, change: Change) -> bool {
        let Some(entry) = self.entry(id) else {
            return false;
        };
        self.request(id, entry.generation, change)
    }
    fn close_children(&self, id: &str, reason: Reason) {
        self.close_children_with_focus(id, reason, false);
    }
    fn close_children_with_focus(&self, id: &str, reason: Reason, restore_focus: bool) {
        if let Some(domain) = self.domain() {
            domain.close_children(id, reason, restore_focus);
            return;
        }
        let children = self.0.borrow().registry.cached_children(id);
        for child in children {
            self.request_id(
                &child,
                Change {
                    open: false,
                    reason,
                    restore_focus: false,
                },
            );
        }
    }
    pub(crate) fn child_path(&self, id: &str, hosts: &[String], triggers: &[String]) -> bool {
        if let Some(domain) = self.domain() {
            return domain.child_path(id, hosts, triggers);
        }
        self.0.borrow().registry.child_path(id, hosts, triggers)
    }
    /// Click discovery includes closed child triggers, but Escape/Tab belong
    /// only to a currently open child. Closed-trigger opening keys still route
    /// to that child rather than the Header's flat keyboard policy.
    pub(crate) fn key_child_path(
        &self,
        parent: &str,
        hosts: &[String],
        triggers: &[String],
        key: &str,
    ) -> bool {
        if !matches!(key, "Escape" | "Tab") {
            return self.child_path(parent, hosts, triggers);
        }
        hosts.iter().chain(triggers).any(|id| {
            let Some(entry) = self
                .entry(id)
                .filter(|entry| entry.kind == OverlayKind::Dropdown)
            else {
                return false;
            };
            if self.read_current(id, &entry) != Some(true)
                || (key == "Escape" && !self.owns_topmost(id))
            {
                return false;
            }
            let host = if hosts.contains(id) {
                vec![id.clone()]
            } else {
                Vec::new()
            };
            let trigger = if triggers.contains(id) {
                vec![id.clone()]
            } else {
                Vec::new()
            };
            self.child_path(parent, &host, &trigger)
        })
    }
    fn owns_topmost(&self, id: &str) -> bool {
        if let Some(domain) = self.domain() {
            return domain.owns_topmost(id);
        }
        let runtime = self.0.borrow();
        runtime.registry.topmost().is_some_and(|top| {
            top == id
                || runtime
                    .registry
                    .live_ancestors(&top)
                    .iter()
                    .any(|ancestor| ancestor == id)
        })
    }
    pub(crate) fn dismiss_root(&self, id: &str, reason: Reason, restore_focus: bool) -> bool {
        if let Some(domain) = self.domain() {
            return domain.dismiss_root(id, reason, restore_focus);
        }
        let root = self.0.borrow().registry.cached_root(id);
        let Some(root) = root else {
            return false;
        };
        // Nested Header Escape in the recorded fresh-small source case ends at
        // BODY. The flat Header path retains its own separately tested policy.
        let restore_focus = restore_focus
            && self
                .entry(&root)
                .is_some_and(|entry| entry.kind != OverlayKind::Header);
        self.request_id(
            &root,
            Change {
                open: false,
                reason,
                restore_focus,
            },
        )
    }
    pub(crate) fn close_submenu(&self, id: &str) {
        let parent = self.domain().map_or_else(
            || self.0.borrow().registry.cached_parent(id),
            |domain| domain.cached_parent(id),
        );
        if let Some(parent) = parent {
            let children = self.domain().map_or_else(
                || self.0.borrow().registry.cached_children(&parent),
                |domain| domain.cached_children(&parent),
            );
            for child in children {
                self.request_id(
                    &child,
                    Change {
                        open: false,
                        reason: Reason::Left,
                        restore_focus: true,
                    },
                );
            }
        }
    }
    fn sync_open(&self, id: &str, generation: u64, open: bool) {
        let mut runtime = self.0.borrow_mut();
        let Some(entry) = runtime
            .nodes
            .get_mut(id)
            .filter(|entry| entry.generation == generation)
        else {
            return;
        };
        entry.revision = entry.revision.wrapping_add(1);
        runtime.registry.set_presence(id, open);
        drop(runtime);
        self.sync_domain_open(id, generation, open);
    }
    fn unregister(&self, id: &str, generation: u64) {
        if !self
            .entry(id)
            .is_some_and(|entry| entry.generation == generation)
        {
            return;
        }
        self.close_children(id, Reason::Drop);
        let mut runtime = self.0.borrow_mut();
        if runtime
            .nodes
            .get(id)
            .is_some_and(|entry| entry.generation == generation)
        {
            runtime.pending_focus.remove_root(id, generation);
            runtime.nodes.remove(id);
            runtime.order.retain(|candidate| candidate != id);
            runtime.registry.unregister(id, generation);
            if let Some(domain) = &runtime.domain {
                domain.unregister(id, generation);
            }
            if runtime
                .ack
                .activation
                .as_ref()
                .is_some_and(|(_, node, _, _)| node == id)
            {
                runtime.ack.activation = None;
            }
        }
    }
    fn start_bridge(&self) {
        if self.0.borrow().bridge.is_some() {
            return;
        }
        let owner = serde_json::to_string(&self.owner_id()).expect("overlay runtime owner");
        let bridge = dioxus::document::eval(
            &crate::overlay_registry_dom::BRIDGE.replace("__OWNER__", &owner),
        );
        self.0.borrow_mut().bridge = Some(bridge);
    }
    fn observe(&self, report: &Report) -> KeyDecision {
        {
            let mut runtime = self.0.borrow_mut();
            if report.owner != runtime.owner || report.sequence <= runtime.sequence {
                return KeyDecision::default();
            }
            runtime.sequence = report.sequence;
            runtime.ack = Ack {
                sequence: report.sequence,
                ..Ack::default()
            };
        }
        self.attach_document(&report.document);
        // Native open attributes may lag the Rust callback during an original
        // click. Reconcile with read(), never overwrite current state from DOM.
        let snapshots = {
            let runtime = self.0.borrow();
            runtime
                .order
                .iter()
                .filter_map(|id| {
                    runtime
                        .nodes
                        .get(id)
                        .map(|entry| (id.clone(), entry.clone()))
                })
                .collect::<Vec<_>>()
        };
        for (id, entry) in &snapshots {
            if let Some(open) = self.read_current(id, entry) {
                self.0.borrow_mut().registry.set_presence(id, open);
            }
        }
        let facts = snapshots
            .iter()
            .map(|(id, entry)| {
                let node = report.nodes.iter().find(|node| {
                    node.id == *id && node.generation == entry.generation && node.connected
                });
                NativeFact {
                    id: id.clone(),
                    generation: entry.generation,
                    connected: node.is_some(),
                    discovery_ready: node.is_some_and(|node| node.discovery_ready),
                    trigger_ancestors: node
                        .map_or_else(Vec::new, |node| node.trigger_ancestors.clone()),
                }
            })
            .collect::<Vec<_>>();
        self.0.borrow_mut().registry.observe(&facts);
        if let Some(domain) = self.domain() {
            if matches!(report.phase.as_str(), "window" | "trap") || report.key == "Tab" {
                let all = report
                    .nodes
                    .iter()
                    .map(|node| NativeFact {
                        id: node.id.clone(),
                        generation: node.generation,
                        connected: node.connected,
                        discovery_ready: node.discovery_ready,
                        trigger_ancestors: node.trigger_ancestors.clone(),
                    })
                    .collect::<Vec<_>>();
                domain.observe_all(&all);
            } else {
                domain.observe_owner(&report.owner, &facts);
            }
        }
        if report.kind == "modality" {
            if let Some(domain) = self.domain() {
                domain.observe_input(&report.input_kind, &report.key);
            }
        }
        let decision = if report.phase == "window" {
            if let Some(domain) = self
                .domain()
                .filter(|domain| domain.accept_event(report.window_sequence))
            {
                let _capture = WindowCapture::enter(&report.active_categories);
                domain.refresh();
                match report.kind.as_str() {
                    "key" if report.key == "Escape" => {
                        domain.escape();
                    }
                    "click" => {
                        let mut hosts = report.hosts.clone();
                        hosts.extend(report.anchors.iter().cloned());
                        domain.outside(&hosts, &report.triggers);
                    }
                    _ => {}
                }
            }
            KeyDecision::default()
        } else {
            match report.kind.as_str() {
                "pointer" => {
                    self.0.borrow_mut().keyboard = false;
                    KeyDecision::default()
                }
                "key" if report.phase == "trap" => self.popover_tab(report),
                "key" => self.key(report),
                "click" => self.click(report),
                "anchor-visibility" => self.anchor_visibility(report),
                _ => KeyDecision::default(),
            }
        };
        let activation = decision
            .activate
            .as_ref()
            .filter(|id| report.candidate_ids.contains(id))
            .and_then(|target| {
                let source = report.nodes.iter().find(|node| {
                    node.items
                        .iter()
                        .any(|id| target == &format!("ix-dropdown-item-{id}"))
                })?;
                let entry = self.entry(&source.id)?;
                (source.connected
                    && source.generation == entry.generation
                    && self.read_current(&source.id, &entry) == Some(true))
                .then(|| {
                    (
                        target.clone(),
                        source.id.clone(),
                        entry.generation,
                        entry.revision,
                    )
                })
            });
        let focus = self
            .0
            .borrow_mut()
            .ack
            .focus
            .take()
            .filter(|authorization| {
                decision.focus.as_ref() == Some(&authorization.0)
                    && (report.tab_order.contains(&authorization.0)
                        || report.focus_targets.contains(&authorization.0))
            });
        let mut runtime = self.0.borrow_mut();
        if runtime.sequence == report.sequence {
            runtime.ack = Ack {
                sequence: report.sequence,
                stop: decision.stop,
                immediate_stop: decision.immediate_stop,
                activation,
                focus,
            };
        }
        decision
    }
    fn popover_tab(&self, report: &Report) -> KeyDecision {
        if report.key != "Tab" || report.default_prevented {
            return KeyDecision::default();
        }
        let Some(domain) = self.domain() else {
            return KeyDecision::default();
        };
        for node in &report.nodes {
            let Some(entry) = self.entry(&node.id).filter(|entry| {
                entry.kind == OverlayKind::Popover
                    && entry.generation == node.generation
                    && node.connected
                    && node.trap_enabled
            }) else {
                continue;
            };
            if self.read_current(&node.id, &entry) != Some(true) {
                continue;
            }
            let Some(excluded) = domain.focus_trap_exclusions(
                &node.id,
                &report.active_hosts,
                &report.active_popover,
            ) else {
                continue;
            };
            let candidates = self.source_tab_candidates(report, &node.tab_candidates);
            let Some(target) =
                crate::popover_focus::tab_boundary_target(&candidates, &excluded, report.shift)
            else {
                continue;
            };
            self.0.borrow_mut().ack.focus = Some((
                target.clone(),
                node.id.clone(),
                entry.generation,
                entry.revision,
                true,
            ));
            return KeyDecision {
                prevent: true,
                stop: true,
                immediate_stop: true,
                focus: Some(target),
                ..KeyDecision::default()
            };
        }
        KeyDecision::default()
    }
    fn key_owner<'a>(&self, report: &'a Report) -> Option<&'a NodeFacts> {
        report
            .hosts
            .iter()
            .chain(report.triggers.iter())
            .find_map(|id| {
                let entry = self.entry(id)?;
                report.nodes.iter().find(|node| {
                    node.id == *id
                        && node.connected
                        && node.generation == entry.generation
                        && (entry.kind == OverlayKind::Dropdown
                            || (entry.kind == OverlayKind::Header
                                && report.key == "Tab"
                                && node.on_trigger))
                })
            })
    }
    fn reported_kind(&self, report: &Report, id: &str) -> Option<OverlayKind> {
        let node = report
            .nodes
            .iter()
            .find(|node| node.id == id && node.connected)?;
        self.entry(id)
            .filter(|entry| entry.generation == node.generation)
            .map(|entry| entry.kind)
            .or_else(|| self.domain()?.node_kind(id, node.generation))
    }
    /// Header's internal controls are native light-DOM buttons in this port,
    /// but live inside the source Header shadow root. Ordinary source queries
    /// do not enter that shadow root. Keep the renderer projection in Rust;
    /// the captured trigger remains separately available for reverse exits.
    fn source_tab_candidates(
        &self,
        report: &Report,
        candidates: &[crate::popover_focus::TabCandidate],
    ) -> Vec<crate::popover_focus::TabCandidate> {
        let controls = report
            .nodes
            .iter()
            .filter(|node| self.reported_kind(report, &node.id) == Some(OverlayKind::Header))
            .flat_map(|node| node.header_controls.iter())
            .collect::<Vec<_>>();
        candidates
            .iter()
            .filter(|candidate| !controls.contains(&&candidate.target))
            .cloned()
            .collect()
    }
    fn source_document_order(&self, report: &Report) -> Vec<String> {
        let Some(candidates) = &report.document_tab_candidates else {
            // Compatibility for pre-native reports and focused transport tests.
            let controls = report
                .nodes
                .iter()
                .filter(|node| self.reported_kind(report, &node.id) == Some(OverlayKind::Header))
                .flat_map(|node| node.header_controls.iter())
                .collect::<Vec<_>>();
            return report
                .tab_order
                .iter()
                .filter(|target| !controls.contains(target))
                .cloned()
                .collect();
        };
        let candidates = self
            .source_tab_candidates(report, candidates)
            .into_iter()
            .filter(|candidate| {
                !candidate.hosts.iter().any(|host| {
                    self.reported_kind(report, host)
                        .is_some_and(OverlayKind::is_dropdown)
                })
            })
            .collect::<Vec<_>>();
        crate::popover_focus::tab_order_targets(&candidates)
    }
    fn key(&self, report: &Report) -> KeyDecision {
        let Some(owner) = self.key_owner(report) else {
            return KeyDecision::default();
        };
        let Some(entry) = self.entry(&owner.id) else {
            return KeyDecision::default();
        };
        let Some(callback) = entry.callbacks.key.clone() else {
            return KeyDecision::default();
        };
        let mut facts = KeyFacts {
            key: report.key.clone(),
            shift: report.shift,
            alt: report.alt,
            ctrl: report.ctrl,
            meta: report.meta,
            default_prevented: report.default_prevented,
            focused_item: owner.focused_item,
            items: owner.items.clone(),
            focus_count: owner.focus_count,
            focused_index: owner.focused_index,
            on_trigger: owner.on_trigger,
            inside: owner.inside,
            popover_ancestor: self
                .domain()
                .is_some_and(|domain| domain.has_popover_ancestor(&owner.id)),
        };
        let capture = owner.mode == "roving-tabindex";
        if report.phase == "capture" && !capture {
            // Mark the native route before Dioxus's own bubble fallback runs.
            // The private callback returns without action for prevented facts.
            facts.default_prevented = true;
            callback(&facts);
            return KeyDecision::default();
        }
        if (matches!(report.phase.as_str(), "bubble" | "fallback") && capture)
            || facts.default_prevented
            || (report.phase == "fallback" && !matches!(report.key.as_str(), "Escape" | "Tab"))
        {
            // Active-descendant arrows/activation belong to the actual trigger
            // listener, not an unrelated input in an externally hosted menu.
            return KeyDecision::default();
        }
        if !facts.inside && !facts.on_trigger {
            return KeyDecision::default();
        }
        let owner_open = self.read_current(&owner.id, &entry) == Some(true);
        if matches!(report.key.as_str(), "Escape" | "Tab") && !owner_open {
            return KeyDecision::default();
        }
        if owner_open
            || facts.on_trigger
                && matches!(
                    report.key.as_str(),
                    "Home" | "End" | "ArrowDown" | "ArrowUp" | "Enter" | " "
                )
        {
            self.0.borrow_mut().keyboard = true;
        }
        if report.key == "Escape" {
            if !self.owns_topmost(&owner.id) {
                return KeyDecision::default();
            }
            self.dismiss_root(&owner.id, Reason::Escape, true);
            return KeyDecision {
                stop: true,
                ..KeyDecision::default()
            };
        }
        let tab_release = if report.key == "Tab" && (capture || facts.popover_ancestor) {
            let root = self.domain().map_or_else(
                || self.0.borrow().registry.cached_root(&owner.id),
                |domain| domain.cached_root(&owner.id),
            );
            root.and_then(|root| {
                let node = report.nodes.iter().find(|node| node.id == root)?;
                let entry = self.entry(&root)?;
                if self.read_current(&root, &entry) != Some(true) {
                    return None;
                }
                let target = if report.shift
                    && report.hosts.iter().any(|host| {
                        self.reported_kind(report, host)
                            .is_some_and(OverlayKind::is_dropdown)
                    }) {
                    (!node.trigger_id.is_empty()).then(|| node.trigger_id.clone())
                } else {
                    self.domain()
                        .and_then(|domain| {
                            let parent = domain.popover_focus_parent(&root)?;
                            let parent_node = report.nodes.iter().find(|node| node.id == parent)?;
                            let excluded = domain
                                .focus_trap_exclusions(&parent, &[], "")
                                .unwrap_or_default();
                            let excluded = if excluded.is_empty() {
                                vec![root.clone()]
                            } else {
                                excluded
                            };
                            let candidates =
                                self.source_tab_candidates(report, &parent_node.tab_candidates);
                            crate::popover_focus::tab_adjacent_target(
                                &candidates,
                                &excluded,
                                &node.trigger_id,
                                report.shift,
                            )
                        })
                        .or_else(|| {
                            let order = self.source_document_order(report);
                            order
                                .iter()
                                .position(|id| id == &node.trigger_id)
                                .and_then(|index| {
                                    let next = if report.shift {
                                        index.checked_sub(1)
                                    } else {
                                        index.checked_add(1)
                                    }?;
                                    order.get(next).cloned()
                                })
                        })
                }?;
                Some((root, entry.generation, target))
            })
        } else {
            None
        };
        let mut decision = callback(&facts);
        if let Some((root, generation, target)) = tab_release.filter(|_| decision.release_focus) {
            if self
                .entry(&root)
                .filter(|entry| entry.generation == generation)
                .and_then(|entry| self.read_current(&root, &entry))
                .is_some()
            {
                // Each source exit queues its precomputed target, including
                // veto and reentrant events. Later ACKs/visibility do not revoke
                // this work; only the owning registration's lifetime does.
                if self.0.borrow_mut().pending_focus.admit(PendingFocus {
                    sequence: report.sequence,
                    target: target.clone(),
                    root,
                    generation,
                }) {
                    decision.prevent = true;
                    decision.focus = Some(target);
                }
            }
        }
        decision
    }
    fn anchor_visibility(&self, report: &Report) -> KeyDecision {
        let Some(fact) = report
            .anchor_visibility
            .as_ref()
            .filter(|fact| !fact.intersecting)
        else {
            return KeyDecision::default();
        };
        let Some(node) = report.nodes.iter().find(|node| {
            node.id == fact.root && node.generation == fact.generation && node.connected
        }) else {
            return KeyDecision::default();
        };
        let expected = if node.anchor_id.is_empty() {
            &node.trigger_id
        } else {
            &node.anchor_id
        };
        if expected.is_empty() || expected != &fact.target {
            return KeyDecision::default();
        }
        let Some(entry) = self.entry(&fact.root).filter(|entry| {
            entry.generation == fact.generation
                && entry.kind.is_dropdown()
                && entry.trigger_visibility_check
        }) else {
            return KeyDecision::default();
        };
        self.with_owner(&fact.root, fact.generation, || {
            if self.read_current(&fact.root, &entry) == Some(true) {
                // A real native observer delivery is a separate dismissal
                // request in the registered owner's scope. Veto retains it.
                self.request(
                    &fact.root,
                    fact.generation,
                    Change {
                        open: false,
                        reason: Reason::AnchorVisibility,
                        restore_focus: false,
                    },
                );
            }
        });
        KeyDecision::default()
    }
    fn click(&self, report: &Report) -> KeyDecision {
        if report.phase == "capture" {
            let mut blocked = Vec::new();
            let mut prevent = false;
            for (index, id) in report.triggers.iter().enumerate() {
                let Some(entry) = self.entry(id).filter(|entry| {
                    entry.kind == OverlayKind::Dropdown
                        && report.nodes.iter().any(|node| {
                            node.id == *id && node.connected && node.generation == entry.generation
                        })
                }) else {
                    continue;
                };
                let child_path = self.child_path(id, &report.hosts, &report.triggers);
                // Block only the composed ancestor's Rust toggle. Preventing
                // the original event here would veto the nearest child before
                // that child's own trigger listener had a chance to run.
                if index > 0 || child_path {
                    blocked.push((id.clone(), entry.generation));
                }
                if index == 0
                    && report.hosts.contains(id)
                    && entry.behavior == CloseBehavior::Outside
                    && !child_path
                {
                    prevent = true;
                    blocked.push((id.clone(), entry.generation));
                }
            }
            self.0.borrow_mut().capture_block = blocked;
            return KeyDecision {
                prevent,
                ..KeyDecision::default()
            };
        }
        self.0.borrow_mut().capture_block.clear();
        let mut decision = KeyDecision::default();
        let keyboard = self.0.borrow().keyboard;
        // Header owns its flat inside/More action in its existing bridge. Only
        // nested Dropdown inside-close policy belongs to this coordinator.
        if !report.default_prevented {
            let leaf = report
                .hosts
                .iter()
                .find_map(|id| {
                    self.entry(id)
                        .filter(|entry| entry.kind == OverlayKind::Dropdown)
                        .map(|entry| (id.clone(), entry.behavior))
                })
                .filter(|(id, behavior)| {
                    behavior.allows_inside()
                        && !report.triggers.contains(id)
                        && !self.child_path(id, &report.hosts, &report.triggers)
                        && {
                            let ancestors = self.domain().map_or_else(
                                || self.0.borrow().registry.live_ancestors(id),
                                |domain| domain.live_ancestors(id),
                            );
                            report
                                .triggers
                                .iter()
                                .all(|trigger| ancestors.contains(trigger))
                        }
                })
                .map(|(id, _)| id);
            if let Some(leaf) = leaf {
                let eligible = {
                    let runtime = self.0.borrow();
                    runtime
                        .order
                        .iter()
                        .filter(|id| {
                            runtime.registry.is_open(id)
                                && (id.as_str() == leaf
                                    || runtime
                                        .nodes
                                        .get(*id)
                                        .is_some_and(|entry| entry.behavior.allows_outside()))
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                };
                for id in eligible {
                    self.request_id(
                        &id,
                        Change {
                            open: false,
                            reason: Reason::Inside,
                            restore_focus: keyboard,
                        },
                    );
                }
            }
        }
        // Protect foreign trigger clicks after their own native/Rust handlers
        // have run. An ended prevented click is not a vetoed opening request.
        if !report.triggers.is_empty() {
            let runtime = self.0.borrow();
            decision.prevent = runtime.order.iter().any(|id| {
                runtime.registry.is_open(id)
                    && runtime
                        .registry
                        .child_path(id, &report.hosts, &report.triggers)
            });
        }
        // Source outside coordination does not consult event.defaultPrevented.
        // In particular More can close children despite veto of its own close.
        let outside = if self.domain().is_some() {
            Vec::new()
        } else {
            self.0
                .borrow()
                .registry
                .outside_candidates(&report.hosts, &report.triggers)
        };
        for id in outside {
            self.request_id(
                &id,
                Change {
                    open: false,
                    reason: Reason::Outside,
                    restore_focus: false,
                },
            );
        }
        decision
    }
    fn input(&self, event: FormEvent) {
        event.stop_propagation();
        let Ok(report) = serde_json::from_str::<Report>(&event.value()) else {
            return;
        };
        let decision = self.observe(&report);
        if decision.prevent {
            event.prevent_default();
        }
    }
    fn query(&self, event: FormEvent) {
        event.stop_propagation();
        let Ok(query) = serde_json::from_str::<Query>(&event.value()) else {
            return;
        };
        if query.owner != self.owner_id() {
            return;
        }
        if let Some(target) = query.schedule_focus_target {
            if self
                .0
                .borrow_mut()
                .pending_focus
                .schedule(query.sequence, &target)
            {
                event.prevent_default();
            }
            return;
        }
        if let Some(target) = query.pending_focus_target {
            let pending = self
                .0
                .borrow_mut()
                .pending_focus
                .take(query.sequence, &target);
            if let Some(pending) = pending {
                // Destination reads can enter another owner. Validate the
                // authorizing registration only after those callbacks finish.
                let destination_visible = query.focus_hosts.is_empty()
                    || self.domain().is_some_and(|domain| {
                        domain.focus_destinations_visible(&query.focus_hosts)
                    });
                let live = self
                    .with_owner(&pending.root, pending.generation, || {
                        let entry = self.entry(&pending.root)?;
                        pending
                            .matches_owner(&pending.root, entry.generation)
                            .then(|| self.read_current(&pending.root, &entry))
                            .flatten()
                    })
                    .flatten()
                    .is_some();
                if live && destination_visible {
                    event.prevent_default();
                }
            }
            return;
        }
        let authorization = {
            let mut runtime = self.0.borrow_mut();
            if query.owner != runtime.owner || query.sequence != runtime.ack.sequence {
                return;
            }
            if let Some(target) = query.activate_target {
                if runtime
                    .ack
                    .activation
                    .as_ref()
                    .is_some_and(|activation| activation.0 == target)
                {
                    runtime
                        .ack
                        .activation
                        .take()
                        .map(|authorization| (authorization, true))
                } else {
                    None
                }
            } else if let Some(target) = query.focus_target {
                if runtime
                    .ack
                    .focus
                    .as_ref()
                    .is_some_and(|focus| focus.0 == target)
                {
                    runtime.ack.focus.take().map(
                        |(target, id, generation, revision, expected_open)| {
                            ((target, id, generation, revision), expected_open)
                        },
                    )
                } else {
                    None
                }
            } else if query.immediate_stop {
                if runtime.ack.immediate_stop {
                    event.prevent_default();
                }
                None
            } else {
                if runtime.ack.stop {
                    event.prevent_default();
                }
                None
            }
        };
        if let Some(((_, id, generation, revision), expected_open)) = authorization {
            if self
                .entry(&id)
                .filter(|entry| entry.generation == generation && entry.revision == revision)
                .and_then(|entry| self.read_current(&id, &entry))
                == Some(expected_open)
            {
                event.prevent_default();
            }
        }
    }
}
impl NodeHandle {
    pub(crate) fn set_trigger_visibility_check(&self, enabled: bool) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let mut state = runtime.0.borrow_mut();
        if let Some(entry) = state.nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.trigger_visibility_check = enabled;
        }
    }
    pub(crate) fn has_popover_ancestor(&self) -> bool {
        self.is_registered()
            && self
                .runtime()
                .and_then(|runtime| runtime.domain())
                .is_some_and(|domain| domain.has_popover_ancestor(&self.id))
    }
    pub(crate) fn runtime(&self) -> Option<RuntimeHandle> {
        self.runtime.upgrade().map(RuntimeHandle)
    }
    pub(crate) fn id(&self) -> &str {
        &self.id
    }
    /// Execute admitted work in the registered component's runtime/scope.
    /// Native completion tasks must not inherit a transient invoker's scope.
    pub(crate) fn with_owner<T>(&self, f: impl FnOnce() -> T) -> Option<T> {
        self.runtime()?.with_owner(&self.id, self.generation, f)
    }
    /// Native default prevention is not copied into Dioxus web MouseEvent
    /// metadata. The component consults the same Rust capture decision before
    /// toggling, after running its own consumer callback.
    pub(crate) fn trigger_click_allowed(&self) -> bool {
        self.runtime().is_some_and(|runtime| {
            let state = runtime.0.borrow();
            state
                .nodes
                .get(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
                && !state
                    .capture_block
                    .iter()
                    .any(|(id, generation)| id == &self.id && *generation == self.generation)
        })
    }
    pub(crate) fn request(&self, open: bool, reason: Reason, restore_focus: bool) -> bool {
        self.runtime().is_some_and(|runtime| {
            // Public handles may be invoked from a short-lived trigger or a
            // different VDOM. Apply can spawn admitted native work: that task
            // belongs to the registered component, not the invoking scope.
            runtime
                .with_owner(&self.id, self.generation, || {
                    runtime.request(
                        &self.id,
                        self.generation,
                        Change {
                            open,
                            reason,
                            restore_focus,
                        },
                    )
                })
                .unwrap_or(false)
        })
    }
    pub(crate) fn is_registered(&self) -> bool {
        self.runtime().is_some_and(|runtime| {
            runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
        })
    }
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn prepare_present(&self) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        if !runtime
            .entry(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
        {
            return false;
        }
        if let Some(domain) = runtime.domain() {
            return domain.prepare_present(&self.id);
        }
        // Before native document attachment, retain controller behavior within
        // this root. Cross-root bootstrap remains a separately tracked boundary.
        let excluded = runtime.0.borrow().registry.cached_path_including(&self.id);
        let candidates = runtime.0.borrow().order.clone();
        for id in candidates {
            if excluded.contains(&id) {
                continue;
            }
            if runtime.entry(&id).is_some_and(|entry| {
                entry.kind == OverlayKind::Popover && entry.behavior.allows_outside()
            }) {
                runtime.request_id(
                    &id,
                    Change {
                        open: false,
                        reason: Reason::Outside,
                        restore_focus: true,
                    },
                );
            }
        }
        let mut state = runtime.0.borrow_mut();
        if !state
            .nodes
            .get(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
        {
            return false;
        }
        state.registry.snapshot_children(&self.id);
        true
    }
    pub(crate) fn did_present(&self) {
        if let Some(runtime) = self.runtime() {
            if runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
            {
                runtime.0.borrow_mut().registry.set_presence(&self.id, true);
                if let Some(domain) = runtime.domain() {
                    domain.presented(&self.id, self.generation);
                }
            }
        }
    }
    pub(crate) fn sync_open(&self, open: bool) {
        if let Some(runtime) = self.runtime() {
            runtime.sync_open(&self.id, self.generation, open);
        }
    }
    pub(crate) fn set_close_behavior(&self, behavior: CloseBehavior) {
        if let Some(runtime) = self.runtime() {
            let mut runtime = runtime.0.borrow_mut();
            if let Some(entry) = runtime
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            {
                entry.behavior = behavior;
                runtime.registry.set_close_behavior(&self.id, behavior);
                if let Some(domain) = &runtime.domain {
                    domain.set_close_behavior(&self.id, self.generation, behavior);
                }
            }
        }
    }
    pub(crate) fn close_children(&self, reason: Reason) {
        if let Some(runtime) = self.runtime() {
            if runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
            {
                runtime.close_children(&self.id, reason);
            }
        }
    }
    pub(crate) fn unregister(&self) {
        if let Some(runtime) = self.runtime() {
            runtime.unregister(&self.id, self.generation);
        }
    }
}

pub(crate) fn use_overlay_runtime() -> RuntimeHandle {
    use_root_context(RuntimeHandle::new)
}

#[component]
pub(crate) fn OverlayTransport(runtime: RuntimeHandle) -> Element {
    let owner = runtime.owner_id();
    let mounted = runtime.clone();
    let input = runtime.clone();
    rsx! { input { r#type:"hidden", "data-ix-overlay-transport":owner,
        onmounted:move |_| mounted.start_bridge(),
        oninput:move |event| input.input(event),
        onchange:move |event| runtime.query(event),
    } }
}

#[derive(Deserialize)]
struct Report {
    owner: String,
    sequence: u64,
    kind: String,
    #[serde(default)]
    document: String,
    #[serde(default)]
    input_kind: String,
    #[serde(default)]
    window_sequence: u64,
    #[serde(default)]
    active_categories: Vec<usize>,
    #[serde(default)]
    anchors: Vec<String>,
    #[serde(default)]
    phase: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    alt: bool,
    #[serde(default)]
    ctrl: bool,
    #[serde(default)]
    meta: bool,
    #[serde(default)]
    default_prevented: bool,
    #[serde(default)]
    hosts: Vec<String>,
    #[serde(default)]
    triggers: Vec<String>,
    #[serde(default)]
    nodes: Vec<NodeFacts>,
    #[serde(default)]
    candidate_ids: Vec<String>,
    #[serde(default)]
    tab_order: Vec<String>,
    #[serde(default)]
    document_tab_candidates: Option<Vec<crate::popover_focus::TabCandidate>>,
    #[serde(default)]
    anchor_visibility: Option<AnchorVisibility>,
    #[serde(default)]
    focus_targets: Vec<String>,
    #[serde(default)]
    active_hosts: Vec<String>,
    #[serde(default)]
    active_popover: String,
}
#[derive(Deserialize)]
struct NodeFacts {
    id: String,
    generation: u64,
    connected: bool,
    #[serde(default)]
    discovery_ready: bool,
    #[serde(default)]
    trigger_ancestors: Vec<String>,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    trigger_id: String,
    #[serde(default)]
    anchor_id: String,
    #[serde(default)]
    header_controls: Vec<String>,
    #[serde(default)]
    items: Vec<usize>,
    #[serde(default)]
    focused_item: Option<usize>,
    #[serde(default)]
    focus_count: usize,
    #[serde(default)]
    focused_index: Option<usize>,
    #[serde(default)]
    on_trigger: bool,
    #[serde(default)]
    inside: bool,
    #[serde(default)]
    trap_enabled: bool,
    #[serde(default)]
    tab_candidates: Vec<crate::popover_focus::TabCandidate>,
}
#[derive(Deserialize)]
struct AnchorVisibility {
    root: String,
    generation: u64,
    target: String,
    intersecting: bool,
}
#[derive(Deserialize)]
struct Query {
    owner: String,
    sequence: u64,
    #[serde(default)]
    activate_target: Option<String>,
    #[serde(default)]
    focus_target: Option<String>,
    #[serde(default)]
    immediate_stop: bool,
    #[serde(default)]
    schedule_focus_target: Option<String>,
    #[serde(default)]
    pending_focus_target: Option<String>,
    /// Current Popover ancestors of the captured native destination object.
    #[serde(default)]
    focus_hosts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::current_scope_id;
    use std::cell::Cell;

    #[derive(Clone)]
    struct RequestTaskFixture(Rc<RequestTaskState>);
    impl PartialEq for RequestTaskFixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    #[derive(Default)]
    struct RequestTaskState {
        node: RefCell<Option<NodeHandle>>,
        caller: Cell<Option<ScopeId>>,
        owner: Cell<Option<ScopeId>>,
        show_caller: Cell<Option<Signal<bool>>>,
        caller_dropped: Cell<bool>,
        open: Cell<bool>,
        complete: Cell<bool>,
        release: Cell<bool>,
        waker: RefCell<Option<std::task::Waker>>,
    }
    #[derive(Clone, Copy)]
    struct RequestTaskContext(&'static str);

    #[component]
    fn RequestTaskOwner(fixture: RequestTaskFixture) -> Element {
        use_context_provider(|| RequestTaskContext("owner"));
        let runtime = use_hook(RuntimeHandle::new);
        let registered = fixture.clone();
        let node = use_hook(move || {
            let scope = current_scope_id();
            registered.0.owner.set(Some(scope));
            registered.0.open.set(true);
            let read = registered.clone();
            let before = registered.clone();
            let apply = registered.clone();
            let node = runtime.register(
                "caller-task-overlay".into(),
                1,
                OverlayKind::Popover,
                CloseBehavior::Both,
                Callbacks {
                    read: Rc::new(move || {
                        assert_eq!(current_scope_id(), scope);
                        assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                        Some(read.0.open.get())
                    }),
                    before: Rc::new(move |_| {
                        assert_eq!(current_scope_id(), before.0.owner.get().unwrap());
                        assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                        true
                    }),
                    apply: Rc::new(move |change| {
                        assert_eq!(current_scope_id(), scope);
                        apply.0.open.set(change.open);
                        let pending = apply.clone();
                        spawn(async move {
                            std::future::poll_fn(|cx| {
                                if pending.0.release.get() {
                                    std::task::Poll::Ready(())
                                } else {
                                    *pending.0.waker.borrow_mut() = Some(cx.waker().clone());
                                    std::task::Poll::Pending
                                }
                            })
                            .await;
                            assert_eq!(current_scope_id(), scope);
                            assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                            pending.0.complete.set(true);
                        });
                    }),
                    key: None,
                },
            );
            *registered.0.node.borrow_mut() = Some(node.clone());
            node
        });
        use_drop(move || node.unregister());
        rsx! { div {} }
    }

    #[component]
    fn RequestTaskCaller(fixture: RequestTaskFixture) -> Element {
        use_context_provider(|| RequestTaskContext("caller"));
        fixture.0.caller.set(Some(current_scope_id()));
        use_drop(move || fixture.0.caller_dropped.set(true));
        rsx! { div {} }
    }

    fn request_task_root(fixture: RequestTaskFixture) -> Element {
        let show = use_signal(|| true);
        fixture.0.show_caller.set(Some(show));
        rsx! {
            RequestTaskOwner { fixture: fixture.clone() }
            if show() { RequestTaskCaller { fixture: fixture.clone() } }
        }
    }

    #[test]
    fn direct_request_native_work_survives_caller_scope_teardown() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let caller = fixture.0.caller.get().unwrap();
        assert_ne!(fixture.0.owner.get(), Some(caller));
        dom.in_scope(caller, || {
            assert!(node.request(false, Reason::Trigger, true));
            assert_eq!(current_scope_id(), caller, "dispatch restores its caller");
            assert_eq!(consume_context::<RequestTaskContext>().0, "caller");
        });
        assert!(!fixture.0.open.get());
        dom.render_immediate_to_vec();
        assert!(
            fixture.0.waker.borrow().is_some(),
            "accepted work reached await"
        );
        dom.in_scope(ScopeId::APP, || {
            let mut show = fixture.0.show_caller.get().unwrap();
            show.set(false);
        });
        dom.render_immediate_to_vec();
        assert!(fixture.0.caller_dropped.get());
        assert!(!fixture.0.complete.get());
        fixture.0.release.set(true);
        fixture.0.waker.borrow_mut().take().unwrap().wake();
        dom.render_immediate_to_vec();
        assert!(
            fixture.0.complete.get(),
            "owner task survived caller teardown"
        );
        dom.in_scope(fixture.0.owner.get().unwrap(), || node.unregister());
        assert!(
            !node.request(false, Reason::Trigger, true),
            "stale registration is rejected"
        );
        assert!(
            node.with_owner(|| panic!("stale owner cannot run work"))
                .is_none()
        );
    }

    struct Model {
        open: Rc<Cell<bool>>,
        veto: Rc<Cell<bool>>,
        node: NodeHandle,
    }
    fn model(
        runtime: &RuntimeHandle,
        id: &str,
        kind: OverlayKind,
        log: Rc<RefCell<Vec<String>>>,
    ) -> Model {
        let open = Rc::new(Cell::new(false));
        let veto = Rc::new(Cell::new(false));
        let read = open.clone();
        let apply = open.clone();
        let before_veto = veto.clone();
        let before_log = log.clone();
        let apply_log = log.clone();
        let before_id = id.to_string();
        let apply_id = id.to_string();
        let node = runtime.register(
            id.to_string(),
            1,
            kind,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(move |change| {
                    before_log
                        .borrow_mut()
                        .push(format!("before:{before_id}:{}", change.open));
                    !before_veto.get()
                }),
                apply: Rc::new(move |change| {
                    apply.set(change.open);
                    apply_log
                        .borrow_mut()
                        .push(format!("apply:{apply_id}:{}", change.open));
                }),
                key: None,
            },
        );
        Model { open, veto, node }
    }
    fn report(
        runtime: &RuntimeHandle,
        sequence: u64,
        kind: &str,
        nodes: serde_json::Value,
    ) -> Report {
        serde_json::from_value(serde_json::json!({
            "owner":runtime.owner_id(),"sequence":sequence,"kind":kind,"nodes":nodes,
        }))
        .unwrap()
    }
    fn nodes(ancestors: &[&str]) -> serde_json::Value {
        serde_json::json!([
            {"id":"header","generation":1,"connected":true,"discovery_ready":true},
            {"id":"child","generation":1,"connected":true,"discovery_ready":true,"trigger_ancestors":ancestors}
        ])
    }

    #[test]
    fn parent_request_child_veto_and_more_outside_are_not_atomic() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let header = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child = model(&runtime, "child", OverlayKind::Dropdown, log.clone());
        runtime.observe(&report(&runtime, 1, "observe", nodes(&["header"])));
        assert!(header.node.request(true, Reason::Trigger, false));
        assert!(child.node.request(true, Reason::Trigger, false));
        log.borrow_mut().clear();
        child.veto.set(true);
        assert!(header.node.request(false, Reason::Trigger, false));
        assert_eq!(
            &*log.borrow(),
            &[
                "before:header:false",
                "before:child:false",
                "apply:header:false"
            ]
        );
        assert!(!header.open.get() && child.open.get());
        child.veto.set(false);
        assert!(header.node.request(true, Reason::Trigger, false));
        header.veto.set(true);
        assert!(!header.node.request(false, Reason::Trigger, false));
        let mut click = report(&runtime, 2, "click", nodes(&["header"]));
        click.triggers.push("header".into());
        click.default_prevented = true;
        runtime.observe(&click);
        assert!(
            header.open.get() && !child.open.get(),
            "outside child dismissal ignores original default and root veto"
        );
    }

    #[test]
    fn cached_discovery_history_controls_escape_and_header_focus_adapter() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let header = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child = model(&runtime, "child", OverlayKind::Dropdown, log);
        runtime.observe(&report(&runtime, 1, "observe", nodes(&[])));
        runtime.observe(&report(&runtime, 2, "observe", nodes(&["header"])));
        assert!(header.node.request(true, Reason::Trigger, false));
        assert!(runtime.child_path("header", &["header".into()], &["child".into()]));
        assert!(!runtime.key_child_path("header", &["header".into()], &["child".into()], "Escape"));
        assert!(!runtime.key_child_path("header", &["header".into()], &["child".into()], "Tab"));
        assert!(runtime.key_child_path("header", &["header".into()], &["child".into()], "Enter"));
        assert!(child.node.request(true, Reason::Trigger, false));
        assert!(runtime.key_child_path("header", &["header".into()], &["child".into()], "Escape"));
        child.veto.set(true);
        assert!(header.node.request(false, Reason::Trigger, false));
        child.veto.set(false);
        assert!(header.node.request(true, Reason::Trigger, false));
        assert!(
            !runtime.key_child_path("header", &["header".into()], &["child".into()], "Escape"),
            "newer topmost Header must retain flat Escape authority"
        );
        assert!(
            runtime.key_child_path("header", &["header".into()], &["child".into()], "Tab"),
            "Tab still defers to an open child"
        );
        let mut open_child = report(&runtime, 3, "key", nodes(&["header"]));
        open_child.phase = "bubble".into();
        open_child.key = "Escape".into();
        open_child.hosts = vec!["header".into()];
        open_child.triggers = vec!["child".into()];
        let raw = open_child
            .nodes
            .iter_mut()
            .find(|node| node.id == "child")
            .unwrap();
        raw.on_trigger = true;
        raw.mode = "active-descendant".into();
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("child")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(|_| panic!("newer topmost Header owns this Escape")));
        let decision = runtime.key(&open_child);
        assert!(!decision.stop && !decision.prevent);
        assert!(header.open.get() && child.open.get());
        assert!(child.node.request(false, Reason::Trigger, false));
        assert!(child.node.request(true, Reason::Trigger, false));
        assert!(runtime.dismiss_root("child", Reason::Escape, true));
        let mut closed = report(&runtime, 3, "key", nodes(&["header"]));
        closed.phase = "bubble".into();
        closed.key = "Escape".into();
        closed.hosts = vec!["header".into()];
        closed.triggers = vec!["child".into()];
        let node = closed
            .nodes
            .iter_mut()
            .find(|node| node.id == "child")
            .unwrap();
        node.on_trigger = true;
        node.mode = "active-descendant".into();
        // Install a callback without changing registration generation or graph.
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("child")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(|_| {
            panic!("closed Escape/Tab cannot reach child policy")
        }));
        let decision = runtime.key(&closed);
        assert!(!decision.stop && !decision.prevent);
        closed.key = "Tab".into();
        let decision = runtime.key(&closed);
        assert!(!decision.stop && !decision.prevent);
        assert!(
            header.open.get() && !child.open.get(),
            "relocation alone must not rewrite cached assignment"
        );
        header.node.unregister();
        child.node.unregister();
        let log = Rc::new(RefCell::new(Vec::new()));
        let header = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child = model(&runtime, "child", OverlayKind::Dropdown, log);
        runtime.observe(&report(&runtime, 3, "observe", nodes(&["header"])));
        assert!(header.node.request(true, Reason::Trigger, false));
        assert!(child.node.request(true, Reason::Trigger, false));
        assert!(runtime.dismiss_root("child", Reason::Escape, true));
        assert!(!header.open.get() && !child.open.get());
    }

    #[test]
    fn before_recursion_aba_and_forced_configuration_reject_superseded_work() {
        let runtime = RuntimeHandle::new();
        let open = Rc::new(Cell::new(true));
        let pending = Rc::new(RefCell::new(None::<NodeHandle>));
        let edit = Rc::new(Cell::new(true));
        let force_inside = Rc::new(Cell::new(false));
        let before_force = force_inside.clone();
        let read = open.clone();
        let before_node = pending.clone();
        let before_open = open.clone();
        let before_edit = edit.clone();
        let apply = open.clone();
        let applied = Rc::new(Cell::new(0));
        let count = applied.clone();
        let node = runtime.register(
            "node".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(move |_| {
                    let node = before_node.borrow().clone().unwrap();
                    assert!(
                        !node.request(false, Reason::Trigger, false),
                        "recursive before must be rejected"
                    );
                    if before_force.replace(false) {
                        assert!(node.request(false, Reason::Configuration, false));
                        return true;
                    }
                    if before_edit.replace(false) {
                        before_open.set(false);
                        node.sync_open(false);
                        before_open.set(true);
                        node.sync_open(true);
                    }
                    true
                }),
                apply: Rc::new(move |change| {
                    apply.set(change.open);
                    count.set(count.get() + 1);
                }),
                key: None,
            },
        );
        *pending.borrow_mut() = Some(node.clone());
        assert!(
            !node.request(false, Reason::Trigger, false),
            "ABA invalidates older command"
        );
        assert!(open.get() && applied.get() == 0);
        assert!(
            node.request(false, Reason::Trigger, false),
            "superseded before does not leave recursion guard stuck"
        );
        force_inside.set(true);
        assert!(
            !node.request(true, Reason::Trigger, false),
            "forced unchanged close supersedes pending opener"
        );
        assert!(!open.get());
        open.set(true);
        node.sync_open(true);
        assert!(
            node.request(false, Reason::Configuration, false),
            "configuration bypasses veto callback"
        );
        assert!(!open.get());

        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&runtime, "header", OverlayKind::Header, log.clone());
        let child_open = Rc::new(Cell::new(false));
        let read = child_open.clone();
        let apply = child_open.clone();
        let reopen = parent.node.clone();
        let child = runtime.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(move |change| {
                    if !change.open {
                        assert!(reopen.request(true, Reason::Trigger, false));
                    }
                    true
                }),
                apply: Rc::new(move |change| apply.set(change.open)),
                key: None,
            },
        );
        runtime.observe(&report(&runtime, 1, "observe", nodes(&["header"])));
        assert!(parent.node.request(true, Reason::Trigger, false));
        assert!(child.request(true, Reason::Trigger, false));
        log.borrow_mut().clear();
        assert!(
            !parent.node.request(false, Reason::Trigger, false),
            "unchanged reopen from child supersedes pending parent close"
        );
        assert!(parent.open.get() && !child_open.get());
        assert!(!log.borrow().contains(&"apply:header:false".into()));
    }

    struct RawForm(String);
    impl dioxus::html::HasFileData for RawForm {
        fn files(&self) -> Vec<dioxus::html::FileData> {
            Vec::new()
        }
    }
    impl dioxus::html::HasFormData for RawForm {
        fn value(&self) -> String {
            self.0.clone()
        }
        fn valid(&self) -> bool {
            true
        }
        fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
            Vec::new()
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    fn event(value: serde_json::Value) -> FormEvent {
        Event::new(
            Rc::new(dioxus::html::FormData::new(RawForm(value.to_string()))),
            true,
        )
    }
    fn key_value(runtime: &RuntimeHandle, sequence: u64, phase: &str) -> serde_json::Value {
        serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"key","phase":phase,"key":"Enter",
            "hosts":["child"],"triggers":["child"],"candidate_ids":["ix-dropdown-item-7"],
            "nodes":[{"id":"child","generation":1,"connected":true,"discovery_ready":true,"items":[7],"inside":true,"mode":"active-descendant"}]})
    }
    #[test]
    fn actual_transport_phases_default_stop_and_activation_are_once_only() {
        let runtime = RuntimeHandle::new();
        let open = Rc::new(Cell::new(true));
        let reads = open.clone();
        let count = Rc::new(Cell::new(0));
        let called = count.clone();
        let node = runtime.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(reads.get())),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: Some(Rc::new(move |facts| {
                    if facts.default_prevented {
                        return KeyDecision::default();
                    }
                    called.set(called.get() + 1);
                    KeyDecision {
                        prevent: true,
                        stop: true,
                        activate: Some("ix-dropdown-item-7".into()),
                        ..KeyDecision::default()
                    }
                })),
            },
        );
        let capture = event(key_value(&runtime, 1, "capture"));
        runtime.input(capture.clone());
        assert!(capture.default_action_enabled() && !capture.propagates() && count.get() == 0);
        let bubble = event(key_value(&runtime, 2, "bubble"));
        runtime.input(bubble.clone());
        assert!(!bubble.default_action_enabled() && count.get() == 1);
        let query_value = serde_json::json!({"owner":runtime.owner_id(),"sequence":2});
        let stop = event(query_value);
        runtime.query(stop.clone());
        assert!(!stop.default_action_enabled());
        let activation = serde_json::json!({"owner":runtime.owner_id(),"sequence":2,"activate_target":"ix-dropdown-item-7"});
        let first = event(activation.clone());
        runtime.query(first.clone());
        assert!(!first.default_action_enabled());
        let repeat = event(activation.clone());
        runtime.query(repeat.clone());
        assert!(repeat.default_action_enabled());
        runtime.input(event(key_value(&runtime, 2, "bubble")));
        assert_eq!(count.get(), 1, "duplicate action sequence rejected");
        runtime.input(event(key_value(&runtime, 3, "bubble")));
        node.unregister();
        let stale = event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":3,"activate_target":"ix-dropdown-item-7"}),
        );
        runtime.query(stale.clone());
        assert!(stale.default_action_enabled());

        // Model Dioxus web's fresh core MouseEvent metadata after an earlier
        // native capture prevention: the component needs the runtime guard.
        let log = Rc::new(RefCell::new(Vec::new()));
        let owner = model(&runtime, "outside", OverlayKind::Dropdown, log);
        owner.node.set_close_behavior(CloseBehavior::Outside);
        assert!(owner.node.request(true, Reason::Trigger, false));
        let mut capture = serde_json::json!({"owner":runtime.owner_id(),"sequence":4,"kind":"click","phase":"capture","hosts":["outside"],"triggers":["outside"],"nodes":[{"id":"outside","generation":1,"connected":true,"discovery_ready":true}]});
        let original = event(capture.clone());
        runtime.input(original.clone());
        assert!(!original.default_action_enabled() && !owner.node.trigger_click_allowed());
        if owner.node.trigger_click_allowed() {
            owner.node.request(false, Reason::Trigger, false);
        }
        assert!(owner.open.get());
        capture["sequence"] = 5.into();
        capture["phase"] = "bubble".into();
        capture["default_prevented"] = true.into();
        runtime.input(event(capture.clone()));
        assert!(owner.node.trigger_click_allowed() && owner.open.get());
        capture["sequence"] = 6.into();
        capture["phase"] = "capture".into();
        capture["triggers"] = serde_json::json!(["foreign", "outside"]);
        let foreign = event(capture);
        runtime.input(foreign.clone());
        assert!(
            foreign.default_action_enabled() && !owner.node.trigger_click_allowed(),
            "foreign trigger is not native-prevented while composed ancestor toggle is blocked"
        );

        // Convenience nested surfaces are actual composed descendants of their
        // ancestor wrapper. Its onclick must not toggle after a child trigger,
        // and an inner leaf still uses its own Both dismissal policy.
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let root = model(&runtime, "root", OverlayKind::Dropdown, log.clone());
        root.node.set_close_behavior(CloseBehavior::Outside);
        let sub = model(&runtime, "sub", OverlayKind::Dropdown, log);
        let facts = serde_json::json!([
            {"id":"root","generation":1,"connected":true,"discovery_ready":true},
            {"id":"sub","generation":1,"connected":true,"discovery_ready":true,"trigger_ancestors":["root"]}
        ]);
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe","nodes":facts})));
        assert!(root.node.request(true, Reason::Trigger, false));
        let mut nested = serde_json::json!({"owner":runtime.owner_id(),"sequence":2,"kind":"click","phase":"capture","hosts":["root"],"triggers":["sub","root"],"nodes":facts});
        let capture = event(nested.clone());
        runtime.input(capture.clone());
        assert!(
            capture.default_action_enabled()
                && sub.node.trigger_click_allowed()
                && !root.node.trigger_click_allowed()
        );
        assert!(sub.node.request(true, Reason::Trigger, false));
        if root.node.trigger_click_allowed() {
            root.node.request(false, Reason::Trigger, false);
        }
        assert!(root.open.get() && sub.open.get());
        nested["sequence"] = 3.into();
        nested["phase"] = "bubble".into();
        let bubble = event(nested.clone());
        runtime.input(bubble.clone());
        assert!(
            !bubble.default_action_enabled() && root.open.get() && sub.open.get(),
            "foreign child opens before original click protection, no ancestor toggle or leaf dismissal"
        );
        nested["sequence"] = 4.into();
        nested["phase"] = "capture".into();
        nested["hosts"] = serde_json::json!(["sub", "root"]);
        nested["triggers"] = serde_json::json!(["root"]);
        let leaf = event(nested.clone());
        runtime.input(leaf.clone());
        assert!(
            leaf.default_action_enabled() && !root.node.trigger_click_allowed(),
            "Outside ancestor must not preprevent its descendant's leaf"
        );
        nested["sequence"] = 5.into();
        nested["phase"] = "bubble".into();
        runtime.input(event(nested));
        assert!(
            !root.open.get() && !sub.open.get(),
            "nearest Both submenu dismisses eligible Outside root even with ancestor trigger in composed path"
        );

        let runtime = RuntimeHandle::new();
        let calls = Rc::new(Cell::new(0));
        let called = calls.clone();
        runtime.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(|| Some(true)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: Some(Rc::new(move |_| {
                    called.set(called.get() + 1);
                    KeyDecision {
                        prevent: true,
                        ..KeyDecision::default()
                    }
                })),
            },
        );
        let fallback = event(key_value(&runtime, 1, "fallback"));
        runtime.input(fallback.clone());
        assert!(
            fallback.default_action_enabled() && calls.get() == 0,
            "ordinary AD activation without actual trigger listener must not run document fallback policy"
        );
        let actual_trigger = event(key_value(&runtime, 2, "bubble"));
        runtime.input(actual_trigger.clone());
        assert!(
            !actual_trigger.default_action_enabled() && calls.get() == 1,
            "actual trigger bubble still owns AD activation"
        );
        let mut fallback = key_value(&runtime, 3, "fallback");
        fallback["nodes"][0]["mode"] = "roving-tabindex".into();
        let roving = event(fallback);
        runtime.input(roving.clone());
        assert!(
            roving.default_action_enabled() && calls.get() == 1,
            "roving must not run document fallback after capture"
        );
    }

    #[test]
    fn roving_tab_authorizes_current_adjacent_focus_once_and_weak_nodes_release_root() {
        let runtime = RuntimeHandle::new();
        let open = Rc::new(Cell::new(true));
        let pending = Rc::new(RefCell::new(None::<NodeHandle>));
        let read = open.clone();
        let apply = open.clone();
        let close = pending.clone();
        let node = runtime.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || Some(read.get())),
                before: Rc::new(|_| true),
                apply: Rc::new(move |change| apply.set(change.open)),
                key: Some(Rc::new(move |_| {
                    let node = close.borrow().clone().unwrap();
                    node.runtime()
                        .unwrap()
                        .dismiss_root(node.id(), Reason::Keyboard, false);
                    KeyDecision {
                        release_focus: true,
                        ..KeyDecision::default()
                    }
                })),
            },
        );
        *pending.borrow_mut() = Some(node.clone());
        let mut key = key_value(&runtime, 1, "capture");
        key["key"] = "Tab".into();
        key["tab_order"] = serde_json::json!(["before", "trigger", "after"]);
        key["nodes"][0]["mode"] = "roving-tabindex".into();
        key["nodes"][0]["trigger_id"] = "trigger".into();
        let input = event(key);
        runtime.input(input.clone());
        assert!(!input.default_action_enabled() && !open.get());
        let early = event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"focus_target":"after"}),
        );
        runtime.query(early.clone());
        assert!(
            early.default_action_enabled(),
            "release is not an immediate focus ACK"
        );
        let schedule = event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"schedule_focus_target":"after"}),
        );
        runtime.query(schedule.clone());
        assert!(!schedule.default_action_enabled());
        let value = serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"pending_focus_target":"after"});
        let first = event(value.clone());
        runtime.query(first.clone());
        assert!(!first.default_action_enabled());
        let repeat = event(value);
        runtime.query(repeat.clone());
        assert!(repeat.default_action_enabled());
        node.unregister();
        assert!(runtime.0.borrow().nodes.is_empty());
        let weak = Rc::downgrade(&runtime.0);
        drop(runtime);
        assert!(weak.upgrade().is_none() && node.runtime().is_none());
    }
}

#[cfg(test)]
#[path = "overlay_domain_runtime_tests.rs"]
mod document_tests;

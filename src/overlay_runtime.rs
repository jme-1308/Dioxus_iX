//! Owner-scoped dispatch with document-scoped Rust overlay coordination. Native code only reports current DOM
//! facts and executes an acknowledged default/stop/activation or focus intent.
use crate::dropdown_inventory::{
    AttributeWrite, HostFacts, InventoryFacts, ItemKind, ItemRef, RovingState,
};
use crate::overlay_completion::CompletionQueue;
use crate::overlay_descendant_focus::{
    DescendantHide, DescendantJob, DescendantPurpose, DescendantQueue,
};
use crate::overlay_domain::{Domain, DomainCallbacks};
use crate::overlay_focus_pending::{PendingFocus, PendingFocusQueue, TriggerRestoreQueue};
use crate::overlay_item_focus::ItemFocusJob;
use crate::overlay_item_focus::ItemFocusQueue;
use crate::overlay_registry::{CloseBehavior, NativeFact, OverlayKind, Registry};
use dioxus::prelude::*;
use serde::Deserialize;
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
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
    TriggerOther,
    AnchorVisibility,
    AnchorVisibilityParent,
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
    pub(crate) route: KeyRoute,
    pub(crate) navigation_mode: Option<String>,
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
    pub(crate) inventory: Option<InventoryFacts>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum KeyRoute {
    #[default]
    Legacy,
    Trigger,
    Navigation,
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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub(crate) struct AnchorRect {
    pub(crate) top: f64,
    pub(crate) bottom: f64,
    pub(crate) left: f64,
    pub(crate) right: f64,
}
impl AnchorRect {
    fn valid(self) -> bool {
        [self.top, self.bottom, self.left, self.right]
            .into_iter()
            .all(f64::is_finite)
            && self.top <= self.bottom
            && self.left <= self.right
    }
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub(crate) struct AnchorIntersection {
    pub(crate) bounding: AnchorRect,
    pub(crate) intersection: AnchorRect,
    #[serde(default)]
    pub(crate) anchor_in_dropdown_item: bool,
}
impl AnchorIntersection {
    fn valid(self) -> bool {
        self.bounding.valid() && self.intersection.valid()
    }
}
type AnchorIntersectionCallback = Rc<dyn Fn(AnchorIntersection)>;

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
    dialog_click: bool,
    root_click_without_host: bool,
    anchor_intersection: Option<AnchorIntersectionCallback>,
    anchor_shape: Option<Rc<dyn Fn(bool)>>,
    item_visual: Option<Rc<dyn Fn() -> RootVisual>>,
    descendant_focus: Option<DescendantFocusCallback>,
    submenu: Option<SubmenuAccess>,
    navigation: Option<NavigationConfiguration>,
    suppress_trigger_restore: bool,
    callbacks: Callbacks,
    scope: Option<(Weak<dioxus::core::Runtime>, ScopeId)>,
}
#[derive(Clone)]
struct NavigationConfiguration {
    version: u64,
    mode: String,
    enabled: bool,
    capture: Option<String>,
    sent: bool,
}
#[derive(Clone, Deserialize)]
struct NavigationBinding {
    version: u64,
    capture: String,
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
    trigger_restores: TriggerRestoreQueue,
    completions: CompletionQueue,
    item_focus: ItemFocusQueue,
    item_focus_callbacks: BTreeMap<u64, ItemFocusCallback>,
    item_visual_deliveries: BTreeMap<u64, (ItemFocusJob, HostFacts)>,
    descendant_focus: DescendantQueue,
    descendant_visual: BTreeMap<u64, (DescendantJob, bool)>,
    submenu_marks: BTreeMap<String, (String, u64)>,
    submenu_own: BTreeMap<String, (u64, InventoryFacts)>,
    item_captures: BTreeMap<String, (u64, BTreeMap<String, String>)>,
    inventories: BTreeMap<String, (u64, InventoryFacts)>,
    roving: RovingState,
    roving_hosts: BTreeMap<(String, u64), Vec<ItemRef>>,
    pending_roving_hosts: Vec<RovingEnrollment>,
    roving_sequence: u64,
    roving_writes: VecDeque<RovingBatch>,
    item_visual: Option<String>,
    item_visual_target: Option<String>,
    item_visual_known: bool,
    item_cues_dirty: bool,
    item_visual_host: Option<HostFacts>,
    item_cue_hosts: crate::item_visual::CueHosts,
    item_root_captures: BTreeMap<String, (u64, String, String)>,
    item_visual_signal: Option<Signal<Option<String>>>,
    bridge: Option<dioxus::document::Eval>,
    domain: Option<Domain>,
    document: String,
    categories: BTreeMap<usize, CategoryCallback>,
}
struct ItemFocusCallback {
    root: String,
    generation: u64,
    callback: Rc<dyn Fn()>,
    visual: Option<HostFacts>,
}
type DescendantFocusCallback = Rc<dyn Fn(&InventoryFacts, bool, bool, bool) -> Option<bool>>;

#[derive(Clone)]
struct SubmenuAccess {
    handle: crate::dropdown::DropdownHandle,
    query_root: Rc<dyn Fn() -> Option<String>>,
    reset: Rc<dyn Fn()>,
    trigger_click: Rc<dyn Fn()>,
}

#[derive(Deserialize)]
struct SubmenuMarkerOffer {
    root: String,
    generation: u64,
    root_capture: String,
    capture: String,
    active: bool,
}

#[derive(Deserialize)]
struct DescendantQuery {
    sequence: u64,
    root: String,
    generation: u64,
    reference: String,
    phase: String,
    #[serde(default)]
    hide_before: Vec<DescendantHide>,
    #[serde(default)]
    facts: Option<crate::descendant_focus::DescendantFacts>,
    #[serde(default)]
    inventory: Option<InventoryFacts>,
    #[serde(default)]
    hosts: Vec<HostFacts>,
    #[serde(default)]
    attempt_path: Vec<String>,
    #[serde(default)]
    target: Option<String>,
}

#[derive(Clone, Default)]
pub(crate) struct RootVisual {
    pub(crate) active: Option<String>,
    pub(crate) aria: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RootCueField {
    Active,
    Aria,
    TriggerAria,
}
#[derive(Deserialize)]
struct RootCueOffer {
    root: String,
    generation: u64,
    capture: String,
    #[serde(default)]
    trigger_capture: Option<String>,
    field: RootCueField,
    value: Option<String>,
}
struct RovingEnrollment {
    root: String,
    generation: u64,
    query_root: String,
    query_generation: u64,
}
struct RovingBatch {
    sequence: u64,
    event_sequence: u64,
    root: String,
    generation: u64,
    writes: VecDeque<AttributeWrite>,
    sent: bool,
}
type CategoryCallback = Rc<dyn Fn(usize, &[usize])>;

impl Drop for Runtime {
    fn drop(&mut self) {
        self.pending_focus.clear();
        self.trigger_restores.clear();
        self.completions.clear();
        self.item_focus.clear();
        self.item_focus_callbacks.clear();
        self.item_captures.clear();
        self.inventories.clear();
        self.roving.clear_all();
        self.roving_hosts.clear();
        self.pending_roving_hosts.clear();
        self.roving_writes.clear();
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
            trigger_restores: TriggerRestoreQueue::default(),
            completions: CompletionQueue::default(),
            item_focus: ItemFocusQueue::default(),
            item_focus_callbacks: BTreeMap::new(),
            item_visual_deliveries: BTreeMap::new(),
            descendant_focus: DescendantQueue::default(),
            descendant_visual: BTreeMap::new(),
            submenu_marks: BTreeMap::new(),
            submenu_own: BTreeMap::new(),
            item_captures: BTreeMap::new(),
            inventories: BTreeMap::new(),
            roving: RovingState::default(),
            roving_hosts: BTreeMap::new(),
            pending_roving_hosts: Vec::new(),
            roving_sequence: 0,
            roving_writes: VecDeque::new(),
            item_visual: None,
            item_visual_target: None,
            item_visual_known: false,
            item_cues_dirty: false,
            item_visual_host: None,
            item_cue_hosts: Default::default(),
            item_root_captures: BTreeMap::new(),
            item_visual_signal: dioxus::core::Runtime::try_current()
                .map(|_| Signal::new_in_scope(None, ScopeId::ROOT)),
            bridge: None,
            domain: None,
            document: String::new(),
            categories: BTreeMap::new(),
        })))
    }
    pub(crate) fn owner_id(&self) -> String {
        self.0.borrow().owner.clone()
    }
    pub(crate) fn item_visual_key(&self) -> Option<String> {
        let (signal, value) = {
            let state = self.0.borrow();
            (state.item_visual_signal, state.item_visual.clone())
        };
        signal
            .and_then(|signal| signal.try_read().ok().map(|value| value.clone()))
            .unwrap_or(value)
    }
    pub(crate) fn item_visual_target(&self) -> Option<String> {
        let _ = self.item_visual_key();
        self.0.borrow().item_visual_target.clone()
    }
    pub(crate) fn register_item_cue(&self, key: String, kind: ItemKind) {
        self.0.borrow_mut().item_cue_hosts.register(key, kind);
    }
    pub(crate) fn unregister_item_cue(&self, key: &str) {
        self.0.borrow_mut().item_cue_hosts.unregister(key);
    }
    pub(crate) fn item_visual_matches(&self, host: &HostFacts) -> bool {
        let state = self.0.borrow();
        state.item_visual_host.as_ref().is_some_and(|current| {
            current.capture == host.capture
                && current.registration_key == host.registration_key
                && current.kind == host.kind
                && state.item_visual_target.as_ref() == Some(&host.target)
        })
    }
    fn set_item_visual(
        &self,
        key: Option<String>,
        target: Option<String>,
        host: Option<HostFacts>,
    ) {
        let signal = {
            let mut state = self.0.borrow_mut();
            let was_known = state.item_visual_known;
            state.item_visual_known = true;
            if was_known
                && state.item_visual == key
                && state.item_visual_target == target
                && state.item_visual_host == host
            {
                return;
            }
            state.item_visual = key.clone();
            state.item_cues_dirty = true;
            state.item_visual_target = target;
            state.item_visual_host = host;
            state.item_visual_signal
        };
        if let Some(mut signal) = signal
            && let Ok(mut value) = signal.try_write()
        {
            *value = key;
        }
        let bridge = self.0.borrow().bridge;
        if let Some(bridge) = bridge {
            let _ = bridge.send(serde_json::json!({"item_cues":true}));
        }
    }
    pub(crate) fn clear_item_visual(&self) {
        self.set_item_visual(None, None, None);
    }
    pub(crate) fn mark_item_visual(&self, host: &HostFacts) {
        if !self.keyboard_mode() {
            return;
        }
        // Source focusItem projects Category into its private parent; that
        // parent's marker never makes the outer Category an AD active host.
        let key = (host.source_focusable && host.kind != ItemKind::Category).then(|| {
            if host.registration_key.is_empty() {
                host.capture.clone()
            } else {
                host.registration_key.clone()
            }
        });
        let target = key.as_ref().map(|_| host.target.clone());
        self.0.borrow_mut().item_cue_hosts.observe(host);
        self.set_item_visual(key, target, Some(host.clone()));
    }
    fn projected_inventory(&self, mut facts: InventoryFacts) -> InventoryFacts {
        let state = self.0.borrow();
        for host in &mut facts.hosts {
            if let Some(write) = state
                .roving_writes
                .iter()
                .rev()
                .flat_map(|batch| batch.writes.iter().rev())
                .find(|write| write.item == host.item)
            {
                host.tabindex_attribute = write.tabindex.clone();
            }
            if state.item_visual_known {
                let key = if host.registration_key.is_empty() {
                    &host.capture
                } else {
                    &host.registration_key
                };
                host.visible_focus = host.source_focusable
                    && host.kind != ItemKind::Category
                    && state.item_visual.as_ref() == Some(key);
            }
        }
        facts
    }
    fn queue_roving_writes(&self, root: &str, generation: u64, writes: Vec<AttributeWrite>) {
        if writes.is_empty() {
            return;
        }
        {
            let mut state = self.0.borrow_mut();
            loop {
                state.roving_sequence = state.roving_sequence.wrapping_add(1).max(1);
                if !state
                    .roving_writes
                    .iter()
                    .any(|batch| batch.sequence == state.roving_sequence)
                {
                    break;
                }
            }
            let sequence = state.roving_sequence;
            let event_sequence = state.sequence;
            state.roving_writes.push_back(RovingBatch {
                sequence,
                event_sequence,
                root: root.into(),
                generation,
                writes: writes.into(),
                sent: false,
            });
        }
        self.flush_roving_writes();
    }
    fn flush_roving_writes(&self) {
        let commands = {
            let state = self.0.borrow();
            state.roving_writes.iter().filter(|batch| !batch.sent).map(|batch| {
                (batch.sequence, serde_json::json!({"roving_attributes":{
                    "sequence":batch.sequence,"root":batch.root,"generation":batch.generation,"writes":batch.writes
                }}))
            }).collect::<Vec<_>>()
        };
        for (sequence, command) in commands {
            if !self
                .0
                .borrow()
                .bridge
                .as_ref()
                .is_some_and(|bridge| bridge.send(command).is_ok())
            {
                break;
            }
            if let Some(batch) = self
                .0
                .borrow_mut()
                .roving_writes
                .iter_mut()
                .find(|batch| batch.sequence == sequence)
            {
                batch.sent = true;
            }
        }
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
                runtime.trigger_restores.remove_root(&id, old_generation);
                runtime.completions.remove_root(&id, old_generation);
                runtime.item_focus.remove_root(&id, old_generation);
                runtime.descendant_focus.remove_root(&id, old_generation);
                runtime
                    .submenu_marks
                    .retain(|_, (root, generation)| root != &id || *generation != old_generation);
                runtime.submenu_own.remove(&id);
                runtime
                    .descendant_visual
                    .retain(|_, (job, _)| job.root != id || job.generation != old_generation);
                runtime
                    .item_visual_deliveries
                    .retain(|_, (job, _)| job.root != id || job.generation != old_generation);
                runtime.item_focus_callbacks.retain(|_, callback| {
                    callback.root != id || callback.generation != old_generation
                });
                runtime.item_captures.remove(&id);
                runtime.item_root_captures.remove(&id);
                runtime.inventories.remove(&id);
                runtime.pending_roving_hosts.retain(|admission| {
                    admission.root != id || admission.generation != old_generation
                });
                if let Some(hosts) = runtime.roving_hosts.remove(&(id.clone(), old_generation)) {
                    for host in hosts {
                        runtime.roving.clear(&host);
                    }
                }
                runtime
                    .roving_writes
                    .retain(|batch| batch.root != id || batch.generation != old_generation);
            }
            let revision = runtime
                .nodes
                .get(&id)
                .map_or(0, |entry| entry.revision.wrapping_add(1));
            let suppress_trigger_restore = runtime.nodes.get(&id).is_some_and(|entry| {
                entry.generation == generation && entry.suppress_trigger_restore
            });
            let navigation = runtime
                .nodes
                .get(&id)
                .filter(|entry| entry.generation == generation)
                .and_then(|entry| entry.navigation.clone());
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
                    dialog_click: false,
                    root_click_without_host: false,
                    anchor_intersection: None,
                    anchor_shape: None,
                    item_visual: None,
                    descendant_focus: None,
                    submenu: None,
                    navigation,
                    suppress_trigger_restore,
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
        let suppress_runtime = read_runtime.clone();
        let read_id = id.to_string();
        let request_id = read_id.clone();
        let suppress_id = read_id.clone();
        domain.register(
            owner,
            id.to_string(),
            generation,
            entry.kind,
            entry.behavior,
            DomainCallbacks {
                suppress_trigger_restore: Rc::new(move || {
                    let Some(runtime) = suppress_runtime.upgrade().map(RuntimeHandle) else {
                        return false;
                    };
                    runtime.mark_trigger_restore_suppressed(&suppress_id, generation)
                }),
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
            state.trigger_restores.clear();
            // Initial native attachment must retain pre-bridge notifications.
            // Adopting another document cancels old-document frame work as an
            // explicit renderer-lifetime adapter.
            if !state.document.is_empty() {
                state.completions.clear();
                state.item_focus.clear();
                state.item_focus_callbacks.clear();
                state.item_visual_deliveries.clear();
                state.descendant_focus.clear();
                state.submenu_marks.clear();
                state.submenu_own.clear();
                state.descendant_visual.clear();
                state.item_captures.clear();
                state.inventories.clear();
                state.item_cue_hosts.reset_captures();
                state.item_root_captures.clear();
                state.item_visual_host = None;
                state.item_visual = None;
                state.item_visual_target = None;
                state.item_visual_known = false;
                state.item_cues_dirty = false;
                state.roving.clear_all();
                state.roving_hosts.clear();
                state.pending_roving_hosts.clear();
                state.roving_writes.clear();
                for entry in state.nodes.values_mut() {
                    if let Some(configuration) = &mut entry.navigation {
                        configuration.version = configuration.version.wrapping_add(1).max(1);
                        configuration.capture = None;
                        configuration.sent = false;
                    }
                }
            }
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
    fn mark_trigger_restore_suppressed(&self, id: &str, generation: u64) -> bool {
        self.with_owner(id, generation, || {
            let Some(snapshot) = self.entry(id).filter(|entry| entry.kind.is_dropdown()) else {
                return false;
            };
            if self.read_current(id, &snapshot) != Some(true) {
                return false;
            }
            let mut state = self.0.borrow_mut();
            let Some(entry) = state.nodes.get_mut(id).filter(|entry| {
                entry.generation == generation && entry.revision == snapshot.revision
            }) else {
                return false;
            };
            entry.suppress_trigger_restore = true;
            true
        })
        .unwrap_or(false)
    }

    /// Source dismissOthers after an actual trigger or host fallthrough.
    /// Closed origins are valid; a dropped/replaced registration is not.
    pub(crate) fn dismiss_dropdown_others(&self, id: &str, generation: u64) -> bool {
        self.dismiss_dropdown_set(id, generation, false)
    }

    fn dismiss_dropdown_set(&self, id: &str, generation: u64, inside: bool) -> bool {
        let admitted = self.with_owner(id, generation, || {
            let entry = self.entry(id)?;
            (entry.kind.is_dropdown() && entry.generation == generation)
                .then(|| self.read_current(id, &entry))
                .flatten()
        });
        if admitted.flatten().is_none() {
            return false;
        }
        if let Some(domain) = self.domain() {
            return if inside {
                domain.dismiss_dropdown_all(id, generation)
            } else {
                domain.dismiss_dropdown_others(id, generation)
            };
        }
        // Reconcile current Rust presence before capturing live ancestry.
        let ids = self.0.borrow().order.clone();
        for candidate in ids {
            let Some(entry) = self.entry(&candidate) else {
                continue;
            };
            if let Some(open) = self
                .with_owner(&candidate, entry.generation, || {
                    self.read_current(&candidate, &entry)
                })
                .flatten()
            {
                self.0.borrow_mut().registry.set_presence(&candidate, open);
            }
        }
        let excluded = if inside {
            Vec::new()
        } else {
            self.0.borrow().registry.dropdown_dismiss_exclusions(id)
        };
        let mut visited = std::collections::BTreeSet::new();
        loop {
            let next = self
                .0
                .borrow()
                .registry
                .next_dropdown_dismiss_other(&visited);
            let Some((candidate, candidate_generation, registration)) = next else {
                break;
            };
            visited.insert(registration);
            if excluded.contains(&candidate) {
                continue;
            }
            self.with_owner(&candidate, candidate_generation, || {
                let Some(entry) = self.entry(&candidate).filter(|entry| {
                    entry.generation == candidate_generation
                        && entry.kind.is_dropdown()
                        && (inside && candidate == id || entry.behavior.allows_outside())
                }) else {
                    return;
                };
                // Closed instances do not receive even an unchanged request:
                // that would incorrectly advance their operation revision.
                if self.read_current(&candidate, &entry) == Some(true) {
                    self.request(
                        &candidate,
                        candidate_generation,
                        Change {
                            open: false,
                            reason: if inside {
                                Reason::Inside
                            } else {
                                Reason::TriggerOther
                            },
                            restore_focus: !inside || self.keyboard_mode(),
                        },
                    );
                }
            });
        }
        true
    }

    pub(crate) fn close_and_release_focus(&self, id: &str) -> bool {
        if let Some(domain) = self.domain() {
            return domain.close_and_release_focus(id);
        }
        let Some(root) = self.0.borrow().registry.cached_root(id) else {
            return false;
        };
        let Some(entry) = self.entry(&root).filter(|entry| entry.kind.is_dropdown()) else {
            return false;
        };
        self.suppress_cached_trigger_restore(&root, &mut Vec::new());
        self.with_owner(&root, entry.generation, || {
            self.request(
                &root,
                entry.generation,
                Change {
                    open: false,
                    reason: Reason::Keyboard,
                    restore_focus: false,
                },
            )
        })
        .unwrap_or(false)
    }

    fn suppress_cached_trigger_restore(&self, id: &str, visited: &mut Vec<String>) {
        if visited.iter().any(|seen| seen == id) {
            return;
        }
        visited.push(id.to_owned());
        let Some(entry) = self.entry(id) else {
            return;
        };
        if !self.mark_trigger_restore_suppressed(id, entry.generation) {
            return;
        }
        let children = self.0.borrow().registry.cached_children(id);
        for child in children {
            self.suppress_cached_trigger_restore(&child, visited);
        }
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
                match change.reason {
                    Reason::Drop => Reason::Drop,
                    Reason::AnchorVisibility | Reason::AnchorVisibilityParent => {
                        Reason::AnchorVisibilityParent
                    }
                    Reason::TriggerOther => Reason::TriggerOther,
                    _ => Reason::Parent,
                },
                change.restore_focus,
            );
        }
        if !self.current(id, generation, operation)
            || self.read_current(id, &current) != Some(previous)
        {
            return false;
        }
        let applied = if previous && !change.open && snapshot.kind.is_dropdown() {
            let ordinary_cleanup = matches!(
                change.reason,
                Reason::AnchorVisibility | Reason::AnchorVisibilityParent | Reason::TriggerOther
            );
            // Each accepted source cleanup observes modality after its own
            // before/cascade, and consumes its CURRENT latch even in pointer
            // mode. Veto and stale operations never reach this point.
            let restore = if ordinary_cleanup {
                self.keyboard_mode()
            } else {
                change.restore_focus
            };
            let suppressed =
                {
                    let mut state = self.0.borrow_mut();
                    let Some(entry) = state.nodes.get_mut(id).filter(|entry| {
                        entry.generation == generation && entry.revision == operation
                    }) else {
                        return false;
                    };
                    std::mem::take(&mut entry.suppress_trigger_restore)
                };
            Change {
                restore_focus: restore
                    && !suppressed
                    && !matches!(change.reason, Reason::Configuration | Reason::Drop),
                ..change
            }
        } else {
            change
        };
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
        (snapshot.callbacks.apply)(applied);
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
                    restore_focus: matches!(
                        reason,
                        Reason::AnchorVisibilityParent | Reason::TriggerOther
                    ) && self
                        .entry(&child)
                        .is_some_and(|entry| entry.kind.is_dropdown()),
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
            let parent_entry = self.entry(&parent);
            let marker = parent_entry
                .as_ref()
                .and_then(|entry| self.first_submenu_marker(&parent, entry.generation));
            let children = self.domain().map_or_else(
                || self.0.borrow().registry.cached_children(&parent),
                |domain| domain.cached_children(&parent),
            );
            let mut hides = Vec::new();
            for child in children {
                let closing = self.entry(&child).and_then(|entry| {
                    (self.read_current(&child, &entry) == Some(true)).then_some(entry)
                });
                let accepted = self.request_id(
                    &child,
                    Change {
                        open: false,
                        reason: Reason::Left,
                        restore_focus: true,
                    },
                );
                if accepted
                    && let Some(closing) = closing
                    && let Some(current) = self.entry(&child)
                    && current.generation == closing.generation
                    && self.read_current(&child, &current) == Some(false)
                    && let Some((_, reference, _)) = self
                        .0
                        .borrow()
                        .item_root_captures
                        .get(&child)
                        .filter(|(generation, _, _)| *generation == current.generation)
                {
                    hides.push(DescendantHide {
                        root: child,
                        generation: current.generation,
                        operation: current.revision,
                        reference: reference.clone(),
                    });
                }
            }
            if let (Some(capture), Some(entry)) = (marker, parent_entry) {
                self.0.borrow_mut().submenu_marks.remove(&capture);
                self.wake_submenu_markers();
                if let Some(access) = entry.submenu {
                    self.with_owner(&parent, entry.generation, || (access.reset)());
                }
                self.queue_submenu_return_after_hide(&parent, entry.generation, capture, hides);
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
            runtime.trigger_restores.remove_root(id, generation);
            runtime.completions.remove_root(id, generation);
            runtime.item_focus.remove_root(id, generation);
            runtime.descendant_focus.remove_root(id, generation);
            runtime
                .submenu_marks
                .retain(|_, (root, version)| root != id || *version != generation);
            runtime.submenu_own.remove(id);
            runtime
                .descendant_visual
                .retain(|_, (job, _)| job.root != id || job.generation != generation);
            runtime
                .item_visual_deliveries
                .retain(|_, (job, _)| job.root != id || job.generation != generation);
            runtime
                .item_focus_callbacks
                .retain(|_, callback| callback.root != id || callback.generation != generation);
            runtime.item_captures.remove(id);
            runtime.item_root_captures.remove(id);
            runtime.inventories.remove(id);
            runtime
                .pending_roving_hosts
                .retain(|admission| admission.root != id || admission.generation != generation);
            if let Some(hosts) = runtime.roving_hosts.remove(&(id.into(), generation)) {
                for host in hosts {
                    runtime.roving.clear(&host);
                }
            }
            runtime
                .roving_writes
                .retain(|batch| batch.root != id || batch.generation != generation);
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
            &crate::overlay_registry_dom::BRIDGE
                .replace(
                    "__DESCENDANT_PLUMBING__",
                    crate::descendant_focus_dom::PLUMBING,
                )
                .replace("__OWNER__", &owner),
        );
        self.0.borrow_mut().bridge = Some(bridge);
        self.flush_navigation_configurations();
        self.flush_completion_frames();
        self.flush_roving_writes();
        self.flush_item_focus();
        self.flush_descendant_focus();
    }
    fn configure_navigation(&self, id: &str, generation: u64, mode: &str, enabled: bool) -> bool {
        if !matches!(mode, "active-descendant" | "roving-tabindex") {
            return false;
        }
        let Some(entry) = self
            .entry(id)
            .filter(|entry| entry.generation == generation && entry.kind == OverlayKind::Dropdown)
        else {
            return false;
        };
        let roving_root = if enabled && mode == "roving-tabindex" {
            let root = self.with_owner(id, generation, || {
                entry
                    .submenu
                    .as_ref()
                    .and_then(|access| (access.query_root)())
                    .unwrap_or_else(|| id.into())
            });
            root.and_then(|root| self.entry(&root).map(|entry| (root, entry.generation)))
        } else {
            None
        };
        {
            let mut state = self.0.borrow_mut();
            let Some(entry) = state.nodes.get_mut(id).filter(|entry| {
                entry.generation == generation && entry.kind == OverlayKind::Dropdown
            }) else {
                return false;
            };
            let version = entry.navigation.as_ref().map_or(1, |configuration| {
                configuration.version.wrapping_add(1).max(1)
            });
            entry.navigation = Some(NavigationConfiguration {
                version,
                mode: mode.into(),
                enabled,
                capture: None,
                sent: false,
            });
            if let Some((query_root, query_generation)) = roving_root {
                state.pending_roving_hosts.push(RovingEnrollment {
                    root: id.into(),
                    generation,
                    query_root,
                    query_generation,
                });
            }
        }
        self.enroll_configured_roving_hosts();
        self.flush_navigation_configurations();
        true
    }
    fn enroll_configured_roving_hosts(&self) {
        let mut state = self.0.borrow_mut();
        let pending = std::mem::take(&mut state.pending_roving_hosts);
        for admission in pending {
            if !state
                .nodes
                .get(&admission.root)
                .is_some_and(|entry| entry.generation == admission.generation)
                || !state
                    .nodes
                    .get(&admission.query_root)
                    .is_some_and(|entry| entry.generation == admission.query_generation)
            {
                continue;
            }
            let host = state
                .submenu_own
                .get(&admission.query_root)
                .filter(|(version, facts)| {
                    *version == admission.query_generation && !facts.items_host.0.is_empty()
                })
                .map(|(_, facts)| facts.items_host.clone());
            let Some(host) = host else {
                state.pending_roving_hosts.push(admission);
                continue;
            };
            let hosts = state
                .roving_hosts
                .entry((admission.root, admission.generation))
                .or_default();
            if !hosts.contains(&host) {
                hosts.push(host);
            }
        }
    }
    fn flush_navigation_configurations(&self) {
        let commands =
            {
                let state = self.0.borrow();
                state.order.iter().filter_map(|id| {
                let entry = state.nodes.get(id)?;
                let configuration = entry.navigation.as_ref().filter(|config| !config.sent)?;
                Some((id.clone(), entry.generation, configuration.version,
                    serde_json::json!({"navigation_config":{
                        "root":id,"generation":entry.generation,"version":configuration.version,
                        "mode":configuration.mode,"enabled":configuration.enabled
                    }})))
            }).collect::<Vec<_>>()
            };
        for (id, generation, version, command) in commands {
            if !self
                .0
                .borrow()
                .bridge
                .as_ref()
                .is_some_and(|bridge| bridge.send(command).is_ok())
            {
                break;
            }
            if let Some(configuration) = self
                .0
                .borrow_mut()
                .nodes
                .get_mut(&id)
                .filter(|entry| entry.generation == generation)
                .and_then(|entry| entry.navigation.as_mut())
                .filter(|configuration| configuration.version == version)
            {
                configuration.sent = true;
            }
        }
    }
    fn flush_item_focus(&self) {
        let jobs = self.0.borrow().item_focus.unsent();
        for job in jobs {
            let sent =
                self.0.borrow().bridge.as_ref().is_some_and(|bridge| {
                    bridge.send(serde_json::json!({"item_focus":{
                    "sequence":job.sequence,"root":job.root,"generation":job.generation,
                    "target":job.target,"focus_native":job.focus_native,"capture":job.capture
                }})).is_ok()
                });
            if !sent {
                break;
            }
            self.0.borrow_mut().item_focus.mark_sent(&job);
        }
    }
    fn flush_descendant_focus(&self) {
        let jobs = self.0.borrow().descendant_focus.unsent();
        for job in jobs {
            let sent = self.0.borrow().bridge.as_ref().is_some_and(|bridge| {
                bridge
                    .send(serde_json::json!({"descendant_focus":{
                        "sequence":job.sequence,"root":job.root,"generation":job.generation,
                        "focus_version":job.focus_version,"last":job.last,
                        "allow_checked":job.allow_checked,"reference":job.reference,"purpose":job.purpose,
                        "hide_before":job.hide_before
                    }}))
                    .is_ok()
            });
            if !sent {
                break;
            }
            self.0.borrow_mut().descendant_focus.mark_sent(&job);
        }
    }
    #[cfg(test)]
    pub(crate) fn pending_item_focus_jobs(&self) -> Vec<ItemFocusJob> {
        self.0.borrow().item_focus.jobs()
    }
    fn flush_completion_frames(&self) {
        let frames = self.0.borrow().completions.unsent();
        for frame in frames {
            let sent =
                {
                    let state = self.0.borrow();
                    let Some(bridge) = &state.bridge else { return };
                    bridge.send(serde_json::json!({"completion_frame":{
                    "sequence":frame.sequence,"root":frame.root,"generation":frame.generation
                }})).is_ok()
                };
            if !sent {
                // Keep this and later admissions for a subsequent bridge;
                // never turn a failed native enrollment into sync delivery.
                break;
            }
            self.0.borrow_mut().completions.mark_sent(&frame);
        }
    }
    #[cfg(test)]
    pub(crate) fn pending_completion_frames(&self) -> Vec<(u64, String, u64)> {
        self.0
            .borrow()
            .completions
            .frames()
            .into_iter()
            .map(|frame| (frame.sequence, frame.root, frame.generation))
            .collect()
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
        {
            let mut runtime = self.0.borrow_mut();
            for node in &report.nodes {
                if runtime
                    .nodes
                    .get(&node.id)
                    .is_some_and(|entry| node.connected && entry.generation == node.generation)
                {
                    runtime.submenu_own.insert(
                        node.id.clone(),
                        (
                            node.generation,
                            node.own_inventory
                                .as_ref()
                                .or(node.inventory.as_ref())
                                .cloned()
                                .unwrap_or_else(|| InventoryFacts {
                                    items_host: ItemRef(String::new()),
                                    hosts: Vec::new(),
                                    navigation_order: Vec::new(),
                                    query_order: Vec::new(),
                                    slots: None,
                                }),
                        ),
                    );
                    let captures = runtime
                        .item_root_captures
                        .entry(node.id.clone())
                        .or_insert_with(|| {
                            (
                                node.generation,
                                node.root_capture.clone(),
                                node.trigger_capture.clone(),
                            )
                        });
                    if captures.0 != node.generation || captures.1.is_empty() {
                        *captures = (
                            node.generation,
                            node.root_capture.clone(),
                            node.trigger_capture.clone(),
                        );
                    } else if captures.1 == node.root_capture {
                        captures.2 = node.trigger_capture.clone();
                    }
                    let mut captures = node
                        .item_focus_captures
                        .iter()
                        .filter(|capture| !capture.target.is_empty() && !capture.capture.is_empty())
                        .map(|capture| (capture.target.clone(), capture.capture.clone()))
                        .collect::<BTreeMap<_, _>>();
                    if let Some(inventory) = &node.inventory {
                        for host in &inventory.hosts {
                            runtime.item_cue_hosts.observe(host);
                        }
                        captures.extend(
                            inventory
                                .hosts
                                .iter()
                                .map(|host| (host.target.clone(), host.capture.clone())),
                        );
                        runtime
                            .inventories
                            .insert(node.id.clone(), (node.generation, inventory.clone()));
                    }
                    runtime
                        .item_captures
                        .insert(node.id.clone(), (node.generation, captures));
                    if let Some(binding) = &node.navigation_binding
                        && !binding.capture.is_empty()
                        && !node.root_capture.is_empty()
                        && runtime.item_root_captures.get(&node.id).is_some_and(
                            |(generation, root, _)| {
                                *generation == node.generation && *root == node.root_capture
                            },
                        )
                        && let Some(configuration) = runtime
                            .nodes
                            .get_mut(&node.id)
                            .and_then(|entry| entry.navigation.as_mut())
                            .filter(|configuration| {
                                configuration.enabled
                                    && configuration.version == binding.version
                                    && configuration.capture.is_none()
                            })
                    {
                        configuration.capture = Some(binding.capture.clone());
                    }
                }
            }
        }
        self.enroll_configured_roving_hosts();
        self.flush_navigation_configurations();
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
                    assignment_ancestors: node.and_then(|node| node.assignment_ancestors.clone()),
                    assignment_emissions: node.and_then(|node| node.assignment_emissions.clone()),
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
                        assignment_ancestors: node.assignment_ancestors.clone(),
                        assignment_emissions: node.assignment_emissions.clone(),
                        trigger_ancestors: node.trigger_ancestors.clone(),
                    })
                    .collect::<Vec<_>>();
                domain.observe_all(&all);
            } else {
                domain.observe_owner(&report.owner, &facts);
            }
        }
        // Both registries now own the immutable records, including ancestry
        // awaiting a foreign owner's readiness. Native may release its copies
        // only after this admission, never merely after sending a report.
        for node in &report.nodes {
            if !node.connected
                || !self
                    .entry(&node.id)
                    .is_some_and(|entry| entry.generation == node.generation)
            {
                continue;
            }
            let Some(sequence) = node
                .assignment_emissions
                .as_ref()
                .and_then(|records| records.iter().map(|record| record.sequence).max())
                .filter(|sequence| *sequence > 0)
            else {
                continue;
            };
            if let Some(bridge) = self.0.borrow().bridge.as_ref() {
                let _ = bridge.send(serde_json::json!({"assignment_ack":{
                    "root":node.id,"generation":node.generation,"sequence":sequence
                }}));
            }
        }
        if report.kind == "modality" {
            if let Some(domain) = self.domain() {
                domain.observe_input(&report.input_kind, &report.key);
            }
            if matches!(report.input_kind.as_str(), "mousedown" | "touchstart") {
                self.clear_item_visual();
            }
        }
        if report.kind == "item-focusin" {
            let host = report
                .item_focus_host
                .as_ref()
                .filter(|host| self.0.borrow_mut().item_cue_hosts.observe(host));
            if let Some(host) = host {
                self.mark_item_visual(host);
            } else if self.keyboard_mode() {
                self.clear_item_visual();
            }
        }
        // Submenu anchor ancestry is a current DOM fact, independent of the
        // optional visibility observer. Suppressing visibility must not disable
        // the source's submenu placement override.
        for node in &report.nodes {
            let Some(in_dropdown_item) = node.anchor_in_dropdown_item else {
                continue;
            };
            let Some(entry) = self.entry(&node.id).filter(|entry| {
                node.connected && node.generation == entry.generation && entry.kind.is_dropdown()
            }) else {
                continue;
            };
            if let Some(callback) = &entry.anchor_shape {
                self.with_owner(&node.id, node.generation, || {
                    if self.read_current(&node.id, &entry) == Some(true) {
                        callback(in_dropdown_item);
                    }
                });
            }
        }
        // Bind activation to this report's captured hosts and registration
        // before a key callback can open, replace or remove an owner. Native
        // execution retains the corresponding actual elements independently.
        let activation_sources = report
            .nodes
            .iter()
            .filter_map(|node| {
                let targets = report
                    .candidate_ids
                    .iter()
                    .filter(|target| {
                        node.inventory.as_ref().map_or_else(
                            || {
                                node.items
                                    .iter()
                                    .any(|id| target.as_str() == format!("ix-dropdown-item-{id}"))
                            },
                            |inventory| {
                                inventory.hosts.iter().any(|host| {
                                    host.target == target.as_str() && !host.capture.is_empty()
                                })
                            },
                        )
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if targets.is_empty() {
                    return None;
                }
                let entry = self.entry(&node.id)?;
                if !node.connected
                    || node.generation != entry.generation
                    || self.read_current(&node.id, &entry) != Some(true)
                {
                    return None;
                }
                Some((node.id.clone(), entry.generation, entry.revision, targets))
            })
            .collect::<Vec<_>>();
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
        let activation = decision.activate.as_ref().and_then(|target| {
            let (id, generation, revision, _) = activation_sources
                .iter()
                .find(|(_, _, _, targets)| targets.contains(target))?;
            self.current(id, *generation, *revision)
                .then(|| (target.clone(), id.clone(), *generation, *revision))
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
        if matches!(report.phase.as_str(), "navigation" | "trigger-key") {
            return report.nodes.iter().find(|node| {
                let Some(entry) = self.entry(&node.id).filter(|entry| {
                    entry.kind == OverlayKind::Dropdown
                        && entry.generation == node.generation
                        && node.connected
                }) else {
                    return false;
                };
                if !self.matches_root_capture(node) {
                    return false;
                }
                if report.phase == "navigation" {
                    let Some(configuration) = entry
                        .navigation
                        .filter(|configuration| configuration.enabled)
                    else {
                        return false;
                    };
                    node.navigation_listener.as_ref().is_some_and(|listener| {
                        listener.version == configuration.version
                            && !listener.capture.is_empty()
                            && configuration.capture.as_deref() == Some(listener.capture.as_str())
                    })
                } else {
                    let installed = node
                        .trigger_listener_capture
                        .as_deref()
                        .unwrap_or(&node.trigger_capture);
                    !installed.is_empty()
                        && node.configured_trigger_listener.as_deref() == Some(installed)
                }
            });
        }
        if report.phase == "capture" {
            if let Some(owner) = report.hosts.iter().rev().find_map(|id| {
                let entry = self
                    .entry(id)
                    .filter(|entry| entry.kind == OverlayKind::Dropdown)?;
                report.nodes.iter().find(|node| {
                    node.id == *id
                        && node.connected
                        && node.generation == entry.generation
                        && node.mode == "roving-tabindex"
                        && self.read_current(id, &entry) == Some(true)
                })
            }) {
                return Some(owner);
            }
        }
        // A linked Dropdown trigger can be physically inside another menu's
        // host. Its own trigger listener owns the key before that enclosing
        // host's navigation mode is considered.
        if let Some(trigger) = report.triggers.iter().find_map(|id| {
            let entry = self
                .entry(id)
                .filter(|entry| entry.kind == OverlayKind::Dropdown)?;
            report.nodes.iter().find(|node| {
                node.id == *id
                    && node.connected
                    && node.generation == entry.generation
                    && node.on_trigger
            })
        }) {
            return Some(trigger);
        }
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
    fn matches_root_capture(&self, node: &NodeFacts) -> bool {
        !node.root_capture.is_empty()
            && self
                .0
                .borrow()
                .item_root_captures
                .get(&node.id)
                .is_some_and(|(generation, root, _)| {
                    *generation == node.generation && *root == node.root_capture
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
        let route = match report.phase.as_str() {
            "trigger-key" => KeyRoute::Trigger,
            "navigation" => KeyRoute::Navigation,
            _ => KeyRoute::Legacy,
        };
        let navigation_mode = if route == KeyRoute::Navigation {
            entry
                .navigation
                .as_ref()
                .map(|configuration| configuration.mode.clone())
        } else {
            None
        };
        if route == KeyRoute::Navigation
            && navigation_mode.as_deref() == Some("active-descendant")
            && matches!(report.key.as_str(), "Escape" | "Tab")
        {
            return KeyDecision::default();
        }
        let domain_root = entry
            .submenu
            .as_ref()
            .and_then(|access| (access.query_root)());
        let domain_node = domain_root.as_ref().and_then(|root| {
            report.nodes.iter().find(|node| {
                node.id == *root
                    && self
                        .entry(root)
                        .is_some_and(|entry| entry.generation == node.generation)
            })
        });
        let navigation_inventory = domain_node
            .and_then(|node| node.own_inventory.as_ref().or(node.inventory.as_ref()))
            .or(owner.inventory.as_ref());
        let mut facts = KeyFacts {
            route,
            navigation_mode,
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
            inventory: navigation_inventory
                .cloned()
                .map(|facts| self.projected_inventory(facts)),
        };
        let capture = if route == KeyRoute::Navigation {
            facts.navigation_mode.as_deref() == Some("roving-tabindex")
        } else {
            owner.mode == "roving-tabindex"
        };
        if route == KeyRoute::Legacy && report.phase == "capture" && !capture {
            // Mark the native route before Dioxus's own bubble fallback runs.
            // The private callback returns without action for prevented facts.
            facts.default_prevented = true;
            callback(&facts);
            return KeyDecision::default();
        }
        if route == KeyRoute::Legacy
            && ((matches!(report.phase.as_str(), "bubble" | "fallback") && capture)
                || facts.default_prevented
                || (report.phase == "fallback" && !matches!(report.key.as_str(), "Escape" | "Tab")))
        {
            // Active-descendant arrows/activation belong to the actual trigger
            // listener, not an unrelated input in an externally hosted menu.
            return KeyDecision::default();
        }
        if route == KeyRoute::Legacy
            && !capture
            && !facts.on_trigger
            && !owner.trigger_on_path
            && !report.triggers.contains(&owner.id)
            && !matches!(report.key.as_str(), "Escape" | "Tab")
        {
            return KeyDecision::default();
        }
        if route == KeyRoute::Legacy && !facts.inside && !facts.on_trigger {
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
        if report.key == "Escape" && route != KeyRoute::Navigation {
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
        let mut decision = if route == KeyRoute::Legacy {
            callback(&facts)
        } else {
            self.with_owner(&owner.id, entry.generation, || callback(&facts))
                .unwrap_or_default()
        };
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
        let Some(fact) = report.anchor_visibility.as_ref() else {
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
                if fact.intersecting {
                    if let Some(geometry) = fact.geometry.filter(|geometry| geometry.valid())
                        && let Some(callback) = &entry.anchor_intersection
                    {
                        callback(geometry);
                    }
                    return;
                }
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
    fn host_click_sites(&self, report: &Report) -> KeyDecision {
        let mut prevented = report.default_prevented;
        // Header's source content-background listener runs before the inner
        // Dropdown handlers. This raw target fact projects that same guard.
        if report.click_sites.iter().any(|site| {
            site.connected
                && site.target_is_host
                && self.entry(&site.root).is_some_and(|entry| {
                    entry.generation == site.generation && entry.kind == OverlayKind::Header
                })
        }) {
            prevented = true;
        }
        for site in &report.click_sites {
            let Some(entry) = self.entry(&site.root).filter(|entry| {
                site.connected
                    && entry.generation == site.generation
                    && entry.kind.is_dropdown()
                    && report.nodes.iter().any(|node| {
                        node.id == site.root && node.generation == site.generation && node.connected
                    })
            }) else {
                continue;
            };
            let source_receiver = match site.receiver.as_str() {
                "host" => entry.dialog_click && site.popover && report.hosts.contains(&site.root),
                "root" => entry.root_click_without_host || report.hosts.contains(&site.root),
                _ => false,
            };
            if !source_receiver || self.read_current(&site.root, &entry).is_none() {
                continue;
            }
            // Keep each captured receiver, including repeated dialog/Host
            // calls. Re-read registration and policy after earlier callbacks.
            if self.child_path(&site.root, &report.hosts, &report.triggers) {
                prevented |= report.triggers.first().is_some_and(|id| id != &site.root);
                continue;
            }
            if let Some(trigger) = report.triggers.first() {
                prevented |= trigger != &site.root || entry.behavior == CloseBehavior::Outside;
                continue;
            }
            let inside = !prevented && entry.behavior.allows_inside();
            self.dismiss_dropdown_set(&site.root, site.generation, inside);
        }
        KeyDecision {
            prevent: prevented,
            ..KeyDecision::default()
        }
    }

    fn click(&self, report: &Report) -> KeyDecision {
        if report.phase == "trigger" {
            if !report.default_prevented {
                for node in &report.nodes {
                    let installed = node
                        .trigger_listener_capture
                        .as_deref()
                        .unwrap_or(&node.trigger_capture);
                    if installed.is_empty()
                        || node.configured_trigger_listener.as_deref() != Some(installed)
                    {
                        continue;
                    }
                    let access = self
                        .entry(&node.id)
                        .filter(|entry| {
                            node.connected
                                && entry.generation == node.generation
                                && !node.root_capture.is_empty()
                                && self
                                    .0
                                    .borrow()
                                    .item_root_captures
                                    .get(&node.id)
                                    .is_some_and(|(generation, root, _)| {
                                        *generation == node.generation && *root == node.root_capture
                                    })
                                && !self
                                    .0
                                    .borrow()
                                    .capture_block
                                    .contains(&(node.id.clone(), node.generation))
                        })
                        .and_then(|entry| entry.submenu);
                    if let Some(access) = access {
                        self.with_owner(&node.id, node.generation, || (access.trigger_click)());
                    }
                }
            }
            // Native default prevention can be established at the target,
            // after document capture but before Dioxus's delegated callback.
            // Preserve the existing ancestor-toggle guards; this fact alone
            // must neither toggle the owner nor dismiss another registration.
            if report.default_prevented
                && let Some((id, generation)) = report.triggers.iter().find_map(|id| {
                    let entry = self
                        .entry(id)
                        .filter(|entry| entry.kind == OverlayKind::Dropdown)?;
                    report
                        .nodes
                        .iter()
                        .any(|node| {
                            node.id == *id && node.connected && node.generation == entry.generation
                        })
                        .then(|| (id.clone(), entry.generation))
                })
            {
                let mut state = self.0.borrow_mut();
                if !state.capture_block.contains(&(id.clone(), generation)) {
                    state.capture_block.push((id, generation));
                }
            }
            return KeyDecision::default();
        }
        if report.phase == "capture" {
            let mut blocked = Vec::new();
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
                if index > 0
                    || child_path
                    || (entry.behavior == CloseBehavior::Outside
                        && report.enclosing_triggers.contains(id))
                {
                    blocked.push((id.clone(), entry.generation));
                }
            }
            self.0.borrow_mut().capture_block = blocked;
            return KeyDecision::default();
        }
        self.0.borrow_mut().capture_block.clear();
        let mut decision = KeyDecision::default();
        let keyboard = self.0.borrow().keyboard;
        // Native reports retain source-equivalent receivers. Renderer-only
        // reports without receiver facts keep their earlier flat test seam.
        if !report.click_sites.is_empty() {
            decision = self.host_click_sites(report);
        } else if !report.default_prevented {
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
            decision.prevent |= runtime.order.iter().any(|id| {
                runtime.registry.is_open(id)
                    && runtime
                        .nodes
                        .get(id)
                        .is_some_and(|entry| entry.kind.is_dropdown())
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
        if let Some(observation) = query.item_focus_observation {
            let projected = self.0.borrow_mut().item_cue_hosts.focus_host(&observation);
            let Ok(host) = projected else {
                return;
            };
            if self.keyboard_mode()
                && let Some(host) = host
            {
                self.mark_item_visual(&host);
            } else {
                self.clear_item_visual();
            }
            event.prevent_default();
            return;
        }
        if let Some(offer) = query.submenu_marker {
            if self.permits_submenu_marker(&offer) {
                event.prevent_default();
            }
            return;
        }
        if let Some(descendant) = query.descendant_focus {
            self.query_descendant_focus(descendant, &event);
            return;
        }
        if query.item_cues_pending {
            if std::mem::take(&mut self.0.borrow_mut().item_cues_dirty) {
                event.prevent_default();
            }
            return;
        }
        if let Some(offer) = query.item_cue {
            let state = self.0.borrow();
            if state.item_cue_hosts.permits(
                &offer,
                state.item_visual_host.as_ref(),
                state.item_visual_known,
            ) {
                event.prevent_default();
            }
            return;
        }
        if let Some(offer) = query.item_root_cue {
            let authorized = {
                let state = self.0.borrow();
                state.item_visual_known
                    && state.item_root_captures.get(&offer.root).is_some_and(
                        |(generation, root, trigger)| {
                            *generation == offer.generation
                                && !root.is_empty()
                                && *root == offer.capture
                                && (!matches!(offer.field, RootCueField::TriggerAria)
                                    || offer.trigger_capture.as_ref().is_some_and(|capture| {
                                        !capture.is_empty() && capture == trigger
                                    }))
                        },
                    )
            };
            if !authorized {
                return;
            }
            // The reader is internal projection only: no application callbacks,
            // queue consumption or renderer work is permitted in this query.
            let expected = self
                .with_owner(&offer.root, offer.generation, || {
                    let entry = self.entry(&offer.root)?;
                    if entry.generation != offer.generation
                        || self.read_current(&offer.root, &entry).is_none()
                    {
                        return None;
                    }
                    let projection = (entry.item_visual.as_ref()?)();
                    Some(match offer.field {
                        RootCueField::Active => projection.active,
                        RootCueField::Aria | RootCueField::TriggerAria => projection.aria,
                    })
                })
                .flatten();
            if expected.is_some_and(|expected| expected == offer.value)
                && self
                    .entry(&offer.root)
                    .is_some_and(|entry| entry.generation == offer.generation)
            {
                event.prevent_default();
            }
            return;
        }
        if let Some(sequence) = query.roving_pending {
            if self
                .0
                .borrow()
                .roving_writes
                .iter()
                .any(|batch| batch.event_sequence == sequence)
            {
                event.prevent_default();
            }
            return;
        }
        if query.roving_write {
            let Some(item) = query.roving_item else {
                return;
            };
            let candidate = {
                let state = self.0.borrow();
                state
                    .roving_writes
                    .front()
                    .filter(|batch| {
                        if let Some(sequence) = query.roving_sequence {
                            batch.sequence == sequence
                                && query.roving_root.as_deref() == Some(batch.root.as_str())
                                && query.roving_generation == Some(batch.generation)
                        } else {
                            query.roving_event_sequence == Some(batch.event_sequence)
                        }
                    })
                    .and_then(|batch| {
                        batch
                            .writes
                            .front()
                            .filter(|write| {
                                write.item.0 == item && write.tabindex == query.roving_tabindex
                            })
                            .map(|_| (batch.sequence, batch.root.clone(), batch.generation))
                    })
            };
            let Some((sequence, root, generation)) = candidate else {
                return;
            };
            let live = self
                .with_owner(&root, generation, || {
                    let entry = self.entry(&root)?;
                    let value = self.read_current(&root, &entry)?;
                    self.entry(&root)
                        .filter(|entry| entry.generation == generation)
                        .map(|_| value)
                })
                .flatten()
                .is_some();
            let consumed = {
                let mut state = self.0.borrow_mut();
                let Some(index) = state
                    .roving_writes
                    .iter()
                    .position(|batch| batch.sequence == sequence)
                else {
                    return;
                };
                if index != 0 {
                    return;
                }
                let batch = &mut state.roving_writes[index];
                if !batch.writes.front().is_some_and(|write| {
                    write.item.0 == item && write.tabindex == query.roving_tabindex
                }) {
                    return;
                }
                batch.writes.pop_front();
                if batch.writes.is_empty() {
                    state.roving_writes.remove(index);
                }
                true
            };
            if live && consumed {
                event.prevent_default();
            }
            return;
        }
        if let Some(phase) = query.item_focus_phase {
            if phase == "retain" {
                if query.item_focus_capture.as_deref().is_some_and(|capture| {
                    let state = self.0.borrow();
                    state.item_focus.retains_token(capture)
                        || state.descendant_focus.retains_token(capture)
                        || state.submenu_marks.contains_key(capture)
                        || state.item_visual_host.as_ref().is_some_and(|host| {
                            host.capture == capture
                                || host.parts.iter().any(|part| part.capture == capture)
                        })
                        || state
                            .roving_hosts
                            .values()
                            .flatten()
                            .any(|host| host.0 == capture)
                        || state.roving.retains_token(capture)
                        || state
                            .roving_writes
                            .iter()
                            .any(|batch| batch.writes.iter().any(|write| write.item.0 == capture))
                }) {
                    event.prevent_default();
                }
                return;
            }
            if phase == "discard" {
                let (Some(sequence), Some(root), Some(generation), Some(target)) = (
                    query.item_focus_sequence,
                    query.item_focus_root,
                    query.item_focus_generation,
                    query.item_focus_target,
                ) else {
                    return;
                };
                let mut state = self.0.borrow_mut();
                if state.item_focus.discard(
                    sequence,
                    &root,
                    generation,
                    &target,
                    query.item_focus_capture.as_deref(),
                ) {
                    state.item_focus_callbacks.remove(&sequence);
                }
                return;
            }
            let (Some(root), Some(generation), Some(target), Some(capture)) = (
                query.item_focus_root,
                query.item_focus_generation,
                query.item_focus_target,
                query.item_focus_capture,
            ) else {
                return;
            };
            let Some(sequence) = query.item_focus_sequence else {
                return;
            };
            if phase == "arm" {
                let live = self
                    .with_owner(&root, generation, || {
                        let entry = self.entry(&root)?;
                        let current = self.read_current(&root, &entry)?;
                        self.entry(&root)
                            .filter(|entry| entry.generation == generation)
                            .map(|_| current)
                    })
                    .flatten()
                    .is_some();
                if live
                    && self
                        .0
                        .borrow_mut()
                        .item_focus
                        .arm(sequence, &root, generation, &target, &capture)
                {
                    event.prevent_default();
                }
            } else if phase == "focus" {
                let native_visual_delivery = query.item_focus_host.is_some();
                let visual = {
                    let state = self.0.borrow();
                    let original = state
                        .item_focus_callbacks
                        .get(&sequence)
                        .and_then(|callback| callback.visual.as_ref());
                    match (&query.item_focus_host, original) {
                        (Some(live), Some(original)) => {
                            if live.capture != capture
                                || live.registration_key != original.registration_key
                                || live.kind != original.kind
                            {
                                return;
                            }
                            Some(live.clone())
                        }
                        (_, original) => original.cloned(),
                    }
                };
                let (job, callback) = {
                    let mut state = self.0.borrow_mut();
                    let Some(job) = state
                        .item_focus
                        .take_focus(sequence, &root, generation, &target, &capture)
                    else {
                        return;
                    };
                    (job, state.item_focus_callbacks.remove(&sequence))
                };
                let live = self
                    .with_owner(&root, generation, || {
                        let Some(entry) = self.entry(&root) else {
                            return false;
                        };
                        if self.read_current(&root, &entry).is_none()
                            || !self
                                .entry(&root)
                                .is_some_and(|entry| entry.generation == generation)
                        {
                            return false;
                        }
                        if let Some(callback) = callback {
                            // Legacy renderer-only callers have no native
                            // post-attempt receipt. Actual native calls publish
                            // only in focusin or their later exact visual phase.
                            if !native_visual_delivery && let Some(visual) = &visual {
                                self.mark_item_visual(visual);
                            }
                            (callback.callback)();
                        }
                        // A first-frame callback can unmount or replace its owner.
                        // Never grant native execution to the replacement.
                        self.entry(&root)
                            .filter(|entry| entry.generation == generation)
                            .and_then(|entry| self.read_current(&root, &entry))
                            .is_some()
                            && self
                                .entry(&root)
                                .is_some_and(|entry| entry.generation == generation)
                    })
                    .unwrap_or(false);
                if live {
                    if let Some(visual) = visual {
                        self.0
                            .borrow_mut()
                            .item_visual_deliveries
                            .insert(sequence, (job, visual));
                    }
                    event.prevent_default();
                } else {
                    // No native scroll is enrolled after a denied first frame.
                    self.0
                        .borrow_mut()
                        .item_focus
                        .take_scroll(sequence, &root, generation, &target, &capture);
                }
            } else if phase == "visual" {
                let original = {
                    let state = self.0.borrow();
                    state
                        .item_visual_deliveries
                        .get(&sequence)
                        .filter(|(job, _)| {
                            job.root == root
                                && job.generation == generation
                                && job.target == target
                                && job.capture.as_deref() == Some(capture.as_str())
                        })
                        .map(|(_, facts)| facts.clone())
                };
                let Some(original) = original else {
                    return;
                };
                let live = query.item_focus_host.unwrap_or_else(|| original.clone());
                if live.capture != capture
                    || live.registration_key != original.registration_key
                    || live.kind != original.kind
                {
                    return;
                }
                let accepted = self
                    .with_owner(&root, generation, || {
                        let entry = self.entry(&root)?;
                        let _ = self.read_current(&root, &entry)?;
                        if !self
                            .entry(&root)
                            .is_some_and(|entry| entry.generation == generation)
                        {
                            return None;
                        }
                        self.0
                            .borrow_mut()
                            .item_visual_deliveries
                            .remove(&sequence)?;
                        self.mark_item_visual(&live);
                        Some(())
                    })
                    .flatten()
                    .is_some();
                if accepted {
                    event.prevent_default();
                }
            } else if phase == "scroll" {
                let job = self
                    .0
                    .borrow_mut()
                    .item_focus
                    .take_scroll(sequence, &root, generation, &target, &capture);
                if job.is_some() {
                    self.0.borrow_mut().item_visual_deliveries.remove(&sequence);
                    let live = self
                        .with_owner(&root, generation, || {
                            let entry = self.entry(&root)?;
                            let current = self.read_current(&root, &entry)?;
                            self.entry(&root)
                                .filter(|entry| entry.generation == generation)
                                .map(|_| current)
                        })
                        .flatten()
                        .is_some();
                    if live {
                        event.prevent_default();
                    }
                }
            }
            return;
        }
        if let Some(sequence) = query.completion_sequence {
            let (Some(root), Some(generation)) =
                (query.completion_root, query.completion_generation)
            else {
                return;
            };
            let callback = self
                .0
                .borrow_mut()
                .completions
                .take(sequence, &root, generation);
            if let Some(callback) = callback {
                self.with_owner(&root, generation, || {
                    let Some(entry) = self
                        .entry(&root)
                        .filter(|entry| entry.generation == generation)
                    else {
                        return;
                    };
                    if self.read_current(&root, &entry).is_some() {
                        event.prevent_default();
                        callback();
                    }
                });
            }
            return;
        }
        if let Some(sequence) = query.trigger_restore_sequence {
            let (Some(root), Some(generation)) =
                (query.trigger_restore_root, query.trigger_restore_generation)
            else {
                return;
            };
            let job = self
                .0
                .borrow_mut()
                .trigger_restores
                .take(sequence, &root, generation);
            if job.is_some()
                && self
                    .with_owner(&root, generation, || {
                        let entry = self.entry(&root)?;
                        (entry.generation == generation)
                            .then(|| self.read_current(&root, &entry))
                            .flatten()
                    })
                    .flatten()
                    .is_some()
            {
                // A source cleanup RAF survives later same-owner reopen and
                // newer reports. Only this registration's lifetime authorizes it.
                event.prevent_default();
            }
            return;
        }
        if let Some(target) = query.schedule_focus_target {
            let armed = self
                .0
                .borrow_mut()
                .pending_focus
                .schedule(query.sequence, &target);
            if armed {
                event.prevent_default();
            }
            return;
        }
        if let Some(target) = query.dispatch_focus_target {
            let route = {
                let mut state = self.0.borrow_mut();
                let has_pending_trigger = !state.trigger_restores.is_empty();
                state
                    .pending_focus
                    .route(query.sequence, &target, has_pending_trigger)
            };
            if route == Some(false) {
                // Preserve the source's synchronous RAF scheduling when no
                // earlier queued cleanup requires native command ordering.
                event.prevent_default();
            } else if route == Some(true) {
                let sent = self.0.borrow().bridge.as_ref().is_some_and(|bridge| {
                    bridge
                        .send(serde_json::json!({
                            "tab_focus":{"sequence":query.sequence,"target":target}
                        }))
                        .is_ok()
                });
                if !sent {
                    // Only this routed admission failed to enter the native
                    // FIFO; independent source jobs remain retained.
                    self.0
                        .borrow_mut()
                        .pending_focus
                        .take(query.sequence, &target);
                }
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
    /// Revision reflected by the component's next native renderer commit.
    pub(crate) fn render_revision(&self) -> u64 {
        self.runtime()
            .and_then(|runtime| runtime.entry(&self.id))
            .filter(|entry| entry.generation == self.generation)
            .map_or(0, |entry| entry.revision)
    }
    pub(crate) fn native_observed(&self) -> bool {
        self.runtime().is_some_and(|runtime| {
            let state = runtime.0.borrow();
            state
                .nodes
                .get(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
                && state.registry.is_connected(&self.id)
                && state
                    .item_root_captures
                    .get(&self.id)
                    .is_some_and(|(generation, root, _)| {
                        *generation == self.generation && !root.is_empty()
                    })
        })
    }
    pub(crate) fn configure_navigation(&self, mode: &str, enabled: bool) -> bool {
        self.runtime().is_some_and(|runtime| {
            runtime.configure_navigation(&self.id, self.generation, mode, enabled)
        })
    }
    pub(crate) fn clear_navigation(&self) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        let mode = runtime
            .entry(&self.id)
            .filter(|entry| entry.generation == self.generation)
            .and_then(|entry| entry.navigation.map(|configuration| configuration.mode))
            .unwrap_or_else(|| "active-descendant".into());
        runtime.configure_navigation(&self.id, self.generation, &mode, false)
    }
    pub(crate) async fn query_inventory(&self) -> Option<InventoryFacts> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                let entry = runtime.entry(&self.id)?;
                runtime.read_current(&self.id, &entry)
            })
            .flatten()?;
        let query = crate::dom_query::eval(
            r#"const [owner,root,generation]=await dioxus.recv();return document[Symbol.for('dioxus-ix.document-native-domain.v1')]?.inventoryReaders?.get(owner)?.(root,generation)??null;"#,
        );
        query
            .send((runtime.owner_id(), self.id.clone(), self.generation))
            .ok()?;
        let facts = query.join::<Option<InventoryFacts>>().await.ok()??;
        runtime
            .with_owner(&self.id, self.generation, || {
                let entry = runtime.entry(&self.id)?;
                runtime.read_current(&self.id, &entry)?;
                if !runtime
                    .entry(&self.id)
                    .is_some_and(|entry| entry.generation == self.generation)
                {
                    return None;
                }
                {
                    let mut state = runtime.0.borrow_mut();
                    state.item_captures.insert(
                        self.id.clone(),
                        (
                            self.generation,
                            facts
                                .hosts
                                .iter()
                                .map(|host| (host.target.clone(), host.capture.clone()))
                                .collect(),
                        ),
                    );
                    state
                        .inventories
                        .insert(self.id.clone(), (self.generation, facts.clone()));
                }
                Some(runtime.projected_inventory(facts))
            })
            .flatten()
    }
    pub(crate) fn inventory(&self) -> Option<InventoryFacts> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                let entry = runtime.entry(&self.id)?;
                runtime.read_current(&self.id, &entry)?;
                let facts = runtime
                    .0
                    .borrow()
                    .inventories
                    .get(&self.id)
                    .filter(|(generation, _)| *generation == self.generation)
                    .map(|(_, facts)| facts.clone())?;
                Some(runtime.projected_inventory(facts))
            })
            .flatten()
    }
    pub(crate) fn select_roving(&self, facts: &InventoryFacts, target: &ItemRef) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime.entry(&self.id) else {
                    return false;
                };
                if runtime.read_current(&self.id, &entry).is_none() {
                    return false;
                }
                let facts = runtime.projected_inventory(facts.clone());
                let inventory = crate::dropdown_inventory::Inventory::new(
                    crate::dropdown_inventory::InventoryMode::RovingTabindex,
                    facts.clone(),
                );
                let writes = {
                    let mut state = runtime.0.borrow_mut();
                    // Ordinary movement remembers baselines, but only source
                    // configure/initialize enroll hosts for this owner's cleanup.
                    state
                        .roving
                        .select(&inventory.items_host, &inventory.query_hosts(), target)
                };
                runtime.queue_roving_writes(&self.id, self.generation, writes);
                true
            })
            .unwrap_or(false)
    }
    pub(crate) fn init_roving(
        &self,
        facts: &InventoryFacts,
        last: bool,
        focus_checked: bool,
    ) -> Option<HostFacts> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                let entry = runtime.entry(&self.id)?;
                runtime.read_current(&self.id, &entry)?;
                let facts = runtime.projected_inventory(facts.clone());
                let inventory = crate::dropdown_inventory::Inventory::new(
                    crate::dropdown_inventory::InventoryMode::RovingTabindex,
                    facts,
                );
                let (target, writes) = {
                    let mut state = runtime.0.borrow_mut();
                    let hosts = state
                        .roving_hosts
                        .entry((self.id.clone(), self.generation))
                        .or_default();
                    if !hosts.contains(&inventory.items_host) {
                        hosts.push(inventory.items_host.clone());
                    }
                    state.roving.init(
                        &inventory.items_host,
                        &inventory.query_hosts(),
                        last,
                        focus_checked,
                    )?
                };
                runtime.queue_roving_writes(&self.id, self.generation, writes);
                inventory.host(&target).cloned()
            })
            .flatten()
    }
    pub(crate) fn clear_roving(&self) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if !runtime
            .entry(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
        {
            return;
        }
        let writes = {
            let mut state = runtime.0.borrow_mut();
            state.pending_roving_hosts.retain(|admission| {
                admission.root != self.id || admission.generation != self.generation
            });
            let hosts = state
                .roving_hosts
                .remove(&(self.id.clone(), self.generation))
                .unwrap_or_default();
            hosts
                .iter()
                .flat_map(|host| state.roving.clear(host))
                .collect()
        };
        runtime.queue_roving_writes(&self.id, self.generation, writes);
    }
    pub(crate) fn queue_item_host_focus(&self, host: &HostFacts, callback: Rc<dyn Fn()>) -> bool {
        if host.capture.is_empty() {
            return false;
        }
        self.queue_item_focus_captured(
            host.target.clone(),
            true,
            Some(host.capture.clone()),
            callback,
            Some(host.clone()),
        )
    }
    /// Admit one captured item operation. Newer keys, mode changes and accepted
    /// hides do not revoke its two native frames while this owner remains live.
    pub(crate) fn queue_item_focus(
        &self,
        target: String,
        focus_native: bool,
        callback: Rc<dyn Fn()>,
    ) -> bool {
        self.queue_item_focus_captured(target, focus_native, None, callback, None)
    }
    fn queue_item_focus_captured(
        &self,
        target: String,
        focus_native: bool,
        captured: Option<String>,
        callback: Rc<dyn Fn()>,
        visual: Option<HostFacts>,
    ) -> bool {
        if target.is_empty() {
            return false;
        }
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime.entry(&self.id) else {
                    return false;
                };
                if runtime.read_current(&self.id, &entry).is_none()
                    || !runtime
                        .entry(&self.id)
                        .is_some_and(|entry| entry.generation == self.generation)
                {
                    return false;
                }
                {
                    let mut state = runtime.0.borrow_mut();
                    let capture = captured.or_else(|| {
                        state
                            .item_captures
                            .get(&self.id)
                            .filter(|(generation, _)| *generation == self.generation)
                            .and_then(|(_, captures)| captures.get(&target))
                            .cloned()
                    });
                    let job = state.item_focus.admit(
                        self.id.clone(),
                        self.generation,
                        target,
                        focus_native,
                        capture,
                    );
                    state.item_focus_callbacks.insert(
                        job.sequence,
                        ItemFocusCallback {
                            root: self.id.clone(),
                            generation: self.generation,
                            callback,
                            visual,
                        },
                    );
                }
                runtime.flush_item_focus();
                true
            })
            .unwrap_or(false)
    }
    /// Retain each changed-state completion until an explicit native frame,
    /// including changes admitted before the shared bridge is mounted.
    pub(crate) fn queue_completion(&self, callback: Rc<dyn Fn()>) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime
                    .entry(&self.id)
                    .filter(|entry| entry.generation == self.generation)
                else {
                    return false;
                };
                if runtime.read_current(&self.id, &entry).is_none() {
                    return false;
                }
                runtime.0.borrow_mut().completions.admit(
                    self.id.clone(),
                    self.generation,
                    callback,
                );
                runtime.flush_completion_frames();
                true
            })
            .unwrap_or(false)
    }
    pub(crate) fn queue_trigger_restore(&self, focus_version: u64) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime
                    .entry(&self.id)
                    .filter(|entry| entry.generation == self.generation)
                else {
                    return false;
                };
                if runtime.read_current(&self.id, &entry).is_none() {
                    return false;
                }
                let mut state = runtime.0.borrow_mut();
                if state.document.is_empty() || state.bridge.is_none() {
                    return false;
                }
                let job =
                    state
                        .trigger_restores
                        .admit(self.id.clone(), self.generation, focus_version);
                let sent = state.bridge.as_ref().is_some_and(|bridge| {
                    bridge
                        .send(serde_json::json!({
                            "restore_trigger":{"sequence":job.sequence,"root":job.root,
                                "generation":job.generation,"focus_version":job.focus_version}
                        }))
                        .is_ok()
                });
                if !sent {
                    state
                        .trigger_restores
                        .take(job.sequence, &self.id, self.generation);
                }
                sent
            })
            .unwrap_or(false)
    }
    pub(crate) fn set_anchor_shape_handler(&self, handler: Rc<dyn Fn(bool)>) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if let Some(entry) = runtime.0.borrow_mut().nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.anchor_shape = Some(handler);
        }
    }
    pub(crate) fn set_anchor_intersection_handler(&self, handler: AnchorIntersectionCallback) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if let Some(entry) = runtime.0.borrow_mut().nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.anchor_intersection = Some(handler);
        }
    }
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
    pub(crate) fn set_host_click_receivers(&self, dialog: bool, root_without_host: bool) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if let Some(entry) = runtime.0.borrow_mut().nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.dialog_click = dialog;
            entry.root_click_without_host = root_without_host;
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
    /// metadata. The component consults Rust's capture and actual-trigger facts
    /// before toggling, after running its own consumer callback.
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
    pub(crate) fn set_descendant_focus_handler(&self, handler: DescendantFocusCallback) {
        if let Some(runtime) = self.runtime() {
            let mut state = runtime.0.borrow_mut();
            if let Some(entry) = state
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            {
                entry.descendant_focus = Some(handler);
            }
        }
    }
    pub(crate) fn queue_descendant_focus(
        &self,
        focus_version: u64,
        last: bool,
        allow_checked: bool,
    ) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        let admitted = runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime.entry(&self.id) else {
                    return false;
                };
                if runtime.read_current(&self.id, &entry).is_none() {
                    return false;
                }
                let mut state = runtime.0.borrow_mut();
                let reference = state
                    .item_root_captures
                    .get(&self.id)
                    .filter(|(generation, capture, _)| {
                        *generation == self.generation && !capture.is_empty()
                    })
                    .map(|(_, capture, _)| capture.clone());
                state.descendant_focus.admit(
                    self.id.clone(),
                    self.generation,
                    focus_version,
                    last,
                    allow_checked,
                    reference,
                );
                true
            })
            .unwrap_or(false);
        if admitted {
            runtime.flush_descendant_focus();
        }
        admitted
    }
    pub(crate) fn set_item_visual_reader(&self, reader: Rc<dyn Fn() -> RootVisual>) {
        if let Some(runtime) = self.runtime() {
            let mut state = runtime.0.borrow_mut();
            if let Some(entry) = state
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            {
                entry.item_visual = Some(reader);
            }
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

#[path = "submenu_runtime.rs"]
mod submenu_runtime;

#[cfg(test)]
#[path = "submenu_runtime_tests.rs"]
mod submenu_runtime_tests;

#[cfg(test)]
#[path = "overlay_assignment_tests.rs"]
mod overlay_assignment_tests;

#[cfg(test)]
#[path = "overlay_navigation_tests.rs"]
mod overlay_navigation_tests;

#[path = "descendant_transport.rs"]
mod descendant_transport;

#[cfg(test)]
#[path = "descendant_transport_tests.rs"]
mod descendant_transport_tests;

#[cfg(test)]
#[path = "item_visual_transport_tests.rs"]
mod item_visual_transport_tests;

#[derive(Deserialize)]
struct Report {
    owner: String,
    sequence: u64,
    kind: String,
    #[serde(default)]
    item_focus_host: Option<HostFacts>,
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
    click_sites: Vec<ClickSite>,
    #[serde(default)]
    triggers: Vec<String>,
    #[serde(default)]
    enclosing_triggers: Vec<String>,
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
struct ClickSite {
    root: String,
    generation: u64,
    connected: bool,
    receiver: String,
    #[serde(default)]
    popover: bool,
    #[serde(default)]
    target_is_host: bool,
}
#[derive(Deserialize)]
struct NodeFacts {
    id: String,
    generation: u64,
    connected: bool,
    #[serde(default)]
    root_capture: String,
    #[serde(default)]
    trigger_capture: String,
    #[serde(default)]
    trigger_listener_capture: Option<String>,
    #[serde(default)]
    navigation_binding: Option<NavigationBinding>,
    #[serde(default)]
    navigation_listener: Option<NavigationBinding>,
    #[serde(default)]
    discovery_ready: bool,
    #[serde(default)]
    trigger_ancestors: Vec<String>,
    #[serde(default)]
    assignment_ancestors: Option<Vec<String>>,
    #[serde(default)]
    assignment_emissions: Option<Vec<crate::overlay_registry::AssignmentEmission>>,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    trigger_id: String,
    #[serde(default)]
    anchor_id: String,
    #[serde(default)]
    anchor_in_dropdown_item: Option<bool>,
    #[serde(default)]
    header_controls: Vec<String>,
    #[serde(default)]
    items: Vec<usize>,
    #[serde(default)]
    item_focus_captures: Vec<ItemFocusCapture>,
    #[serde(default)]
    inventory: Option<InventoryFacts>,
    #[serde(default)]
    own_inventory: Option<InventoryFacts>,
    #[serde(default)]
    trigger_on_path: bool,
    #[serde(default)]
    configured_trigger_listener: Option<String>,
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
struct ItemFocusCapture {
    target: String,
    capture: String,
}
#[derive(Deserialize)]
struct AnchorVisibility {
    root: String,
    generation: u64,
    target: String,
    intersecting: bool,
    #[serde(default)]
    geometry: Option<AnchorIntersection>,
}
#[derive(Deserialize)]
struct Query {
    owner: String,
    sequence: u64,
    #[serde(default)]
    item_cues_pending: bool,
    #[serde(default)]
    item_focus_observation: Option<crate::item_visual::FocusObservation>,
    #[serde(default)]
    descendant_focus: Option<DescendantQuery>,
    #[serde(default)]
    submenu_marker: Option<SubmenuMarkerOffer>,
    #[serde(default)]
    item_cue: Option<crate::item_visual::CueOffer>,
    #[serde(default)]
    item_root_cue: Option<RootCueOffer>,
    #[serde(default)]
    item_focus_host: Option<HostFacts>,
    #[serde(default)]
    roving_pending: Option<u64>,
    #[serde(default)]
    roving_write: bool,
    #[serde(default)]
    roving_sequence: Option<u64>,
    #[serde(default)]
    roving_event_sequence: Option<u64>,
    #[serde(default)]
    roving_root: Option<String>,
    #[serde(default)]
    roving_generation: Option<u64>,
    #[serde(default)]
    roving_item: Option<String>,
    #[serde(default)]
    roving_tabindex: Option<String>,
    #[serde(default)]
    item_focus_sequence: Option<u64>,
    #[serde(default)]
    item_focus_root: Option<String>,
    #[serde(default)]
    item_focus_generation: Option<u64>,
    #[serde(default)]
    item_focus_target: Option<String>,
    #[serde(default)]
    item_focus_capture: Option<String>,
    #[serde(default)]
    item_focus_phase: Option<String>,
    #[serde(default)]
    activate_target: Option<String>,
    #[serde(default)]
    focus_target: Option<String>,
    #[serde(default)]
    immediate_stop: bool,
    #[serde(default)]
    schedule_focus_target: Option<String>,
    #[serde(default)]
    dispatch_focus_target: Option<String>,
    #[serde(default)]
    pending_focus_target: Option<String>,
    /// Current Popover ancestors of the captured native destination object.
    #[serde(default)]
    focus_hosts: Vec<String>,
    #[serde(default)]
    trigger_restore_sequence: Option<u64>,
    #[serde(default)]
    trigger_restore_root: Option<String>,
    #[serde(default)]
    trigger_restore_generation: Option<u64>,
    #[serde(default)]
    completion_sequence: Option<u64>,
    #[serde(default)]
    completion_root: Option<String>,
    #[serde(default)]
    completion_generation: Option<u64>,
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

    fn host_click_value(runtime: &RuntimeHandle, sequence: u64) -> serde_json::Value {
        serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),
            "sequence":sequence,"kind":"click","phase":"bubble","hosts":["origin"],
            "nodes":[{"id":"origin","generation":1,"connected":true,"discovery_ready":true},
                {"id":"both","generation":1,"connected":true,"discovery_ready":true},
                {"id":"outside","generation":1,"connected":true,"discovery_ready":true},
                {"id":"inside","generation":1,"connected":true,"discovery_ready":true},
                {"id":"none","generation":1,"connected":true,"discovery_ready":true}],
            "click_sites":[{"root":"origin","generation":1,"connected":true,"receiver":"host","popover":true},
                {"root":"origin","generation":1,"connected":true,"receiver":"root"}]})
    }

    #[test]
    fn host_click_fallthrough_keeps_origin_and_respects_each_peer_policy() {
        for (behavior, prevented) in [
            (CloseBehavior::Outside, false),
            (CloseBehavior::None, false),
            (CloseBehavior::Both, true),
            (CloseBehavior::Inside, true),
        ] {
            let runtime = RuntimeHandle::new();
            let log = Rc::new(RefCell::new(Vec::new()));
            let origin = model(&runtime, "origin", OverlayKind::Dropdown, log.clone());
            origin.node.set_close_behavior(behavior);
            origin.node.set_host_click_receivers(true, true);
            let peers = [
                model(&runtime, "both", OverlayKind::Dropdown, log.clone()),
                model(&runtime, "outside", OverlayKind::Dropdown, log.clone()),
                model(&runtime, "inside", OverlayKind::Dropdown, log.clone()),
                model(&runtime, "none", OverlayKind::Dropdown, log.clone()),
            ];
            peers[1].node.set_close_behavior(CloseBehavior::Outside);
            peers[2].node.set_close_behavior(CloseBehavior::Inside);
            peers[3].node.set_close_behavior(CloseBehavior::None);
            origin.open.set(true);
            for peer in &peers {
                peer.open.set(true);
            }
            let mut value = host_click_value(&runtime, 1);
            value["default_prevented"] = prevented.into();
            runtime.input(event(value));
            assert!(
                origin.open.get(),
                "host fallthrough never closes its retained origin"
            );
            assert_eq!(
                peers.map(|peer| peer.open.get()),
                [false, false, true, true]
            );
            assert_eq!(
                *log.borrow(),
                [
                    "before:both:false",
                    "apply:both:false",
                    "before:outside:false",
                    "apply:outside:false"
                ]
            );
        }
    }

    #[test]
    fn host_click_receiver_guards_reject_old_disconnected_and_non_source_sites() {
        for (kind, receiver, generation, connected, popup_path) in [
            (OverlayKind::Dropdown, "root", 0, true, true),
            (OverlayKind::Dropdown, "root", 1, false, true),
            (OverlayKind::Dropdown, "unknown", 1, true, true),
            (OverlayKind::Header, "root", 1, true, false),
            (OverlayKind::Dropdown, "root", 1, true, false),
            (OverlayKind::Popover, "root", 1, true, true),
        ] {
            let runtime = RuntimeHandle::new();
            let log = Rc::new(RefCell::new(Vec::new()));
            let origin = model(&runtime, "origin", kind, log.clone());
            origin.open.set(true);
            let peer = model(&runtime, "both", OverlayKind::Dropdown, log.clone());
            peer.open.set(true);
            let mut value = host_click_value(&runtime, 1);
            if !popup_path {
                value["hosts"] = serde_json::json!([]);
            }
            value["click_sites"] = serde_json::json!([{"root":"origin","generation":generation,"connected":connected,"receiver":receiver,"popover":true}]);
            runtime.input(event(value));
            assert!(origin.open.get());
            assert!(peer.open.get());
            assert!(log.borrow().is_empty());
        }
    }

    #[test]
    fn host_click_uses_source_layer_and_live_trigger_guards_without_early_outside_veto() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let origin = model(&runtime, "origin", OverlayKind::Dropdown, log.clone());
        origin.node.set_close_behavior(CloseBehavior::Outside);
        origin.open.set(true);
        let peer = model(&runtime, "both", OverlayKind::Dropdown, log.clone());
        peer.open.set(true);
        peer.veto.set(true);
        // A source non-top-layer component can still use a native popover in
        // the renderer. Its physical dialog is not another source receiver.
        runtime.input(event(host_click_value(&runtime, 1)));
        assert_eq!(*log.borrow(), ["before:both:false"]);
        log.borrow_mut().clear();
        let mut capture = host_click_value(&runtime, 2);
        capture["phase"] = "capture".into();
        capture["triggers"] = serde_json::json!(["origin"]);
        let captured = event(capture);
        runtime.input(captured.clone());
        assert!(captured.default_action_enabled());
        assert!(origin.node.trigger_click_allowed());
        // The own native target may attempt its toggle before the later host
        // guard prevents that original click; the host does not run others.
        let mut own = host_click_value(&runtime, 3);
        own["triggers"] = serde_json::json!(["origin"]);
        let own = event(own);
        runtime.input(own.clone());
        assert!(!own.default_action_enabled());
        assert!(log.borrow().is_empty());
        let mut foreign = host_click_value(&runtime, 4);
        foreign["triggers"] = serde_json::json!(["both"]);
        let foreign = event(foreign);
        runtime.input(foreign.clone());
        assert!(!foreign.default_action_enabled());
        assert!(log.borrow().is_empty());
        let mut enclosing = host_click_value(&runtime, 5);
        enclosing["phase"] = "capture".into();
        enclosing["triggers"] = serde_json::json!(["origin"]);
        enclosing["enclosing_triggers"] = serde_json::json!(["origin"]);
        let enclosing = event(enclosing);
        runtime.input(enclosing.clone());
        assert!(
            enclosing.default_action_enabled(),
            "wrapper guard does not preempt native target defaults"
        );
        assert!(
            !origin.node.trigger_click_allowed(),
            "an Outside trigger enclosing its host waits for the host guard"
        );
    }

    #[test]
    fn host_click_retains_two_local_veto_passes_and_independent_window_retry() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let origin = model(&runtime, "origin", OverlayKind::Dropdown, log.clone());
        origin.open.set(true);
        origin.node.set_close_behavior(CloseBehavior::Outside);
        origin.node.set_host_click_receivers(true, true);
        let both = model(&runtime, "both", OverlayKind::Dropdown, log.clone());
        both.open.set(true);
        both.veto.set(true);
        let outside = model(&runtime, "outside", OverlayKind::Dropdown, log.clone());
        outside.open.set(true);
        runtime.input(event(host_click_value(&runtime, 1)));
        assert_eq!(
            *log.borrow(),
            [
                "before:both:false",
                "before:outside:false",
                "apply:outside:false",
                "before:both:false"
            ]
        );
        assert!(origin.open.get() && both.open.get());
        assert!(!outside.open.get());
        let mut window = host_click_value(&runtime, 2);
        window["phase"] = "window".into();
        window["window_sequence"] = 1.into();
        runtime.input(event(window.clone()));
        assert_eq!(
            log.borrow()
                .iter()
                .filter(|s| s.as_str() == "before:both:false")
                .count(),
            3
        );
        window["sequence"] = 3.into();
        runtime.input(event(window));
        assert_eq!(
            log.borrow()
                .iter()
                .filter(|s| s.as_str() == "before:both:false")
                .count(),
            3,
            "same Window event is not a fourth pass"
        );
    }

    #[test]
    fn header_more_click_does_not_acquire_a_popover_parents_dropdown_guard() {
        for kind in [
            OverlayKind::Popover,
            OverlayKind::Dropdown,
            OverlayKind::Header,
        ] {
            for child_open in [false, true] {
                let runtime = RuntimeHandle::new();
                let log = Rc::new(RefCell::new(Vec::new()));
                let parent = model(&runtime, "parent", kind, log.clone());
                parent.open.set(true);
                let header = model(&runtime, "header", OverlayKind::Header, log.clone());
                header.open.set(child_open);
                let report = serde_json::json!({"owner":runtime.owner_id(),"document":runtime.owner_id(),"sequence":1,
                    "kind":"click","phase":"bubble","hosts":["parent"],"triggers":["header"],
                    "nodes":[{"id":"parent","generation":1,"connected":true,"discovery_ready":true},
                        {"id":"header","generation":1,"connected":true,"discovery_ready":true,"trigger_ancestors":["parent"]}],
                    "click_sites":[{"root":"parent","generation":1,"connected":true,"receiver":"root"}]});
                let input = event(report);
                runtime.input(input.clone());
                assert_eq!(
                    input.default_action_enabled(),
                    kind == OverlayKind::Popover,
                    "kind={kind:?},child_open={child_open}"
                );
                assert!(parent.open.get());
                assert_eq!(header.open.get(), child_open);
                assert!(log.borrow().is_empty());
            }
        }
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
    fn completion_event(runtime: &RuntimeHandle, frame: &(u64, String, u64)) -> FormEvent {
        event(serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
            "completion_sequence":frame.0,"completion_root":frame.1,"completion_generation":frame.2}))
    }

    fn item_event(
        runtime: &RuntimeHandle,
        job: &ItemFocusJob,
        phase: &str,
        capture: &str,
    ) -> FormEvent {
        event(serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
            "item_focus_sequence":job.sequence,"item_focus_root":job.root,
            "item_focus_generation":job.generation,"item_focus_target":job.target,
            "item_focus_capture":capture,"item_focus_phase":phase}))
    }

    fn mixed_inventory_facts() -> InventoryFacts {
        serde_json::from_value(serde_json::json!({"items_host":"actual-items-host",
            "hosts":[
                {"item":"actual-a","kind":"dropdown","target":"a","capture":"actual-a","registration_key":"key-a"},
                {"item":"actual-b","kind":"menu","target":"b","capture":"actual-b","registration_key":"key-b","tabindex_attribute":"4"}
            ],"navigation_order":["actual-a","actual-b"],"query_order":["actual-a","actual-b"]})).unwrap()
    }
    #[derive(Clone, Default)]
    struct PublicationFixture {
        runtime: Rc<RefCell<Option<RuntimeHandle>>>,
        visible: Rc<Cell<Option<Signal<bool>>>>,
    }
    impl PartialEq for PublicationFixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.runtime, &other.runtime)
        }
    }
    fn publication_root(fixture: PublicationFixture) -> Element {
        let runtime = use_overlay_runtime();
        *fixture.runtime.borrow_mut() = Some(runtime.clone());
        let visible = use_signal(|| true);
        fixture.visible.set(Some(visible));
        rsx! {
            if visible() {
                crate::Dropdown { label: "Publication", suppress_trigger_visibility_check: true,
                    crate::DropdownItem { id: "publication-dropdown", label: "Dropdown" }
                    crate::SelectItem { id: "publication-select", value: "select", label: "Select" }
                    crate::MenuItem { id: "publication-menu", label: "Menu" }
                    crate::MenuCategory { id: "publication-category", label: "Category", span { "Child" } }
                }
            }
            OverlayTransport { runtime }
        }
    }
    struct PublicationConverter;
    macro_rules! publication_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for PublicationConverter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;publication_converter!(@data $group,event)})*
            }
        };
        (@data Form,$event:ident)=>{dioxus::html::FormData::new(RawForm($event.downcast::<String>().unwrap().clone()))};
        (@data $other:ident,$event:ident)=>{panic!("Unexpected publication test event")};
    }
    dioxus::html::with_html_event_groups!(publication_converter);
    fn publication_dispatch(
        dom: &VirtualDom,
        transport: dioxus::core::ElementId,
        name: &str,
        value: serde_json::Value,
    ) -> bool {
        let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
            value.to_string(),
        )));
        let original = Event::new(data, true);
        dom.runtime()
            .handle_event(name, original.clone(), transport);
        !original.default_action_enabled()
    }
    fn publication_attr(tag: &str, name: &str) -> Option<String> {
        let rest = tag.split_once(&format!(" {name}="))?.1;
        if let Some(quoted) = rest.strip_prefix('"') {
            Some(quoted.split('"').next()?.to_string())
        } else {
            Some(rest.split([' ', '>']).next()?.to_string())
        }
    }
    fn publication_tag<'a>(html: &'a str, id: &str) -> &'a str {
        html.split('>')
            .find(|tag| publication_attr(tag, "id").as_deref() == Some(id))
            .unwrap_or_else(|| panic!("Missing rendered source host {id}"))
    }
    fn publication_cues(html: &str, expected: Option<&str>) {
        for id in [
            "publication-dropdown",
            "publication-select",
            "publication-menu",
        ] {
            assert_eq!(
                publication_attr(publication_tag(html, id), "data-active").as_deref(),
                Some(if expected == Some(id) {
                    "true"
                } else {
                    "false"
                }),
                "rendered cue on {id}, expected {expected:?}"
            );
        }
        assert_eq!(
            publication_attr(publication_tag(html, "publication-category"), "data-active"),
            None,
            "Category's outer host never gains the private parent's marker"
        );
        let select_inner = html
            .split('>')
            .find(|tag| {
                publication_attr(tag, "class")
                    .is_some_and(|class| class.contains("ix-select-item-option"))
            })
            .unwrap();
        assert_eq!(
            publication_attr(select_inner, "data-active").as_deref(),
            Some(if expected == Some("publication-select") {
                "true"
            } else {
                "false"
            }),
            "Select's generated inner cue follows its source host"
        );
        let root = html
            .split('>')
            .find(|tag| {
                publication_attr(tag, "aria-label").as_deref() == Some("Publication")
                    && publication_attr(tag, "data-ix-overlay-root").is_some()
            })
            .unwrap();
        assert_eq!(
            publication_attr(root, "aria-activedescendant").as_deref(),
            expected
        );
    }
    fn publication_focus(
        dom: &VirtualDom,
        transport: dioxus::core::ElementId,
        runtime: &RuntimeHandle,
        job: &ItemFocusJob,
    ) {
        for phase in ["arm", "focus", "visual"] {
            let host = runtime
                .0
                .borrow()
                .inventories
                .get(&job.root)
                .and_then(|(_, facts)| {
                    facts
                        .hosts
                        .iter()
                        .find(|host| Some(host.capture.as_str()) == job.capture.as_deref())
                })
                .cloned();
            assert!(publication_dispatch(
                dom,
                transport,
                "change",
                serde_json::json!({
                    "owner":runtime.owner_id(),"sequence":0,"item_focus_sequence":job.sequence,
                    "item_focus_root":job.root,"item_focus_generation":job.generation,
                    "item_focus_target":job.target,"item_focus_capture":job.capture,"item_focus_phase":phase,
                    "item_focus_host":if phase == "arm" { None } else { host }
                })
            ));
        }
    }
    #[test]
    fn actual_vdom_focus_ack_publishes_mixed_host_cues_without_another_input() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(PublicationConverter));
        let fixture = PublicationFixture::default();
        let mut dom = VirtualDom::new_with_props(publication_root, fixture.clone());
        let edits = dom.rebuild_to_vec();
        let runtime = fixture.runtime.borrow().clone().unwrap();
        assert!(
            runtime.0.borrow().item_visual_signal.is_some(),
            "real VDOM signal path"
        );
        let mut current = dioxus::core::ElementId::ROOT;
        let mut transport = None;
        for edit in edits.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "data-ix-overlay-transport" && value == runtime.owner_id() => {
                    transport = Some(current)
                }
                _ => {}
            }
        }
        let transport = transport.expect("actual rendered overlay transport");
        let initial = dioxus_ssr::render(&dom);
        let root_tag = initial
            .split('>')
            .find(|tag| {
                publication_attr(tag, "aria-label").as_deref() == Some("Publication")
                    && publication_attr(tag, "data-ix-overlay-root").is_some()
            })
            .unwrap();
        let root = publication_attr(root_tag, "data-ix-overlay-root").unwrap();
        let generation: u64 = publication_attr(root_tag, "data-ix-overlay-generation")
            .unwrap()
            .parse()
            .unwrap();
        let node = NodeHandle {
            runtime: Rc::downgrade(&runtime.0),
            id: root.clone(),
            generation,
        };
        dom.in_runtime(|| assert!(node.request(true, Reason::Trigger, false)));
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        let html = dioxus_ssr::render(&dom);
        let hosts = [
            ("publication-dropdown", "dropdown"),
            ("publication-select", "select"),
            ("publication-menu", "menu"),
            ("publication-category", "category"),
        ].into_iter().map(|(id,kind)| {
            let tag = publication_tag(&html,id);
            serde_json::json!({"item":format!("actual-{id}"),"capture":format!("actual-{id}"),
                "kind":kind,"target":id,"registration_key":publication_attr(tag,"data-ix-item-key").unwrap(),
                "source_focusable":true})
        }).collect::<Vec<_>>();
        publication_cues(&html, None);
        let order = hosts
            .iter()
            .map(|host| host["item"].clone())
            .collect::<Vec<_>>();
        let report = serde_json::json!({"owner":runtime.owner_id(),"sequence":1,
            "kind":"key","phase":"bubble","key":"Home","hosts":[root],"triggers":[root],
            "nodes":[{"id":root,"generation":generation,"connected":true,"discovery_ready":true,
                "mode":"active-descendant","inside":true,"on_trigger":true,
                "inventory":{"items_host":"actual-publication-items","hosts":hosts,
                    "navigation_order":order,"query_order":order}}]});
        assert!(publication_dispatch(&dom, transport, "input", report));
        // Admit independent A/B operations before either first stage. Their
        // first ACKs must each publish without a later original native input.
        let first = runtime
            .pending_item_focus_jobs()
            .into_iter()
            .last()
            .unwrap();
        assert_eq!(first.target, "publication-dropdown");
        let second_facts: HostFacts = serde_json::from_value(hosts[1].clone()).unwrap();
        dom.in_runtime(|| assert!(node.queue_item_host_focus(&second_facts, Rc::new(|| {}))));
        let second = runtime
            .pending_item_focus_jobs()
            .into_iter()
            .last()
            .unwrap();
        assert_ne!(first.sequence, second.sequence);
        publication_focus(&dom, transport, &runtime, &first);
        assert_eq!(
            runtime.item_visual_target().as_deref(),
            Some("publication-dropdown")
        );
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        publication_cues(&dioxus_ssr::render(&dom), Some("publication-dropdown"));
        publication_focus(&dom, transport, &runtime, &second);
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        publication_cues(&dioxus_ssr::render(&dom), Some("publication-select"));
        // No source focusin is synthesized: a nonfocusable host must publish
        // from its frame callback alone. All remaining subscribers use the
        // same registered queue, without forced parent notification.
        for id in ["publication-menu", "publication-category"] {
            let facts: HostFacts = serde_json::from_value(
                hosts
                    .iter()
                    .find(|host| host["target"] == id)
                    .unwrap()
                    .clone(),
            )
            .unwrap();
            dom.in_runtime(|| assert!(node.queue_item_host_focus(&facts, Rc::new(|| {}))));
            let job = runtime
                .pending_item_focus_jobs()
                .into_iter()
                .last()
                .unwrap();
            publication_focus(&dom, transport, &runtime, &job);
            for _ in 0..4 {
                dom.render_immediate_to_vec();
            }
            publication_cues(
                &dioxus_ssr::render(&dom),
                (id != "publication-category").then_some(id),
            );
        }
        let facts: HostFacts = serde_json::from_value(hosts[0].clone()).unwrap();
        dom.in_runtime(|| assert!(node.queue_item_host_focus(&facts, Rc::new(|| {}))));
        let dropped = runtime
            .pending_item_focus_jobs()
            .into_iter()
            .last()
            .unwrap();
        dom.in_runtime(|| fixture.visible.get().unwrap().set(false));
        for _ in 0..4 {
            dom.render_immediate_to_vec();
        }
        assert!(!publication_dispatch(
            &dom,
            transport,
            "change",
            serde_json::json!({
                "owner":runtime.owner_id(),"sequence":0,"item_focus_sequence":dropped.sequence,
                "item_focus_root":dropped.root,"item_focus_generation":dropped.generation,
                "item_focus_target":dropped.target,"item_focus_capture":dropped.capture,"item_focus_phase":"focus"
            })
        ));
        assert!(!dioxus_ssr::render(&dom).contains("publication-dropdown"));
    }
    fn roving_event(
        runtime: &RuntimeHandle,
        sequence: u64,
        item: &str,
        value: Option<&str>,
    ) -> FormEvent {
        event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,
            "roving_write":true,"roving_event_sequence":sequence,"roving_item":item,"roving_tabindex":value}),
        )
    }

    #[test]
    fn registered_mixed_inventory_keeps_logical_roving_writes_until_exact_fifo_ack_and_restores_raw_attrs()
     {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "mixed-root",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        assert!(root.node.request(true, Reason::Trigger, false));
        assert!(root.node.configure_navigation("roving-tabindex", true));
        let selected = root.node.clone();
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("mixed-root")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(move |facts| {
            let raw = facts.inventory.clone().unwrap();
            let inventory = crate::dropdown_inventory::Inventory::new(
                crate::dropdown_inventory::InventoryMode::RovingTabindex,
                raw.clone(),
            );
            let target = inventory
                .navigate(&facts.key, inventory.active().as_ref())
                .unwrap();
            assert!(selected.select_roving(&raw, &target));
            KeyDecision {
                prevent: true,
                ..KeyDecision::default()
            }
        }));
        let raw = mixed_inventory_facts();
        for (sequence, key) in [(1, "End"), (2, "ArrowUp")] {
            let input = event(
                serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"key","phase":"capture","key":key,
                "hosts":["mixed-root"],"nodes":[{"id":"mixed-root","generation":1,"connected":true,"discovery_ready":true,
                    "mode":"roving-tabindex","inside":true,"inventory":raw}]}),
            );
            runtime.input(input.clone());
            assert!(!input.default_action_enabled());
        }
        let logical = root.node.inventory().unwrap();
        assert_eq!(logical.hosts[0].tabindex_attribute.as_deref(), Some("0"));
        assert_eq!(logical.hosts[1].tabindex_attribute.as_deref(), Some("-1"));
        assert_eq!(
            raw.hosts[0].tabindex_attribute, None,
            "raw diagnostics are not rewritten"
        );
        let overtaking = roving_event(&runtime, 2, "actual-b", Some("-1"));
        runtime.query(overtaking.clone());
        assert!(
            overtaking.default_action_enabled(),
            "later event cannot overtake an earlier batch"
        );
        for (sequence, item, value) in [
            (1, "actual-a", "-1"),
            (1, "actual-b", "0"),
            (2, "actual-b", "-1"),
            (2, "actual-a", "0"),
        ] {
            let wrong = roving_event(&runtime, sequence, item, Some("wrong"));
            runtime.query(wrong.clone());
            assert!(wrong.default_action_enabled());
            let ack = roving_event(&runtime, sequence, item, Some(value));
            runtime.query(ack.clone());
            assert!(!ack.default_action_enabled());
        }
        assert!(runtime.0.borrow().roving_writes.is_empty());
        root.node.clear_roving();
        let (sequence, writes) = {
            let state = runtime.0.borrow();
            let batch = state.roving_writes.front().unwrap();
            (
                batch.sequence,
                batch.writes.iter().cloned().collect::<Vec<_>>(),
            )
        };
        assert_eq!(
            writes,
            [
                AttributeWrite {
                    item: ItemRef("actual-a".into()),
                    tabindex: None
                },
                AttributeWrite {
                    item: ItemRef("actual-b".into()),
                    tabindex: Some("4".into())
                }
            ]
        );
        assert!(root.node.request(false, Reason::Trigger, false));
        for write in writes {
            let value = serde_json::json!({"owner":runtime.owner_id(),"sequence":0,"roving_write":true,
                "roving_sequence":sequence,"roving_root":"mixed-root","roving_generation":1,
                "roving_item":write.item,"roving_tabindex":write.tabindex});
            let ack = event(value.clone());
            runtime.query(ack.clone());
            assert!(
                !ack.default_action_enabled(),
                "accepted hide does not erase admitted restore writes"
            );
            let replay = event(value);
            runtime.query(replay.clone());
            assert!(replay.default_action_enabled());
        }
        assert!(runtime.0.borrow().roving_writes.is_empty());
    }

    #[test]
    fn item_visual_marks_only_on_keyboard_focus_delivery_and_never_infers_category_outer_marker() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "visual-root",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        let mut raw = mixed_inventory_facts();
        raw.hosts[1].visible_focus = true;
        for host in &raw.hosts {
            runtime.register_item_cue(host.registration_key.clone(), host.kind);
        }
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"document":"mixed-visual-runtime-test","kind":"observe",
            "nodes":[{"id":"visual-root","generation":1,"connected":true,"inventory":raw}]})));
        assert_eq!(
            runtime.item_visual_key(),
            None,
            "ordinary observation cannot invent a focus notification"
        );
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":2,"document":"mixed-visual-runtime-test","kind":"modality","input_kind":"keydown","key":"Tab"})));
        assert!(runtime.keyboard_mode());
        let host = raw.hosts[0].clone();
        let observed = runtime.clone();
        assert!(root.node.queue_item_host_focus(
            &host,
            Rc::new(move || {
                assert_eq!(observed.item_visual_key().as_deref(), Some("key-a"));
                assert_eq!(observed.item_visual_target().as_deref(), Some("a"));
            })
        ));
        let job = runtime.pending_item_focus_jobs().pop().unwrap();
        assert_eq!(job.capture.as_deref(), Some("actual-a"));
        assert!(job.focus_native);
        for phase in ["arm", "focus", "scroll"] {
            let ack = item_event(&runtime, &job, phase, "actual-a");
            runtime.query(ack.clone());
            assert!(!ack.default_action_enabled());
        }
        let mut category = host.clone();
        category.kind = ItemKind::Category;
        category.registration_key = "outer-category".into();
        runtime.mark_item_visual(&category);
        assert_eq!(runtime.item_visual_key(), None);
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":3,"document":"mixed-visual-runtime-test","kind":"item-focusin","item_focus_host":raw.hosts[1]})));
        assert_eq!(runtime.item_visual_key().as_deref(), Some("key-b"));
        let mut disabled_property = raw.hosts[1].clone();
        disabled_property.source_focusable = false;
        runtime.mark_item_visual(&disabled_property);
        assert_eq!(runtime.item_visual_key(), None);
        runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":4,"document":"mixed-visual-runtime-test","kind":"modality","input_kind":"mousedown"})));
        runtime.mark_item_visual(&host);
        assert_eq!(
            runtime.item_visual_key(),
            None,
            "pointer mode may not create a visible-focus marker"
        );
    }

    #[test]
    fn item_frames_restore_owner_keep_each_capture_and_allow_next_focus_before_scroll() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let caller = fixture.0.caller.get().unwrap();
        let raw = event(
            serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"kind":"observe",
            "nodes":[{"id":node.id(),"generation":node.generation(),"connected":true,
                "item_focus_captures":[{"target":"a","capture":"actual-a"},{"target":"b","capture":"actual-b"}]}]}),
        );
        dom.in_scope(owner, || runtime.input(raw));
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let first = delivered.clone();
        let nested = node.clone();
        dom.in_scope(caller, || {
            assert!(node.queue_item_focus(
                "a".into(),
                false,
                Rc::new(move || {
                    assert_eq!(current_scope_id(), owner);
                    assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                    first.borrow_mut().push("a");
                    let later = first.clone();
                    assert!(nested.queue_item_focus(
                        "c".into(),
                        true,
                        Rc::new(move || later.borrow_mut().push("c"))
                    ));
                })
            ));
            let second = delivered.clone();
            assert!(node.queue_item_focus(
                "b".into(),
                true,
                Rc::new(move || {
                    assert_eq!(current_scope_id(), owner);
                    second.borrow_mut().push("b");
                })
            ));
        });
        assert!(delivered.borrow().is_empty());
        let jobs = runtime.pending_item_focus_jobs();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].capture.as_deref(), Some("actual-a"));
        assert_eq!(jobs[1].capture.as_deref(), Some("actual-b"));
        assert!(!jobs[0].focus_native && jobs[1].focus_native);
        dom.in_scope(owner, || runtime.attach_document("item-initial-bootstrap"));
        assert_eq!(runtime.pending_item_focus_jobs(), jobs);
        dom.in_scope(ScopeId::APP, || {
            let mut show = fixture.0.show_caller.get().unwrap();
            show.set(false);
        });
        dom.render_immediate_to_vec();
        assert!(fixture.0.caller_dropped.get());
        fixture.0.open.set(false);
        for job in &jobs {
            let arm = item_event(&runtime, job, "arm", job.capture.as_deref().unwrap());
            dom.in_scope(ScopeId::APP, || runtime.query(arm.clone()));
            assert!(!arm.default_action_enabled());
        }
        // A's second frame is still pending when B's first frame runs. New C
        // is admitted reentrantly, but is not part of this captured frame batch.
        for job in &jobs {
            let focus = item_event(&runtime, job, "focus", job.capture.as_deref().unwrap());
            runtime.query(focus.clone());
            assert!(!focus.default_action_enabled());
        }
        assert_eq!(*delivered.borrow(), ["a", "b"]);
        for job in &jobs {
            let scroll = item_event(&runtime, job, "scroll", job.capture.as_deref().unwrap());
            runtime.query(scroll.clone());
            assert!(!scroll.default_action_enabled());
            let repeat = item_event(&runtime, job, "scroll", job.capture.as_deref().unwrap());
            runtime.query(repeat.clone());
            assert!(repeat.default_action_enabled());
        }
        let later = runtime.pending_item_focus_jobs();
        assert_eq!(later.len(), 1);
        assert_eq!(later[0].target, "c");
        drop(dom);
        assert!(runtime.pending_item_focus_jobs().is_empty());
        assert!(!node.queue_item_focus("d".into(), true, Rc::new(|| panic!("dropped owner"))));
        assert_eq!(*delivered.borrow(), ["a", "b"]);
    }

    #[test]
    fn item_queries_reject_wrong_identity_and_discard_failed_bootstrap_without_losing_peers() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let owner = model(&runtime, "item-root", OverlayKind::Dropdown, log.clone());
        let delivered = Rc::new(Cell::new(0));
        let observed = delivered.clone();
        assert!(owner.node.queue_item_focus(
            "item".into(),
            true,
            Rc::new(move || observed.set(observed.get() + 1))
        ));
        let job = runtime.pending_item_focus_jobs().pop().unwrap();
        let retained = Rc::new(());
        let released = Rc::downgrade(&retained);
        assert!(owner.node.queue_item_focus(
            "missing".into(),
            false,
            Rc::new(move || {
                let _ = &retained;
            })
        ));
        let missing = runtime.pending_item_focus_jobs().pop().unwrap();
        let discard = event(serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
            "item_focus_phase":"discard","item_focus_sequence":missing.sequence,
            "item_focus_root":missing.root,"item_focus_generation":missing.generation,"item_focus_target":missing.target}));
        runtime.query(discard);
        assert!(released.upgrade().is_none());
        assert_eq!(
            runtime.pending_item_focus_jobs().as_slice(),
            std::slice::from_ref(&job)
        );
        for phase in ["focus", "scroll", "unknown"] {
            let ack = item_event(&runtime, &job, phase, "actual");
            runtime.query(ack.clone());
            assert!(ack.default_action_enabled());
        }
        let wrong_owner = event(serde_json::json!({"owner":"wrong-runtime","sequence":0,
            "item_focus_sequence":job.sequence,"item_focus_root":job.root,
            "item_focus_generation":job.generation,"item_focus_target":job.target,
            "item_focus_capture":"actual","item_focus_phase":"arm"}));
        runtime.query(wrong_owner.clone());
        assert!(wrong_owner.default_action_enabled());
        let arm = item_event(&runtime, &job, "arm", "actual");
        runtime.query(arm.clone());
        assert!(!arm.default_action_enabled());
        let wrong_capture = item_event(&runtime, &job, "focus", "replacement-object");
        runtime.query(wrong_capture.clone());
        assert!(wrong_capture.default_action_enabled());
        let joined = model(&runtime, "item-root", OverlayKind::Dropdown, log);
        let focus = item_event(&runtime, &job, "focus", "actual");
        runtime.query(focus.clone());
        assert!(!focus.default_action_enabled());
        assert_eq!(delivered.get(), 1);
        let replacement = runtime.register(
            "item-root".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(|| Some(false)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        );
        assert!(runtime.pending_item_focus_jobs().is_empty());
        let stale = item_event(&runtime, &job, "scroll", "actual");
        runtime.query(stale.clone());
        assert!(stale.default_action_enabled());
        assert!(!joined.node.queue_item_focus(
            "old".into(),
            true,
            Rc::new(|| panic!("old generation"))
        ));
        assert!(replacement.queue_item_focus("new".into(), true, Rc::new(|| {})));
        replacement.unregister();
        assert!(runtime.pending_item_focus_jobs().is_empty());
        assert!(runtime.0.borrow().item_focus_callbacks.is_empty());
    }

    #[test]
    fn item_first_frame_reentrant_owner_replacement_revokes_native_execution() {
        let runtime = RuntimeHandle::new();
        let owner = model(
            &runtime,
            "item-root",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        let called = Rc::new(Cell::new(0));
        let changed = called.clone();
        let weak = Rc::downgrade(&runtime.0);
        assert!(owner.node.queue_item_focus(
            "item".into(),
            false,
            Rc::new(move || {
                changed.set(changed.get() + 1);
                let runtime = RuntimeHandle(weak.upgrade().unwrap());
                runtime.register(
                    "item-root".into(),
                    2,
                    OverlayKind::Dropdown,
                    CloseBehavior::Both,
                    Callbacks {
                        read: Rc::new(|| Some(false)),
                        before: Rc::new(|_| true),
                        apply: Rc::new(|_| {}),
                        key: None,
                    },
                );
            })
        ));
        let job = runtime.pending_item_focus_jobs().pop().unwrap();
        let arm = item_event(&runtime, &job, "arm", "actual");
        runtime.query(arm.clone());
        assert!(!arm.default_action_enabled());
        let focus = item_event(&runtime, &job, "focus", "actual");
        runtime.query(focus.clone());
        assert!(
            focus.default_action_enabled(),
            "old callback may not authorize the replacement's native focus"
        );
        assert_eq!(called.get(), 1);
        assert!(runtime.pending_item_focus_jobs().is_empty());
        let repeat = item_event(&runtime, &job, "focus", "actual");
        runtime.query(repeat.clone());
        assert!(repeat.default_action_enabled());
        assert_eq!(called.get(), 1);

        // A component can lose its readable state without a new registration
        // being installed. Denied native focus must not strand a scroll stage
        // that native code will never schedule.
        let available = Rc::new(Cell::new(true));
        let read = available.clone();
        let unreadable = runtime.register(
            "unreadable".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(move || read.get().then_some(false)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        );
        assert!(unreadable.queue_item_focus(
            "item".into(),
            false,
            Rc::new(move || available.set(false))
        ));
        let job = runtime.pending_item_focus_jobs().pop().unwrap();
        let arm = item_event(&runtime, &job, "arm", "readable-object");
        runtime.query(arm.clone());
        assert!(!arm.default_action_enabled());
        let focus = item_event(&runtime, &job, "focus", "readable-object");
        runtime.query(focus.clone());
        assert!(focus.default_action_enabled());
        assert!(runtime.pending_item_focus_jobs().is_empty());
        assert!(runtime.0.borrow().item_focus_callbacks.is_empty());
    }

    #[test]
    fn completion_frames_restore_registered_owner_and_defer_reentrant_work_past_snapshot() {
        let fixture = RequestTaskFixture(Rc::new(RequestTaskState::default()));
        let mut dom = VirtualDom::new_with_props(request_task_root, fixture.clone());
        dom.rebuild_in_place();
        let node = fixture.0.node.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        let owner = fixture.0.owner.get().unwrap();
        let caller = fixture.0.caller.get().unwrap();
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let first_log = delivered.clone();
        let nested_node = node.clone();
        dom.in_scope(caller, || {
            assert!(node.queue_completion(Rc::new(move || {
                assert_eq!(current_scope_id(), owner);
                assert_eq!(consume_context::<RequestTaskContext>().0, "owner");
                first_log.borrow_mut().push(1);
                let nested_log = first_log.clone();
                assert!(nested_node.queue_completion(Rc::new(move || {
                    assert_eq!(current_scope_id(), owner);
                    nested_log.borrow_mut().push(3);
                })));
            })));
            let second_log = delivered.clone();
            assert!(node.queue_completion(Rc::new(move || {
                assert_eq!(current_scope_id(), owner);
                second_log.borrow_mut().push(2);
            })));
        });
        assert!(
            delivered.borrow().is_empty(),
            "no synchronous fallback before the native bridge"
        );
        dom.in_scope(owner, || {
            runtime.attach_document("completion-owner-bootstrap")
        });
        let frame = runtime.pending_completion_frames();
        assert_eq!(
            frame.len(),
            2,
            "initial document attachment retains admitted writes"
        );
        dom.in_scope(ScopeId::APP, || {
            let mut show = fixture.0.show_caller.get().unwrap();
            show.set(false);
        });
        dom.render_immediate_to_vec();
        assert!(fixture.0.caller_dropped.get());
        // Later reports and the current closed state do not erase captured
        // completion payloads. Delivery still reads through the owner scope.
        fixture.0.open.set(false);
        dom.in_scope(owner, || {
            runtime.observe(&report(&runtime, 80, "observe", serde_json::json!([])));
        });
        let wrong_owner = event(serde_json::json!({"owner":"foreign-runtime","sequence":0,
            "completion_sequence":frame[0].0,"completion_root":frame[0].1,"completion_generation":frame[0].2}));
        runtime.query(wrong_owner.clone());
        assert!(wrong_owner.default_action_enabled());
        let out_of_order = completion_event(&runtime, &frame[1]);
        runtime.query(out_of_order.clone());
        assert!(out_of_order.default_action_enabled());
        assert!(delivered.borrow().is_empty());
        for captured in &frame {
            let ack = completion_event(&runtime, captured);
            dom.in_scope(ScopeId::APP, || runtime.query(ack.clone()));
            assert!(!ack.default_action_enabled());
            let repeat = completion_event(&runtime, captured);
            runtime.query(repeat.clone());
            assert!(repeat.default_action_enabled());
        }
        assert_eq!(*delivered.borrow(), [1, 2]);
        let next_frame = runtime.pending_completion_frames();
        assert_eq!(
            next_frame.len(),
            1,
            "reentrant work waits for the next frame snapshot"
        );
        let ack = completion_event(&runtime, &next_frame[0]);
        runtime.query(ack.clone());
        assert!(!ack.default_action_enabled());
        assert_eq!(*delivered.borrow(), [1, 2, 3]);
        let dropped_log = delivered.clone();
        assert!(node.queue_completion(Rc::new(move || dropped_log.borrow_mut().push(4))));
        let dropped_frame = runtime.pending_completion_frames().pop().unwrap();
        drop(dom);
        assert!(runtime.pending_completion_frames().is_empty());
        let stale = completion_event(&runtime, &dropped_frame);
        runtime.query(stale.clone());
        assert!(stale.default_action_enabled());
        assert_eq!(*delivered.borrow(), [1, 2, 3]);
        assert!(!node.queue_completion(Rc::new(|| panic!("dropped owner"))));
    }

    #[test]
    fn completion_ack_cannot_retarget_replaced_registration_or_consume_wrong_tuple() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let old = model(
            &runtime,
            "completion-root",
            OverlayKind::Dropdown,
            log.clone(),
        );
        let delivered = Rc::new(Cell::new(0));
        let first = delivered.clone();
        assert!(
            old.node
                .queue_completion(Rc::new(move || first.set(first.get() + 1)))
        );
        let old_frame = runtime.pending_completion_frames().pop().unwrap();
        let joined = model(&runtime, "completion-root", OverlayKind::Dropdown, log);
        assert_eq!(
            runtime.pending_completion_frames().as_slice(),
            std::slice::from_ref(&old_frame),
            "same-generation settings joins retain completion jobs"
        );
        let wrong_generation = completion_event(&runtime, &(old_frame.0, old_frame.1.clone(), 2));
        runtime.query(wrong_generation.clone());
        assert!(wrong_generation.default_action_enabled());
        let replacement = runtime.register(
            "completion-root".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(|| Some(false)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        );
        assert!(runtime.pending_completion_frames().is_empty());
        assert!(
            !old.node
                .queue_completion(Rc::new(|| panic!("old generation")))
        );
        assert!(
            !joined
                .node
                .queue_completion(Rc::new(|| panic!("same old generation")))
        );
        let current = delivered.clone();
        assert!(replacement.queue_completion(Rc::new(move || current.set(current.get() + 10))));
        let current_frame = runtime.pending_completion_frames().pop().unwrap();
        let stale = completion_event(&runtime, &old_frame);
        runtime.query(stale.clone());
        assert!(stale.default_action_enabled());
        assert_eq!(delivered.get(), 0);
        let current = completion_event(&runtime, &current_frame);
        runtime.query(current.clone());
        assert!(!current.default_action_enabled());
        assert_eq!(delivered.get(), 10);
    }
    fn key_value(runtime: &RuntimeHandle, sequence: u64, phase: &str) -> serde_json::Value {
        serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"key","phase":phase,"key":"Enter",
            "hosts":["child"],"triggers":["child"],"candidate_ids":["ix-dropdown-item-7"],
            "nodes":[{"id":"child","generation":1,"connected":true,"discovery_ready":true,"items":[7],"inside":true,"mode":"active-descendant"}]})
    }
    #[test]
    fn registered_mixed_host_activation_uses_captured_inventory_and_exact_live_owner() {
        use crate::dropdown_inventory::{Activation, Inventory, InventoryMode};

        for (kind, guard) in [
            ("dropdown", "valid"),
            ("select", "valid"),
            ("menu", "valid"),
            ("category", "valid"),
            ("native-opt-in", "native-default"),
            ("select", "no-active"),
            ("menu", "missing-capture"),
            ("category", "missing-candidate"),
            ("select", "outside-inventory"),
            ("select", "disconnected"),
            ("menu", "stale-generation"),
            ("category", "closed"),
            ("select", "opened-by-callback"),
            ("menu", "removed-by-callback"),
            ("category", "replaced-by-callback"),
            ("select", "dropped-before-ack"),
            ("menu", "closed-before-ack"),
        ] {
            let runtime = RuntimeHandle::new();
            let target = format!("consumer-{kind}-custom-id");
            let open = Rc::new(Cell::new(!matches!(guard, "closed" | "opened-by-callback")));
            let reads = open.clone();
            let callback_open = open.clone();
            let callback_runtime = runtime.clone();
            let callback_target = target.clone();
            let node = runtime.register(
                "mixed-activation".into(),
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
                        let inventory = Inventory::new(
                            InventoryMode::RovingTabindex,
                            facts.inventory.clone().unwrap(),
                        );
                        let activation = inventory.activation(
                            &facts.key,
                            &["Enter".into()],
                            inventory.active().as_ref(),
                        );
                        let (prevent, mut activate) = match activation {
                            Activation::SyntheticHost { item } => (
                                true,
                                item.and_then(|item| inventory.host(&item))
                                    .map(|host| host.target.clone()),
                            ),
                            Activation::NativeDefault | Activation::None => (false, None),
                            Activation::Submenu { .. } => unreachable!(),
                        };
                        match guard {
                            "outside-inventory" => activate = Some(callback_target.clone()),
                            "opened-by-callback" => callback_open.set(true),
                            "removed-by-callback" => {
                                callback_runtime.unregister("mixed-activation", 1);
                            }
                            "replaced-by-callback" => {
                                callback_runtime.register(
                                    "mixed-activation".into(),
                                    2,
                                    OverlayKind::Dropdown,
                                    CloseBehavior::Both,
                                    Callbacks {
                                        read: Rc::new(|| Some(true)),
                                        before: Rc::new(|_| true),
                                        apply: Rc::new(|_| {}),
                                        key: None,
                                    },
                                );
                            }
                            _ => {}
                        }
                        KeyDecision {
                            prevent,
                            activate,
                            ..KeyDecision::default()
                        }
                    })),
                },
            );
            let reported_target = if guard == "outside-inventory" {
                "different-actual-host"
            } else {
                &target
            };
            let original = event(serde_json::json!({
                "owner":runtime.owner_id(),"sequence":1,"kind":"key","phase":"capture","key":"Enter",
                "hosts":["mixed-activation"],
                "candidate_ids":if guard == "missing-candidate" { vec![] } else { vec![target.clone()] },
                "nodes":[{"id":"mixed-activation","generation":if guard == "stale-generation" {2} else {1},
                    "connected":guard != "disconnected","discovery_ready":true,
                    "inside":true,"mode":"roving-tabindex","inventory":{
                        "items_host":"actual-items-object",
                        "hosts":[{"item":"actual-host-object","kind":kind,"target":reported_target,
                            "capture":if guard == "missing-capture" {""} else {"actual-host-object"},
                            "tabindex_attribute":if guard == "no-active" {"-1"} else {"0"},
                            "roving_opt_in":kind == "native-opt-in"}],
                        "navigation_order":["actual-host-object"],"query_order":["actual-host-object"]
                    }}]
            }));
            runtime.input(original.clone());
            if guard == "native-default" {
                assert!(original.default_action_enabled(), "{kind}/{guard}");
            } else if matches!(guard, "valid" | "no-active") {
                assert!(!original.default_action_enabled(), "{kind}/{guard}");
            }
            if guard == "dropped-before-ack" {
                node.unregister();
            }
            if guard == "closed-before-ack" {
                open.set(false);
            }
            // Wrong tuples never consume the pending authorization.
            for (owner, sequence, query_target) in [
                ("foreign-owner".to_string(), 1, target.clone()),
                (runtime.owner_id(), 2, target.clone()),
                (runtime.owner_id(), 1, "another-target".into()),
            ] {
                let wrong = event(
                    serde_json::json!({"owner":owner,"sequence":sequence,"activate_target":query_target}),
                );
                runtime.query(wrong.clone());
                assert!(wrong.default_action_enabled(), "{kind}/{guard}");
            }
            let query = serde_json::json!({"owner":runtime.owner_id(),"sequence":1,"activate_target":target});
            let first = event(query.clone());
            runtime.query(first.clone());
            assert_eq!(
                !first.default_action_enabled(),
                guard == "valid",
                "{kind}/{guard}"
            );
            let replay = event(query);
            runtime.query(replay.clone());
            assert!(replay.default_action_enabled(), "{kind}/{guard} replay");
            if let Some(entry) = runtime.entry("mixed-activation") {
                runtime.unregister("mixed-activation", entry.generation);
            }
        }
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

        // Source114's own Outside trigger toggles before bubbling host
        // prevention. A veto retains visibility without blocking admission.
        let log = Rc::new(RefCell::new(Vec::new()));
        let owner = model(&runtime, "outside", OverlayKind::Dropdown, log);
        owner.node.set_close_behavior(CloseBehavior::Outside);
        assert!(owner.node.request(true, Reason::Trigger, false));
        owner.veto.set(true);
        let mut capture = serde_json::json!({"owner":runtime.owner_id(),"sequence":4,"kind":"click","phase":"capture","hosts":["outside"],"triggers":["outside"],"nodes":[{"id":"outside","generation":1,"connected":true,"discovery_ready":true}]});
        let original = event(capture.clone());
        runtime.input(original.clone());
        assert!(original.default_action_enabled() && owner.node.trigger_click_allowed());
        if owner.node.trigger_click_allowed() {
            assert!(!owner.node.request(false, Reason::Trigger, false));
        }
        assert!(owner.open.get());
        capture["sequence"] = 5.into();
        capture["phase"] = "bubble".into();
        capture["default_prevented"] = true.into();
        capture["click_sites"] = serde_json::json!([{"root":"outside","generation":1,"connected":true,"receiver":"root"}]);
        let host = event(capture.clone());
        runtime.input(host.clone());
        assert!(!host.default_action_enabled());
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
    // Insert into overlay_runtime.rs's existing #[cfg(test)] tests module.
    // Uses the real FormEvent input/query handlers; no native DOM/frame claim.
    fn configured_roving_observe(runtime: &RuntimeHandle, sequence: u64, nodes: serde_json::Value) {
        runtime.input(event(serde_json::json!({
            "owner":runtime.owner_id(),"sequence":sequence,"kind":"observe","nodes":nodes
        })));
    }
    fn configured_roving_empty(host: &str) -> InventoryFacts {
        serde_json::from_value(serde_json::json!({"items_host":host,"hosts":[],
        "navigation_order":[],"query_order":[]}))
        .unwrap()
    }
    fn configured_roving_retained(runtime: &RuntimeHandle, capture: &str) -> bool {
        let query = event(serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
        "item_focus_phase":"retain","item_focus_capture":capture}));
        runtime.query(query.clone());
        !query.default_action_enabled()
    }
    fn configured_roving_drain(
        runtime: &RuntimeHandle,
        attributes: &mut BTreeMap<ItemRef, Option<String>>,
    ) {
        loop {
            let next = {
                let state = runtime.0.borrow();
                state.roving_writes.front().map(|batch| {
                    (
                        batch.sequence,
                        batch.root.clone(),
                        batch.generation,
                        batch.writes.front().unwrap().clone(),
                    )
                })
            };
            let Some((sequence, root, generation, write)) = next else {
                break;
            };
            let query = event(serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
            "roving_write":true,"roving_sequence":sequence,"roving_root":root,
            "roving_generation":generation,"roving_item":write.item,
            "roving_tabindex":write.tabindex}));
            runtime.query(query.clone());
            assert!(
                !query.default_action_enabled(),
                "exact FIFO captured write must be authorized"
            );
            attributes.insert(write.item, write.tabindex);
        }
    }

    #[test]
    fn configured_roving_empty_host_is_retained_without_remembering_any_attributes() {
        let runtime = RuntimeHandle::new();
        let root = model(
            &runtime,
            "empty-roving",
            OverlayKind::Dropdown,
            Rc::new(RefCell::new(Vec::new())),
        );
        let facts = configured_roving_empty("actual-empty-host");
        configured_roving_observe(
            &runtime,
            1,
            serde_json::json!([
                {"id":"empty-roving","generation":1,"connected":true,"discovery_ready":true,
                    "root_capture":"actual-empty-root","inventory":facts}
            ]),
        );
        assert!(root.node.request(true, Reason::Trigger, false));
        assert!(root.node.configure_navigation("roving-tabindex", true));
        assert!(configured_roving_retained(&runtime, "actual-empty-host"));
        assert!(!configured_roving_retained(&runtime, "copied-empty-host"));
        assert!(
            !runtime.0.borrow().roving.retains_token("actual-empty-host"),
            "configuration enrolls the host but does not snapshot original item attributes"
        );
        assert!(runtime.0.borrow().roving_writes.is_empty());
        root.node.clear_roving();
        assert!(!configured_roving_retained(&runtime, "actual-empty-host"));
        assert!(runtime.0.borrow().roving_writes.is_empty());
    }

    #[test]
    fn pending_roving_configuration_survives_newer_modes_but_not_owner_replacement_or_clear() {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let root = model(
            &runtime,
            "pending-roving",
            OverlayKind::Dropdown,
            log.clone(),
        );
        assert!(root.node.configure_navigation("roving-tabindex", true));
        assert!(root.node.configure_navigation("active-descendant", false));
        configured_roving_observe(
            &runtime,
            1,
            serde_json::json!([
                {"id":"pending-roving","generation":1,"connected":true,"discovery_ready":true,
                    "inventory":configured_roving_empty("first-config-host")}
            ]),
        );
        assert!(
            configured_roving_retained(&runtime, "first-config-host"),
            "later mode configuration does not erase an earlier source cleanup enrollment"
        );
        root.node.clear_roving();
        assert!(!configured_roving_retained(&runtime, "first-config-host"));

        let replaced = model(
            &runtime,
            "replaced-roving",
            OverlayKind::Dropdown,
            log.clone(),
        );
        assert!(replaced.node.configure_navigation("roving-tabindex", true));
        let callbacks = runtime.entry("replaced-roving").unwrap().callbacks;
        let replacement = runtime.register(
            "replaced-roving".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            callbacks,
        );
        configured_roving_observe(
            &runtime,
            2,
            serde_json::json!([
                {"id":"replaced-roving","generation":1,"connected":true,"discovery_ready":true,
                    "inventory":configured_roving_empty("stale-generation-host")}
            ]),
        );
        configured_roving_observe(
            &runtime,
            3,
            serde_json::json!([
                {"id":"replaced-roving","generation":2,"connected":true,"discovery_ready":true,
                    "inventory":configured_roving_empty("replacement-host")}
            ]),
        );
        assert!(!configured_roving_retained(
            &runtime,
            "stale-generation-host"
        ));
        assert!(
            !configured_roving_retained(&runtime, "replacement-host"),
            "a replacement must not inherit old pending configuration"
        );
        assert!(!replaced.node.configure_navigation("roving-tabindex", true));
        assert!(replacement.configure_navigation("roving-tabindex", true));
        assert!(configured_roving_retained(&runtime, "replacement-host"));
        replacement.unregister();
        assert!(!configured_roving_retained(&runtime, "replacement-host"));

        let cleared = model(&runtime, "cleared-pending", OverlayKind::Dropdown, log);
        assert!(cleared.node.configure_navigation("roving-tabindex", true));
        cleared.node.clear_roving();
        configured_roving_observe(
            &runtime,
            4,
            serde_json::json!([
                {"id":"cleared-pending","generation":1,"connected":true,"discovery_ready":true,
                    "inventory":configured_roving_empty("after-clear-host")}
            ]),
        );
        assert!(
            !configured_roving_retained(&runtime, "after-clear-host"),
            "an unfulfilled enrollment cannot reappear after actual cleanup"
        );
    }

    fn configured_roving_forwarded_cleanup(initialize: bool) {
        let runtime = RuntimeHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let parent = model(&runtime, "header", OverlayKind::Dropdown, log.clone());
        let child = model(&runtime, "child", OverlayKind::Dropdown, log.clone());
        let parent_facts = configured_roving_empty("parent-direct-host");
        let mut child_facts = mixed_inventory_facts();
        let originals = child_facts
            .hosts
            .iter()
            .map(|host| (host.item.clone(), host.tabindex_attribute.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut attributes = originals.clone();
        configured_roving_observe(
            &runtime,
            1,
            serde_json::json!([
                {"id":"header","generation":1,"connected":true,"discovery_ready":true,
                    "inventory":parent_facts},
                {"id":"child","generation":1,"connected":true,"discovery_ready":true,
                    "trigger_ancestors":["header"],"inventory":child_facts}
            ]),
        );
        assert!(parent.node.request(true, Reason::Trigger, false));
        assert!(parent.node.configure_navigation("roving-tabindex", true));
        assert!(child.node.request(true, Reason::Trigger, false));
        assert!(child.node.configure_navigation("roving-tabindex", true));
        assert_eq!(
            runtime.0.borrow().registry.cached_children("header"),
            ["child"]
        );
        // The component's apply hook clears its enrolled set only on accepted hide.
        // Preserve the model callback so actual requests/cascades remain observable.
        let accepted = parent.node.clone();
        let original_apply = runtime.entry("header").unwrap().callbacks.apply;
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("header")
            .unwrap()
            .callbacks
            .apply = Rc::new(move |change| {
            if !change.open {
                accepted.clear_roving();
            }
            original_apply(change);
        });
        let selected = parent.node.clone();
        runtime
            .0
            .borrow_mut()
            .nodes
            .get_mut("header")
            .unwrap()
            .callbacks
            .key = Some(Rc::new(move |facts| {
            let raw = facts.inventory.as_ref().unwrap();
            if initialize {
                assert_eq!(selected.init_roving(raw, true, false).unwrap().target, "b");
            } else {
                assert!(selected.select_roving(raw, &ItemRef("actual-b".into())));
            }
            KeyDecision {
                prevent: true,
                ..KeyDecision::default()
            }
        }));
        // Parent's existing listener now queries Child's forwarded direct host;
        // its own direct host and the physical receiver remain separate raw facts.
        let key = event(serde_json::json!({"owner":runtime.owner_id(),"sequence":2,
        "kind":"key","phase":"capture","key":"End","hosts":["header"],
        "nodes":[
            {"id":"header","generation":1,"connected":true,"discovery_ready":true,
                "mode":"roving-tabindex","inside":true,"inventory":child_facts,
                "own_inventory":parent_facts},
            {"id":"child","generation":1,"connected":true,"discovery_ready":true,
                "mode":"roving-tabindex","trigger_ancestors":["header"],"inventory":child_facts}
        ]}));
        runtime.input(key.clone());
        assert!(!key.default_action_enabled());
        configured_roving_drain(&runtime, &mut attributes);
        assert_eq!(
            attributes.get(&ItemRef("actual-a".into())),
            Some(&Some("-1".into()))
        );
        assert_eq!(
            attributes.get(&ItemRef("actual-b".into())),
            Some(&Some("0".into()))
        );
        for host in &mut child_facts.hosts {
            host.tabindex_attribute = attributes[&host.item].clone();
        }
        configured_roving_observe(
            &runtime,
            3,
            serde_json::json!([
                {"id":"header","generation":1,"connected":true,"discovery_ready":true,
                    "inventory":child_facts,"own_inventory":parent_facts},
                {"id":"child","generation":1,"connected":true,"discovery_ready":true,
                    "trigger_ancestors":["header"],"inventory":child_facts}
            ]),
        );
        child.veto.set(true);
        log.borrow_mut().clear();
        assert!(parent.node.request(false, Reason::Trigger, false));
        assert!(!parent.open.get() && child.open.get());
        assert_eq!(
            *log.borrow(),
            [
                "before:header:false",
                "before:child:false",
                "apply:header:false"
            ]
        );
        configured_roving_drain(&runtime, &mut attributes);
        if initialize {
            assert_eq!(
                attributes, originals,
                "explicit initialization enrolled the forwarded host and restores exact None/4 originals"
            );
        } else {
            assert_eq!(
                attributes.get(&ItemRef("actual-a".into())),
                Some(&Some("-1".into()))
            );
            assert_eq!(
                attributes.get(&ItemRef("actual-b".into())),
                Some(&Some("0".into())),
                "ordinary selection did not enroll Child in Parent's cleanup set"
            );
            child.node.clear_roving();
            configured_roving_drain(&runtime, &mut attributes);
            assert_eq!(
                attributes, originals,
                "Child still owns the exact shared baseline after its veto"
            );
        }
    }

    #[test]
    fn ordinary_forwarded_roving_selection_preserves_vetoed_child_attributes_on_parent_hide() {
        configured_roving_forwarded_cleanup(false);
    }

    #[test]
    fn explicit_roving_initialization_enrolls_forwarded_host_for_parent_cleanup() {
        configured_roving_forwarded_cleanup(true);
    }
}

#[cfg(test)]
#[path = "overlay_domain_runtime_tests.rs"]
mod document_tests;

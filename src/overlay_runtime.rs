//! Owner-scoped dispatch with document-scoped Rust overlay coordination.
//! Native adapters supply read-only DOM observations and minimal global input.
//! Dioxus handlers own event defaults, propagation, activation and mounted focus.
use crate::dropdown_inventory::{
    HostFacts, HostPartFacts, HostPartKind, InventoryFacts, ItemKind, ItemRef,
};
use crate::overlay_completion::CompletionQueue;
use crate::overlay_domain::{Domain, DomainCallbacks, SurfaceValidity};
use crate::overlay_registry::{CloseBehavior, NativeFact, OverlayKind, Registry};
use dioxus::prelude::*;
use dioxus_core::Task;
use serde::Deserialize;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

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
    pub(crate) navigation_mode: Option<String>,
    pub(crate) key: String,
    pub(crate) shift: bool,
    pub(crate) alt: bool,
    pub(crate) ctrl: bool,
    pub(crate) meta: bool,
    pub(crate) default_prevented: bool,
    pub(crate) focused_item: Option<usize>,
    pub(crate) focused_target: Option<String>,
    pub(crate) items: Vec<usize>,
    pub(crate) focus_count: usize,
    pub(crate) focusable_ids: Vec<String>,
    pub(crate) focused_index: Option<usize>,
    pub(crate) on_trigger: bool,
    pub(crate) inside: bool,
    pub(crate) popover_ancestor: bool,
    pub(crate) inventory: Option<InventoryFacts>,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct KeyDecision {
    pub(crate) prevent: bool,
    pub(crate) stop: bool,
    pub(crate) release_focus: bool,
    pub(crate) activate: Option<String>,
}
type KeyCallback = Rc<dyn Fn(&KeyFacts) -> KeyDecision>;

/// Shared read-only native focus facts. Consumers project only their explicitly
/// registered Dioxus controls; this observation never enrolls new controls.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FocusObservation {
    pub(crate) candidates: Vec<crate::popover_focus::NativeFocusCandidate>,
    pub(crate) active_id: String,
    pub(crate) active_default: bool,
}
/// Surface notifications reuse the canonical observation and shared document
/// input adapters. Pointer invalidation has no independent native transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SurfaceObservation {
    Focus(FocusObservation),
    RequestedFocus {
        request: u64,
        focus: FocusObservation,
    },
    Pointer,
}
type SurfaceObserver = Rc<dyn Fn(SurfaceObservation)>;

/// Passive consumers share canonical focus observations without entering the
/// overlay hierarchy or authorizing overlay dismissal/keyboard ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScopedFocusFacts {
    pub(crate) observation: FocusObservation,
    pub(crate) active_hosts: Vec<String>,
    pub(crate) eligible_focus_ids: Vec<String>,
    pub(crate) nested: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScopedFocusEvent {
    Observe(ScopedFocusFacts),
    Requested {
        request: u64,
        facts: ScopedFocusFacts,
    },
}
#[derive(Clone)]
struct ScopedFocusRegistration {
    generation: u64,
    callback: Rc<dyn Fn(ScopedFocusEvent)>,
    renderer: Weak<dioxus::core::Runtime>,
    scope: ScopeId,
    live: CopyValue<()>,
}

/// Authored Popover role for an already registered Dioxus mounted control.
/// Nested Popovers replace this scope; native ancestry never infers a header.
#[derive(Clone, Default)]
pub(crate) struct PopoverFocusScope {
    pub(crate) owner: Option<NodeHandle>,
    pub(crate) header: bool,
}

/// Dioxus-authored physical surfaces used to partition canonical native facts.
/// Empty fields do not enroll arbitrary DOM or supply focus destinations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SurfaceMetadata {
    pub(crate) trigger_id: String,
    pub(crate) items_host: String,
    pub(crate) popover_host: String,
}

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
    mounted: bool,
    ownership_ready: bool,
    ancestors: Vec<NodeHandle>,
    before_operation: Option<u64>,
    kind: OverlayKind,
    behavior: CloseBehavior,
    trigger_visibility_check: bool,
    anchor_intersection: Option<AnchorIntersectionCallback>,
    surface_observer: Option<SurfaceObserver>,
    surface_metadata: SurfaceMetadata,
    submenu: Option<SubmenuAccess>,
    navigation_mode: Option<String>,
    suppress_trigger_restore: bool,
    callbacks: Callbacks,
    scope: Option<(Weak<dioxus::core::Runtime>, ScopeId)>,
}
struct Runtime {
    owner: String,
    registry: Registry,
    nodes: BTreeMap<String, Entry>,
    order: Vec<String>,
    sequence: u64,
    observation: Option<Report>,
    keyboard: bool,
    completions: CompletionQueue,
    item_mounts: BTreeMap<String, ItemMount>,
    owned_item_focus: BTreeMap<String, OwnedItemFocus>,
    item_focus_sequence: u64,
    submenu_marks: BTreeMap<String, (String, u64)>,
    item_descriptions: BTreeMap<String, OwnedItemDescription>,
    item_description_sequence: u64,
    item_tabindices: BTreeMap<(String, u64), BTreeMap<String, ItemTabindex>>,
    item_tabindex_revision: u64,
    item_tabindex_signal: Option<Signal<u64>>,
    item_visual: Option<String>,
    item_visual_target: Option<String>,
    item_visual_known: bool,
    item_visual_host: Option<HostFacts>,
    item_kinds: BTreeMap<String, ItemKind>,
    item_visual_signal: Option<Signal<Option<String>>>,
    bridge: Option<dioxus::document::Eval>,
    bridge_task: Option<Task>,
    bridge_leases: usize,
    bridge_epoch: u64,
    bridge_owner: Option<(Weak<dioxus::core::Runtime>, CopyValue<()>)>,
    bridge_ready: Option<Signal<bool>>,
    domain: Option<Domain>,
    document: String,
    document_generation: u64,
    categories: BTreeMap<usize, CategoryCallback>,
    focus_observers: BTreeMap<String, ScopedFocusRegistration>,
    focus_observer_sequence: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnedItemDescription {
    owner: Option<(String, u64)>,
    host: HostFacts,
    registration_sequence: u64,
    order: ItemOrder,
}

/// Authored sibling position and optional containing item, scoped to one owner.
/// Omitted positions preserve component registration order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ItemOrder {
    pub(crate) position: Option<usize>,
    pub(crate) parent: Option<String>,
}

#[derive(Clone)]
struct ItemMount {
    target: String,
    focus_id: String,
    owner: Option<(String, u64)>,
    node: Rc<MountedData>,
    renderer: Option<Weak<dioxus::core::Runtime>>,
    popover_scope: Option<PopoverFocusScope>,
}
/// A captured Dioxus destination retains no element or renderer ownership.
/// Replacing/unmounting its registration or changing documents invalidates it.
#[derive(Clone)]
pub(crate) struct MountedFocusTarget {
    runtime: Weak<RefCell<Runtime>>,
    key: String,
    logical_target: String,
    focus_id: String,
    node: Weak<MountedData>,
    document_generation: u64,
    renderer: Option<Weak<dioxus::core::Runtime>>,
}
impl MountedFocusTarget {
    /// Cleanup compares the captured element even after its registration was
    /// removed. Identity alone does not authorize a new focus operation.
    pub(crate) fn matches_node(&self, node: &Rc<MountedData>) -> bool {
        Weak::ptr_eq(&self.node, &Rc::downgrade(node))
    }
    pub(crate) fn live_node(&self) -> Option<Rc<MountedData>> {
        // Component cleanup unregisters its handle. Also reject a retained
        // registry after the renderer VirtualDom itself has been dropped.
        let _renderer = match &self.renderer {
            Some(owner) => Some(owner.upgrade()?),
            None => None,
        };
        let runtime = self.runtime.upgrade()?;
        let node = self.node.upgrade()?;
        let state = runtime.borrow();
        (state.document_generation == self.document_generation
            && state.item_mounts.get(&self.key).is_some_and(|mount| {
                mount.target == self.logical_target
                    && mount.focus_id == self.focus_id
                    && Rc::ptr_eq(&mount.node, &node)
            }))
        .then_some(node)
    }
}
struct OwnedItemFocus {
    sequence: u64,
    generation: u64,
    key: String,
    callback: Option<Rc<dyn Fn()>>,
    visual: Option<HostFacts>,
    focus_native: bool,
    exit_target: Option<MountedFocusTarget>,
    // A repair is an ordinary mounted item job with an exact weak destination.
    // Its tag lets later real input cancel repairs without disturbing key jobs.
    repair_target: Option<MountedFocusTarget>,
    expected_open: Option<bool>,
    destination_popovers: Vec<String>,
    task: Option<(Weak<dioxus::core::Runtime>, dioxus::core::Task)>,
}

#[derive(Clone)]
struct SubmenuAccess {
    handle: crate::dropdown::DropdownHandle,
    query_root: Rc<dyn Fn() -> Option<String>>,
    reset: Rc<dyn Fn()>,
}

#[derive(Clone, Copy)]
struct ItemTabindex {
    revision: u64,
    value: i32,
}
type CategoryCallback = Rc<dyn Fn(usize, &[usize])>;

impl Runtime {
    /// Only explicitly described Dioxus members belong to an item inventory.
    /// Rust descriptions own sibling order and containing-item relationships.
    fn owned_inventory(&self, root: &str, generation: u64) -> Option<InventoryFacts> {
        if !self
            .nodes
            .get(root)
            .is_some_and(|entry| entry.generation == generation)
        {
            return None;
        }
        let owner = (root.to_string(), generation);
        let mut members = self
            .item_descriptions
            .values()
            .filter(|item| item.owner.as_ref() == Some(&owner))
            .collect::<Vec<_>>();
        members.sort_by_key(|item| {
            (
                item.order.position.is_none(),
                item.order.position.unwrap_or_default(),
                item.registration_sequence,
            )
        });
        let member_keys = members
            .iter()
            .map(|item| item.host.registration_key.as_str())
            .collect::<BTreeSet<_>>();
        let mut children = BTreeMap::<Option<String>, Vec<String>>::new();
        for item in &members {
            // A foreign or removed parent cannot place a member in another
            // owner's tree. Such a member remains an ordinary top-level item.
            let parent = item
                .order
                .parent
                .as_ref()
                .filter(|key| member_keys.contains(key.as_str()))
                .cloned();
            children
                .entry(parent)
                .or_default()
                .push(item.host.registration_key.clone());
        }
        let mut keys = Vec::with_capacity(members.len());
        let mut visited = BTreeSet::new();
        let append = |seed: String, visited: &mut BTreeSet<String>, keys: &mut Vec<String>| {
            let mut pending = vec![seed];
            while let Some(key) = pending.pop() {
                if !visited.insert(key.clone()) {
                    continue;
                }
                keys.push(key.clone());
                if let Some(descendants) = children.get(&Some(key)) {
                    pending.extend(descendants.iter().rev().cloned());
                }
            }
        };
        for key in children.get(&None).into_iter().flatten() {
            append(key.clone(), &mut visited, &mut keys);
        }
        // Invalid cyclic relationships have no root. Visit every remaining
        // registered member once, without recursive traversal or losing items.
        for item in &members {
            append(item.host.registration_key.clone(), &mut visited, &mut keys);
        }
        let hosts = keys
            .iter()
            .filter_map(|key| {
                self.item_descriptions
                    .get(key)
                    .map(|item| item.host.clone())
            })
            .collect::<Vec<_>>();
        let order = hosts
            .iter()
            .map(|host| host.item.clone())
            .collect::<Vec<_>>();
        Some(InventoryFacts {
            items_host: ItemRef(root.into()),
            hosts,
            navigation_order: order.clone(),
            query_order: order,
            slots: None,
        })
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.completions.clear();

        self.item_descriptions.clear();
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
impl PartialEq for NodeHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.generation == other.generation
            && Weak::ptr_eq(&self.runtime, &other.runtime)
    }
}
impl Eq for NodeHandle {}

#[derive(Clone, Copy)]
struct OverlayScope(Signal<Vec<NodeHandle>>);

/// Consume the parent once: a provider installed by this component must never
/// become its own parent on subsequent renders.
pub(crate) fn use_overlay_ancestors() -> Vec<NodeHandle> {
    let parent = try_use_context::<OverlayScope>();
    parent
        .map(|scope| scope.0.read().clone())
        .unwrap_or_default()
}

pub(crate) fn use_overlay_scope(node: Option<NodeHandle>) -> Vec<NodeHandle> {
    let ancestors = use_overlay_ancestors();
    use_overlay_scope_with_ancestors(node, ancestors.clone());
    ancestors
}

/// Provide the effective authored lineage for a surface rendered elsewhere
/// than its trigger. This hook deliberately consumes no lexical parent scope.
pub(crate) fn use_overlay_scope_with_ancestors(
    node: Option<NodeHandle>,
    ancestors: Vec<NodeHandle>,
) {
    let mut scope = use_context_provider(|| OverlayScope(Signal::new(Vec::new())));
    let mut lineage = Vec::new();
    for current in node.into_iter().chain(ancestors) {
        if current.is_registered() && !lineage.contains(&current) {
            lineage.push(current);
        }
    }
    let changed = *scope.0.peek() != lineage;
    if changed {
        scope.0.set(lineage);
    }
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
            observation: None,
            keyboard: false,
            completions: CompletionQueue::default(),
            item_mounts: BTreeMap::new(),
            owned_item_focus: BTreeMap::new(),
            item_focus_sequence: 0,
            submenu_marks: BTreeMap::new(),
            item_descriptions: BTreeMap::new(),
            item_description_sequence: 0,
            item_tabindices: BTreeMap::new(),
            item_tabindex_revision: 0,
            item_tabindex_signal: dioxus::core::Runtime::try_current()
                .map(|_| Signal::new_in_scope(0, ScopeId::ROOT)),
            item_visual: None,
            item_visual_target: None,
            item_visual_known: false,
            item_visual_host: None,
            item_kinds: BTreeMap::new(),
            item_visual_signal: dioxus::core::Runtime::try_current()
                .map(|_| Signal::new_in_scope(None, ScopeId::ROOT)),
            bridge: None,
            bridge_task: None,
            bridge_leases: 0,
            bridge_epoch: 0,
            bridge_owner: None,
            bridge_ready: dioxus::core::Runtime::try_current()
                .map(|_| Signal::new_in_scope(false, ScopeId::ROOT)),
            domain: None,
            document: String::new(),
            document_generation: 0,
            categories: BTreeMap::new(),
            focus_observers: BTreeMap::new(),
            focus_observer_sequence: 0,
        })))
    }
    pub(crate) fn register_focus_observer(
        &self,
        root: String,
        callback: Rc<dyn Fn(ScopedFocusEvent)>,
    ) -> u64 {
        let renderer = dioxus::core::Runtime::current();
        let scope = dioxus::core::current_scope_id();
        let live = CopyValue::new(());
        let mut state = self.0.borrow_mut();
        state.focus_observer_sequence = state.focus_observer_sequence.wrapping_add(1).max(1);
        let generation = state.focus_observer_sequence;
        state.focus_observers.insert(
            root,
            ScopedFocusRegistration {
                generation,
                callback,
                renderer: Rc::downgrade(&renderer),
                scope,
                live,
            },
        );
        generation
    }
    pub(crate) fn unregister_focus_observer(&self, root: &str, generation: u64) {
        let mut state = self.0.borrow_mut();
        if state
            .focus_observers
            .get(root)
            .is_some_and(|entry| entry.generation == generation)
        {
            state.focus_observers.remove(root);
        }
    }
    pub(crate) fn request_scoped_focus(&self, root: &str, generation: u64, request: u64) -> bool {
        self.read_bridge_ready();
        let state = self.0.borrow();
        if !state.focus_observers.get(root).is_some_and(|entry| {
            entry.generation == generation
                && entry.live.try_read().is_ok()
                && entry.renderer.upgrade().is_some()
        }) {
            return false;
        }
        state.bridge.as_ref().is_some_and(|bridge| {
            bridge
                .send(serde_json::json!({"focus_request": {
                    "id": root, "generation": generation, "request": request
                }}))
                .is_ok()
        })
    }
    fn deliver_scoped_focus(
        &self,
        root: &str,
        generation: u64,
        sequence: u64,
        event: ScopedFocusEvent,
    ) {
        let entry = self.0.borrow().focus_observers.get(root).cloned();
        let Some(entry) =
            entry.filter(|entry| entry.generation == generation && entry.live.try_read().is_ok())
        else {
            return;
        };
        let Some(renderer) = entry.renderer.upgrade() else {
            return;
        };
        renderer.in_scope(entry.scope, || {
            // A previous callback may remove/replace this registration, dispose
            // its scope, or admit newer native facts. None authorizes old work.
            let callback = {
                let state = self.0.borrow();
                state
                    .focus_observers
                    .get(root)
                    .filter(|current| {
                        state.sequence == sequence
                            && current.generation == generation
                            && current.live.try_read().is_ok()
                    })
                    .map(|entry| entry.callback.clone())
            };
            if let Some(callback) = callback {
                callback(event);
            }
        });
    }
    fn notify_focus_observers(&self, report: &Report) {
        let recipients = self
            .0
            .borrow()
            .focus_observers
            .iter()
            .map(|(root, entry)| (root.clone(), entry.generation))
            .collect::<Vec<_>>();
        let facts = ScopedFocusFacts::from(report);
        for (root, generation) in recipients {
            self.deliver_scoped_focus(
                &root,
                generation,
                report.sequence,
                ScopedFocusEvent::Observe(facts.clone()),
            );
        }
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
    /// Reads the same reactive selection as the outer host. Select's option
    /// inherits its host cue; Category marks only its eligible parent control.
    pub(crate) fn part_active(&self, key: &str, kind: HostPartKind) -> bool {
        let _ = self.item_visual_key();
        let state = self.0.borrow();
        let Some(host) = state
            .item_visual_host
            .as_ref()
            .filter(|host| host.registration_key == key)
        else {
            return false;
        };
        match (host.kind, kind) {
            (ItemKind::Select, HostPartKind::SelectOption) => host.source_focusable,
            (ItemKind::Category, HostPartKind::CategoryParent) => host
                .parts
                .iter()
                .any(|part| part.kind == kind && part.source_focusable),
            _ => false,
        }
    }
    pub(crate) fn item_visual_target(&self) -> Option<String> {
        let _ = self.item_visual_key();
        self.0.borrow().item_visual_target.clone()
    }
    pub(crate) fn item_visual_known(&self) -> bool {
        let _ = self.item_visual_key();
        self.0.borrow().item_visual_known
    }
    /// Explicit Rust composition supplies membership and ownership.
    /// A sibling or replacement owner cannot claim the selected control.
    pub(crate) fn item_visual_owned_by(&self, root: &str, generation: u64) -> bool {
        let _ = self.item_visual_key();
        let state = self.0.borrow();
        let Some(host) = state.item_visual_host.as_ref() else {
            return false;
        };
        state.item_visual_target.as_ref() == Some(&host.target)
            && Self::registered_item_live(&state, host)
            && state
                .nodes
                .get(root)
                .is_some_and(|entry| entry.generation == generation)
            && state
                .item_mounts
                .get(&host.registration_key)
                .is_some_and(|mount| {
                    mount
                        .owner
                        .as_ref()
                        .is_some_and(|(id, version)| id == root && *version == generation)
                })
    }
    pub(crate) fn register_item_cue(&self, key: String, kind: ItemKind) {
        if !key.is_empty() {
            self.0.borrow_mut().item_kinds.entry(key).or_insert(kind);
        }
    }
    /// Publish current component props and its explicit overlay owner.
    /// A description grants membership, while only a mounted handle grants focus.
    pub(crate) fn register_owned_item_description(
        &self,
        key: &str,
        owner: Option<(String, u64)>,
        mut host: HostFacts,
        order: ItemOrder,
    ) {
        if key.is_empty() {
            return;
        }
        host.item = ItemRef(key.into());
        host.capture = key.into();
        host.registration_key = key.into();
        let identity_changed = self
            .0
            .borrow()
            .item_descriptions
            .get(key)
            .is_some_and(|old| {
                old.owner != owner || old.host.target != host.target || old.host.kind != host.kind
            });
        if identity_changed {
            self.cancel_item_key(key);
        } else if host.disabled_attribute || host.hidden_attribute {
            // Closing an inline owner may hide a still-focused registered item.
            // Preserve its completed cue, but never execute newly ineligible work.
            let roots = self
                .0
                .borrow()
                .owned_item_focus
                .iter()
                .filter(|(_, job)| job.key == key && job.exit_target.is_none())
                .map(|(root, _)| root.clone())
                .collect::<Vec<_>>();
            for root in roots {
                self.cancel_owned_item_focus(&root);
            }
        }
        let mut state = self.0.borrow_mut();
        let registration_sequence = state
            .item_descriptions
            .get(key)
            .map(|old| old.registration_sequence)
            .unwrap_or_else(|| {
                state.item_description_sequence =
                    state.item_description_sequence.wrapping_add(1).max(1);
                state.item_description_sequence
            });
        if let Some(selected) = state
            .item_visual_host
            .as_mut()
            .filter(|selected| selected.registration_key == key)
        {
            *selected = host.clone();
        }
        for job in state
            .owned_item_focus
            .values_mut()
            .filter(|job| job.key == key)
        {
            if let Some(visual) = job.visual.as_mut() {
                *visual = host.clone();
            }
        }
        state.item_descriptions.insert(
            key.into(),
            OwnedItemDescription {
                owner,
                host,
                registration_sequence,
                order,
            },
        );
    }
    #[cfg(test)]
    pub(crate) fn register_item_mount(&self, key: &str, target: &str, node: Rc<MountedData>) {
        self.register_item_control_mount(key, target, target, node);
    }
    pub(crate) fn mounted_focus_target(&self, id: &str) -> Option<MountedFocusTarget> {
        if id.is_empty() {
            return None;
        }
        let state = self.0.borrow();
        let (key, mount) = state
            .item_mounts
            .iter()
            .find(|(_, mount)| mount.focus_id == id)?;
        Some(MountedFocusTarget {
            runtime: Rc::downgrade(&self.0),
            key: key.clone(),
            logical_target: mount.target.clone(),
            focus_id: id.into(),
            node: Rc::downgrade(&mount.node),
            document_generation: state.document_generation,
            renderer: mount.renderer.clone(),
        })
    }
    pub(crate) fn capture_registered_focus(&self, id: &str) -> Option<MountedFocusTarget> {
        self.mounted_focus_target(id)
            .or_else(|| self.domain()?.focus_target(id))
            .filter(|target| target.live_node().is_some())
    }
    #[cfg(test)]
    pub(crate) fn register_item_control_mount(
        &self,
        key: &str,
        target: &str,
        focus_id: &str,
        node: Rc<MountedData>,
    ) {
        self.register_owned_item_control_mount(key, target, focus_id, node, None);
    }
    pub(crate) fn register_owned_item_control_mount(
        &self,
        key: &str,
        target: &str,
        focus_id: &str,
        node: Rc<MountedData>,
        owner: Option<(String, u64)>,
    ) {
        let renderer = dioxus::core::Runtime::try_current().map(|owner| Rc::downgrade(&owner));
        let changed = self.0.borrow().item_mounts.get(key).is_some_and(|old| {
            old.target != target
                || old.focus_id != focus_id
                || old.owner != owner
                || !Rc::ptr_eq(&old.node, &node)
        });
        if changed {
            self.cancel_item_key(key);
        }
        // Repeated mounting of the same handle retains its authored role. A
        // replacement has no metadata until its binding supplies the new scope.
        let popover_scope = (!changed)
            .then(|| {
                self.0
                    .borrow()
                    .item_mounts
                    .get(key)
                    .and_then(|old| old.popover_scope.clone())
            })
            .flatten();
        self.0.borrow_mut().item_mounts.insert(
            key.into(),
            ItemMount {
                target: target.into(),
                focus_id: focus_id.into(),
                owner,
                node,
                renderer,
                popover_scope,
            },
        );
    }
    pub(crate) fn set_item_popover_scope(&self, key: &str, scope: Option<PopoverFocusScope>) {
        if let Some(mount) = self.0.borrow_mut().item_mounts.get_mut(key) {
            mount.popover_scope = scope;
        }
    }
    pub(crate) fn unregister_item_cue(&self, key: &str) {
        self.cancel_item_key(key);
        let mut state = self.0.borrow_mut();
        state.item_mounts.remove(key);
        state.item_kinds.remove(key);
        state.item_descriptions.remove(key);
        state.submenu_marks.remove(key);
    }
    fn cancel_item_key(&self, key: &str) {
        let selected = self
            .0
            .borrow()
            .item_visual_host
            .as_ref()
            .is_some_and(|host| host.registration_key == key);
        if selected {
            self.clear_item_visual();
        }
        let roots: Vec<_> = self
            .0
            .borrow()
            .owned_item_focus
            .iter()
            .filter(|(_, job)| job.key == key)
            .map(|(root, _)| root.clone())
            .collect();
        for root in roots {
            self.cancel_owned_item_focus(&root);
        }
        let changed = {
            let mut state = self.0.borrow_mut();
            let mut changed = false;
            state.item_tabindices.retain(|_, values| {
                changed |= values.remove(key).is_some();
                !values.is_empty()
            });
            changed
        };
        if changed {
            self.notify_item_tabindex();
        }
    }
    fn cancel_item_repairs(&self, focused: Option<&str>) {
        let roots = self
            .0
            .borrow()
            .owned_item_focus
            .iter()
            .filter(|(_, job)| {
                job.repair_target
                    .as_ref()
                    .is_some_and(|target| focused != Some(target.focus_id.as_str()))
            })
            .map(|(root, _)| root.clone())
            .collect::<Vec<_>>();
        for root in roots {
            self.cancel_owned_item_focus(&root);
        }
    }

    fn repair_moved_item_focus(&self, report: &Report) {
        let Some(moved) = report
            .moved_focus
            .as_ref()
            .filter(|_| report.active_default)
        else {
            return;
        };
        let Some((root, generation, host, target)) = ({
            let state = self.0.borrow();
            let described = state.item_descriptions.get(&moved.key);
            described.and_then(|description| {
                let (root, generation) = description.owner.as_ref()?;
                let entry = state.nodes.get(root)?;
                let mount = state.item_mounts.get(&moved.key)?;
                if entry.generation != *generation
                    || entry.kind != OverlayKind::Dropdown
                    || !entry.mounted
                    || !entry.ownership_ready
                    || entry.navigation_mode.as_deref() != Some("roving-tabindex")
                    || mount.focus_id != moved.focus_id
                    || mount.target != description.host.target
                    || mount.owner.as_ref() != Some(&(root.clone(), *generation))
                    || !description
                        .host
                        .eligible(crate::dropdown_inventory::InventoryMode::RovingTabindex)
                    || !report
                        .nodes
                        .iter()
                        .any(|node| node.id == *root && node.generation == *generation)
                    || state.owned_item_focus.contains_key(root)
                {
                    return None;
                }
                Some((root.clone(), *generation, description.host.clone()))
            })
        })
        .and_then(|(root, generation, host)| {
            let target = self
                .mounted_focus_target(&moved.focus_id)
                .filter(|target| target.key == moved.key)?;
            target.live_node()?;
            Some((root, generation, host, target))
        }) else {
            return;
        };
        let node = NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id: root.clone(),
            generation,
        };
        // This remains the existing Dioxus executor: no native focus command,
        // timer, DOM target retention or alternative unregistered destination.
        let _ = node.with_owner(|| {
            if node.queue_item_host_focus(&host, Rc::new(|| {})) {
                if let Some(job) = self.0.borrow_mut().owned_item_focus.get_mut(&root) {
                    job.repair_target = Some(target);
                }
                dioxus::core::needs_update();
            }
        });
    }

    fn cancel_owned_item_focus(&self, root: &str) {
        let job = self.0.borrow_mut().owned_item_focus.remove(root);
        if let Some((owner, task)) = job.and_then(|job| job.task)
            && let Some(owner) = owner.upgrade()
        {
            owner.in_scope(ScopeId::ROOT, || task.cancel());
        }
    }
    fn retire_owned_item_focus(&self, root: &str, sequence: u64) {
        let mut state = self.0.borrow_mut();
        if state
            .owned_item_focus
            .get(root)
            .is_some_and(|job| job.sequence == sequence)
        {
            state.owned_item_focus.remove(root);
        }
    }
    fn item_owner_open(&self, root: &str, generation: u64) -> bool {
        self.with_owner(root, generation, || {
            let entry = self.entry(root)?;
            (self.read_current(root, &entry) == Some(true)
                && self
                    .entry(root)
                    .is_some_and(|entry| entry.generation == generation))
            .then_some(())
        })
        .flatten()
        .is_some()
    }
    fn focus_owner_registered(&self, root: &str, generation: u64) -> bool {
        self.with_owner(root, generation, || {
            let entry = self.entry(root)?;
            (self.read_current(root, &entry).is_some()
                && self
                    .entry(root)
                    .is_some_and(|entry| entry.generation == generation))
            .then_some(())
        })
        .flatten()
        .is_some()
    }
    fn destination_popovers_visible(&self, popovers: &[String]) -> bool {
        self.domain().map_or_else(
            || {
                popovers.iter().all(|id| {
                    self.entry(id).is_some_and(|entry| {
                        entry.kind == OverlayKind::Popover
                            && self.read_current(id, &entry) == Some(true)
                    })
                })
            },
            |domain| domain.focus_destinations_visible(popovers),
        )
    }
    fn queue_exit_focus(
        &self,
        root: &str,
        generation: u64,
        target: MountedFocusTarget,
        destination_popovers: Vec<String>,
    ) -> bool {
        self.queue_registered_focus(root, generation, target, None, destination_popovers)
    }
    fn queue_registered_focus(
        &self,
        root: &str,
        generation: u64,
        target: MountedFocusTarget,
        expected_open: Option<bool>,
        destination_popovers: Vec<String>,
    ) -> bool {
        self.with_owner(root, generation, || {
            if !self.focus_owner_registered(root, generation)
                || target.live_node().is_none()
                || !self.destination_popovers_visible(&destination_popovers)
                || expected_open.is_some_and(|expected| {
                    self.entry(root)
                        .and_then(|entry| self.read_current(root, &entry))
                        != Some(expected)
                })
            {
                return false;
            }
            if self
                .entry(root)
                .is_none_or(|entry| entry.generation != generation || entry.scope.is_none())
                || target.live_node().is_none()
            {
                return false;
            }
            self.cancel_owned_item_focus(root);
            let sequence = {
                let mut state = self.0.borrow_mut();
                state.item_focus_sequence = state.item_focus_sequence.wrapping_add(1).max(1);
                let sequence = state.item_focus_sequence;
                state.owned_item_focus.insert(
                    root.into(),
                    OwnedItemFocus {
                        sequence,
                        generation,
                        key: target.key.clone(),
                        callback: None,
                        visual: None,
                        focus_native: true,
                        exit_target: Some(target),
                        repair_target: None,
                        expected_open,
                        destination_popovers,
                        task: None,
                    },
                );
                sequence
            };
            let weak = Rc::downgrade(&self.0);
            let owned_root = root.to_string();
            dioxus::core::queue_effect(move || {
                if let Some(state) = weak.upgrade() {
                    let runtime = RuntimeHandle(state);
                    runtime.with_owner(&owned_root, generation, || {
                        runtime.start_owned_item_focus(owned_root.clone(), generation, sequence)
                    });
                }
            });
            // A raw native report need not modify component state. Wake the
            // owner now so the framework runs this post-render focus effect.
            dioxus::core::needs_update();
            true
        })
        .unwrap_or(false)
    }
    fn owned_item_live(
        &self,
        root: &str,
        generation: u64,
        sequence: u64,
        node: &Rc<MountedData>,
    ) -> bool {
        let Some((expected_open, popovers)) = self
            .0
            .borrow()
            .owned_item_focus
            .get(root)
            .map(|job| (job.expected_open, job.destination_popovers.clone()))
        else {
            return false;
        };
        let owner_live = self.focus_owner_registered(root, generation)
            && expected_open.is_none_or(|expected| {
                self.with_owner(root, generation, || {
                    self.entry(root)
                        .and_then(|entry| self.read_current(root, &entry))
                        == Some(expected)
                })
                .unwrap_or(false)
            });
        if !owner_live || !self.destination_popovers_visible(&popovers) {
            return false;
        }
        // Presence readers can run consumer code. Recheck identity only after
        // those callbacks, without holding a registry borrow across them.
        let state = self.0.borrow();
        state.owned_item_focus.get(root).is_some_and(|job| {
            job.sequence == sequence
                && job.generation == generation
                && job.repair_target.as_ref().is_none_or(|target| {
                    target
                        .live_node()
                        .is_some_and(|current| Rc::ptr_eq(&current, node))
                        && state.nodes.get(root).is_some_and(|entry| {
                            entry.navigation_mode.as_deref() == Some("roving-tabindex")
                        })
                        && state
                            .item_descriptions
                            .get(&job.key)
                            .is_some_and(|description| {
                                description.owner.as_ref() == Some(&(root.into(), generation))
                                    && description.host.eligible(
                                        crate::dropdown_inventory::InventoryMode::RovingTabindex,
                                    )
                            })
                })
                && if let Some(target) = &job.exit_target {
                    target
                        .live_node()
                        .is_some_and(|current| Rc::ptr_eq(&current, node))
                } else {
                    state
                        .item_mounts
                        .get(&job.key)
                        .is_some_and(|mount| Rc::ptr_eq(&mount.node, node))
                }
        })
    }

    fn start_owned_item_focus(&self, root: String, generation: u64, sequence: u64) {
        let selected = {
            let state = self.0.borrow();
            state
                .owned_item_focus
                .get(&root)
                .filter(|job| job.sequence == sequence)
                .and_then(|job| {
                    let node = if let Some(target) = &job.exit_target {
                        target.live_node()
                    } else {
                        state
                            .item_mounts
                            .get(&job.key)
                            .map(|mount| mount.node.clone())
                    }?;
                    Some((node, job.focus_native, job.exit_target.is_some()))
                })
        };
        let Some((node, focus_native, exiting)) = selected else {
            self.retire_owned_item_focus(&root, sequence);
            return;
        };
        if !self.owned_item_live(&root, generation, sequence, &node) {
            self.cancel_owned_item_focus(&root);
            return;
        }
        let callback = self
            .0
            .borrow_mut()
            .owned_item_focus
            .get_mut(&root)
            .and_then(|job| job.callback.take());
        if let Some(callback) = callback {
            callback();
        }
        if !self.owned_item_live(&root, generation, sequence, &node) {
            self.retire_owned_item_focus(&root, sequence);
            return;
        }
        let weak = Rc::downgrade(&self.0);
        let task_root = root.clone();
        let task = spawn(async move {
            let live = || {
                weak.upgrade().map(RuntimeHandle).filter(|runtime| {
                    runtime.owned_item_live(&task_root, generation, sequence, &node)
                })
            };
            let finish = || {
                if let Some(state) = weak.upgrade() {
                    RuntimeHandle(state).retire_owned_item_focus(&task_root, sequence);
                }
            };
            if live().is_none() {
                finish();
                return;
            }
            // Desktop alpha can complete focus but report a null decode error.
            // Cues and nearest scrolling follow a live attempt in either case.
            if focus_native {
                let _ = node.set_focus(true).await;
            }
            if exiting {
                finish();
                return;
            }
            let Some(runtime) = live() else {
                finish();
                return;
            };
            let visual = runtime
                .0
                .borrow()
                .owned_item_focus
                .get(&task_root)
                .and_then(|job| job.visual.clone());
            if let Some(visual) = visual {
                runtime.mark_item_visual(&visual);
            }
            let repair = runtime
                .0
                .borrow()
                .owned_item_focus
                .get(&task_root)
                .is_some_and(|job| job.repair_target.is_some());
            drop(runtime);
            if repair || live().is_none() {
                finish();
                return;
            }
            let _ = node
                .scroll_to_with_options(dioxus::html::ScrollToOptions {
                    behavior: dioxus::html::ScrollBehavior::Instant,
                    vertical: dioxus::html::ScrollLogicalPosition::Nearest,
                    horizontal: dioxus::html::ScrollLogicalPosition::Nearest,
                })
                .await;
            // Recheck after the await, then retire only this operation. A
            // newer reentrant intent remains owned by its own effect/task.
            let _ = live();
            finish();
        });
        if let Some(job) = self
            .0
            .borrow_mut()
            .owned_item_focus
            .get_mut(&root)
            .filter(|job| job.sequence == sequence)
        {
            job.task = Some((Rc::downgrade(&dioxus::core::Runtime::current()), task));
        }
    }
    pub(crate) fn item_visual_matches(&self, host: &HostFacts) -> bool {
        let state = self.0.borrow();
        state.item_visual_host.as_ref().is_some_and(|current| {
            current.registration_key == host.registration_key
                && current.kind == host.kind
                && current.target == host.target
                && state.item_visual_target.as_ref() == Some(&host.target)
                && Self::registered_item_live(&state, host)
        })
    }
    fn registered_item_live(state: &Runtime, host: &HostFacts) -> bool {
        state.item_kinds.get(&host.registration_key) == Some(&host.kind)
            && state
                .item_mounts
                .get(&host.registration_key)
                .is_some_and(|mount| {
                    mount.target == host.target
                        && mount
                            .renderer
                            .as_ref()
                            .is_none_or(|renderer| renderer.upgrade().is_some())
                })
    }
    /// Physical Dioxus focus is authoritative for a registered mounted item.
    /// Descriptions come from Rust components; no native capture receipt is needed.
    pub(crate) fn focus_registered_item(
        &self,
        key: &str,
        source_focusable: bool,
        part: Option<HostPartKind>,
    ) {
        let host = {
            let state = self.0.borrow();
            let Some(kind) = state.item_kinds.get(key).copied() else {
                return;
            };
            let Some(mount) = state.item_mounts.get(key) else {
                return;
            };
            if mount
                .renderer
                .as_ref()
                .is_some_and(|renderer| renderer.upgrade().is_none())
            {
                return;
            }
            if !matches!(
                (kind, part),
                (
                    ItemKind::Dropdown
                        | ItemKind::Menu
                        | ItemKind::Select
                        | ItemKind::Category
                        | ItemKind::NativeOptIn,
                    None
                ) | (ItemKind::Select, Some(HostPartKind::SelectOption))
                    | (ItemKind::Category, Some(HostPartKind::CategoryParent))
            ) {
                return;
            }
            let mut host = state
                .item_descriptions
                .get(key)
                .filter(|item| item.host.kind == kind && item.host.target == mount.target)
                .map(|item| item.host.clone())
                .unwrap_or_else(|| HostFacts {
                    root_path: Vec::new(),
                    item: ItemRef(key.into()),
                    kind,
                    target: mount.target.clone(),
                    capture: key.into(),
                    registration_key: key.into(),
                    source_focusable,
                    disabled_attribute: false,
                    hidden_attribute: false,
                    tabindex_attribute: None,
                    checked_attribute: false,
                    visible_focus: false,
                    roving_opt_in: false,
                    linked_trigger: None,
                    parts: Vec::new(),
                });
            host.source_focusable = source_focusable && kind != ItemKind::Category;
            if kind == ItemKind::Category {
                host.parts = if part == Some(HostPartKind::CategoryParent) {
                    vec![HostPartFacts {
                        kind: HostPartKind::CategoryParent,
                        capture: mount.focus_id.clone(),
                        source_focusable,
                    }]
                } else {
                    Vec::new()
                };
            }
            host
        };
        self.mark_item_visual(&host);
    }
    /// A late blur of the previous control cannot erase a newer selection.
    pub(crate) fn blur_registered_item(&self, key: &str) {
        let selected = self
            .0
            .borrow()
            .item_visual_host
            .as_ref()
            .is_some_and(|host| host.registration_key == key);
        if selected {
            self.clear_item_visual();
        }
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
            state.item_visual_target = target;
            state.item_visual_host = host;
            state.item_visual_signal
        };
        if let Some(mut signal) = signal
            && let Ok(mut value) = signal.try_write()
        {
            *value = key;
        }
    }
    pub(crate) fn clear_item_visual(&self) {
        self.set_item_visual(None, None, None);
    }
    pub(crate) fn mark_item_visual(&self, host: &HostFacts) {
        if !self.keyboard_mode() || !Self::registered_item_live(&self.0.borrow(), host) {
            return;
        }
        // Source focusItem projects Category into its private parent; that
        // parent's marker never makes the outer Category an AD active host.
        let key = (host.source_focusable
            && matches!(
                host.kind,
                ItemKind::Dropdown | ItemKind::Select | ItemKind::Menu
            ))
        .then(|| host.registration_key.clone());
        let target = key.as_ref().map(|_| host.target.clone());
        self.set_item_visual(key, target, Some(host.clone()));
    }
    fn mounted_item_key(state: &Runtime, host: &HostFacts) -> Option<String> {
        if state.item_mounts.contains_key(&host.registration_key) {
            return Some(host.registration_key.clone());
        }
        state
            .item_mounts
            .iter()
            .find(|(_, mount)| mount.target == host.target)
            .map(|(key, _)| key.clone())
    }
    fn current_item_tabindex(state: &Runtime, key: &str) -> Option<i32> {
        state
            .item_tabindices
            .values()
            .filter_map(|values| values.get(key))
            .max_by_key(|value| value.revision)
            .map(|value| value.value)
    }
    pub(crate) fn item_tabindex(&self, key: &str) -> Option<i32> {
        let signal = self.0.borrow().item_tabindex_signal;
        let _revision = signal.and_then(|signal| signal.try_read().ok().map(|value| *value));
        Self::current_item_tabindex(&self.0.borrow(), key)
    }
    fn notify_item_tabindex(&self) {
        let (signal, revision) = {
            let mut state = self.0.borrow_mut();
            state.item_tabindex_revision = state.item_tabindex_revision.wrapping_add(1).max(1);
            (state.item_tabindex_signal, state.item_tabindex_revision)
        };
        if let Some(mut signal) = signal
            && let Ok(mut value) = signal.try_write()
        {
            *value = revision;
        }
    }
    fn clear_item_tabindices(&self, root: &str, generation: u64) {
        let removed = self
            .0
            .borrow_mut()
            .item_tabindices
            .remove(&(root.into(), generation))
            .is_some();
        if removed {
            self.notify_item_tabindex();
        }
    }
    fn set_roving_target(
        &self,
        root: &str,
        generation: u64,
        candidates: &[HostFacts],
        target: &ItemRef,
    ) -> bool {
        let mut state = self.0.borrow_mut();
        let target_key = candidates
            .iter()
            .find(|host| host.item == *target)
            .and_then(|host| Self::mounted_item_key(&state, host));
        let Some(target_key) = target_key else {
            return false;
        };
        let revision = state.item_tabindex_revision.wrapping_add(1).max(1);
        let values = candidates
            .iter()
            .filter_map(|host| {
                Self::mounted_item_key(&state, host).map(|key| {
                    let value = if key == target_key { 0 } else { -1 };
                    (key, ItemTabindex { revision, value })
                })
            })
            .collect();
        state
            .item_tabindices
            .insert((root.into(), generation), values);
        drop(state);
        self.notify_item_tabindex();
        true
    }
    fn projected_inventory(&self, mut facts: InventoryFacts) -> InventoryFacts {
        let state = self.0.borrow();
        for host in &mut facts.hosts {
            if let Some(value) = Self::mounted_item_key(&state, host)
                .and_then(|key| Self::current_item_tabindex(&state, &key))
            {
                host.tabindex_attribute = Some(value.to_string());
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
        if self
            .entry(&id)
            .is_some_and(|entry| entry.generation < generation)
        {
            self.cancel_owned_item_focus(&id);
            if let Some(old) = self.entry(&id) {
                self.clear_item_tabindices(&id, old.generation);
            }
        }
        let mut runtime = self.0.borrow_mut();
        if !runtime
            .nodes
            .get(&id)
            .is_some_and(|entry| entry.generation > generation)
        {
            if let Some(old_generation) = runtime.nodes.get(&id).map(|entry| entry.generation)
                && old_generation != generation
            {
                runtime.completions.remove_root(&id, old_generation);

                runtime
                    .submenu_marks
                    .retain(|_, (root, generation)| root != &id || *generation != old_generation);
                runtime
                    .item_descriptions
                    .retain(|_, item| item.owner.as_ref() != Some(&(id.clone(), old_generation)));
            }
            let revision = runtime
                .nodes
                .get(&id)
                .map_or(0, |entry| entry.revision.wrapping_add(1));
            let suppress_trigger_restore = runtime.nodes.get(&id).is_some_and(|entry| {
                entry.generation == generation && entry.suppress_trigger_restore
            });
            let surface_metadata = runtime
                .nodes
                .get(&id)
                .filter(|entry| entry.generation == generation)
                .map_or_else(SurfaceMetadata::default, |entry| {
                    entry.surface_metadata.clone()
                });
            let navigation_mode = runtime
                .nodes
                .get(&id)
                .filter(|entry| entry.generation == generation)
                .and_then(|entry| entry.navigation_mode.clone());
            runtime
                .registry
                .register(id.clone(), generation, kind, behavior);
            if !runtime.order.contains(&id) {
                runtime.order.push(id.clone());
            }
            let ownership = runtime
                .nodes
                .get(&id)
                .filter(|entry| entry.generation == generation)
                .map(|entry| {
                    (
                        entry.mounted,
                        entry.ownership_ready,
                        entry.ancestors.clone(),
                    )
                })
                .unwrap_or((false, false, Vec::new()));
            runtime.nodes.insert(
                id.clone(),
                Entry {
                    generation,
                    revision,
                    mounted: ownership.0,
                    ownership_ready: ownership.1,
                    ancestors: ownership.2,
                    before_operation: None,
                    kind,
                    behavior,
                    trigger_visibility_check: kind.is_dropdown(),
                    anchor_intersection: None,
                    surface_observer: None,
                    surface_metadata,
                    submenu: None,
                    navigation_mode,
                    suppress_trigger_restore,
                    callbacks,
                    scope,
                },
            );
        }
        drop(runtime);
        self.join_node(&id);
        self.publish_ownership();
        NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id,
            generation,
        }
    }
    #[cfg(test)]
    pub(crate) fn node_handle(&self, id: &str) -> Option<NodeHandle> {
        let generation = self.entry(id)?.generation;
        Some(NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id: id.into(),
            generation,
        })
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
        let metadata_runtime = Rc::downgrade(&self.0);
        let metadata_id = id.to_owned();
        domain.set_surface_reader(
            &self.owner_id(),
            id,
            generation,
            Rc::new(move || {
                RuntimeHandle(metadata_runtime.upgrade()?)
                    .surface_validity(&metadata_id, generation)
            }),
        );
        domain.set_surface_metadata(
            &self.owner_id(),
            id,
            generation,
            entry.surface_metadata.clone(),
        );
        if let Some(open) = self
            .with_owner(id, generation, || self.read_current(id, &entry))
            .flatten()
        {
            domain.sync_open(id, generation, open);
        }
    }
    /// Read after the owning callback: a callback may replace/unmount a surface
    /// or update its current trigger/forwarded host without a native mutation.
    fn surface_metadata(&self, id: &str, generation: u64) -> Option<SurfaceMetadata> {
        self.with_owner(id, generation, || {
            let snapshot = self.entry(id).filter(|entry| entry.mounted)?;
            self.read_current(id, &snapshot)?;
            self.entry(id)
                .filter(|entry| entry.mounted && self.current(id, generation, snapshot.revision))
                .map(|entry| entry.surface_metadata)
        })
        .flatten()
    }
    /// A callback-free final lease rejects a renderer dropped or registration
    /// removed/replaced by a later surface read, without replaying callbacks.
    fn surface_validity(&self, id: &str, generation: u64) -> Option<SurfaceValidity> {
        self.surface_metadata(id, generation)?;
        let revision = self.entry(id)?.revision;
        let weak = Rc::downgrade(&self.0);
        let id = id.to_owned();
        Some(Rc::new(move || {
            let Some(runtime) = weak.upgrade().map(RuntimeHandle) else {
                return false;
            };
            runtime.current(&id, generation, revision)
                && runtime.entry(&id).is_some_and(|entry| {
                    entry.mounted
                        && entry
                            .scope
                            .is_none_or(|(owner, _)| owner.upgrade().is_some())
                })
        }))
    }
    fn project_report(&self, mut report: Report) -> Report {
        let surfaces = if let Some(domain) = self.domain() {
            domain.surface_metadata()
        } else {
            let snapshots = {
                let state = self.0.borrow();
                state
                    .order
                    .iter()
                    .filter_map(|id| {
                        state
                            .nodes
                            .get(id)
                            .map(|entry| (id.clone(), entry.generation, entry.revision))
                    })
                    .collect::<Vec<_>>()
            };
            let surfaces = snapshots
                .into_iter()
                .filter_map(|(id, generation, revision)| {
                    self.surface_validity(&id, generation)
                        .map(|valid| (id, generation, revision, valid))
                })
                .collect::<Vec<_>>();
            // A later read may remove an earlier surface. Never retain that
            // stale destination in a projection after callback reentry.
            surfaces
                .into_iter()
                .filter_map(|(id, generation, revision, valid)| {
                    if !valid() || !self.current(&id, generation, revision) {
                        return None;
                    }
                    self.entry(&id)
                        .filter(|entry| entry.mounted)
                        .map(|entry| (id, generation, entry.surface_metadata))
                })
                .collect()
        };
        report.nodes = surfaces
            .into_iter()
            .map(|(id, generation, metadata)| NodeFacts {
                id,
                generation,
                trigger_id: metadata.trigger_id,
                items_host: metadata.items_host,
                popover_host: metadata.popover_host,
                focusable_ids: Vec::new(),
                tab_candidates: Vec::new(),
            })
            .collect();
        report.project_focus();
        report
    }
    /// Dioxus lifetimes and authored relationships are the only graph input.
    /// Native observations are limited to read-only geometry/focus/Tab facts.
    fn publish_ownership(&self) {
        let entries = self
            .0
            .borrow()
            .nodes
            .iter()
            .map(|(id, entry)| (id.clone(), entry.clone()))
            .collect::<Vec<_>>();
        let facts = entries
            .into_iter()
            .map(|(id, entry)| {
                let mut ancestors = Vec::new();
                for ancestor in &entry.ancestors {
                    let same_document = ancestor.runtime().is_some_and(|runtime| {
                        let document = runtime.0.borrow().document.clone();
                        let own_document = self.0.borrow().document.clone();
                        document.is_empty() || own_document.is_empty() || document == own_document
                    });
                    if ancestor.id != id
                        && ancestor.is_registered()
                        && same_document
                        && !ancestors.contains(&ancestor.id)
                    {
                        ancestors.push(ancestor.id.clone());
                    }
                }
                NativeFact {
                    id,
                    generation: entry.generation,
                    connected: entry.mounted,
                    discovery_ready: entry.ownership_ready,
                    assignment_ancestors: Some(ancestors.clone()),
                    trigger_ancestors: ancestors,
                }
            })
            .collect::<Vec<_>>();
        self.0.borrow_mut().registry.observe(&facts);
        if let Some(domain) = self.domain() {
            domain.observe_owner(&self.owner_id(), &facts);
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
        if !self.0.borrow().document.is_empty() {
            let changed = {
                let mut state = self.0.borrow_mut();
                let changed = !state.item_tabindices.is_empty();
                state.item_tabindices.clear();
                changed
            };
            if changed {
                self.notify_item_tabindex();
            }
            let roots: Vec<_> = self.0.borrow().owned_item_focus.keys().cloned().collect();
            for root in roots {
                self.cancel_owned_item_focus(&root);
            }
        }
        let (ids, categories) = {
            let mut state = self.0.borrow_mut();
            // Adopting another document cancels old-document native work.
            if !state.document.is_empty() {
                // The initial typed report binds an already-mounted renderer;
                // only a later document change invalidates its saved handles.
                state.document_generation = state.document_generation.wrapping_add(1);
                state.submenu_marks.clear();

                state.item_visual_host = None;
                state.item_visual = None;
                state.item_visual_target = None;
                state.item_visual_known = false;
                state.observation = None;
            }
            state.document = document.to_string();
            state.domain = Some(domain);
            (state.order.clone(), state.categories.clone())
        };
        if let Some(domain) = self.domain() {
            let weak = Rc::downgrade(&self.0);
            domain.register_focus_reader(
                self.owner_id(),
                Rc::new(move |id| RuntimeHandle(weak.upgrade()?).mounted_focus_target(id)),
            );
        }
        for id in ids {
            self.join_node(&id);
        }
        self.publish_ownership();
        for (id, callback) in categories {
            if let Some(domain) = self.domain() {
                domain.register_category(self.owner_id(), id, callback);
            }
        }
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
        } else {
            // Components can open before the document transport attaches. Reuse
            // their guarded Rust registrations without a second coordinator.
            let callbacks = self
                .0
                .borrow()
                .categories
                .iter()
                .filter(|(id, _)| **id != source)
                .map(|(_, callback)| callback.clone())
                .collect::<Vec<_>>();
            for callback in callbacks {
                callback(source, active);
            }
        }
        true
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
        self.dismiss_dropdown_set(id, generation, false, None)
    }

    fn dismiss_dropdown_set(
        &self,
        id: &str,
        generation: u64,
        inside: bool,
        restore_focus: Option<bool>,
    ) -> bool {
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
                domain.dismiss_dropdown_all(id, generation, restore_focus)
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
                            restore_focus: restore_focus
                                .unwrap_or_else(|| !inside || self.keyboard_mode()),
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
            // Cancel the previous open-state intent before apply. A synchronous
            // close completion may admit a new expected-closed return target.
            self.cancel_owned_item_focus(id);
        }
        (snapshot.callbacks.apply)(applied);
        if !change.open && self.read_current(id, &current) == Some(false) {
            self.clear_item_tabindices(id, generation);
        }
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
    fn owns_dioxus_topmost(&self, id: &str) -> bool {
        let observed_top = self
            .domain()
            .and_then(|domain| domain.topmost())
            .or_else(|| self.0.borrow().registry.topmost());
        if observed_top.is_some() {
            return self.owns_topmost(id);
        }
        // An explicitly mounted Dioxus control can receive its first key before
        // the observation bridge publishes connectivity. Current Rust owners
        // still determine the local open layer; no native key route is needed.
        let owners = {
            let state = self.0.borrow();
            state
                .order
                .iter()
                .rev()
                .filter_map(|id| state.nodes.get(id).map(|entry| (id.clone(), entry.clone())))
                .collect::<Vec<_>>()
        };
        owners
            .into_iter()
            .find(|(id, entry)| self.read_current(id, entry) == Some(true))
            .is_some_and(|(top, _)| {
                top == id
                    || self
                        .0
                        .borrow()
                        .registry
                        .live_ancestors(&top)
                        .iter()
                        .any(|parent| parent == id)
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
        let Some(parent) = parent else { return };
        let parent_entry = self.entry(&parent);
        let marker = parent_entry
            .as_ref()
            .and_then(|entry| self.first_submenu_marker(&parent, entry.generation));
        // Capture the parent's own inventory before forwarding is reset. Native
        // hide receipts and arbitrary-DOM return targets are unnecessary: the
        // existing Dioxus item executor validates the current open owner/mount.
        let own = parent_entry
            .as_ref()
            .and_then(|entry| self.0.borrow().owned_inventory(&parent, entry.generation));
        let host = marker.as_ref().and_then(|capture| {
            own.as_ref()?
                .hosts
                .iter()
                .find(|host| host.capture == *capture)
                .cloned()
        });
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
        let (Some(capture), Some(entry)) = (marker, parent_entry) else {
            return;
        };
        self.0.borrow_mut().submenu_marks.remove(&capture);
        if let Some(access) = entry.submenu {
            self.with_owner(&parent, entry.generation, || (access.reset)());
        }
        let Some(host) = host.filter(|host| !host.disabled_attribute && !host.hidden_attribute)
        else {
            return;
        };
        let node = NodeHandle {
            runtime: Rc::downgrade(&self.0),
            id: parent.clone(),
            generation: entry.generation,
        };
        if !self.item_owner_open(&parent, entry.generation) || !node.is_item_registered(&host) {
            return;
        }
        let roving = self
            .entry(&parent)
            .is_some_and(|entry| entry.navigation_mode.as_deref() == Some("roving-tabindex"));
        if roving
            && !own
                .as_ref()
                .is_some_and(|facts| node.select_roving(facts, &host.item))
        {
            return;
        }
        node.queue_item_host_focus(&host, Rc::new(|| {}));
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
        if !open {
            self.clear_item_tabindices(id, generation);
        }
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
        self.cancel_owned_item_focus(id);
        self.clear_item_tabindices(id, generation);
        let mut runtime = self.0.borrow_mut();
        if runtime
            .nodes
            .get(id)
            .is_some_and(|entry| entry.generation == generation)
        {
            runtime.completions.remove_root(id, generation);

            runtime
                .submenu_marks
                .retain(|_, (root, version)| root != id || *version != generation);
            runtime
                .item_descriptions
                .retain(|_, item| item.owner.as_ref() != Some(&(id.to_string(), generation)));
            runtime.nodes.remove(id);
            runtime.order.retain(|candidate| candidate != id);
            runtime.registry.unregister(id, generation);
            if let Some(domain) = &runtime.domain {
                domain.unregister(id, generation);
            }
        }
        drop(runtime);
        self.publish_ownership();
    }
    /// Existing request effects subscribe to source availability. A renderless
    /// lease may start after its parent's initial effect; no request is queued.
    fn read_bridge_ready(&self) {
        if let Some(ready) = self.0.borrow().bridge_ready {
            let _ = ready.try_read();
        }
    }
    fn publish_bridge_ready(&self, value: bool) {
        let ready = self.0.borrow().bridge_ready;
        if let Some(mut ready) = ready
            && let Ok(mut current) = ready.try_write()
            && *current != value
        {
            *current = value;
        }
    }
    fn acquire_bridge(
        &self,
        document: Option<Rc<dyn dioxus::document::Document>>,
        renderer: Weak<dioxus::core::Runtime>,
    ) {
        {
            let mut state = self.0.borrow_mut();
            state.bridge_leases += 1;
            if state.bridge.is_some() {
                return;
            }
        }
        let (Some(document), Some(renderer)) = (document, renderer.upgrade()) else {
            return;
        };
        renderer.in_scope(ScopeId::ROOT, || {
            let (owner, sequence, epoch) = {
                let mut state = self.0.borrow_mut();
                state.bridge_epoch = state.bridge_epoch.wrapping_add(1);
                state.bridge_owner = Some((
                    Rc::downgrade(&renderer),
                    CopyValue::new_in_scope((), ScopeId::ROOT),
                ));
                (
                    serde_json::to_string(&state.owner).expect("overlay runtime owner"),
                    state.sequence,
                    state.bridge_epoch,
                )
            };
            // Desktop's real Document provider is below ROOT. Use the provider
            // captured by the lease while the Eval/task belong to ROOT.
            let script = crate::overlay_registry_dom::BRIDGE
                .replace("__OWNER__", &owner)
                .replace("__START_SEQUENCE__", &sequence.to_string());
            let bridge = document.eval(script);
            let mut receiver = bridge;
            let weak = Rc::downgrade(&self.0);
            let task = spawn(async move {
                while let Ok(report) = receiver.recv::<Report>().await {
                    let Some(runtime) = weak.upgrade().map(RuntimeHandle) else {
                        break;
                    };
                    if !runtime.receive_report(epoch, report) {
                        break;
                    }
                }
            });
            {
                let mut state = self.0.borrow_mut();
                state.bridge = Some(bridge);
                state.bridge_task = Some(task);
            }
            self.publish_bridge_ready(true);
        });
    }
    fn release_bridge(&self) {
        let stopped = {
            let mut state = self.0.borrow_mut();
            if state.bridge_leases == 0 {
                return;
            }
            state.bridge_leases -= 1;
            if state.bridge_leases != 0 {
                return;
            }
            state.bridge_epoch = state.bridge_epoch.wrapping_add(1);
            state.bridge_owner = None;
            (state.bridge.take(), state.bridge_task.take())
        };
        if let Some(bridge) = stopped.0 {
            let _ = bridge.send(serde_json::json!({"finish":true}));
        }
        if let Some(task) = stopped.1 {
            task.cancel();
        }
        self.publish_bridge_ready(false);
    }
    fn receive_report(&self, epoch: u64, report: Report) -> bool {
        let current = {
            let state = self.0.borrow();
            state.bridge_epoch == epoch
                && state.bridge_leases > 0
                && state.bridge.is_some()
                && state.bridge_owner.as_ref().is_some_and(|(renderer, live)| {
                    renderer.upgrade().is_some() && live.try_read().is_ok()
                })
        };
        if !current {
            return false;
        }
        self.observe(&report);
        true
    }
    #[cfg(test)]
    pub(crate) fn test_report(&self, value: serde_json::Value) {
        if let Ok(report) = serde_json::from_value(value) {
            self.observe(&report);
        }
    }
    fn set_navigation_mode(&self, id: &str, generation: u64, mode: &str) -> bool {
        if !matches!(mode, "active-descendant" | "roving-tabindex") {
            return false;
        }
        let Some(entry) = self
            .entry(id)
            .filter(|entry| entry.generation == generation && entry.kind == OverlayKind::Dropdown)
        else {
            return false;
        };
        if entry.navigation_mode.as_deref() != Some(mode) {
            self.clear_item_tabindices(id, generation);
        }
        let mut state = self.0.borrow_mut();
        let Some(entry) = state
            .nodes
            .get_mut(id)
            .filter(|entry| entry.generation == generation)
        else {
            return false;
        };
        entry.navigation_mode = Some(mode.into());
        true
    }
    #[cfg(test)]
    pub(crate) fn pending_item_focus_count(&self) -> usize {
        self.0.borrow().owned_item_focus.len()
    }
    #[cfg(test)]
    pub(crate) fn pending_completions(&self) -> usize {
        self.0.borrow().completions.len()
    }
    fn observe_document(&self, report: &Report) -> KeyDecision {
        let Some(event) = report.document_event.as_ref().filter(|event| event.valid()) else {
            return KeyDecision::default();
        };
        let domain = {
            let state = self.0.borrow();
            if report.owner != state.owner
                || report.sequence <= state.sequence
                || report.document.is_empty()
                || report.document != state.document
            {
                return KeyDecision::default();
            }
            let Some(domain) = state.domain.clone() else {
                return KeyDecision::default();
            };
            domain
        };
        let admitted = domain.accept_event(event.sequence);
        if !admitted && !domain.is_current_event(event.sequence) {
            return KeyDecision::default();
        }
        {
            let mut state = self.0.borrow_mut();
            state.sequence = report.sequence;
        }
        match &event.action {
            DocumentAction::Input { input_kind, key } => {
                self.cancel_item_repairs(None);
                if admitted {
                    domain.observe_input(input_kind, key);
                }
                if matches!(input_kind.as_str(), "mousedown" | "touchstart") {
                    self.0.borrow_mut().keyboard = false;
                    self.clear_item_visual();
                    // Each renderer owner receives the current shared event,
                    // even when another owner admitted its document sequence.
                    self.notify_surfaces(SurfaceObservation::Pointer, None);
                } else if let Some(keyboard) = domain.keyboard_mode() {
                    self.0.borrow_mut().keyboard = keyboard;
                }
            }
            DocumentAction::Blur => self.cancel_item_repairs(None),
            DocumentAction::Outside {
                hosts,
                triggers,
                anchors,
            } if admitted => {
                self.cancel_item_repairs(None);
                let mut inside = hosts.clone();
                inside.extend(anchors.iter().cloned());
                domain.outside(&inside, triggers);
            }
            DocumentAction::Escape if admitted => {
                domain.escape();
            }
            _ => {}
        }
        KeyDecision::default()
    }
    fn observe(&self, report: &Report) -> KeyDecision {
        if report.kind == "document" {
            // Minimal global input is not a DOM observation and cannot attach a
            // document, disconnect nodes or replace any retained inventory.
            return self.observe_document(report);
        }
        if report.kind != "observe" {
            return KeyDecision::default();
        }
        {
            let mut runtime = self.0.borrow_mut();
            if report.owner != runtime.owner || report.sequence <= runtime.sequence {
                return KeyDecision::default();
            }
            runtime.sequence = report.sequence;
        }
        self.attach_document(&report.document);
        let projected = self.project_report(report.clone());
        let report = &projected;
        {
            let mut runtime = self.0.borrow_mut();
            // A snapshot supplies read-only order, focus and Tab destinations.
            // Dioxus event handlers supply all keyboard policy and origin.
            runtime.observation = Some(report.clone());
        }
        if report.focus_changed
            || (report.focus_request.is_none()
                && !report.active_default
                && !report.active_id.is_empty())
        {
            self.cancel_item_repairs((!report.active_default).then_some(report.active_id.as_str()));
        }
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
            if let Some(open) = self
                .with_owner(id, entry.generation, || self.read_current(id, entry))
                .flatten()
            {
                self.0.borrow_mut().registry.set_presence(id, open);
            }
        }
        self.repair_moved_item_focus(report);
        self.notify_surfaces(
            SurfaceObservation::Focus(FocusObservation::from(report)),
            Some(report),
        );
        self.notify_focus_observers(report);
        if let Some(request) = &report.focus_request {
            self.deliver_scoped_focus(
                &request.id,
                request.generation,
                report.sequence,
                ScopedFocusEvent::Requested {
                    request: request.request,
                    facts: ScopedFocusFacts::from(report),
                },
            );
        }
        if let Some(request) = &report.focus_request
            && self.0.borrow().sequence == report.sequence
        {
            // Ordinary observers may have removed/replaced the requester.
            // A request is delivered only to its live registration, never as a
            // broadcast or a synthetic focus-change notification.
            self.with_owner(&request.id, request.generation, || {
                if let Some(observer) = self
                    .entry(&request.id)
                    .filter(|entry| {
                        entry.mounted
                            && entry.generation == request.generation
                            && report.nodes.iter().any(|node| {
                                node.id == request.id && node.generation == request.generation
                            })
                    })
                    .and_then(|entry| entry.surface_observer)
                {
                    observer(SurfaceObservation::RequestedFocus {
                        request: request.request,
                        focus: FocusObservation::from(report),
                    });
                }
            });
        }
        KeyDecision::default()
    }
    fn notify_surfaces(&self, observation: SurfaceObservation, report: Option<&Report>) {
        let surfaces = {
            let runtime = self.0.borrow();
            runtime
                .order
                .iter()
                .filter_map(|id| {
                    let entry = runtime.nodes.get(id)?;
                    entry.surface_observer.as_ref()?;
                    Some((id.clone(), entry.generation))
                })
                .collect::<Vec<_>>()
        };
        for (id, generation) in surfaces {
            if report.is_some_and(|report| {
                !report
                    .nodes
                    .iter()
                    .any(|node| node.id == id && node.generation == generation)
            }) {
                continue;
            }
            // An earlier callback may remove or replace a later surface.
            // Re-read its mounted generation/callback outside Runtime borrows.
            self.with_owner(&id, generation, || {
                if let Some(observer) = self
                    .entry(&id)
                    .filter(|entry| entry.mounted && entry.generation == generation)
                    .and_then(|entry| entry.surface_observer)
                {
                    observer(observation.clone());
                }
            });
        }
    }
    fn reported_kind(&self, report: &Report, id: &str) -> Option<OverlayKind> {
        let node = report.nodes.iter().find(|node| node.id == id)?;
        self.entry(id)
            .filter(|entry| entry.generation == node.generation)
            .map(|entry| entry.kind)
            .or_else(|| self.domain()?.node_kind(id, node.generation))
    }
    fn document_tab_order(&self, report: &Report) -> Vec<String> {
        let candidates = report
            .document_candidates
            .iter()
            .filter(|candidate| {
                !candidate.hosts.iter().any(|host| {
                    self.reported_kind(report, host)
                        .is_some_and(OverlayKind::is_dropdown)
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        crate::popover_focus::tab_order_targets(&candidates)
    }
    fn dispatch_key(
        &self,
        id: &str,
        generation: u64,
        facts: KeyFacts,
        report: &Report,
    ) -> KeyDecision {
        self.with_owner(id, generation, || {
            let Some(entry) = self
                .entry(id)
                .filter(|entry| entry.generation == generation)
            else {
                return KeyDecision::default();
            };
            if facts.default_prevented || (!facts.inside && !facts.on_trigger) {
                return KeyDecision::default();
            }
            let Some(owner_open) = self.read_current(id, &entry) else {
                return KeyDecision::default();
            };
            if matches!(facts.key.as_str(), "Escape" | "Tab") && !owner_open {
                return KeyDecision::default();
            }
            if let Some(domain) = self.domain() {
                domain.observe_input("keydown", &facts.key);
            }
            self.0.borrow_mut().keyboard = true;
            if facts.key == "Escape" {
                if !self.owns_dioxus_topmost(id) {
                    return KeyDecision::default();
                }
                self.dismiss_root(id, Reason::Escape, true);
                return KeyDecision {
                    // The shared window listener must not dismiss another
                    // owner when this handled event leaves the Dioxus root.
                    prevent: true,
                    stop: true,
                    ..KeyDecision::default()
                };
            }
            let Some(callback) = entry.callbacks.key.clone() else {
                return KeyDecision::default();
            };
            // Capture the mounted destination before close callbacks cancel old
            // focus intents or change the current graph. Admission occurs after.
            let current_report = self.project_report(report.clone());
            if !self.current(id, entry.generation, entry.revision) {
                return KeyDecision::default();
            }
            let tab_release = self.tab_release_target(id, &facts, &current_report);
            let mut decision = callback(&facts);
            if !self.current(id, entry.generation, entry.revision)
                || self.read_current(id, &entry) != Some(true)
            {
                decision.activate = None;
            }
            if let Some((root, generation, target, popovers)) =
                tab_release.filter(|_| decision.release_focus)
                && self.queue_exit_focus(&root, generation, target, popovers)
            {
                decision.prevent = true;
            }
            decision
        })
        .unwrap_or_default()
    }
    fn tab_release_target(
        &self,
        id: &str,
        facts: &KeyFacts,
        report: &Report,
    ) -> Option<(String, u64, MountedFocusTarget, Vec<String>)> {
        if facts.key != "Tab"
            || (facts.navigation_mode.as_deref() != Some("roving-tabindex")
                && !facts.popover_ancestor)
        {
            return None;
        }
        let root = self.domain().map_or_else(
            || self.0.borrow().registry.cached_root(id),
            |domain| domain.cached_root(id),
        );
        root.and_then(|root| {
            let entry = self.entry(&root)?;
            let node = report
                .nodes
                .iter()
                .find(|node| node.id == root && node.generation == entry.generation)?;
            if self.read_current(&root, &entry) != Some(true) {
                return None;
            }
            // A linked child trigger belongs to its enclosing Dropdown,
            // although it is outside the child's own surface. Preserve
            // that registered relationship without changing event origin.
            let linked_parent = facts.on_trigger
                && self
                    .domain()
                    .map_or_else(
                        || self.0.borrow().registry.cached_parent(id),
                        |domain| domain.cached_parent(id),
                    )
                    .is_some_and(|parent| {
                        self.reported_kind(report, &parent)
                            .is_some_and(OverlayKind::is_dropdown)
                    });
            let target = if facts.shift
                && (linked_parent
                    || report.hosts.iter().any(|host| {
                        self.reported_kind(report, host)
                            .is_some_and(OverlayKind::is_dropdown)
                    })) {
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
                        crate::popover_focus::tab_adjacent_target(
                            &parent_node.tab_candidates,
                            &excluded,
                            &node.trigger_id,
                            facts.shift,
                        )
                    })
                    .or_else(|| {
                        let order = self.document_tab_order(report);
                        order
                            .iter()
                            .position(|id| id == &node.trigger_id)
                            .and_then(|index| {
                                let next = if facts.shift {
                                    index.checked_sub(1)
                                } else {
                                    index.checked_add(1)
                                }?;
                                order.get(next).cloned()
                            })
                    })
            }?;
            let mounted = self
                .mounted_focus_target(&target)
                .or_else(|| self.domain()?.focus_target(&target))?;
            let mut popovers = Vec::new();
            for candidate in report
                .focus_candidates
                .iter()
                .filter(|candidate| candidate.target == target)
            {
                for host in &candidate.hosts {
                    if !popovers.contains(host)
                        && self.reported_kind(report, host) == Some(OverlayKind::Popover)
                    {
                        popovers.push(host.clone());
                    }
                }
            }
            Some((root, entry.generation, mounted, popovers))
        })
    }
    fn anchor_visibility(
        &self,
        id: &str,
        generation: u64,
        intersecting: bool,
        geometry: Option<AnchorIntersection>,
    ) {
        let Some(entry) = self.entry(id).filter(|entry| {
            entry.generation == generation
                && entry.kind.is_dropdown()
                && entry.trigger_visibility_check
        }) else {
            return;
        };
        // Dioxus owns the observed trigger and event lifecycle. Enter the
        // registered surface owner before callbacks; no DOM identity receipt or
        // native visibility transport is needed for an explicit component.
        self.with_owner(id, generation, || {
            if self.read_current(id, &entry) != Some(true) {
                return;
            }
            if intersecting {
                if let Some(geometry) = geometry.filter(|geometry| geometry.valid())
                    && let Some(callback) = &entry.anchor_intersection
                {
                    callback(geometry);
                }
            } else {
                self.request(
                    id,
                    generation,
                    Change {
                        open: false,
                        reason: Reason::AnchorVisibility,
                        restore_focus: false,
                    },
                );
            }
        });
    }
    #[cfg(test)]
    fn input(&self, event: FormEvent) {
        event.stop_propagation();
        let Ok(report) = serde_json::from_str::<Report>(&event.value()) else {
            return;
        };
        self.observe(&report);
    }
}

impl NodeHandle {
    /// Publish current authored metadata without signal writes or callbacks.
    /// Domain readers share this current registration, never a native fallback.
    pub(crate) fn set_surface_metadata(&self, metadata: SurfaceMetadata) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        {
            let mut state = runtime.0.borrow_mut();
            let Some(entry) = state
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            else {
                return;
            };
            entry.surface_metadata = metadata.clone();
        }
        if let Some(domain) = runtime.domain() {
            domain.set_surface_metadata(&runtime.owner_id(), &self.id, self.generation, metadata);
        }
    }
    #[cfg(test)]
    pub(crate) fn surface_metadata(&self) -> Option<SurfaceMetadata> {
        self.runtime()?
            .entry(&self.id)
            .filter(|entry| entry.generation == self.generation)
            .map(|entry| entry.surface_metadata)
    }
    pub(crate) fn set_ownership(&self, ancestors: Vec<NodeHandle>, ready: bool) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let mut lineage = Vec::new();
        for ancestor in ancestors {
            if ancestor != *self && ancestor.is_registered() && !lineage.contains(&ancestor) {
                lineage.push(ancestor);
            }
        }
        let changed = {
            let mut state = runtime.0.borrow_mut();
            let Some(entry) = state
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            else {
                return;
            };
            if entry.ancestors == lineage && entry.ownership_ready == ready {
                false
            } else {
                entry.ancestors = lineage;
                entry.ownership_ready = ready;
                true
            }
        };
        if changed {
            runtime.publish_ownership();
        }
    }
    pub(crate) fn set_mounted(&self) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let changed = {
            let mut state = runtime.0.borrow_mut();
            let Some(entry) = state
                .nodes
                .get_mut(&self.id)
                .filter(|entry| entry.generation == self.generation)
            else {
                return;
            };
            let changed = !entry.mounted;
            entry.mounted = true;
            changed
        };
        if changed {
            runtime.publish_ownership();
        }
    }

    pub(crate) fn owns_keyboard_topmost(&self) -> bool {
        self.runtime().is_some_and(|runtime| {
            runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
                && runtime.owns_dioxus_topmost(&self.id)
        })
    }

    /// Plan a boundary wrap for a directly delivered Dioxus Tab event.
    /// Callers apply the returned default/propagation decision to that event.
    pub(crate) fn dioxus_popover_tab(&self, event: &KeyboardEvent, enabled: bool) -> KeyDecision {
        if !enabled || !event.default_action_enabled() || event.key() != Key::Tab {
            return KeyDecision::default();
        }
        let Some(runtime) = self.runtime() else {
            return KeyDecision::default();
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                let Some(entry) = runtime.entry(&self.id).filter(|entry| {
                    entry.generation == self.generation
                        && entry.kind == OverlayKind::Popover
                        && entry.mounted
                }) else {
                    return KeyDecision::default();
                };
                if runtime.read_current(&self.id, &entry) != Some(true) {
                    return KeyDecision::default();
                }
                let observation = runtime.0.borrow().observation.clone();
                let Some(report) = observation.map(|report| runtime.project_report(report)) else {
                    return KeyDecision::default();
                };
                let Some(node) = report
                    .nodes
                    .iter()
                    .find(|node| node.id == self.id && node.generation == self.generation)
                else {
                    return KeyDecision::default();
                };
                let Some(domain) = runtime.domain() else {
                    return KeyDecision::default();
                };
                let Some(excluded) = domain.focus_trap_exclusions(
                    &self.id,
                    &report.active_hosts,
                    &report.active_popover,
                ) else {
                    return KeyDecision::default();
                };
                let Some(target) = crate::popover_focus::tab_boundary_target(
                    &node.tab_candidates,
                    &excluded,
                    event.modifiers().contains(Modifiers::SHIFT),
                ) else {
                    return KeyDecision::default();
                };
                let Some(target) = self.capture_registered_focus(&target) else {
                    return KeyDecision::default();
                };
                if !self.queue_registered_focus(target, Some(true)) {
                    return KeyDecision::default();
                }
                KeyDecision {
                    prevent: true,
                    stop: true,
                    ..Default::default()
                }
            })
            .unwrap_or_default()
    }
    /// A current Dioxus content surface owns its virtual click propagation.
    /// Keep browser defaults intact, including native input and form actions.
    pub(crate) fn dioxus_inside_click(&self, event: &MouseEvent) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let _ = runtime.with_owner(&self.id, self.generation, || {
            let Some(entry) = runtime
                .entry(&self.id)
                .filter(|entry| entry.generation == self.generation && entry.kind.is_dropdown())
            else {
                return;
            };
            if runtime.read_current(&self.id, &entry) != Some(true) {
                return;
            }
            event.stop_propagation();
            if event.default_action_enabled() && entry.behavior.allows_inside() {
                runtime.dismiss_dropdown_set(&self.id, self.generation, true, None);
            }
        });
    }
    /// Semantic keyboard activation uses the same inside-dismissal policy as a
    /// pointer item, with an explicit keyboard focus return intent.
    pub(crate) fn dioxus_item_activation(&self) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        let _ = runtime.with_owner(&self.id, self.generation, || {
            let Some(entry) = runtime
                .entry(&self.id)
                .filter(|entry| entry.generation == self.generation && entry.kind.is_dropdown())
            else {
                return;
            };
            if runtime.read_current(&self.id, &entry) == Some(true)
                && entry.behavior.allows_inside()
            {
                runtime.dismiss_dropdown_set(&self.id, self.generation, true, Some(true));
            }
        });
    }
    /// Build keyboard facts from an explicit Dioxus control and the latest
    /// read-only inventory/Tab observation. Registration, not native listener
    /// installation or a captured DOM receipt, authorizes the event.
    pub(crate) fn dioxus_key_facts(
        &self,
        event: &KeyboardEvent,
        on_trigger: bool,
        focused_target: Option<String>,
        inside: bool,
    ) -> Option<KeyFacts> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                let entry = runtime.entry(&self.id)?;
                runtime.read_current(&self.id, &entry)?;
                let navigation_root = entry
                    .submenu
                    .as_ref()
                    .and_then(|access| (access.query_root)())
                    .filter(|root| runtime.entry(root).is_some());
                let navigation_id = navigation_root.as_deref().unwrap_or(&self.id);
                let navigation_entry = runtime.entry(navigation_id)?;
                let observation = runtime.0.borrow().observation.clone();
                let projected = observation.map(|report| runtime.project_report(report));
                let observed = projected.as_ref();
                if !runtime.current(&self.id, self.generation, entry.revision) {
                    return None;
                }
                let state = runtime.0.borrow();
                let owner = observed.and_then(|report| {
                    report
                        .nodes
                        .iter()
                        .find(|node| node.id == self.id && node.generation == self.generation)
                });
                let inventory = state.owned_inventory(navigation_id, navigation_entry.generation);
                let focused = focused_target.or_else(|| {
                    if inside && !on_trigger {
                        observed
                            .map(|report| report.active_id.clone())
                            .filter(|target| !target.is_empty())
                    } else {
                        None
                    }
                });
                let physical_target = focused.as_ref().map(|target| {
                    state
                        .item_mounts
                        .values()
                        .find(|mount| mount.target == *target || mount.focus_id == *target)
                        .map_or_else(|| target.clone(), |mount| mount.focus_id.clone())
                });
                let focused_target = focused.map(|target| {
                    state
                        .item_mounts
                        .values()
                        .find(|mount| mount.focus_id == target)
                        .map_or(target.clone(), |mount| mount.target.clone())
                });
                let focusable_ids =
                    owner.map_or_else(Vec::new, |owner| owner.focusable_ids.clone());
                let focused_index = physical_target.as_ref().and_then(|target| {
                    focusable_ids
                        .iter()
                        .position(|candidate| candidate == target)
                });
                let focused_item = focused_target
                    .as_ref()
                    .and_then(|target| target.strip_prefix("ix-dropdown-item-")?.parse().ok());
                let items = Vec::new();
                drop(state);
                Some(KeyFacts {
                    navigation_mode: navigation_entry.navigation_mode,
                    key: event.key().to_string(),
                    shift: event.modifiers().contains(Modifiers::SHIFT),
                    alt: event.modifiers().contains(Modifiers::ALT),
                    ctrl: event.modifiers().contains(Modifiers::CONTROL),
                    meta: event.modifiers().contains(Modifiers::META),
                    default_prevented: !event.default_action_enabled(),
                    focused_item,
                    focused_target,
                    items,
                    focus_count: focusable_ids.len(),
                    focusable_ids,
                    focused_index,
                    on_trigger,
                    inside,
                    popover_ancestor: runtime
                        .domain()
                        .is_some_and(|domain| domain.has_popover_ancestor(&self.id)),
                    inventory: inventory.map(|facts| runtime.projected_inventory(facts)),
                })
            })
            .flatten()
    }
    /// Dispatch one Dioxus key in this surface's current owner. The snapshot is
    /// used only to select registered mounted destinations before callbacks.
    pub(crate) fn dioxus_key(&self, mut facts: KeyFacts) -> KeyDecision {
        let Some(runtime) = self.runtime() else {
            return KeyDecision::default();
        };
        let Some(entry) = runtime
            .entry(&self.id)
            .filter(|entry| entry.generation == self.generation)
        else {
            return KeyDecision::default();
        };
        facts.navigation_mode = entry
            .submenu
            .as_ref()
            .and_then(|access| (access.query_root)())
            .and_then(|root| runtime.entry(&root))
            .and_then(|entry| entry.navigation_mode)
            .or(entry.navigation_mode);
        facts.popover_ancestor = runtime
            .domain()
            .is_some_and(|domain| domain.has_popover_ancestor(&self.id));
        let mut report = runtime.0.borrow().observation.clone().unwrap_or_default();
        report.hosts = if facts.inside {
            vec![self.id.clone()]
        } else {
            Vec::new()
        };
        report.triggers = if facts.on_trigger {
            vec![self.id.clone()]
        } else {
            Vec::new()
        };
        runtime.dispatch_key(&self.id, self.generation, facts, &report)
    }
    pub(crate) fn capture_registered_focus(&self, id: &str) -> Option<MountedFocusTarget> {
        let runtime = self.runtime()?;
        runtime
            .with_owner(&self.id, self.generation, || {
                runtime.capture_registered_focus(id)
            })
            .flatten()
    }
    pub(crate) fn queue_registered_focus(
        &self,
        target: MountedFocusTarget,
        expected_open: Option<bool>,
    ) -> bool {
        self.runtime().is_some_and(|runtime| {
            runtime.queue_registered_focus(
                &self.id,
                self.generation,
                target,
                expected_open,
                Vec::new(),
            )
        })
    }
    pub(crate) fn cancel_registered_focus(&self) {
        if let Some(runtime) = self.runtime()
            && runtime
                .entry(&self.id)
                .is_some_and(|entry| entry.generation == self.generation)
        {
            runtime.cancel_owned_item_focus(&self.id);
        }
    }
    /// Revision reflected by the component's next native renderer commit.
    pub(crate) fn render_revision(&self) -> u64 {
        self.runtime()
            .and_then(|runtime| runtime.entry(&self.id))
            .filter(|entry| entry.generation == self.generation)
            .map_or(0, |entry| entry.revision)
    }
    pub(crate) fn set_navigation_mode(&self, mode: &str) -> bool {
        self.runtime()
            .is_some_and(|runtime| runtime.set_navigation_mode(&self.id, self.generation, mode))
    }
    pub(crate) async fn query_inventory(&self) -> Option<InventoryFacts> {
        self.inventory()
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
                    .owned_inventory(&self.id, self.generation)?;
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
                if !runtime.item_owner_open(&self.id, self.generation) {
                    return false;
                }
                let inventory = crate::dropdown_inventory::Inventory::new(
                    crate::dropdown_inventory::InventoryMode::RovingTabindex,
                    runtime.projected_inventory(facts.clone()),
                );
                runtime.set_roving_target(
                    &self.id,
                    self.generation,
                    &inventory.query_hosts(),
                    target,
                )
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
                if !runtime.item_owner_open(&self.id, self.generation) {
                    return None;
                }
                let inventory = crate::dropdown_inventory::Inventory::new(
                    crate::dropdown_inventory::InventoryMode::RovingTabindex,
                    runtime.projected_inventory(facts.clone()),
                );
                let candidates = {
                    let state = runtime.0.borrow();
                    inventory
                        .query_hosts()
                        .into_iter()
                        .filter(|host| RuntimeHandle::mounted_item_key(&state, host).is_some())
                        .collect::<Vec<_>>()
                };
                let edge = if last {
                    candidates.last()
                } else {
                    candidates.first()
                }?;
                let target = if focus_checked {
                    candidates
                        .iter()
                        .find(|host| host.checked_attribute)
                        .unwrap_or(edge)
                } else {
                    edge
                };
                runtime
                    .set_roving_target(&self.id, self.generation, &candidates, &target.item)
                    .then(|| target.clone())
            })
            .flatten()
    }
    pub(crate) fn clear_roving(&self) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if runtime
            .entry(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
        {
            runtime.clear_item_tabindices(&self.id, self.generation);
        }
    }
    pub(crate) fn is_item_registered(&self, host: &HostFacts) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime
            .with_owner(&self.id, self.generation, || {
                RuntimeHandle::mounted_item_key(&runtime.0.borrow(), host).is_some()
            })
            .unwrap_or(false)
    }
    pub(crate) fn queue_item_host_focus(&self, host: &HostFacts, callback: Rc<dyn Fn()>) -> bool {
        self.queue_mounted_item_focus(host.target.clone(), true, callback, Some(host.clone()))
    }
    /// Native candidate order is observational only; registered Dioxus handles
    /// remain the sole executor, including items with a distinct inner control.
    pub(crate) fn queue_tab_boundary_focus(
        &self,
        ids: &[String],
        last: bool,
        callback: Rc<dyn Fn()>,
    ) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        if !runtime.item_owner_open(&self.id, self.generation) {
            return false;
        }
        let Some(inventory) = self.inventory() else {
            return false;
        };
        let host = {
            let state = runtime.0.borrow();
            let matching = |id: &String| {
                if id.is_empty() {
                    return None;
                }
                inventory
                    .hosts
                    .iter()
                    .find(|host| {
                        !host.disabled_attribute
                            && !host.hidden_attribute
                            && RuntimeHandle::mounted_item_key(&state, host)
                                .and_then(|key| state.item_mounts.get(&key))
                                .is_some_and(|mount| mount.focus_id == *id)
                    })
                    .cloned()
            };
            if last {
                ids.iter().rev().find_map(matching)
            } else {
                ids.iter().find_map(matching)
            }
        };
        host.is_some_and(|host| self.queue_item_host_focus(&host, callback))
    }
    /// Only explicitly registered Dioxus mounted elements can receive item focus.
    /// Consumers opt in by registering their own mounted control.
    pub(crate) fn queue_item_focus(
        &self,
        target: String,
        focus_native: bool,
        callback: Rc<dyn Fn()>,
    ) -> bool {
        self.queue_mounted_item_focus(target, focus_native, callback, None)
    }
    fn queue_mounted_item_focus(
        &self,
        target: String,
        focus_native: bool,
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
                if entry.scope.is_none()
                    || runtime.read_current(&self.id, &entry) != Some(true)
                    || !runtime
                        .entry(&self.id)
                        .is_some_and(|entry| entry.generation == self.generation)
                {
                    return false;
                }
                let visual = visual.or_else(|| {
                    runtime
                        .0
                        .borrow()
                        .owned_inventory(&self.id, self.generation)
                        .and_then(|inventory| {
                            inventory
                                .hosts
                                .into_iter()
                                .find(|host| host.target == target)
                        })
                });
                let key = {
                    let state = runtime.0.borrow();
                    visual
                        .as_ref()
                        .filter(|host| state.item_mounts.contains_key(&host.registration_key))
                        .map(|host| host.registration_key.clone())
                        .or_else(|| {
                            state
                                .item_mounts
                                .iter()
                                .find(|(_, mount)| mount.target == target)
                                .map(|(key, _)| key.clone())
                        })
                };
                let Some(key) = key else {
                    return false;
                };
                runtime.cancel_owned_item_focus(&self.id);
                let sequence = {
                    let mut state = runtime.0.borrow_mut();
                    state.item_focus_sequence = state.item_focus_sequence.wrapping_add(1).max(1);
                    let sequence = state.item_focus_sequence;
                    state.owned_item_focus.insert(
                        self.id.clone(),
                        OwnedItemFocus {
                            sequence,
                            generation: self.generation,
                            key,
                            callback: Some(callback),
                            visual,
                            focus_native,
                            exit_target: None,
                            repair_target: None,
                            expected_open: Some(true),
                            destination_popovers: Vec::new(),
                            task: None,
                        },
                    );
                    sequence
                };
                let weak = Rc::downgrade(&runtime.0);
                let root = self.id.clone();
                let generation = self.generation;
                dioxus::core::queue_effect(move || {
                    if let Some(state) = weak.upgrade() {
                        let runtime = RuntimeHandle(state);
                        runtime.with_owner(&root, generation, || {
                            runtime.start_owned_item_focus(root.clone(), generation, sequence)
                        });
                    }
                });
                true
            })
            .unwrap_or(false)
    }
    /// Deliver changed-state notifications through the registered component's
    /// Dioxus effect queue. Admission never invokes a listener synchronously.
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
                // A completion belongs to a real Dioxus component. There is no
                // native transport or synchronous fallback for unowned work.
                if entry.scope.is_none() || runtime.read_current(&self.id, &entry).is_none() {
                    return false;
                }
                let ticket = runtime.0.borrow_mut().completions.admit(
                    self.id.clone(),
                    self.generation,
                    callback,
                );
                let node = self.clone();
                dioxus::core::queue_effect(move || {
                    let Some(runtime) = node.runtime() else {
                        return;
                    };
                    let callback =
                        runtime
                            .0
                            .borrow_mut()
                            .completions
                            .take(ticket, &node.id, node.generation);
                    if let Some(callback) = callback {
                        runtime.with_owner(&node.id, node.generation, || {
                            let Some(entry) = runtime
                                .entry(&node.id)
                                .filter(|entry| entry.generation == node.generation)
                            else {
                                return;
                            };
                            if runtime.read_current(&node.id, &entry).is_some() {
                                callback();
                            }
                        });
                    }
                });
                // An async native completion can arrive without another
                // component state change. Wake this owner to flush its effect.
                dioxus::core::needs_update();
                true
            })
            .unwrap_or(false)
    }
    /// Request current canonical native facts through the existing bridge.
    /// The caller owns its operation token; the runtime retains no request.
    pub(crate) fn request_focus_observation(&self, request: u64) -> bool {
        let Some(runtime) = self.runtime() else {
            return false;
        };
        runtime.read_bridge_ready();
        if !runtime.entry(&self.id).is_some_and(|entry| {
            entry.mounted
                && entry.generation == self.generation
                && entry.kind == OverlayKind::Popover
                && entry
                    .scope
                    .as_ref()
                    .is_none_or(|(owner, _)| owner.upgrade().is_some())
        }) {
            return false;
        }
        runtime.0.borrow().bridge.as_ref().is_some_and(|bridge| {
            bridge
                .send(serde_json::json!({"focus_request": {
                    "id": self.id, "generation": self.generation, "request": request
                }}))
                .is_ok()
        })
    }
    /// Use native eligibility/document order once, then intersect it with
    /// current mounted Dioxus controls and the physical Popover partition.
    pub(crate) fn opening_focus_candidates(
        &self,
        observation: &FocusObservation,
    ) -> Vec<crate::popover_focus::FocusCandidate> {
        let Some(runtime) = self.runtime() else {
            return Vec::new();
        };
        let Some(entry) = runtime.entry(&self.id).filter(|entry| {
            entry.mounted
                && entry.generation == self.generation
                && entry.kind == OverlayKind::Popover
        }) else {
            return Vec::new();
        };
        let host = entry.surface_metadata.popover_host;
        if host.is_empty() {
            return Vec::new();
        }
        let state = runtime.0.borrow();
        observation
            .candidates
            .iter()
            .filter_map(|candidate| {
                if candidate.target.is_empty()
                    || candidate.target == host
                    || candidate.popover != host
                {
                    return None;
                }
                let mount = state.item_mounts.values().find(|mount| {
                    mount.focus_id == candidate.target
                        && mount
                            .renderer
                            .as_ref()
                            .is_none_or(|owner| owner.upgrade().is_some())
                        && mount.owner.as_ref().is_none_or(|(id, generation)| {
                            state.nodes.get(id).is_some_and(|entry| {
                                entry.mounted && entry.generation == *generation
                            })
                        })
                })?;
                let in_header = mount
                    .popover_scope
                    .as_ref()
                    .is_some_and(|scope| scope.header && scope.owner.as_ref() == Some(self));
                Some(crate::popover_focus::FocusCandidate {
                    target: candidate.target.clone(),
                    tab_index: candidate.tab_index,
                    in_header,
                })
            })
            .collect()
    }
    /// Reuse canonical focus facts and shared mouse/touch input in this surface's
    /// Dioxus scope. Installation never synchronously delivers during render.
    pub(crate) fn set_surface_observer(&self, observer: SurfaceObserver) {
        let Some(runtime) = self.runtime() else {
            return;
        };
        if let Some(entry) = runtime.0.borrow_mut().nodes.get_mut(&self.id)
            && entry.generation == self.generation
        {
            entry.surface_observer = Some(observer);
        }
    }
    #[cfg(test)]
    pub(crate) fn current_focus_observation(&self) -> Option<FocusObservation> {
        let runtime = self.runtime()?;
        runtime
            .entry(&self.id)
            .filter(|entry| entry.mounted && entry.generation == self.generation)?;
        let state = runtime.0.borrow();
        let report = state.observation.as_ref()?;
        report
            .nodes
            .iter()
            .any(|node| node.id == self.id && node.generation == self.generation)
            .then(|| FocusObservation::from(report))
    }
    /// Deliver visibility from a registered Dioxus trigger's `onvisible`.
    /// The component binding owns trigger lifetime; this handle additionally
    /// rejects dropped/replaced surfaces and dispatches in the surface owner.
    pub(crate) fn observe_anchor_visibility(
        &self,
        intersecting: bool,
        geometry: Option<AnchorIntersection>,
    ) {
        if let Some(runtime) = self.runtime() {
            runtime.anchor_visibility(&self.id, self.generation, intersecting, geometry);
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
    pub(crate) fn runtime(&self) -> Option<RuntimeHandle> {
        self.runtime.upgrade().map(RuntimeHandle)
    }
    pub(crate) fn id(&self) -> &str {
        &self.id
    }
    /// Execute admitted work in the registered component's runtime/scope.
    /// Queued effects must not inherit a transient invoker's scope.
    pub(crate) fn with_owner<T>(&self, f: impl FnOnce() -> T) -> Option<T> {
        self.runtime()?.with_owner(&self.id, self.generation, f)
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
        runtime
            .entry(&self.id)
            .is_some_and(|entry| entry.generation == self.generation)
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
    let document = use_hook(try_consume_context::<Rc<dyn dioxus::document::Document>>);
    let renderer = use_hook(|| Rc::downgrade(&dioxus::core::Runtime::current()));
    let leased = use_hook(|| Rc::new(std::cell::Cell::new(false)));
    let active = leased.clone();
    let source = runtime.clone();
    use_effect(move || {
        if !active.replace(true) {
            source.acquire_bridge(document.clone(), renderer.clone());
        }
    });
    use_drop(move || {
        if leased.replace(false) {
            runtime.release_bridge();
        }
    });
    VNode::empty()
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
#[path = "item_visual_transport_tests.rs"]
mod item_visual_transport_tests;

#[derive(Clone, Deserialize)]
struct FocusRequest {
    id: String,
    generation: u64,
    request: u64,
}

#[derive(Clone, Default, Deserialize)]
struct Report {
    owner: String,
    sequence: u64,
    kind: String,
    #[serde(default)]
    document: String,
    #[serde(default)]
    document_event: Option<DocumentEvent>,
    #[serde(default)]
    active_id: String,
    #[serde(default)]
    active_default: bool,
    #[serde(default)]
    active_nested: bool,
    #[serde(default)]
    focus_changed: bool,
    #[serde(default)]
    focus_request: Option<FocusRequest>,
    #[serde(default)]
    moved_focus: Option<MovedFocus>,
    #[serde(default)]
    hosts: Vec<String>,
    #[serde(default)]
    triggers: Vec<String>,
    // Root descriptors belong to Dioxus registrations. Ignore legacy native
    // input even when supplied; unknown DOM cannot enroll itself here.
    #[serde(skip)]
    nodes: Vec<NodeFacts>,
    #[serde(default)]
    focus_candidates: Vec<crate::popover_focus::NativeFocusCandidate>,
    #[serde(skip)]
    document_candidates: Vec<crate::popover_focus::TabCandidate>,
    #[serde(default)]
    active_hosts: Vec<String>,
    #[serde(default)]
    eligible_focus_ids: Vec<String>,
    #[serde(default)]
    active_popover: String,
}
impl From<&Report> for ScopedFocusFacts {
    fn from(report: &Report) -> Self {
        Self {
            observation: FocusObservation::from(report),
            active_hosts: report.active_hosts.clone(),
            eligible_focus_ids: report.eligible_focus_ids.clone(),
            nested: report.active_nested,
        }
    }
}
impl From<&Report> for FocusObservation {
    fn from(report: &Report) -> Self {
        Self {
            candidates: report.focus_candidates.clone(),
            active_id: report.active_id.clone(),
            active_default: report.active_default,
        }
    }
}
impl Report {
    fn project_focus(&mut self) {
        use crate::popover_focus::{project_tab_candidates, tab_order_targets};
        self.document_candidates = project_tab_candidates(&self.focus_candidates, |_| true);
        for node in &mut self.nodes {
            // Physical host IDs distinguish responsive Header surfaces and a
            // Dropdown's forwarded items host from its logical Rust owner.
            // Missing native IDs never group unrelated/unidentified elements.
            let host_candidates = project_tab_candidates(&self.focus_candidates, |candidate| {
                !node.items_host.is_empty()
                    && candidate.nearest_host == node.items_host
                    && candidate.target != node.items_host
            });
            node.focusable_ids = tab_order_targets(&host_candidates)
                .into_iter()
                .filter(|target| !target.is_empty())
                .collect();
            node.tab_candidates = project_tab_candidates(&self.focus_candidates, |candidate| {
                !node.popover_host.is_empty()
                    && candidate.popover == node.popover_host
                    && candidate.target != node.popover_host
            });
        }
    }
}
#[derive(Clone, Deserialize)]
struct MovedFocus {
    key: String,
    focus_id: String,
}
#[derive(Clone, Deserialize)]
struct DocumentEvent {
    sequence: u64,
    #[serde(flatten)]
    action: DocumentAction,
}
impl DocumentEvent {
    fn valid(&self) -> bool {
        self.sequence != 0
            && match &self.action {
                DocumentAction::Input { input_kind, .. } => {
                    matches!(input_kind.as_str(), "keydown" | "mousedown" | "touchstart")
                }
                _ => true,
            }
    }
}
#[derive(Clone, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum DocumentAction {
    Outside {
        #[serde(default)]
        hosts: Vec<String>,
        #[serde(default)]
        triggers: Vec<String>,
        #[serde(default)]
        anchors: Vec<String>,
    },
    Escape,
    Blur,
    Input {
        input_kind: String,
        #[serde(default)]
        key: String,
    },
}
#[derive(Clone, Default)]
struct NodeFacts {
    id: String,
    generation: u64,
    trigger_id: String,
    items_host: String,
    popover_host: String,
    focusable_ids: Vec<String>,
    tab_candidates: Vec<crate::popover_focus::TabCandidate>,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "overlay_domain_runtime_tests.rs"]
mod document_tests;

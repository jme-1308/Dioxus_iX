//! Hierarchical navigation and selection using native Rust state.
//!
//! The 32px rows and indentation follow the upstream iX tree. This renderer uses
//! ARIA levels for a flat visible list, as does the upstream virtualized tree.
use crate::Icon;
use dioxus::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TREE_FOCUS_REQUEST: AtomicU64 = AtomicU64::new(1);

/// Flat, stable-keyed item for [`TreeModel`]. `data` belongs to the application;
/// the ordered child IDs define the hierarchy, independently of map ordering.
/// A branch with `has_children: true` and no children can load them on demand.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeModelItem<T = String> {
    pub id: String,
    pub data: T,
    pub has_children: bool,
    pub children: Vec<String>,
    pub disabled: bool,
    pub loading: bool,
}
impl<T> TreeModelItem<T> {
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Self {
            id: id.into(),
            data,
            has_children: false,
            children: vec![],
            disabled: false,
            loading: false,
        }
    }
    pub fn branch(id: impl Into<String>, data: T, children: Vec<String>) -> Self {
        Self {
            has_children: true,
            children,
            ..Self::new(id, data)
        }
    }
}

/// Per-item expansion, selection and optional interaction override. Model-level
/// disability always wins. Context can be prepared for items not yet visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TreeItemContext {
    pub is_expanded: bool,
    pub is_selected: bool,
    pub is_disabled: bool,
}
pub type TreeContext = BTreeMap<String, TreeItemContext>;

/// Invalid flat models are rejected before a controller publishes any changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeModelError {
    MissingRoot(String),
    DuplicateId(String),
    KeyMismatch(String),
    MissingChild { parent: String, child: String },
    MultipleParents(String),
    Cycle(String),
    LeafWithChildren(String),
    MissingItem(String),
    CannotRemoveRoot,
}
impl std::fmt::Display for TreeModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid tree model: {self:?}")
    }
}
impl std::error::Error for TreeModelError {}

/// Upstream-style flat model with a synthetic root which is never rendered.
/// Detached entries are permitted for staging/lazy loads and remain hidden.
/// Every reference must exist; cycles, duplicate children and multiple parents
/// are rejected even in detached subtrees, so expansion cannot recurse forever.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeModel<T = String> {
    pub root: String,
    pub items: BTreeMap<String, TreeModelItem<T>>,
}
impl<T> TreeModel<T> {
    pub fn new(
        root: impl Into<String>,
        items: impl IntoIterator<Item = TreeModelItem<T>>,
    ) -> Result<Self, TreeModelError> {
        let mut entries = BTreeMap::new();
        for item in items {
            if entries.contains_key(&item.id) {
                return Err(TreeModelError::DuplicateId(item.id));
            }
            entries.insert(item.id.clone(), item);
        }
        let model = Self {
            root: root.into(),
            items: entries,
        };
        model.validate()?;
        Ok(model)
    }
    pub fn validate(&self) -> Result<(), TreeModelError> {
        if !self.items.contains_key(&self.root) {
            return Err(TreeModelError::MissingRoot(self.root.clone()));
        }
        let mut parents = BTreeSet::new();
        for (id, item) in &self.items {
            if id != &item.id {
                return Err(TreeModelError::KeyMismatch(id.clone()));
            }
            if !item.has_children && !item.children.is_empty() {
                return Err(TreeModelError::LeafWithChildren(id.clone()));
            }
            for child in &item.children {
                if !self.items.contains_key(child) {
                    return Err(TreeModelError::MissingChild {
                        parent: id.clone(),
                        child: child.clone(),
                    });
                }
                if !parents.insert(child.clone()) {
                    return Err(TreeModelError::MultipleParents(child.clone()));
                }
            }
        }
        fn visit<T>(
            model: &TreeModel<T>,
            id: &str,
            visiting: &mut BTreeSet<String>,
            complete: &mut BTreeSet<String>,
        ) -> Result<(), TreeModelError> {
            if complete.contains(id) {
                return Ok(());
            }
            if !visiting.insert(id.to_string()) {
                return Err(TreeModelError::Cycle(id.to_string()));
            }
            for child in &model.items[id].children {
                visit(model, child, visiting, complete)?;
            }
            visiting.remove(id);
            complete.insert(id.to_string());
            Ok(())
        }
        let mut complete = BTreeSet::new();
        for id in self.items.keys() {
            visit(self, id, &mut BTreeSet::new(), &mut complete)?;
        }
        if parents.contains(&self.root) {
            return Err(TreeModelError::Cycle(self.root.clone()));
        }
        Ok(())
    }
    /// Insert a leaf/lazy branch or a subtree whose referenced items are already
    /// staged in `items`. The controller's transaction validates the final graph.
    pub fn append(&mut self, parent: &str, item: TreeModelItem<T>) -> Result<(), TreeModelError> {
        if self.items.contains_key(&item.id) {
            return Err(TreeModelError::DuplicateId(item.id));
        }
        let branch = self
            .items
            .get_mut(parent)
            .ok_or_else(|| TreeModelError::MissingItem(parent.into()))?;
        branch.has_children = true;
        branch.children.push(item.id.clone());
        self.items.insert(item.id.clone(), item);
        Ok(())
    }
    /// Remove a complete subtree, including its incoming reference. The parent
    /// becomes a leaf when its last child is removed. Unrelated staged items stay.
    pub fn remove(&mut self, id: &str) -> Result<Vec<TreeModelItem<T>>, TreeModelError> {
        if id == self.root {
            return Err(TreeModelError::CannotRemoveRoot);
        }
        if !self.items.contains_key(id) {
            return Err(TreeModelError::MissingItem(id.into()));
        }
        fn collect<T>(model: &TreeModel<T>, id: &str, ids: &mut Vec<String>) {
            if ids.iter().any(|existing| existing == id) {
                return;
            }
            ids.push(id.into());
            if let Some(item) = model.items.get(id) {
                for child in &item.children {
                    collect(model, child, ids);
                }
            }
        }
        let mut ids = vec![];
        collect(self, id, &mut ids);
        for item in self.items.values_mut() {
            let before = item.children.len();
            item.children.retain(|child| !ids.contains(child));
            if before > 0 && item.children.is_empty() {
                item.has_children = false;
            }
        }
        Ok(ids
            .into_iter()
            .filter_map(|id| self.items.remove(&id))
            .collect())
    }
}

/// One completed model/context transaction or explicit refresh. `removed`
/// contains model items, rather than upstream's detached HTMLElement objects.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeRefreshEvent<T = String> {
    pub revision: u64,
    pub dirty: BTreeSet<String>,
    pub force: bool,
    pub removed: Vec<TreeModelItem<T>>,
    pub context: TreeContext,
    /// Exact typed model snapshot at the time of this notification, including
    /// the hidden root and detached staged entries.
    pub items: Vec<TreeModelItem<T>>,
}
#[derive(Clone, Debug, PartialEq)]
struct TreeStore<T> {
    model: TreeModel<T>,
    context: TreeContext,
    refresh: TreeRefreshEvent<T>,
    pending_refreshes: Vec<TreeRefreshEvent<T>>,
}
fn reconcile_tree_context<T>(model: &TreeModel<T>, context: &mut TreeContext) {
    context.retain(|id, _| model.items.contains_key(id));
    for item in model.items.values() {
        context.entry(item.id.clone()).or_default();
    }
}
impl<T: Clone + PartialEq> TreeStore<T> {
    fn new(model: TreeModel<T>) -> Self {
        let mut context = TreeContext::new();
        reconcile_tree_context(&model, &mut context);
        let items = model.items.values().cloned().collect();
        Self {
            model,
            context: context.clone(),
            refresh: TreeRefreshEvent {
                revision: 0,
                dirty: BTreeSet::new(),
                force: false,
                removed: vec![],
                context,
                items,
            },
            pending_refreshes: vec![],
        }
    }
    fn transact(
        &mut self,
        edit: impl FnOnce(&mut TreeModel<T>) -> Result<(), TreeModelError>,
    ) -> Result<(), TreeModelError> {
        let mut model = self.model.clone();
        edit(&mut model)?;
        model.validate()?;
        let removed = self
            .model
            .items
            .iter()
            .filter(|(id, _)| !model.items.contains_key(*id))
            .map(|(_, item)| item.clone())
            .collect();
        let dirty = model
            .items
            .iter()
            .filter(|(id, item)| self.model.items.get(*id) != Some(*item))
            .map(|(id, _)| id.clone())
            .collect();
        self.model = model;
        reconcile_tree_context(&self.model, &mut self.context);
        self.publish(dirty, false, removed);
        Ok(())
    }
    fn publish(&mut self, dirty: BTreeSet<String>, force: bool, removed: Vec<TreeModelItem<T>>) {
        self.refresh = TreeRefreshEvent {
            revision: self.refresh.revision.wrapping_add(1),
            dirty,
            force,
            removed,
            context: self.context.clone(),
            items: self.model.items.values().cloned().collect(),
        };
        self.pending_refreshes.push(self.refresh.clone());
    }
}

/// Scope-owned reactive handle. Mutations and context changes are synchronous;
/// [`ModelTree`] receives them on the next Dioxus render. Unlike retained DOM
/// renderer updaters, custom Rust renderers receive fresh typed data every render.
pub struct TreeController<T: Clone + PartialEq + 'static = String> {
    store: Signal<TreeStore<T>>,
}
impl<T: Clone + PartialEq + 'static> Copy for TreeController<T> {}
impl<T: Clone + PartialEq + 'static> Clone for TreeController<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: Clone + PartialEq + 'static> PartialEq for TreeController<T> {
    fn eq(&self, other: &Self) -> bool {
        self.store == other.store
    }
}
/// Create a tree handle in the current component scope. Initialize it with a
/// validated model; later replacements and transactions are validated atomically.
pub fn use_tree_model<T: Clone + PartialEq + 'static>(
    model: impl FnOnce() -> TreeModel<T>,
) -> TreeController<T> {
    let store = use_signal(|| {
        let model = model();
        // Public fields support ergonomic bulk construction; reject invalid initial
        // graphs here before the renderer can traverse them.
        model
            .validate()
            .expect("use_tree_model requires a valid TreeModel");
        TreeStore::new(model)
    });
    TreeController { store }
}
impl<T: Clone + PartialEq + 'static> TreeController<T> {
    /// Reactive model snapshot. Call inside a component to subscribe to edits.
    pub fn model(self) -> TreeModel<T> {
        self.store.read().model.clone()
    }
    pub fn context(self) -> TreeContext {
        self.store.read().context.clone()
    }
    pub fn revision(self) -> u64 {
        self.store.read().refresh.revision
    }
    /// Apply a complete edit transaction. Errors leave model, context, revision
    /// and pending refresh notifications untouched.
    pub fn update(
        self,
        edit: impl FnOnce(&mut TreeModel<T>) -> Result<(), TreeModelError>,
    ) -> Result<(), TreeModelError> {
        let mut next = self.store.peek().clone();
        next.transact(edit)?;
        let mut store = self.store;
        store.set(next);
        Ok(())
    }
    pub fn replace(self, model: TreeModel<T>) -> Result<(), TreeModelError> {
        self.update(|current| {
            *current = model;
            Ok(())
        })
    }
    pub fn append(self, parent: &str, item: TreeModelItem<T>) -> Result<(), TreeModelError> {
        self.update(|model| model.append(parent, item))
    }
    pub fn remove(self, id: &str) -> Result<(), TreeModelError> {
        self.update(|model| model.remove(id).map(|_| ()))
    }
    /// Edit typed payload without changing stable identity or hierarchy.
    pub fn update_data(self, id: &str, edit: impl FnOnce(&mut T)) -> Result<(), TreeModelError> {
        self.update(|model| {
            let item = model
                .items
                .get_mut(id)
                .ok_or_else(|| TreeModelError::MissingItem(id.into()))?;
            edit(&mut item.data);
            Ok(())
        })
    }
    pub fn set_context(self, mut context: TreeContext) {
        let mut next = self.store.peek().clone();
        reconcile_tree_context(&next.model, &mut context);
        if next.context == context {
            return;
        }
        next.context = context.clone();
        next.publish(BTreeSet::new(), false, vec![]);
        let mut store = self.store;
        store.set(next);
    }
    /// Notify custom renderers of current data. Dioxus refreshes declarative
    /// content without discarding focused row elements, including `force` refresh.
    pub fn refresh(self, force: bool) {
        self.refresh_items(BTreeSet::new(), force);
    }
    pub fn mark_items_as_dirty(self, ids: impl IntoIterator<Item = String>) {
        self.refresh_items(ids.into_iter().collect(), false);
    }
    fn refresh_items(self, mut dirty: BTreeSet<String>, force: bool) {
        let mut next = self.store.peek().clone();
        dirty.retain(|id| next.model.items.contains_key(id));
        if force {
            dirty.extend(next.model.items.keys().cloned());
        }
        next.publish(dirty, force, vec![]);
        let mut store = self.store;
        store.set(next);
    }
}

/// Data passed to a flat-model custom renderer. `visible_items` includes the
/// complete expanded list, even when only a viewport window is mounted.
#[derive(Clone, PartialEq)]
pub struct TreeRenderContext<T: Clone + PartialEq + 'static = String> {
    pub index: usize,
    pub item: TreeModelItem<T>,
    pub visual: TreeItemData,
    pub visible_items: Vec<TreeModelItem<T>>,
    pub context: TreeContext,
    pub controller: TreeController<T>,
}

fn model_tree_nodes<T: Clone>(
    model: &TreeModel<T>,
    context: &TreeContext,
    label: impl Fn(&T) -> String + Copy,
) -> Vec<TreeNode> {
    fn node<T: Clone>(
        model: &TreeModel<T>,
        context: &TreeContext,
        id: &str,
        label: impl Fn(&T) -> String + Copy,
    ) -> TreeNode {
        let item = &model.items[id];
        TreeNode {
            id: id.into(),
            label: label(&item.data),
            icon: None,
            disabled: item.disabled || context.get(id).is_some_and(|context| context.is_disabled),
            lazy: item.has_children && item.children.is_empty(),
            loading: item.loading,
            children: item
                .children
                .iter()
                .map(|id| node(model, context, id, label))
                .collect(),
        }
    }
    let root = &model.items[&model.root];
    if !root.has_children {
        return vec![];
    }
    root.children
        .iter()
        .map(|id| node(model, context, id, label))
        .collect()
}
fn model_tree_state(context: &TreeContext) -> TreeState {
    TreeState {
        selected: context
            .iter()
            .find(|(_, value)| value.is_selected)
            .map(|(id, _)| id.clone()),
        expanded: context
            .iter()
            .filter(|(_, value)| value.is_expanded)
            .map(|(id, _)| id.clone())
            .collect(),
        checked: BTreeSet::new(),
    }
}

/// Flat-model adapter with the same 32px windowing and keyboard interaction as
/// [`Tree`]. Supply an accessible label for typed payloads using `item_label`.
/// `onrefresh` replaces upstream's imperative retained-element update callbacks;
/// custom content is ordinary Dioxus content and updates reactively.
#[component]
pub fn ModelTree<T: Clone + PartialEq + 'static>(
    controller: TreeController<T>,
    item_label: Callback<T, String>,
    #[props(default)] render_item: Option<Callback<TreeRenderContext<T>, Element>>,
    #[props(default)] onrefresh: EventHandler<TreeRefreshEvent<T>>,
    #[props(default)] oncontextchange: EventHandler<TreeContext>,
    #[props(default)] onselect: EventHandler<String>,
    #[props(default)] onexpand: EventHandler<TreeToggle>,
    #[props(default)] onload: EventHandler<String>,
    #[props(default)] onremove: EventHandler<Vec<TreeModelItem<T>>>,
    #[props(default)] toggle_on_item_click: bool,
    #[props(default = true)] virtualized: bool,
    #[props(default = 4)] overscan: usize,
    #[props(default = 640)] max_height: u32,
    #[props(default = "Tree".to_string())] label: String,
    #[props(default)] class: String,
) -> Element {
    let snapshot = controller.store.read().clone();
    let nodes = model_tree_nodes(&snapshot.model, &snapshot.context, |data| {
        item_label.call(data.clone())
    });
    let state = model_tree_state(&snapshot.context);
    let selected_ids: BTreeSet<_> = snapshot
        .context
        .iter()
        .filter(|(_, context)| context.is_selected)
        .map(|(id, _)| id.clone())
        .collect();
    let mut visual = vec![];
    visible_nodes(&nodes, &state, None, 1, false, &mut visual);
    let visible_items: Vec<_> = visual
        .iter()
        .map(|row| snapshot.model.items[&row.id].clone())
        .collect();
    let renderer_model = snapshot.model.clone();
    let renderer_context = snapshot.context.clone();
    let pending_refreshes = snapshot.pending_refreshes.clone();
    use_effect(use_reactive!(|pending_refreshes| {
        if pending_refreshes.is_empty() {
            return;
        }
        let through = pending_refreshes
            .last()
            .map(|event| event.revision)
            .unwrap_or_default();
        let mut store = controller.store;
        store
            .write()
            .pending_refreshes
            .retain(|event| event.revision > through);
        for refresh in pending_refreshes {
            onrefresh.call(refresh.clone());
            if !refresh.removed.is_empty() {
                onremove.call(refresh.removed);
            }
        }
    }));
    rsx! { Tree {
        nodes, state, selected_ids, refresh_revision: snapshot.refresh.revision, label, class, virtualized, overscan, max_height, toggle_on_item_click,
        onstatechange: move |next: TreeState| {
            let mut context = controller.context();
            for (id, item) in &mut context { item.is_selected = next.selected.as_ref() == Some(id); item.is_expanded = next.expanded.contains(id); }
            controller.set_context(context.clone()); oncontextchange.call(context);
        },
        onselect, onexpand, onload,
        render_item: move |visual: TreeItemData| {
            if let Some(renderer) = render_item {
                let index = visible_items.iter().position(|item| item.id == visual.id).unwrap_or_default();
                renderer.call(TreeRenderContext { index, item: renderer_model.items[&visual.id].clone(), visual, visible_items: visible_items.clone(), context: renderer_context.clone(), controller })
            } else { rsx! { "{visual.label}" } }
        },
    } }
}

/// A stable-keyed tree node. The collection passed to [`Tree`] is the visible root
/// collection; no synthetic root node is rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub children: Vec<TreeNode>,
    pub disabled: bool,
    /// A branch whose children are supplied by the `onload` callback.
    pub lazy: bool,
    /// Set while the application is retrieving children for a lazy branch.
    pub loading: bool,
}
impl TreeNode {
    /// Create an enabled leaf.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: vec![],
            disabled: false,
            lazy: false,
            loading: false,
        }
    }
    /// Create an enabled branch with known children.
    pub fn branch(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeNode>,
    ) -> Self {
        Self {
            children,
            ..Self::new(id, label)
        }
    }
    /// Whether this node has known or lazily supplied child nodes.
    pub fn has_children(&self) -> bool {
        self.lazy || !self.children.is_empty()
    }
}

/// Derived checkbox state. Mixed branches contain both checked and unchecked leaves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeCheckState {
    #[default]
    Unchecked,
    Checked,
    Mixed,
}
impl TreeCheckState {
    fn aria(self) -> &'static str {
        match self {
            Self::Unchecked => "false",
            Self::Checked => "true",
            Self::Mixed => "mixed",
        }
    }
}

/// Controlled tree state, independent of rendering and suitable for serialization
/// by the application. Node IDs must be unique within a tree.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeState {
    pub selected: Option<String>,
    pub expanded: BTreeSet<String>,
    /// Checked node IDs. Use `set_checked` for ancestor reconciliation and cascade.
    pub checked: BTreeSet<String>,
}
impl TreeState {
    /// Derive a branch state from its enabled children. Disabled subtrees keep
    /// their own state and do not prevent a parent from becoming fully checked.
    pub fn check_state(&self, node: &TreeNode) -> TreeCheckState {
        let children: Vec<_> = node
            .children
            .iter()
            .filter(|child| !child.disabled)
            .collect();
        if node.disabled || children.is_empty() {
            return if self.checked.contains(&node.id) {
                TreeCheckState::Checked
            } else {
                TreeCheckState::Unchecked
            };
        }
        let states: Vec<_> = children
            .into_iter()
            .map(|child| self.check_state(child))
            .collect();
        if states.iter().all(|state| *state == TreeCheckState::Checked) {
            TreeCheckState::Checked
        } else if states
            .iter()
            .all(|state| *state == TreeCheckState::Unchecked)
        {
            TreeCheckState::Unchecked
        } else {
            TreeCheckState::Mixed
        }
    }
    /// Select an enabled node. Hidden descendants may be selected by application code.
    pub fn select(&mut self, nodes: &[TreeNode], id: &str) -> bool {
        if find_node(nodes, id, false).is_none_or(|(_, disabled)| disabled) {
            return false;
        }
        let changed = self.selected.as_deref() != Some(id);
        self.selected = Some(id.to_string());
        changed
    }
    /// Expand or collapse an enabled branch, leaving descendant expansion intact.
    pub fn set_expanded(&mut self, nodes: &[TreeNode], id: &str, expanded: bool) -> bool {
        let Some((node, disabled)) = find_node(nodes, id, false) else {
            return false;
        };
        if disabled || !node.has_children() {
            return false;
        }
        if expanded {
            self.expanded.insert(id.to_string())
        } else {
            self.expanded.remove(id)
        }
    }
    /// Check/uncheck a subtree, skipping disabled nodes, then recompute ancestors.
    /// Returns false for missing/disabled nodes or when no check state changed.
    pub fn set_checked(&mut self, nodes: &[TreeNode], id: &str, checked: bool) -> bool {
        let Some((node, disabled)) = find_node(nodes, id, false) else {
            return false;
        };
        if disabled {
            return false;
        }
        let before = self.checked.clone();
        cascade_checked(&mut self.checked, node, checked);
        reconcile_checked(&mut self.checked, nodes);
        before != self.checked
    }
    /// Toggle a subtree. A mixed branch becomes fully checked.
    pub fn toggle_checked(&mut self, nodes: &[TreeNode], id: &str) -> bool {
        let Some((node, disabled)) = find_node(nodes, id, false) else {
            return false;
        };
        if disabled {
            return false;
        }
        self.set_checked(nodes, id, self.check_state(node) != TreeCheckState::Checked)
    }
    /// Expand every enabled branch, preserving disabled subtree state.
    pub fn expand_all(&mut self, nodes: &[TreeNode]) {
        for node in nodes.iter().filter(|node| !node.disabled) {
            if node.has_children() {
                self.expanded.insert(node.id.clone());
            }
            self.expand_all(&node.children);
        }
    }
    /// Collapse every branch.
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }
}
fn find_node<'a>(
    nodes: &'a [TreeNode],
    id: &str,
    inherited_disabled: bool,
) -> Option<(&'a TreeNode, bool)> {
    for node in nodes {
        let disabled = inherited_disabled || node.disabled;
        if node.id == id {
            return Some((node, disabled));
        }
        if let Some(found) = find_node(&node.children, id, disabled) {
            return Some(found);
        }
    }
    None
}
fn cascade_checked(checked: &mut BTreeSet<String>, node: &TreeNode, value: bool) {
    if node.disabled {
        return;
    }
    if value {
        checked.insert(node.id.clone());
    } else {
        checked.remove(&node.id);
    }
    for child in &node.children {
        cascade_checked(checked, child, value);
    }
}
fn reconcile_checked(checked: &mut BTreeSet<String>, nodes: &[TreeNode]) {
    for node in nodes.iter().filter(|node| !node.disabled) {
        reconcile_checked(checked, &node.children);
        let enabled: Vec<_> = node
            .children
            .iter()
            .filter(|child| !child.disabled)
            .collect();
        if !enabled.is_empty() {
            if enabled.iter().all(|child| checked.contains(&child.id)) {
                checked.insert(node.id.clone());
            } else {
                checked.remove(&node.id);
            }
        }
    }
}

/// Expansion event. An expansion with no loaded children also invokes `onload`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeToggle {
    pub id: String,
    pub expanded: bool,
}

/// Read-only row data supplied to an optional custom tree renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeItemData {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub level: usize,
    pub expanded: bool,
    pub selected: bool,
    pub disabled: bool,
    pub checked: TreeCheckState,
}

#[derive(Clone, Debug, PartialEq)]
struct VisibleNode {
    id: String,
    label: String,
    icon: Option<String>,
    parent: Option<String>,
    level: usize,
    position: usize,
    set_size: usize,
    disabled: bool,
    has_children: bool,
    expanded: bool,
    loading: bool,
    lazy: bool,
    check_state: TreeCheckState,
}
fn visible_nodes(
    nodes: &[TreeNode],
    state: &TreeState,
    parent: Option<&str>,
    level: usize,
    inherited_disabled: bool,
    output: &mut Vec<VisibleNode>,
) {
    for (index, node) in nodes.iter().enumerate() {
        let disabled = inherited_disabled || node.disabled;
        let expanded = state.expanded.contains(&node.id);
        output.push(VisibleNode {
            id: node.id.clone(),
            label: node.label.clone(),
            icon: node.icon.clone(),
            parent: parent.map(str::to_string),
            level,
            position: index + 1,
            set_size: nodes.len(),
            disabled,
            has_children: node.has_children(),
            expanded,
            loading: node.loading,
            lazy: node.lazy && node.children.is_empty(),
            check_state: state.check_state(node),
        });
        if expanded {
            visible_nodes(
                &node.children,
                state,
                Some(&node.id),
                level + 1,
                disabled,
                output,
            );
        }
    }
}
const TREE_ROW_HEIGHT: f64 = 32.0;

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
struct TreeViewport {
    top: f64,
    height: f64,
}
impl Default for TreeViewport {
    fn default() -> Self {
        Self {
            top: 0.0,
            height: 640.0,
        }
    }
}

/// The mounted row range includes a small buffer on either side of the viewport.
/// Clamp stale scroll positions after branches collapse or the model shrinks.
fn tree_window(total: usize, viewport: TreeViewport, overscan: usize) -> std::ops::Range<usize> {
    if total == 0 {
        return 0..0;
    }
    let height = if viewport.height.is_finite() && viewport.height > 0.0 {
        viewport.height
    } else {
        TREE_ROW_HEIGHT
    };
    let max_top = (total as f64 * TREE_ROW_HEIGHT - height).max(0.0);
    let top = if viewport.top.is_finite() {
        viewport.top.clamp(0.0, max_top)
    } else {
        0.0
    };
    let first = (top / TREE_ROW_HEIGHT).floor() as usize;
    let last = ((top + height) / TREE_ROW_HEIGHT).ceil() as usize;
    first.saturating_sub(overscan)..last.saturating_add(overscan).min(total)
}
fn reveal_offset(index: usize, viewport: TreeViewport, total: usize) -> f64 {
    let height = viewport.height.max(TREE_ROW_HEIGHT);
    let top = index as f64 * TREE_ROW_HEIGHT;
    let desired = if top < viewport.top {
        top
    } else if top + TREE_ROW_HEIGHT > viewport.top + height {
        top + TREE_ROW_HEIGHT - height
    } else {
        viewport.top
    };
    desired.clamp(0.0, (total as f64 * TREE_ROW_HEIGHT - height).max(0.0))
}

// Event-driven DOM metrics only; Rust derives the window and owns all tree state.
// A hidden tree reports zero height and is measured again when it becomes visible.
const TREE_OBSERVE: &str = r#"
const id = await dioxus.recv(), root = document.getElementById(id);
if (!root) return;
let frame = 0, previous = '', done = () => {};
const cleanup = () => { root.removeEventListener('scroll', schedule); resize.disconnect(); removal.disconnect(); cancelAnimationFrame(frame); done(); };
const measure = () => {
  frame = 0;
  if (!root.isConnected) { cleanup(); return; }
  const value = {top:root.scrollTop,height:root.clientHeight}, json = JSON.stringify(value);
  if (previous !== json) { previous = json; dioxus.send(value); }
};
const schedule = () => { if (!frame) frame = requestAnimationFrame(measure); };
const resize = new ResizeObserver(schedule);
const removal = new MutationObserver(() => { if (!root.isConnected) cleanup(); });
root.addEventListener('scroll', schedule, {passive:true}); resize.observe(root);
removal.observe(document.documentElement, {childList:true,subtree:true});
await new Promise(resolve => { done = resolve; schedule(); });
"#;

fn reveal_tree_row(
    root: String,
    mut focused: Signal<Option<String>>,
    mut viewport: Signal<TreeViewport>,
    index: usize,
    id: String,
    total: usize,
) {
    focused.set(Some(id.clone()));
    let mut next = *viewport.peek();
    next.top = reveal_offset(index, next, total);
    viewport.set(next);
    let sequence = NEXT_TREE_FOCUS_REQUEST.fetch_add(1, Ordering::Relaxed);
    spawn(async move {
        let task = crate::dom_query::eval(
            r#"
const [rootId, id, top, sequence] = await dioxus.recv();
const root = document.getElementById(rootId);
if (!root) return;
// Native VDOM edits cross IPC: one animation frame does not prove that the
// requested virtual row exists. A later request also owns focus immediately.
if ((root.__ixTreePendingFocus?.sequence ?? -1) >= sequence) return;
root.__ixTreePendingFocus?.cancel?.();
const request = {sequence, cancel:null};
root.__ixTreePendingFocus = request;
root.scrollTop = top;
await new Promise(resolve => {
  let observer, timeout, finished = false;
  const cleanup = () => {
    if (finished) return;
    finished = true;
    observer?.disconnect(); clearTimeout(timeout);
    document.removeEventListener('pointerdown', cleanup, true);
    document.removeEventListener('keydown', cancelKey, true);
    document.removeEventListener('focusin', cancelOutsideFocus, true);
    request.cancel = null;
    resolve();
  };
  const cancelKey = event => { if (event.key === 'Tab' || event.key === 'Escape') cleanup(); };
  const cancelOutsideFocus = event => { if (!root.contains(event.target)) cleanup(); };
  request.cancel = cleanup;
  const attempt = () => {
    if (!root.isConnected || root.__ixTreePendingFocus !== request) { cleanup(); return; }
    const node = Array.from(root.querySelectorAll('[data-tree-node-id]')).find(node => node.dataset.treeNodeId === id);
    if (node?.getAttribute('tabindex') !== '0') return;
    root.scrollTop = top;
    node.focus({preventScroll:true});
    cleanup();
  };
  observer = new MutationObserver(attempt);
  observer.observe(document.documentElement, {childList:true,subtree:true,attributes:true,attributeFilter:['tabindex']});
  document.addEventListener('pointerdown', cleanup, true);
  document.addEventListener('keydown', cancelKey, true);
  document.addEventListener('focusin', cancelOutsideFocus, true);
  timeout = setTimeout(cleanup, 2000);
  attempt();
});
"#,
        );
        let _ = task.send((root, id, next.top, sequence));
        let _ = task.await;
    });
}

fn focus_node(
    mut focused: Signal<Option<String>>,
    mounted: Signal<BTreeMap<String, MountedEvent>>,
    id: String,
) {
    focused.set(Some(id.clone()));
    let node = mounted.peek().get(&id).cloned();
    if let Some(node) = node {
        spawn(async move {
            let _ = node.set_focus(true).await;
        });
    }
}
fn publish_tree_state(
    mut local: Signal<TreeState>,
    onchange: EventHandler<TreeState>,
    state: TreeState,
) {
    local.set(state.clone());
    onchange.call(state);
}
fn toggle_branch(
    nodes: &[TreeNode],
    state: &TreeState,
    id: &str,
    local: Signal<TreeState>,
    onchange: EventHandler<TreeState>,
    ontoggle: EventHandler<TreeToggle>,
    onload: EventHandler<String>,
) {
    let mut next = state.clone();
    let expanded = !state.expanded.contains(id);
    if next.set_expanded(nodes, id, expanded) {
        publish_tree_state(local, onchange, next);
        ontoggle.call(TreeToggle {
            id: id.to_string(),
            expanded,
        });
        if expanded
            && find_node(nodes, id, false)
                .is_some_and(|(node, _)| node.lazy && node.children.is_empty() && !node.loading)
        {
            onload.call(id.to_string());
        }
    }
}

/// A hierarchical navigation tree with selectable rows, disclosures and optional
/// cascading checkboxes. Use stable unique IDs; supply `state` for controlled usage.
/// Arrow keys, Home/End, Page Up/Down, Enter/Space and single-letter typeahead
/// operate on the complete visible hierarchy, including rows outside the viewport.
/// Rows use upstream’s fixed 32px height and are windowed by default.
#[component]
pub fn Tree(
    nodes: Vec<TreeNode>,
    /// Changes trigger declarative refresh while preserving stable row elements.
    /// Normally managed by [`ModelTree`]; hierarchical callers may leave it zero.
    #[props(default)]
    refresh_revision: u64,
    /// Optional externally prepared selection flags. User selection remains
    /// single-select and emits one selected ID in `onstatechange`.
    #[props(default)]
    selected_ids: Option<BTreeSet<String>>,
    #[props(default)] state: Option<TreeState>,
    #[props(default)] default_state: TreeState,
    #[props(default)] onstatechange: EventHandler<TreeState>,
    #[props(default)] onselect: EventHandler<String>,
    #[props(default)] onexpand: EventHandler<TreeToggle>,
    #[props(default)] onload: EventHandler<String>,
    /// Replace the label area while retaining row hierarchy and interaction.
    #[props(default)]
    render_item: Option<Callback<TreeItemData, Element>>,
    #[props(default)] checkboxes: bool,
    #[props(default)] toggle_on_item_click: bool,
    /// Keep only viewport rows plus overscan mounted. Disable for applications
    /// that explicitly require the full visible hierarchy in the DOM.
    #[props(default = true)]
    virtualized: bool,
    /// Number of extra rows mounted on either side of the viewport.
    #[props(default = 4)]
    overscan: usize,
    /// Maximum viewport height in pixels. Small trees retain their natural height.
    #[props(default = 640)]
    max_height: u32,
    #[props(default = "Tree".to_string())] label: String,
    #[props(default)] class: String,
) -> Element {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static TREE_ID: AtomicUsize = AtomicUsize::new(1);
    let root_id = use_hook(|| format!("ix-tree-{}", TREE_ID.fetch_add(1, Ordering::Relaxed)));
    let mount_id = root_id.clone();
    let keyboard_root = root_id.clone();
    let mut viewport = use_signal(|| TreeViewport {
        top: 0.0,
        height: f64::from(max_height.max(32)),
    });
    let local = use_signal(|| default_state);
    let snapshot = state.unwrap_or(local());
    let mut focused = use_signal(|| None::<String>);
    let mut has_focus = use_signal(|| false);
    let mut mounted = use_signal(BTreeMap::<String, MountedEvent>::new);
    let mut rows = vec![];
    visible_nodes(&nodes, &snapshot, None, 1, false, &mut rows);
    let active = focused()
        .filter(|id| rows.iter().any(|row| row.id == *id && !row.disabled))
        .or_else(|| {
            // Restore focus to a visible ancestor when a branch/model update hides a row.
            let mut candidate = focused();
            while let Some(id) = candidate {
                if let Some(parent) = parent_id(&nodes, &id, None) {
                    if rows.iter().any(|row| row.id == parent && !row.disabled) {
                        return Some(parent);
                    }
                    candidate = Some(parent);
                } else {
                    break;
                }
            }
            rows.iter()
                .find(|row| !row.disabled)
                .map(|row| row.id.clone())
        });
    let window = if virtualized {
        tree_window(rows.len(), viewport(), overscan)
    } else {
        0..rows.len()
    };
    // After wheel scrolling unmounts the old roving-tabindex row, keep a mounted
    // entry point so Tab can still enter the tree at its current viewport.
    let tab_stop = active
        .as_ref()
        .filter(|id| rows[window.clone()].iter().any(|row| &row.id == *id))
        .cloned()
        .or_else(|| {
            rows[window.clone()]
                .iter()
                .find(|row| !row.disabled)
                .map(|row| row.id.clone())
        });
    let total = rows.len();
    let leading = window.start as f64 * TREE_ROW_HEIGHT;
    let trailing = (total - window.end) as f64 * TREE_ROW_HEIGHT;
    {
        let active = active.clone();
        let rows = rows.clone();
        let root = root_id.clone();
        use_effect(use_reactive!(|active| {
            if has_focus() && focused() != active {
                if let Some(id) = active.clone() {
                    if let Some(index) = rows.iter().position(|row| row.id == id) {
                        reveal_tree_row(root.clone(), focused, viewport, index, id, rows.len());
                    }
                }
            }
        }));
    }
    {
        let visible_ids: BTreeSet<_> = rows[window.clone()]
            .iter()
            .map(|row| row.id.clone())
            .collect();
        use_effect(use_reactive!(|visible_ids| {
            mounted.write().retain(|id, _| visible_ids.contains(id));
        }));
    }
    let callback_nodes = std::rc::Rc::new(nodes.clone());
    let keyboard_rows = rows.clone();
    let keyboard_nodes = nodes.clone();
    let keyboard_state = snapshot.clone();
    let keyboard_active = active.clone();
    rsx! {
        ul { id: root_id, class: "ix-tree {class}", role: "tree", "aria-label": label, "aria-multiselectable": checkboxes,
            style: format!("max-height:{}px", max_height.max(32)),
            "data-virtualized": virtualized, "data-visible-count": total, "data-refresh-revision": refresh_revision,
            onmounted: move |_| {
                let id = mount_id.clone();
                spawn(async move {
                    let mut task = dioxus::document::eval(TREE_OBSERVE);
                    if task.send(id).is_err() { return; }
                    while let Ok(measured) = task.recv::<TreeViewport>().await {
                        if measured.top.is_finite() && measured.height.is_finite() && measured.height > 0.0 && *viewport.peek() != measured { viewport.set(measured); }
                    }
                });
            },
            onfocusin: move |_| has_focus.set(true), onfocusout: move |_| has_focus.set(false),
            onkeydown: move |event: KeyboardEvent| {
                let modifiers = event.modifiers();
                if modifiers.ctrl() || modifiers.meta() || modifiers.alt() { return; }
                let Some(id) = focused().or_else(|| keyboard_active.clone()) else { return; };
                let Some(index) = keyboard_rows.iter().position(|row| row.id == id) else { return; };
                let row = &keyboard_rows[index]; if row.disabled { return; }
                let target = match event.key() {
                    Key::ArrowDown => keyboard_rows.iter().skip(index + 1).find(|row| !row.disabled).map(|row| row.id.clone()),
                    Key::ArrowUp => keyboard_rows[..index].iter().rev().find(|row| !row.disabled).map(|row| row.id.clone()),
                    Key::PageDown | Key::PageUp => {
                        let step = (viewport().height / TREE_ROW_HEIGHT).floor().max(1.0) as usize;
                        let end = if event.key() == Key::PageDown { index.saturating_add(step).min(keyboard_rows.len() - 1) } else { index.saturating_sub(step) };
                        if end >= index { keyboard_rows[..=end].iter().enumerate().rev().find(|(_, row)| !row.disabled).map(|(_, row)| row.id.clone()) }
                        else { keyboard_rows[end..].iter().find(|row| !row.disabled).map(|row| row.id.clone()) }
                    },
                    Key::Home => keyboard_rows.iter().find(|row| !row.disabled).map(|row| row.id.clone()),
                    Key::End => keyboard_rows.iter().rfind(|row| !row.disabled).map(|row| row.id.clone()),
                    Key::ArrowRight => {
                        if row.has_children && !row.expanded { toggle_branch(&keyboard_nodes, &keyboard_state, &id, local, onstatechange, onexpand, onload); None }
                        else { keyboard_rows.iter().skip(index + 1).take_while(|next| next.level > row.level).find(|next| !next.disabled).map(|next| next.id.clone()) }
                    },
                    Key::ArrowLeft => {
                        if row.has_children && row.expanded { toggle_branch(&keyboard_nodes, &keyboard_state, &id, local, onstatechange, onexpand, onload); None } else { row.parent.clone() }
                    },
                    Key::Enter => { let mut next = keyboard_state.clone(); next.select(&keyboard_nodes, &id); publish_tree_state(local, onstatechange, next); onselect.call(id.clone()); None },
                    Key::Character(ref key) if key == " " => {
                        let mut next = keyboard_state.clone();
                        if checkboxes { next.toggle_checked(&keyboard_nodes, &id); }
                        else { next.select(&keyboard_nodes, &id); onselect.call(id.clone()); }
                        publish_tree_state(local, onstatechange, next); None
                    },
                    Key::Character(ref key) if key.chars().count() == 1 && key.chars().all(char::is_alphanumeric) => {
                        let key = key.to_lowercase();
                        keyboard_rows.iter().cycle().skip(index + 1).take(keyboard_rows.len()).find(|next| !next.disabled && next.label.to_lowercase().starts_with(&key)).map(|next| next.id.clone())
                    },
                    _ => return,
                };
                event.prevent_default(); event.stop_propagation();
                if let Some(target) = target {
                    if let Some(index) = keyboard_rows.iter().position(|row| row.id == target) {
                        reveal_tree_row(keyboard_root.clone(), focused, viewport, index, target, keyboard_rows.len());
                    }
                }
            },
            if leading > 0.0 { li { class: "ix-tree-spacer", role: "presentation", "aria-hidden": "true", style: format!("height:{leading}px") } }
            for row in rows.into_iter().skip(window.start).take(window.len()) {
                TreeItem {
                    key: "{row.id}", node_id: row.id.clone(), label: row.label.clone(), icon: if render_item.is_none() { row.icon.clone() } else { None },
                    children: render_item.map(|renderer| renderer.call(TreeItemData { id: row.id.clone(), label: row.label.clone(), icon: row.icon.clone(), level: row.level, expanded: row.expanded, selected: selected_ids.as_ref().map_or(snapshot.selected.as_deref() == Some(&row.id), |ids| ids.contains(&row.id)), disabled: row.disabled, checked: row.check_state })),
                    level: row.level, position: row.position, set_size: row.set_size,
                    has_children: row.has_children, expanded: row.expanded, selected: selected_ids.as_ref().map_or(snapshot.selected.as_deref() == Some(&row.id), |ids| ids.contains(&row.id)),
                    disabled: row.disabled, focusable: tab_stop.as_deref() == Some(&row.id),
                    checked: if checkboxes { Some(row.check_state) } else { None }, loading: row.loading,
                    onmounted: { let id = row.id.clone(); move |event| { mounted.write().insert(id.clone(), event); } },
                    onfocus: { let id = row.id.clone(); move |_| focused.set(Some(id.clone())) },
                    onselect: { let id = row.id.clone(); let snapshot = snapshot.clone(); let nodes = callback_nodes.clone(); move |_| {
                        focus_node(focused, mounted, id.clone());
                        let mut next = snapshot.clone(); next.select(&nodes, &id);
                        if toggle_on_item_click && find_node(&nodes, &id, false).is_some_and(|(node, _)| node.has_children()) { toggle_branch(&nodes, &next, &id, local, onstatechange, onexpand, onload); }
                        else { publish_tree_state(local, onstatechange, next); }
                        onselect.call(id.clone());
                    } },
                    ontoggle: { let id = row.id.clone(); let snapshot = snapshot.clone(); let nodes = callback_nodes.clone(); move |_| { focus_node(focused, mounted, id.clone()); toggle_branch(&nodes, &snapshot, &id, local, onstatechange, onexpand, onload); } },
                    oncheck: { let id = row.id.clone(); let snapshot = snapshot.clone(); let nodes = callback_nodes.clone(); move |_| { focus_node(focused, mounted, id.clone()); let mut next = snapshot.clone(); if next.toggle_checked(&nodes, &id) { publish_tree_state(local, onstatechange, next); } } },
                }
            }
            if trailing > 0.0 { li { class: "ix-tree-spacer", role: "presentation", "aria-hidden": "true", style: format!("height:{trailing}px") } }
        }
    }
}
fn parent_id(nodes: &[TreeNode], id: &str, parent: Option<&str>) -> Option<String> {
    for node in nodes {
        if node.id == id {
            return parent.map(str::to_string);
        }
        if let Some(found) = parent_id(&node.children, id, Some(&node.id)) {
            return Some(found);
        }
    }
    None
}

/// One tree row. Normally created by [`Tree`]; it is also public for custom tree
/// renderers. `level`/`position`/`set_size` are one-based ARIA hierarchy metadata.
#[component]
pub fn TreeItem(
    node_id: String,
    label: String,
    #[props(default)] children: Option<Element>,
    #[props(default)] icon: Option<String>,
    #[props(default = 1)] level: usize,
    #[props(default = 1)] position: usize,
    #[props(default = 1)] set_size: usize,
    #[props(default)] has_children: bool,
    #[props(default)] expanded: bool,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default = true)] focusable: bool,
    #[props(default)] checked: Option<TreeCheckState>,
    #[props(default)] loading: bool,
    #[props(default)] onselect: EventHandler<()>,
    #[props(default)] ontoggle: EventHandler<()>,
    #[props(default)] oncheck: EventHandler<()>,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
    #[props(default)] onfocus: EventHandler<()>,
) -> Element {
    rsx! {
        li {
            class: "ix-tree-item", role: "treeitem", "data-tree-node-id": node_id,
            "aria-level": level, "aria-posinset": position, "aria-setsize": set_size,
            "aria-expanded": if has_children { Some(expanded.to_string()) } else { None },
            "aria-selected": selected, "aria-disabled": disabled, "aria-label": label.clone(),
            "aria-checked": checked.map(TreeCheckState::aria), "aria-busy": loading,
            tabindex: if focusable && !disabled { 0 } else { -1 },
            style: format!("--ix-tree-indent:{}px", level.saturating_sub(1) * 16),
            onmounted: move |event| onmounted.call(event), onfocus: move |_| onfocus.call(()),
            onclick: move |_| if !disabled { onselect.call(()); },
            span { class: "ix-tree-chevron", "data-expanded": expanded, "aria-hidden": "true",
                onclick: move |event| { if has_children { event.stop_propagation(); if !disabled { ontoggle.call(()); } } },
                if has_children { Icon { name: "chevron-right-small", size: 24 } }
            }
            if let Some(checked) = checked {
                span { class: "ix-tree-checkbox", "data-state": checked.aria(), "aria-hidden": "true",
                    onclick: move |event| { event.stop_propagation(); if !disabled { oncheck.call(()); } },
                    if checked == TreeCheckState::Checked { Icon { name: "check", size: 16 } }
                    if checked == TreeCheckState::Mixed { span { class: "ix-tree-checkbox-mixed" } }
                }
            }
            if let Some(icon) = icon { Icon { name: icon, size: 24 } }
            span { class: "ix-tree-label", title: label.clone(), if let Some(children) = children { {children} } else { "{label}" } }
            if loading { span { class: "ix-tree-loading", role: "status", "Loading" } }
        }
    }
}

pub(crate) const STYLES: &str = r#"
.ix-tree{list-style:none;padding:0;margin:0;width:100%;min-width:0;overflow:auto;overflow-anchor:none;max-height:640px;color:var(--ix-color-text);font:inherit}.ix-tree-spacer{padding:0;margin:0;pointer-events:none;list-style:none;flex:none}.ix-tree-item{display:flex;align-items:center;gap:8px;height:32px;min-height:32px;padding:0 16px 0 var(--ix-tree-indent);cursor:pointer;user-select:none;position:relative;white-space:nowrap}.ix-tree-item:hover:not([aria-disabled=true]){background:var(--ix-color-surface-2)}.ix-tree-item[aria-selected=true]{background:var(--ix-color-surface-2)}.ix-tree-item[aria-selected=true]::before{content:"";position:absolute;inset-block:0;left:0;width:2px;background:var(--ix-color-primary)}.ix-tree-item[aria-disabled=true]{opacity:.5;cursor:default}.ix-tree-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-tree-chevron{width:32px;height:32px;min-width:32px;display:inline-flex;align-items:center;justify-content:center}.ix-tree-chevron>.ix-icon{transition:transform .16s ease-out}.ix-tree-chevron[data-expanded=true]>.ix-icon{transform:rotate(90deg)}.ix-tree-label{overflow:hidden;text-overflow:ellipsis;flex:1;min-width:0;line-height:24px}.ix-tree-checkbox{display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;min-width:16px;border:1px solid var(--ix-color-border);border-radius:2px;background:transparent}.ix-tree-checkbox[data-state=true],.ix-tree-checkbox[data-state=mixed]{background:var(--ix-color-primary);border-color:var(--ix-color-primary);color:var(--ix-color-on-primary)}.ix-tree-checkbox-mixed{width:8px;height:2px;background:currentColor}.ix-tree-loading{font-size:12px;line-height:16px;color:var(--ix-color-text-secondary)}
@media(prefers-reduced-motion:reduce){.ix-tree-chevron>.ix-icon{transition:none}}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn flat_sample() -> TreeModel<String> {
        TreeModel::new(
            "root",
            [
                TreeModelItem::branch("root", String::new(), vec!["sample".into()]),
                TreeModelItem::branch("sample", "Sample".into(), vec!["one".into(), "two".into()]),
                TreeModelItem::new("one", "One".into()),
                TreeModelItem::new("two", "Two".into()),
            ],
        )
        .unwrap()
    }
    #[test]
    fn flat_model_rejects_ambiguous_and_recursive_graphs_but_permits_staged_items() {
        let mut model = flat_sample();
        model.items.insert(
            "staged".into(),
            TreeModelItem::new("staged", "Unattached".into()),
        );
        assert!(model.validate().is_ok());
        let context = TreeContext::new();
        let nodes = model_tree_nodes(&model, &context, Clone::clone);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "sample");
        assert!(find_node(&nodes, "root", false).is_none());
        assert!(find_node(&nodes, "staged", false).is_none());
        model.items.get_mut("one").unwrap().has_children = true;
        model
            .items
            .get_mut("one")
            .unwrap()
            .children
            .push("sample".into());
        assert!(matches!(
            model.validate(),
            Err(TreeModelError::MultipleParents(_))
        ));
        model.items.get_mut("root").unwrap().children.clear();
        assert!(matches!(model.validate(), Err(TreeModelError::Cycle(_))));
        model.items.get_mut("one").unwrap().children = vec!["absent".into()];
        assert!(matches!(
            model.validate(),
            Err(TreeModelError::MissingChild { .. })
        ));
        assert!(matches!(
            TreeModel::new(
                "root",
                [TreeModelItem::new("root", 1), TreeModelItem::new("root", 2)]
            ),
            Err(TreeModelError::DuplicateId(_))
        ));
    }
    #[test]
    fn failed_flat_transaction_preserves_state_and_queued_refreshes_atomically() {
        let mut store = TreeStore::new(flat_sample());
        store
            .transact(|model| model.append("sample", TreeModelItem::new("three", "Three".into())))
            .unwrap();
        let before = store.clone();
        let result = store.transact(|model| {
            model.items.get_mut("one").unwrap().data = "Should roll back".into();
            model
                .items
                .get_mut("sample")
                .unwrap()
                .children
                .push("missing".into());
            Ok(())
        });
        assert!(matches!(result, Err(TreeModelError::MissingChild { .. })));
        assert_eq!(store, before);
        assert_eq!(store.refresh.revision, 1);
        assert_eq!(store.pending_refreshes.len(), 1);
    }
    #[test]
    fn flat_mutations_keep_callback_order_and_remove_context_for_deleted_subtrees() {
        let mut store = TreeStore::new(flat_sample());
        store.context.get_mut("sample").unwrap().is_expanded = true;
        store.context.get_mut("one").unwrap().is_selected = true;
        store
            .transact(|model| {
                model.items.get_mut("one").unwrap().data = "Renamed".into();
                Ok(())
            })
            .unwrap();
        store
            .transact(|model| model.remove("sample").map(|_| ()))
            .unwrap();
        assert_eq!(
            store
                .pending_refreshes
                .iter()
                .map(|event| event.revision)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            store.pending_refreshes[0].dirty,
            BTreeSet::from(["one".into()])
        );
        assert!(store.pending_refreshes[0].context["one"].is_selected);
        assert_eq!(
            store.pending_refreshes[1]
                .removed
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["one", "sample", "two"]
        );
        assert_eq!(store.context.len(), 1);
        assert!(!store.context.values().any(|context| context.is_selected));
        assert!(!store.model.items["root"].has_children);
        assert_eq!(
            store.model.remove("root"),
            Err(TreeModelError::CannotRemoveRoot)
        );
    }
    #[test]
    fn flat_replacement_preserves_surviving_context_and_clamps_hidden_focus_target() {
        let mut store = TreeStore::new(flat_sample());
        store.context.get_mut("sample").unwrap().is_expanded = true;
        store.context.get_mut("one").unwrap().is_selected = true;
        store
            .transact(|model| {
                model.items.get_mut("sample").unwrap().children.reverse();
                model.items.get_mut("one").unwrap().data = "Updated".into();
                Ok(())
            })
            .unwrap();
        assert!(store.context["one"].is_selected);
        let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
        let state = model_tree_state(&store.context);
        let mut visible = vec![];
        visible_nodes(&nodes, &state, None, 1, false, &mut visible);
        assert_eq!(
            visible
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            ["sample", "two", "one"]
        );
        store.context.get_mut("sample").unwrap().is_expanded = false;
        let mut collapsed = vec![];
        visible_nodes(
            &nodes,
            &model_tree_state(&store.context),
            None,
            1,
            false,
            &mut collapsed,
        );
        assert_eq!(collapsed.len(), 1);
        assert!(store.context["one"].is_selected);
        assert_eq!(parent_id(&nodes, "one", None).as_deref(), Some("sample"));
    }
    #[test]
    fn flat_lazy_branch_retains_context_through_loading_and_empty_completion() {
        let mut model = flat_sample();
        model
            .append(
                "sample",
                TreeModelItem::branch("remote", "Remote".into(), vec![]),
            )
            .unwrap();
        let mut store = TreeStore::new(model);
        store.context.get_mut("remote").unwrap().is_expanded = true;
        store
            .transact(|model| {
                model.items.get_mut("remote").unwrap().loading = true;
                Ok(())
            })
            .unwrap();
        let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
        let remote = find_node(&nodes, "remote", false).unwrap().0;
        assert!(remote.lazy && remote.loading && remote.has_children());
        store
            .transact(|model| {
                model.append("remote", TreeModelItem::new("loaded", "Loaded".into()))?;
                model.items.get_mut("remote").unwrap().loading = false;
                Ok(())
            })
            .unwrap();
        let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
        let remote = find_node(&nodes, "remote", false).unwrap().0;
        assert!(!remote.lazy && !remote.loading && remote.has_children());
        assert!(store.context["remote"].is_expanded);
        assert!(store.context.contains_key("loaded"));
        store
            .transact(|model| model.remove("loaded").map(|_| ()))
            .unwrap();
        let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
        assert!(!find_node(&nodes, "remote", false).unwrap().0.has_children());
    }
    fn model_tree_fixture() -> Element {
        let controller = use_tree_model(flat_sample);
        use_hook(move || {
            let mut context = controller.context();
            context.get_mut("sample").unwrap().is_expanded = true;
            context.get_mut("one").unwrap().is_selected = true;
            context.get_mut("two").unwrap().is_disabled = true;
            controller.set_context(context);
        });
        rsx! { ModelTree { controller, item_label: |data: String| data, label: "Flat fixture", render_item: |row: TreeRenderContext<String>| rsx! { span { "data-render-index": row.index, "{row.item.data} ({row.visible_items.len()} visible)" } } } }
    }
    #[test]
    fn flat_renderer_exposes_typed_payload_complete_list_context_and_hidden_root() {
        let mut dom = VirtualDom::new(model_tree_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom).replace('"', "");
        assert_eq!(html.matches("role=treeitem").count(), 3);
        assert!(html.contains("aria-label=Flat fixture"));
        assert!(html.contains("One (3 visible)"));
        assert!(html.contains("data-render-index=2"));
        assert!(html.contains("aria-selected=true"));
        assert!(html.contains("aria-disabled=true"));
        assert!(!html.contains("data-tree-node-id=root"));
    }
    #[derive(Default)]
    struct FlatRuntimeCapture {
        controller: Option<TreeController<String>>,
        events: Vec<TreeRefreshEvent<String>>,
        removed: Vec<String>,
    }
    fn flat_runtime_fixture() -> Element {
        let controller = use_tree_model(flat_sample);
        let capture = consume_context::<std::rc::Rc<std::cell::RefCell<FlatRuntimeCapture>>>();
        capture.borrow_mut().controller = Some(controller);
        let removed_capture = capture.clone();
        rsx! { ModelTree { controller, item_label: |data: String| data,
            onrefresh: move |event| capture.borrow_mut().events.push(event),
            onremove: move |items: Vec<TreeModelItem<String>>| removed_capture.borrow_mut().removed.extend(items.into_iter().map(|item| item.id)),
        } }
    }
    #[test]
    fn flat_controller_notifies_every_batched_mutation_once_and_refresh_effect_settles() {
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            use std::{
                future::Future,
                pin::pin,
                task::{Context, Poll, Waker},
            };
            let capture = std::rc::Rc::new(std::cell::RefCell::new(FlatRuntimeCapture::default()));
            let mut dom = VirtualDom::new(flat_runtime_fixture).with_root_context(capture.clone());
            dom.rebuild_in_place();
            let controller = capture.borrow().controller.unwrap();
            dom.in_scope(ScopeId::ROOT, || {
                let mut context = controller.context();
                context.get_mut("sample").unwrap().is_expanded = true;
                controller.set_context(context);
                controller
                    .append("sample", TreeModelItem::new("three", "Three".into()))
                    .unwrap();
                controller
                    .update_data("three", |data| *data = "Renamed three".into())
                    .unwrap();
                controller.mark_items_as_dirty(["three".into(), "missing".into()]);
                controller.refresh(true);
                controller.remove("one").unwrap();
            });
            dom.render_immediate_to_vec();
            let mut context = Context::from_waker(Waker::noop());
            let mut settled = false;
            for _ in 0..8 {
                let work = { pin!(dom.wait_for_work()).poll(&mut context) };
                match work {
                    Poll::Ready(()) => {
                        dom.render_immediate_to_vec();
                    }
                    Poll::Pending => {
                        settled = true;
                        break;
                    }
                }
            }
            assert!(
                settled,
                "refresh effects must settle after draining their notifications"
            );
            let result = capture.borrow();
            assert_eq!(
                result
                    .events
                    .iter()
                    .map(|event| event.revision)
                    .collect::<Vec<_>>(),
                [1, 2, 3, 4, 5, 6]
            );
            assert_eq!(
                result.events[1]
                    .items
                    .iter()
                    .find(|item| item.id == "three")
                    .unwrap()
                    .data,
                "Three"
            );
            assert_eq!(
                result.events[2]
                    .items
                    .iter()
                    .find(|item| item.id == "three")
                    .unwrap()
                    .data,
                "Renamed three"
            );
            assert_eq!(result.events[3].dirty, BTreeSet::from(["three".into()]));
            assert!(result.events[4].force);
            assert_eq!(result.removed, ["one"]);
            assert!(dioxus_ssr::render(&dom).contains("Renamed three"));
            send.send(()).unwrap();
        });
        receive
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect(
                "ModelTree callback queue must terminate and deliver every update exactly once",
            );
    }
    fn sample() -> Vec<TreeNode> {
        vec![TreeNode::branch(
            "sample",
            "Sample",
            vec![
                TreeNode::new("child-1", "Sample Child 1"),
                TreeNode::branch(
                    "child-2",
                    "Sample Child 2",
                    vec![TreeNode::new("grandchild", "Grandchild")],
                ),
            ],
        )]
    }
    #[test]
    fn visible_rows_preserve_hierarchy_and_collapsed_descendant_state() {
        let nodes = sample();
        let mut state = TreeState::default();
        let mut rows = vec![];
        visible_nodes(&nodes, &state, None, 1, false, &mut rows);
        assert_eq!(rows.len(), 1);
        state.set_expanded(&nodes, "sample", true);
        state.set_expanded(&nodes, "child-2", true);
        rows.clear();
        visible_nodes(&nodes, &state, None, 1, false, &mut rows);
        assert_eq!(
            rows.iter()
                .map(|row| (row.id.as_str(), row.level, row.position, row.set_size))
                .collect::<Vec<_>>(),
            vec![
                ("sample", 1, 1, 1),
                ("child-1", 2, 1, 2),
                ("child-2", 2, 2, 2),
                ("grandchild", 3, 1, 1)
            ]
        );
        state.set_expanded(&nodes, "sample", false);
        assert!(state.expanded.contains("child-2"));
        assert_eq!(
            parent_id(&nodes, "grandchild", None),
            Some("child-2".into())
        );
    }
    #[test]
    fn check_cascade_and_ancestor_indeterminate_states_are_consistent() {
        let nodes = sample();
        let mut state = TreeState::default();
        assert!(state.set_checked(&nodes, "child-1", true));
        assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Mixed);
        assert!(!state.checked.contains("sample"));
        state.toggle_checked(&nodes, "sample");
        assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Checked);
        assert_eq!(state.checked.len(), 4);
        state.set_checked(&nodes, "grandchild", false);
        assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Mixed);
        assert!(!state.checked.contains("child-2"));
        state.set_checked(&nodes, "sample", false);
        assert!(state.checked.is_empty());
    }
    #[test]
    fn disabled_subtrees_are_immutable_and_do_not_block_parent_checking() {
        let mut nodes = sample();
        nodes[0].children[1].disabled = true;
        let mut state = TreeState::default();
        state.set_checked(&nodes, "sample", true);
        assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Checked);
        assert!(!state.checked.contains("child-2"));
        assert!(!state.checked.contains("grandchild"));
        assert!(!state.select(&nodes, "grandchild"));
        assert!(!state.set_expanded(&nodes, "child-2", true));
        assert!(!state.set_checked(&nodes, "grandchild", true));
        state.checked.insert("grandchild".into());
        state.set_checked(&nodes, "sample", false);
        assert!(state.checked.contains("grandchild"));
    }
    #[test]
    fn expansion_rejects_leaf_and_missing_nodes_and_handles_lazy_branches() {
        let mut nodes = sample();
        nodes.push(TreeNode {
            lazy: true,
            ..TreeNode::new("lazy", "Lazy branch")
        });
        let mut state = TreeState::default();
        assert!(!state.set_expanded(&nodes, "missing", true));
        assert!(!state.set_expanded(&nodes, "child-1", true));
        assert!(state.set_expanded(&nodes, "lazy", true));
        state.expand_all(&nodes);
        assert_eq!(state.expanded.len(), 3);
        state.collapse_all();
        assert!(state.expanded.is_empty());
    }

    #[test]
    fn window_bounds_dom_work_for_large_scrolls_and_clamped_collapses() {
        let range = tree_window(
            100_000,
            TreeViewport {
                top: 1_600_000.0,
                height: 320.0,
            },
            4,
        );
        assert_eq!(range, 49_996..50_014);
        assert_eq!(range.len(), 18);
        assert_eq!(
            tree_window(
                3,
                TreeViewport {
                    top: 1_600_000.0,
                    height: 320.0
                },
                4
            ),
            0..3
        );
        assert_eq!(tree_window(0, TreeViewport::default(), 4), 0..0);
        assert_eq!(
            tree_window(
                100,
                TreeViewport {
                    top: f64::NAN,
                    height: 320.0
                },
                4
            ),
            0..14
        );
    }

    #[test]
    fn keyboard_reveal_reaches_offscreen_end_without_losing_identity_or_hierarchy() {
        let mut nodes = sample();
        nodes[0]
            .children
            .extend((0..2000).map(|i| TreeNode::new(format!("device-{i}"), format!("Device {i}"))));
        let mut state = TreeState::default();
        state.expand_all(&nodes);
        state.select(&nodes, "device-1999");
        let mut rows = vec![];
        visible_nodes(&nodes, &state, None, 1, false, &mut rows);
        let index = rows.iter().position(|row| row.id == "device-1999").unwrap();
        let mut viewport = TreeViewport {
            top: 0.0,
            height: 320.0,
        };
        assert!(!tree_window(rows.len(), viewport, 4).contains(&index));
        viewport.top = reveal_offset(index, viewport, rows.len());
        assert!(tree_window(rows.len(), viewport, 4).contains(&index));
        assert_eq!(viewport.top + viewport.height, rows.len() as f64 * 32.0);
        assert_eq!(
            (
                rows[index].level,
                rows[index].position,
                rows[index].set_size
            ),
            (2, 2002, 2002)
        );
        state.set_expanded(&nodes, "sample", false);
        assert_eq!(state.selected.as_deref(), Some("device-1999"));
        assert_eq!(
            parent_id(&nodes, "device-1999", None).as_deref(),
            Some("sample")
        );
        assert_eq!(reveal_offset(0, viewport, 1), 0.0);
    }

    fn unwindowed_tree_fixture() -> Element {
        rsx! { Tree { nodes: (0..2000).map(|i| TreeNode::new(format!("device-{i}"), format!("Device {i}"))).collect::<Vec<_>>(), virtualized: false, max_height: 320 } }
    }
    #[test]
    fn consumers_can_explicitly_mount_the_full_visible_tree() {
        let mut dom = VirtualDom::new(unwindowed_tree_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom).replace('"', "");
        assert_eq!(html.matches("role=treeitem").count(), 2000);
        assert!(html.contains("Device 1999"));
        assert!(!html.contains("class=ix-tree-spacer"));
    }

    fn virtual_tree_fixture() -> Element {
        rsx! { Tree { nodes: (0..2000).map(|i| TreeNode::new(format!("device-{i}"), format!("Device {i}"))).collect::<Vec<_>>(), max_height: 320, label: "Large fixture" } }
    }
    #[test]
    fn large_tree_renders_bounded_rows_with_full_sibling_metadata() {
        let mut dom = VirtualDom::new(virtual_tree_fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom).replace('"', "");
        assert_eq!(html.matches("role=treeitem").count(), 14);
        assert!(html.contains("aria-setsize=2000"));
        assert!(html.contains("height:63552px"));
        assert!(!html.contains("Device 1999"));
    }

    fn accessible_tree_fixture() -> Element {
        let nodes = sample();
        let mut state = TreeState::default();
        state.set_expanded(&nodes, "sample", true);
        state.set_checked(&nodes, "child-1", true);
        state.select(&nodes, "child-1");
        rsx! { Tree { nodes, state, checkboxes: true, label: "Fixture tree" } }
    }

    #[test]
    fn rendered_tree_exposes_hierarchy_selection_mixed_checks_and_hidden_descendants() {
        let mut dom = VirtualDom::new(accessible_tree_fixture);
        dom.rebuild_in_place();
        // Dioxus emits valid unquoted HTML for numeric/bool attribute values.
        let html = dioxus_ssr::render(&dom).replace('"', "");
        assert!(html.contains("role=tree"));
        assert!(html.contains("aria-label=Fixture tree"));
        assert_eq!(html.matches("role=treeitem").count(), 3);
        assert!(html.contains("aria-level=2"), "{html}");
        assert!(html.contains("aria-checked=mixed"));
        assert!(html.contains("aria-selected=true"));
        assert!(html.contains("Sample Child 1"));
        assert!(!html.contains("Grandchild"));
    }
}

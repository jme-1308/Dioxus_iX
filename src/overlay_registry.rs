// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Pure overlay hierarchy graph for Header overflow, Dropdown and Popover instances.
//!
//! Tracks two distinct source-derived graphs without DOM or callbacks:
//! 1. Cached submenu assignment and presented parent->children snapshots
//!    (`dropdown.tsx:313-340`, `dropdown-controller.ts:68-149`, `nested-overlay-registry.ts:58-94`).
//!    Popover discovery synchronizes its controller cache even while closed.
//! 2. Live composed-trigger overlay ancestry and presentation order
//!    (`overlay-coordinator.ts:96-119, 188-363`).

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum OverlayKind {
    Header,
    Dropdown,
    Popover,
}

impl OverlayKind {
    /// Header overflow is an actual Dropdown in the source coordinator.
    pub(crate) fn is_dropdown(self) -> bool {
        matches!(self, Self::Header | Self::Dropdown)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum CloseBehavior {
    #[default]
    Both,
    Inside,
    Outside,
    None,
}

impl CloseBehavior {
    pub(crate) fn allows_inside(self) -> bool {
        matches!(self, Self::Both | Self::Inside)
    }

    pub(crate) fn allows_outside(self) -> bool {
        matches!(self, Self::Both | Self::Outside)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) struct NativeFact {
    #[serde(default)]
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) generation: u64,
    #[serde(default)]
    pub(crate) connected: bool,
    #[serde(default)]
    pub(crate) trigger_ancestors: Vec<String>,
    /// Discovery is emitted from a real trigger, while live coordinator
    /// ancestry may come from an anchor. None preserves legacy fact producers.
    #[serde(default)]
    pub(crate) assignment_ancestors: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) discovery_ready: bool,
}

#[derive(Clone, Debug)]
struct NodeEntry {
    generation: u64,
    registration: u64,
    kind: OverlayKind,
    close_behavior: CloseBehavior,
    open: bool,
    connected: bool,
    discovery_ready: bool,
    discovered: bool,
    trigger_ancestors: Vec<String>,
    assignment_ancestors: Option<Vec<String>>,
    assigned_children: Vec<String>,
}

impl NodeEntry {
    fn new(
        generation: u64,
        registration: u64,
        kind: OverlayKind,
        close_behavior: CloseBehavior,
    ) -> Self {
        Self {
            generation,
            registration,
            kind,
            close_behavior,
            open: false,
            connected: false,
            discovery_ready: false,
            discovered: false,
            trigger_ancestors: Vec::new(),
            assignment_ancestors: None,
            assigned_children: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Registry {
    nodes: BTreeMap<String, NodeEntry>,
    order: Vec<String>,
    presentation_order: Vec<String>,
    presented_children: Vec<(String, Vec<String>)>,
    registration: u64,
}

impl Registry {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register(
        &mut self,
        id: String,
        generation: u64,
        kind: OverlayKind,
        close_behavior: CloseBehavior,
    ) {
        if let Some(existing) = self.nodes.get_mut(&id) {
            if generation < existing.generation {
                return;
            }
            if generation == existing.generation {
                existing.kind = kind;
                existing.close_behavior = close_behavior;
                return;
            }
            self.prune_node_references(&id);
            self.order.retain(|k| k != &id);
        }
        self.registration = self.registration.wrapping_add(1);
        self.order.push(id.clone());
        self.nodes.insert(
            id,
            NodeEntry::new(generation, self.registration, kind, close_behavior),
        );
    }

    pub(crate) fn unregister(&mut self, id: &str, generation: u64) {
        if self.generation(id) != Some(generation) {
            return;
        }
        self.nodes.remove(id);
        self.order.retain(|k| k != id);
        self.prune_node_references(id);
    }

    pub(crate) fn observe(&mut self, facts: &[NativeFact]) {
        let mut newly_ready = Vec::new();
        let mut rediscover_all = false;

        for fact in facts {
            if self.generation(&fact.id) != Some(fact.generation) {
                continue;
            }
            if !fact.connected {
                let should_prune = self.nodes.get(&fact.id).is_some_and(|n| {
                    n.connected
                        || n.discovered
                        || !n.assigned_children.is_empty()
                        || !n.trigger_ancestors.is_empty()
                });
                if should_prune {
                    self.prune_hierarchy_references(&fact.id);
                    if let Some(node) = self.nodes.get_mut(&fact.id) {
                        node.connected = false;
                        node.discovery_ready = false;
                        node.discovered = false;
                        node.trigger_ancestors.clear();
                        node.assignment_ancestors = None;
                        node.assigned_children.clear();
                    }
                }
                continue;
            }

            let mut cleaned_ancestors = Vec::with_capacity(fact.trigger_ancestors.len());
            for anc in &fact.trigger_ancestors {
                if anc != &fact.id && !cleaned_ancestors.contains(anc) {
                    cleaned_ancestors.push(anc.clone());
                }
            }
            let assignment_ancestors = fact.assignment_ancestors.as_ref().map(|ancestors| {
                let mut cleaned = Vec::with_capacity(ancestors.len());
                for ancestor in ancestors {
                    if ancestor != &fact.id && !cleaned.contains(ancestor) {
                        cleaned.push(ancestor.clone());
                    }
                }
                cleaned
            });

            if let Some(node) = self.nodes.get_mut(&fact.id) {
                node.connected = true;
                node.discovery_ready = fact.discovery_ready;
                node.trigger_ancestors = cleaned_ancestors;
                node.assignment_ancestors = assignment_ancestors;
                if node.discovery_ready && !node.discovered {
                    if node.kind == OverlayKind::Header {
                        node.discovered = true;
                        rediscover_all = true;
                    } else {
                        newly_ready.push(fact.id.clone());
                    }
                }
            }
        }

        if rediscover_all {
            for id in self.order.clone() {
                if self
                    .nodes
                    .get(&id)
                    .is_some_and(|n| n.connected && n.discovery_ready)
                {
                    self.discover_one(&id);
                }
            }
        } else {
            for id in newly_ready {
                self.discover_one(&id);
            }
        }
    }

    pub(crate) fn set_open(&mut self, id: &str, open: bool) {
        let Some(currently_open) = self.nodes.get(id).map(|n| n.open) else {
            return;
        };
        if open {
            if currently_open {
                return;
            }
            if let Some(node) = self.nodes.get_mut(id) {
                node.open = true;
            }
            self.presentation_order.retain(|k| k != id);
            self.presentation_order.push(id.to_string());

            self.snapshot_children(id);
        } else {
            if let Some(node) = self.nodes.get_mut(id) {
                node.open = false;
            }
            self.presentation_order.retain(|k| k != id);
            self.presented_children.retain(|(parent, _)| parent != id);
        }
    }

    /// Synchronize controller assignments without presenting the surface.
    /// Popover discovery does this while closed; Dropdown does it only when its
    /// controller presents, so property visibility must never call this method.
    pub(crate) fn snapshot_children(&mut self, id: &str) {
        if !self.nodes.contains_key(id) {
            return;
        }
        let assigned = self.assigned_children(id);
        if let Some(node) = self.nodes.get_mut(id) {
            node.assigned_children.clone_from(&assigned);
        }
        let mut snapshot = Vec::with_capacity(assigned.len());
        for child_id in assigned {
            if child_id != id
                && !snapshot.contains(&child_id)
                && !self.presented_reaches(&child_id, id)
            {
                snapshot.push(child_id);
            }
        }
        if let Some((_, existing)) = self
            .presented_children
            .iter_mut()
            .find(|(parent, _)| parent == id)
        {
            *existing = snapshot;
        } else {
            self.presented_children.push((id.to_string(), snapshot));
        }
    }

    /// Source buildPathIncluding contains the target and cached ancestors, not
    /// descendants or siblings. Opening a Popover may therefore dismiss those.
    pub(crate) fn cached_path_including(&self, id: &str) -> Vec<String> {
        if !self.nodes.contains_key(id) {
            return Vec::new();
        }
        let mut path = vec![id.to_string()];
        let mut cursor = 0;
        while cursor < path.len() {
            let current = path[cursor].clone();
            for (parent, children) in &self.presented_children {
                if children.contains(&current) && !path.contains(parent) {
                    path.push(parent.clone());
                }
            }
            cursor += 1;
        }
        path
    }

    /// Dropdown trigger dismissal protects cached ancestors and current live
    /// Dropdown ancestors, including those reached through a Popover parent.
    /// Descendants and siblings remain eligible for the subsequent pass.
    pub(crate) fn dropdown_dismiss_exclusions(&self, id: &str) -> Vec<String> {
        let mut excluded = self.cached_path_including(id);
        for ancestor in self.live_ancestors(id) {
            if self
                .nodes
                .get(&ancestor)
                .is_some_and(|node| node.kind.is_dropdown())
                && !excluded.contains(&ancestor)
            {
                excluded.push(ancestor);
            }
        }
        excluded
    }

    /// Select the next connected-lifetime registration in source Map order.
    /// The caller marks it visited before checking policy, exclusions or open
    /// state, so a later callback cannot revisit an already passed instance.
    pub(crate) fn next_dropdown_dismiss_other(
        &self,
        visited: &BTreeSet<u64>,
    ) -> Option<(String, u64, u64)> {
        self.order.iter().find_map(|id| {
            let node = self.nodes.get(id)?;
            (node.kind.is_dropdown() && !visited.contains(&node.registration))
                .then(|| (id.clone(), node.generation, node.registration))
        })
    }

    /// Property-only visibility does not erase controller child assignments.
    pub(crate) fn set_presence(&mut self, id: &str, open: bool) {
        let Some(node) = self.nodes.get_mut(id) else {
            return;
        };
        let previous = node.open;
        node.open = open;
        if previous != open {
            self.presentation_order.retain(|candidate| candidate != id);
            if open {
                self.presentation_order.push(id.to_string());
            }
        }
    }

    pub(crate) fn forget_children(&mut self, id: &str) {
        self.presented_children.retain(|(parent, _)| parent != id);
    }

    pub(crate) fn set_close_behavior(&mut self, id: &str, close_behavior: CloseBehavior) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.close_behavior = close_behavior;
        }
    }

    pub(crate) fn is_open(&self, id: &str) -> bool {
        self.nodes.get(id).is_some_and(|n| n.open)
    }

    pub(crate) fn is_connected(&self, id: &str) -> bool {
        self.nodes.get(id).is_some_and(|node| node.connected)
    }

    pub(crate) fn generation(&self, id: &str) -> Option<u64> {
        self.nodes.get(id).map(|n| n.generation)
    }

    pub(crate) fn close_behavior(&self, id: &str) -> Option<CloseBehavior> {
        self.nodes.get(id).map(|n| n.close_behavior)
    }

    pub(crate) fn assigned_children(&self, id: &str) -> Vec<String> {
        let Some(node) = self.nodes.get(id) else {
            return Vec::new();
        };
        node.assigned_children
            .iter()
            .filter(|cid| self.nodes.get(*cid).is_some_and(|n| n.connected))
            .cloned()
            .collect()
    }

    pub(crate) fn cached_parent(&self, id: &str) -> Option<String> {
        for (parent, children) in &self.presented_children {
            if children.iter().any(|c| c == id) {
                return Some(parent.clone());
            }
        }
        None
    }

    pub(crate) fn cached_root(&self, id: &str) -> Option<String> {
        if !self.nodes.contains_key(id) {
            return None;
        }
        let mut cur = id.to_string();
        let mut visited = vec![cur.clone()];
        while let Some(parent) = self.cached_parent(&cur) {
            if visited.contains(&parent) {
                break;
            }
            visited.push(parent.clone());
            cur = parent;
        }
        Some(cur)
    }

    pub(crate) fn cached_children(&self, id: &str) -> Vec<String> {
        self.presented_children
            .iter()
            .find(|(parent, _)| parent == id)
            .map(|(_, children)| {
                children
                    .iter()
                    .filter(|cid| self.nodes.contains_key(*cid))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn live_parent(&self, id: &str) -> Option<String> {
        self.parent_matching(id)
    }

    /// Focus uses the live source coordinator chain, including Header overflow.
    pub(crate) fn focus_parent(&self, id: &str) -> Option<String> {
        self.parent_matching(id)
    }

    fn parent_matching(&self, id: &str) -> Option<String> {
        let node = self.nodes.get(id)?;
        if !node.connected {
            return None;
        }
        for ancestor_id in &node.trigger_ancestors {
            if ancestor_id != id
                && let Some(anc) = self.nodes.get(ancestor_id)
                && anc.connected
                && anc.open
            {
                return Some(ancestor_id.clone());
            }
        }
        None
    }

    pub(crate) fn focus_ancestors(&self, id: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut current = self.focus_parent(id);
        while let Some(parent) = current {
            if parent == id || ancestors.contains(&parent) {
                break;
            }
            current = self.focus_parent(&parent);
            ancestors.push(parent);
        }
        ancestors
    }

    pub(crate) fn live_ancestors(&self, id: &str) -> Vec<String> {
        let mut ancestors = Vec::new();
        let mut cur = self.live_parent(id);
        while let Some(parent) = cur {
            if parent == id || ancestors.contains(&parent) {
                break;
            }
            cur = self.live_parent(&parent);
            ancestors.push(parent);
        }
        ancestors
    }

    pub(crate) fn path_in_hierarchy(
        &self,
        parent: &str,
        hosts: &[String],
        triggers: &[String],
    ) -> bool {
        self.nodes.contains_key(parent)
            && (hosts.iter().any(|h| h == parent)
                || triggers.iter().any(|t| t == parent)
                || self.child_path(parent, hosts, triggers))
    }

    pub(crate) fn child_path(&self, parent: &str, hosts: &[String], triggers: &[String]) -> bool {
        if !self.nodes.contains_key(parent) {
            return false;
        }
        for id in &self.order {
            if id == parent {
                continue;
            }
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if !node.connected {
                continue;
            }
            let in_triggers = triggers.iter().any(|t| t == id);
            let in_open_hosts = node.open && hosts.iter().any(|h| h == id);
            let direct_trigger = in_triggers && self.live_parent(id).as_deref() == Some(parent);
            let presented_descendant = node.open
                && (in_triggers || in_open_hosts)
                && self.live_ancestors(id).iter().any(|a| a == parent);
            if direct_trigger || presented_descendant {
                return true;
            }
        }
        false
    }

    pub(crate) fn topmost(&self) -> Option<String> {
        self.presentation_order
            .iter()
            .rev()
            .chain(self.order.iter().rev())
            .find(|id| {
                self.nodes
                    .get(*id)
                    .is_some_and(|node| node.connected && node.open)
            })
            .cloned()
    }

    pub(crate) fn outside_candidates(&self, hosts: &[String], triggers: &[String]) -> Vec<String> {
        let mut presented = Vec::with_capacity(self.presentation_order.len());
        for id in self.presentation_order.iter().chain(&self.order) {
            if self.is_open(id) && !presented.contains(id) {
                presented.push(id.clone());
            }
        }
        presented
            .into_iter()
            .rev()
            .filter(|id| {
                self.close_behavior(id)
                    .is_some_and(CloseBehavior::allows_outside)
                    && !self.path_in_hierarchy(id, hosts, triggers)
            })
            .collect()
    }

    fn discover_one(&mut self, child_id: &str) {
        let Some(ancestors) = self
            .nodes
            .get(child_id)
            .filter(|n| n.connected && n.discovery_ready)
            .map(|n| {
                n.assignment_ancestors
                    .as_ref()
                    .unwrap_or(&n.trigger_ancestors)
                    .clone()
            })
        else {
            return;
        };
        let kind = self.nodes.get(child_id).map(|node| node.kind);
        let unknown_ancestor = ancestors.iter().any(|id| !self.nodes.contains_key(id));
        let nearest = ancestors.into_iter().find(|anc_id| {
            anc_id != child_id
                && self.nodes.get(anc_id).is_some_and(|node| {
                    Some(node.kind) == kind
                        || kind.is_some_and(|kind| kind.is_dropdown() && node.kind.is_dropdown())
                })
        });
        let Some(parent_id) = nearest else {
            // A foreign renderer may have committed its native host before its
            // Rust owner joins. Retry discovery instead of latching no parent.
            if unknown_ancestor {
                return;
            }
            if let Some(child) = self.nodes.get_mut(child_id) {
                child.discovered = true;
            }
            return;
        };
        if !self
            .nodes
            .get(&parent_id)
            .is_some_and(|n| n.connected && n.discovery_ready)
        {
            return;
        }
        if let Some(child) = self.nodes.get_mut(child_id) {
            child.discovered = true;
        }
        if let Some(parent) = self.nodes.get_mut(&parent_id)
            && !parent.assigned_children.iter().any(|c| c == child_id)
        {
            parent.assigned_children.push(child_id.to_string());
            if parent.kind == OverlayKind::Popover {
                // cacheNestedPopoverId immediately syncs, including while the
                // parent is closed; Dropdown deliberately retains its separate
                // controller-presentation snapshot boundary.
                self.snapshot_children(&parent_id);
            }
        }
    }

    fn presented_reaches(&self, start: &str, target: &str) -> bool {
        let mut stack = vec![start];
        let mut visited = Vec::new();
        while let Some(cur) = stack.pop() {
            if cur == target {
                return true;
            }
            if !visited.contains(&cur) {
                visited.push(cur);
                if let Some((_, children)) = self.presented_children.iter().find(|(p, _)| p == cur)
                {
                    stack.extend(children.iter().map(String::as_str));
                }
            }
        }
        false
    }

    fn prune_hierarchy_references(&mut self, id: &str) {
        self.presented_children.retain(|(parent, _)| parent != id);
        for (_, children) in &mut self.presented_children {
            children.retain(|c| c != id);
        }
        for node in self.nodes.values_mut() {
            node.assigned_children.retain(|c| c != id);
        }
    }

    fn prune_node_references(&mut self, id: &str) {
        self.presentation_order.retain(|k| k != id);
        self.prune_hierarchy_references(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(id: &str, generation: u64, ancestors: &[&str], ready: bool) -> NativeFact {
        NativeFact {
            id: id.to_string(),
            generation,
            connected: true,
            trigger_ancestors: ancestors.iter().map(|s| (*s).to_string()).collect(),
            assignment_ancestors: None,
            discovery_ready: ready,
        }
    }

    #[test]
    fn dropdown_trigger_exclusions_keep_cached_and_live_ancestors_through_popover() {
        let mut reg = Registry::new();
        for id in ["old", "active", "sibling", "descendant", "live"] {
            reg.register(id.into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        }
        reg.register(
            "middle".into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
        );
        reg.observe(&[
            fact("old", 1, &[], true),
            fact("active", 1, &["old"], true),
            fact("sibling", 1, &["old"], true),
            fact("descendant", 1, &["active", "old"], true),
            fact("live", 1, &[], true),
            fact("middle", 1, &["live"], true),
        ]);
        for id in ["old", "active", "sibling", "descendant", "live", "middle"] {
            reg.set_open(id, true);
        }
        reg.observe(&[fact("active", 1, &["middle", "live"], true)]);
        assert_eq!(reg.cached_parent("active").as_deref(), Some("old"));
        assert_eq!(reg.live_ancestors("active"), ["middle", "live"]);
        assert_eq!(
            reg.dropdown_dismiss_exclusions("active"),
            ["active", "old", "live"]
        );
    }

    #[test]
    fn dropdown_trigger_iteration_tracks_registration_lifetime_before_policy() {
        let mut reg = Registry::new();
        reg.register(
            "protected".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Inside,
        );
        reg.register(
            "foreign".into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
        );
        reg.register("later".into(), 1, OverlayKind::Header, CloseBehavior::Both);
        let mut visited = BTreeSet::new();
        let first = reg.next_dropdown_dismiss_other(&visited).unwrap();
        assert_eq!((first.0.as_str(), first.1), ("protected", 1));
        visited.insert(first.2);
        reg.set_close_behavior("protected", CloseBehavior::Both);
        reg.register(
            "protected".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        let later = reg.next_dropdown_dismiss_other(&visited).unwrap();
        assert_eq!(later.0, "later");
        visited.insert(later.2);
        assert!(reg.next_dropdown_dismiss_other(&visited).is_none());
        reg.unregister("protected", 1);
        reg.register(
            "protected".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        let reconnected = reg.next_dropdown_dismiss_other(&visited).unwrap();
        assert_eq!((reconnected.0.as_str(), reconnected.1), ("protected", 1));
        assert_ne!(first.2, reconnected.2);
        visited.insert(reconnected.2);
        reg.register("later".into(), 2, OverlayKind::Header, CloseBehavior::Both);
        let replaced = reg.next_dropdown_dismiss_other(&visited).unwrap();
        assert_eq!((replaced.0.as_str(), replaced.1), ("later", 2));
        assert_ne!(later.2, replaced.2);
    }

    #[test]
    fn popover_discovery_syncs_closed_cache_without_changing_presence() {
        let mut reg = Registry::new();
        for id in ["parent", "child"] {
            reg.register(id.into(), 1, OverlayKind::Popover, CloseBehavior::Both);
        }
        reg.observe(&[
            fact("parent", 1, &[], true),
            fact("child", 1, &["parent"], true),
        ]);
        assert!(!reg.is_open("parent") && !reg.is_open("child"));
        assert!(reg.topmost().is_none());
        assert_eq!(reg.cached_children("parent"), ["child"]);
        reg.observe(&[fact("child", 1, &[], true)]);
        assert!(reg.live_ancestors("child").is_empty());
        assert_eq!(reg.cached_parent("child").as_deref(), Some("parent"));
        reg.forget_children("parent");
        assert!(reg.cached_children("parent").is_empty());
        reg.snapshot_children("parent");
        assert_eq!(reg.cached_children("parent"), ["child"]);
        assert!(!reg.is_open("parent"));
    }

    #[test]
    fn cached_path_including_excludes_siblings_and_descendants() {
        let mut reg = Registry::new();
        for id in ["outer", "parent", "sibling", "child"] {
            reg.register(id.into(), 1, OverlayKind::Popover, CloseBehavior::Both);
        }
        reg.observe(&[
            fact("outer", 1, &[], true),
            fact("parent", 1, &["outer"], true),
            fact("sibling", 1, &["outer"], true),
            fact("child", 1, &["parent", "outer"], true),
        ]);
        assert_eq!(reg.cached_path_including("parent"), ["parent", "outer"]);
        assert_eq!(
            reg.cached_path_including("child"),
            ["child", "parent", "outer"]
        );
        assert!(reg.cached_path_including("missing").is_empty());
    }

    #[test]
    fn property_open_does_not_snapshot_controller_children() {
        let mut reg = Registry::new();
        for id in ["parent", "child"] {
            reg.register(id.into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        }
        reg.observe(&[
            fact("parent", 1, &[], true),
            fact("child", 1, &["parent"], true),
        ]);
        reg.set_presence("parent", true);
        assert!(reg.cached_children("parent").is_empty());
        reg.set_presence("parent", false);
        reg.set_open("parent", true);
        assert_eq!(reg.cached_children("parent"), vec!["child"]);
    }

    #[test]
    fn partial_owner_observation_defers_discovery_until_registered_parent_connects() {
        let mut reg = Registry::new();
        for id in ["parent", "child"] {
            reg.register(id.into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        }
        reg.observe(&[fact("child", 1, &["parent"], true)]);
        reg.observe(&[
            fact("parent", 1, &[], true),
            fact("child", 1, &["parent"], true),
        ]);
        reg.set_open("parent", true);
        assert_eq!(reg.cached_children("parent"), vec!["child"]);
    }

    #[test]
    fn property_hidden_parent_retains_cached_assignment_until_controller_cleanup() {
        let mut reg = Registry::new();
        for id in ["parent", "child"] {
            reg.register(id.into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        }
        reg.observe(&[
            fact("parent", 1, &[], true),
            fact("child", 1, &["parent"], true),
        ]);
        reg.set_open("parent", true);
        reg.set_open("child", true);
        reg.set_presence("parent", false);
        assert!(reg.is_open("child"));
        assert_eq!(reg.cached_root("child").as_deref(), Some("parent"));
        assert_eq!(reg.cached_children("parent"), vec!["child"]);
        reg.forget_children("parent");
        assert_eq!(reg.cached_root("child").as_deref(), Some("child"));
    }

    #[test]
    fn cached_discovery_skips_foreign_type_while_live_parent_uses_it() {
        let mut reg = Registry::new();
        reg.register(
            "outer".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.register(
            "middle".into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
        );
        reg.register(
            "inner".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.observe(&[
            fact("outer", 1, &[], true),
            fact("middle", 1, &["outer"], true),
            fact("inner", 1, &["middle", "outer"], true),
        ]);
        for id in ["outer", "middle", "inner"] {
            reg.set_open(id, true);
        }
        assert_eq!(reg.cached_root("inner").as_deref(), Some("outer"));
        assert_eq!(reg.cached_root("middle").as_deref(), Some("middle"));
        assert_eq!(reg.live_parent("inner").as_deref(), Some("middle"));
    }

    #[test]
    fn anchor_only_live_ancestor_does_not_emit_assignment_but_receives_real_children() {
        let mut reg = Registry::new();
        for id in ["root", "category", "child"] {
            reg.register(id.into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        }
        let mut category = fact("category", 1, &["root"], true);
        category.assignment_ancestors = Some(Vec::new());
        let mut child = fact("child", 1, &["category", "root"], true);
        child.assignment_ancestors = Some(vec!["category".into(), "root".into()]);
        reg.observe(&[fact("root", 1, &[], true), category.clone(), child]);
        assert!(reg.assigned_children("root").is_empty());
        assert_eq!(reg.assigned_children("category"), ["child"]);

        for id in ["root", "category", "child"] {
            reg.set_open(id, true);
        }
        assert_eq!(reg.live_parent("category").as_deref(), Some("root"));
        assert_eq!(reg.live_ancestors("child"), ["category", "root"]);
        assert!(reg.cached_children("root").is_empty());
        assert_eq!(reg.cached_children("category"), ["child"]);
        assert_eq!(reg.cached_root("child").as_deref(), Some("category"));

        // A callback/policy refresh is the same lifetime: it keeps the explicit
        // no-emission fact and the accepted controller child snapshot.
        reg.register(
            "category".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Inside,
        );
        reg.observe(&[category.clone()]);
        assert!(reg.assigned_children("root").is_empty());
        assert_eq!(reg.cached_children("category"), ["child"]);

        // Replacement starts a fresh capture namespace. The retired explicit
        // empty list cannot suppress a new legacy producer's real trigger.
        reg.register(
            "category".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        assert!(reg.nodes["category"].assignment_ancestors.is_none());
        reg.observe(&[category]);
        assert!(reg.assigned_children("root").is_empty());
        assert!(reg.cached_children("category").is_empty());
        assert_eq!(reg.cached_parent("child"), None);
        reg.observe(&[fact("category", 2, &["root"], true)]);
        assert_eq!(reg.assigned_children("root"), ["category"]);
        assert!(
            reg.cached_children("root").is_empty(),
            "discovery is not presentation"
        );
        reg.set_open("root", false);
        reg.set_open("root", true);
        assert_eq!(reg.cached_children("root"), ["category"]);
    }

    #[test]
    fn nearest_first_discovery_and_separate_snapshots() {
        let mut reg = Registry::new();
        reg.register("header".into(), 1, OverlayKind::Header, CloseBehavior::Both);
        reg.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.register("sub".into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        reg.observe(&[
            fact("header", 1, &[], true),
            fact("child", 1, &["header"], true),
            fact("sub", 1, &["child", "header"], true),
        ]);
        assert_eq!(reg.assigned_children("header"), vec!["child"]);
        assert_eq!(reg.assigned_children("child"), vec!["sub"]);
        assert!(reg.cached_children("header").is_empty());

        reg.set_open("header", true);
        reg.set_open("child", true);
        reg.set_open("sub", true);
        assert_eq!(reg.cached_children("header"), vec!["child"]);
        assert_eq!(reg.cached_root("sub").as_deref(), Some("header"));

        reg.set_open("child", false);
        assert!(reg.cached_children("child").is_empty());
        assert_eq!(reg.cached_children("header"), vec!["child"]);
    }

    #[test]
    fn header_discovery_gated_until_ready_and_rediscovers_all() {
        let mut reg = Registry::new();
        reg.register("header".into(), 1, OverlayKind::Header, CloseBehavior::Both);
        reg.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.observe(&[fact("header", 1, &[], false), fact("child", 1, &[], true)]);
        assert!(reg.assigned_children("header").is_empty());

        reg.observe(&[
            fact("header", 1, &[], true),
            fact("child", 1, &["header"], true),
        ]);
        assert_eq!(reg.assigned_children("header"), vec!["child"]);
    }

    #[test]
    fn md_to_sm_projection_preserves_cached_root_vs_live_parent_split() {
        let mut reg = Registry::new();
        reg.register("header".into(), 1, OverlayKind::Header, CloseBehavior::Both);
        reg.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.register("sub".into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        reg.observe(&[
            fact("header", 1, &[], true),
            fact("child", 1, &[], true),
            fact("sub", 1, &["child"], true),
        ]);
        reg.observe(&[
            fact("header", 1, &[], true),
            fact("child", 1, &["header"], true),
            fact("sub", 1, &["child", "header"], true),
        ]);
        reg.set_open("header", true);
        reg.set_open("child", true);
        reg.set_open("sub", true);
        assert_eq!(reg.cached_root("sub").as_deref(), Some("child"));
        assert_eq!(reg.live_parent("child").as_deref(), Some("header"));
        assert_eq!(reg.focus_parent("child").as_deref(), Some("header"));
        assert_eq!(reg.live_ancestors("sub"), vec!["child", "header"]);
        assert_eq!(reg.focus_ancestors("sub"), vec!["child", "header"]);
        assert!(reg.path_in_hierarchy("header", &["sub".into()], &[]));
        assert!(reg.child_path("header", &[], &["child".into()]));
    }

    #[test]
    fn outside_candidates_newest_first_policy_and_more_click_quirk() {
        let mut reg = Registry::new();
        reg.register("header".into(), 1, OverlayKind::Header, CloseBehavior::Both);
        reg.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.register(
            "sub".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Outside,
        );
        reg.register(
            "inside_only".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Inside,
        );
        reg.observe(&[
            fact("header", 1, &[], true),
            fact("child", 1, &["header"], true),
            fact("sub", 1, &["child"], true),
            fact("inside_only", 1, &[], true),
        ]);
        reg.set_open("header", true);
        reg.set_open("child", true);
        reg.set_open("sub", true);
        reg.set_open("inside_only", true);
        reg.set_open("header", true);
        assert_eq!(
            reg.topmost().as_deref(),
            Some("inside_only"),
            "topmost ignores click policy and unchanged opens"
        );
        assert_eq!(
            reg.outside_candidates(&[], &[]),
            vec!["sub", "child", "header"]
        );
        assert_eq!(
            reg.outside_candidates(&[], &["header".into()]),
            vec!["sub", "child"]
        );
        assert_eq!(reg.outside_candidates(&["child".into()], &[]), vec!["sub"]);
    }

    #[test]
    fn generations_reconnect_cycles_and_stale_pruning() {
        let mut reg = Registry::new();
        reg.register("a".into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        reg.register("b".into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        reg.observe(&[fact("a", 1, &["b"], true), fact("b", 1, &["a"], true)]);
        reg.set_open("a", true);
        reg.set_open("b", true);
        assert_eq!(reg.cached_children("a"), vec!["b"]);
        assert!(reg.cached_children("b").is_empty());
        assert_eq!(reg.cached_root("b").as_deref(), Some("a"));
        assert_eq!(reg.live_ancestors("a"), vec!["b"]);

        reg.observe(&[NativeFact {
            id: "b".into(),
            generation: 1,
            connected: false,
            trigger_ancestors: Vec::new(),
            assignment_ancestors: None,
            discovery_ready: false,
        }]);
        assert!(reg.cached_children("a").is_empty());
        assert!(reg.assigned_children("a").is_empty());
        reg.observe(&[fact("b", 1, &["a"], true)]);
        assert_eq!(reg.assigned_children("a"), vec!["b"]);

        reg.register("b".into(), 2, OverlayKind::Dropdown, CloseBehavior::Both);
        reg.unregister("b", 1);
        reg.observe(&[fact("b", 1, &["a"], true)]);
        assert!(!reg.is_open("b"));
        assert!(reg.assigned_children("a").is_empty());
    }
}

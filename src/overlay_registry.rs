// SPDX-FileCopyrightText: 2026 Siemens AG
// SPDX-License-Identifier: MIT

//! Pure overlay hierarchy graph for Header overflow, Dropdown and Popover instances.
//!
//! Latest connected, ready trigger facts determine logical same-kind parents.
//! Dropdown and Header are compatible; cross-kind live focus ancestry remains
//! a separate relation. Visibility changes never freeze a hierarchy snapshot.

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
    /// Explicit trigger ancestry determines logical membership; live focus
    /// ancestry may also come from an anchor. None uses current trigger ancestry,
    /// while Some([]) explicitly declares no logical parent.
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
    trigger_ancestors: Vec<String>,
    assignment_ancestors: Option<Vec<String>>,
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
            trigger_ancestors: Vec::new(),
            assignment_ancestors: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Registry {
    nodes: BTreeMap<String, NodeEntry>,
    order: Vec<String>,
    presentation_order: Vec<String>,
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
        for fact in facts {
            let Some(node) = self
                .nodes
                .get_mut(&fact.id)
                .filter(|node| node.generation == fact.generation)
            else {
                continue;
            };
            node.connected = fact.connected;
            node.discovery_ready = fact.connected && fact.discovery_ready;
            let clean = |ancestors: &[String]| {
                let mut current = Vec::new();
                for ancestor in ancestors {
                    if ancestor != &fact.id && !current.contains(ancestor) {
                        current.push(ancestor.clone());
                    }
                }
                current
            };
            if fact.connected {
                node.trigger_ancestors = clean(&fact.trigger_ancestors);
                node.assignment_ancestors = fact
                    .assignment_ancestors
                    .as_ref()
                    .map(|ancestors| clean(ancestors));
            } else {
                node.trigger_ancestors.clear();
                node.assignment_ancestors = None;
            }
        }
    }

    pub(crate) fn set_open(&mut self, id: &str, open: bool) {
        self.set_presence(id, open);
    }

    /// Historical name retained for clients: this is the current logical path,
    /// including the target and ancestors, never siblings or descendants.
    pub(crate) fn cached_path_including(&self, id: &str) -> Vec<String> {
        if !self.nodes.contains_key(id) {
            return Vec::new();
        }
        let parents = self.logical_parents();
        let mut path = vec![id.to_string()];
        while let Some(parent) = parents.get(path.last().unwrap()) {
            if path.contains(parent) {
                break;
            }
            path.push(parent.clone());
        }
        path
    }

    /// Dropdown trigger dismissal protects current logical and live Dropdown
    /// ancestors, including those reached through a Popover parent.
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

    /// Select the next registration lifetime in current registration order.
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

    /// Visibility only changes presentation order, independently of hierarchy.
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

    /// Historical compatibility names now query only current ready relations.
    pub(crate) fn cached_parent(&self, id: &str) -> Option<String> {
        self.logical_parents().get(id).cloned()
    }

    pub(crate) fn cached_root(&self, id: &str) -> Option<String> {
        self.cached_path_including(id).last().cloned()
    }

    pub(crate) fn cached_children(&self, id: &str) -> Vec<String> {
        let parents = self.logical_parents();
        self.order
            .iter()
            .filter(|child| parents.get(*child).is_some_and(|parent| parent == id))
            .cloned()
            .collect()
    }

    pub(crate) fn live_parent(&self, id: &str) -> Option<String> {
        self.parent_matching(id)
    }

    /// Focus follows the current open cross-kind ancestry, including Header.
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

    fn logical_parents(&self) -> BTreeMap<String, String> {
        let mut parents = BTreeMap::new();
        for (id, node) in &self.nodes {
            if !node.connected || !node.discovery_ready {
                continue;
            }
            for ancestor in node
                .assignment_ancestors
                .as_ref()
                .unwrap_or(&node.trigger_ancestors)
            {
                // An unknown nearer owner cannot be classified yet. Retain the
                // ID in the latest fact so its later admission needs no replay.
                let Some(parent) = self.nodes.get(ancestor) else {
                    break;
                };
                if parent.kind == node.kind || parent.kind.is_dropdown() && node.kind.is_dropdown()
                {
                    if parent.connected && parent.discovery_ready {
                        parents.insert(id.clone(), ancestor.clone());
                    }
                    break;
                }
            }
        }
        // Malformed mutually enclosing facts must still produce a forest.
        // The earliest registered member of a cycle becomes its root.
        for id in &self.order {
            let mut path = Vec::new();
            let mut current = id.clone();
            while let Some(parent) = parents.get(&current).cloned() {
                if let Some(at) = path.iter().position(|seen| seen == &current) {
                    let root = path[at..]
                        .iter()
                        .min_by_key(|id| self.nodes[*id].registration)
                        .cloned()
                        .unwrap();
                    parents.remove(&root);
                    break;
                }
                path.push(current);
                current = parent;
            }
        }
        parents
    }

    fn prune_node_references(&mut self, id: &str) {
        self.presentation_order.retain(|candidate| candidate != id);
        // An ID is not a lifetime. A different owner may still retain its old
        // facts when this parent is removed or replaced; require an explicit
        // fresh relationship instead of adopting the replacement automatically.
        for node in self.nodes.values_mut() {
            node.trigger_ancestors.retain(|ancestor| ancestor != id);
            if let Some(ancestors) = &mut node.assignment_ancestors {
                ancestors.retain(|ancestor| ancestor != id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fact(id: &str, generation: u64, ancestors: &[&str], ready: bool) -> NativeFact {
        NativeFact {
            id: id.into(),
            generation,
            connected: true,
            trigger_ancestors: ancestors.iter().map(|id| (*id).into()).collect(),
            assignment_ancestors: None,
            discovery_ready: ready,
        }
    }
    fn dropdowns(ids: &[&str]) -> Registry {
        let mut reg = Registry::new();
        for id in ids {
            reg.register((*id).into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        }
        reg
    }

    #[test]
    fn current_children_admit_after_parent_open_and_follow_rebind_and_detach() {
        let mut reg = dropdowns(&["a", "b", "child", "peer"]);
        reg.observe(&[
            fact("a", 1, &[], true),
            fact("b", 1, &[], true),
            fact("peer", 1, &["a"], true),
        ]);
        reg.set_open("a", true);
        reg.observe(&[fact("child", 1, &["a"], true)]);
        assert_eq!(reg.cached_children("a"), ["child", "peer"]);
        assert_eq!(reg.cached_parent("child").as_deref(), Some("a"));
        reg.observe(&[fact("child", 1, &["b"], true)]);
        assert_eq!(reg.cached_children("a"), ["peer"]);
        assert_eq!(reg.cached_children("b"), ["child"]);
        reg.observe(&[fact("child", 1, &[], true)]);
        assert!(reg.cached_children("b").is_empty());
        assert_eq!(reg.cached_root("child").as_deref(), Some("child"));
    }

    #[test]
    fn current_nearest_parent_readiness_and_unknown_owner_need_no_child_replay() {
        let mut reg = dropdowns(&["outer", "child"]);
        reg.observe(&[
            fact("outer", 1, &[], true),
            fact("child", 1, &["late", "outer"], true),
        ]);
        assert_eq!(reg.cached_parent("child"), None);
        reg.register("late".into(), 1, OverlayKind::Dropdown, CloseBehavior::Both);
        reg.observe(&[fact("late", 1, &["outer"], false)]);
        assert_eq!(reg.cached_parent("child"), None);
        reg.observe(&[fact("late", 1, &["outer"], true)]);
        assert_eq!(
            reg.cached_path_including("child"),
            ["child", "late", "outer"]
        );
        reg.observe(&[fact("child", 1, &["late", "outer"], false)]);
        assert!(reg.cached_children("late").is_empty());
        reg.observe(&[fact("child", 1, &["late", "outer"], true)]);
        assert_eq!(reg.cached_children("late"), ["child"]);
        reg.observe(&[fact("late", 1, &["outer"], false)]);
        assert_eq!(reg.cached_parent("child"), None);
    }

    #[test]
    fn generation_disconnect_and_registration_changes_reject_stale_facts() {
        let mut reg = dropdowns(&["parent", "child"]);
        reg.observe(&[
            fact("parent", 1, &[], true),
            fact("child", 1, &["parent"], true),
        ]);
        reg.register(
            "child".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.unregister("child", 1);
        reg.observe(&[fact("child", 1, &["parent"], true)]);
        assert!(reg.cached_children("parent").is_empty());
        reg.observe(&[fact("child", 2, &["parent"], true)]);
        assert_eq!(reg.cached_children("parent"), ["child"]);
        let mut disconnected = fact("child", 2, &["parent"], true);
        disconnected.connected = false;
        reg.observe(&[disconnected]);
        assert!(reg.cached_children("parent").is_empty());
        reg.observe(&[fact("child", 2, &["parent"], true)]);
        reg.unregister("parent", 1);
        assert_eq!(reg.cached_parent("child"), None);
        reg.register(
            "parent".into(),
            2,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
        );
        reg.observe(&[fact("parent", 1, &[], true)]);
        assert_eq!(reg.cached_parent("child"), None);
        reg.observe(&[fact("parent", 2, &[], true)]);
        assert_eq!(reg.cached_parent("child"), None);
        reg.observe(&[fact("child", 2, &["parent"], true)]);
        assert_eq!(reg.cached_parent("child").as_deref(), Some("parent"));
    }

    #[test]
    fn header_layout_moves_update_logical_parent_and_cross_kind_focus_stays_distinct() {
        let mut reg = dropdowns(&["child", "sub"]);
        reg.register("header".into(), 1, OverlayKind::Header, CloseBehavior::Both);
        reg.register(
            "popover".into(),
            1,
            OverlayKind::Popover,
            CloseBehavior::Both,
        );
        reg.observe(&[
            fact("header", 1, &[], true),
            fact("popover", 1, &["header"], true),
            fact("child", 1, &[], true),
            fact("sub", 1, &["child"], true),
        ]);
        for id in ["header", "popover", "child", "sub"] {
            reg.set_open(id, true);
        }
        reg.observe(&[
            fact("child", 1, &["popover", "header"], true),
            fact("sub", 1, &["child", "popover", "header"], true),
        ]);
        assert_eq!(reg.cached_root("sub").as_deref(), Some("header"));
        assert_eq!(reg.cached_parent("child").as_deref(), Some("header"));
        assert_eq!(reg.focus_parent("child").as_deref(), Some("popover"));
        assert_eq!(reg.live_ancestors("sub"), ["child", "popover", "header"]);
        assert!(reg.path_in_hierarchy("header", &["sub".into()], &[]));
        reg.observe(&[fact("child", 1, &[], true)]);
        assert_eq!(reg.cached_root("sub").as_deref(), Some("child"));
    }

    #[test]
    fn explicit_empty_logical_ancestry_keeps_anchor_parentless_but_accepts_its_children() {
        let mut reg = dropdowns(&["root", "anchor", "child"]);
        let mut anchor = fact("anchor", 1, &["root"], true);
        anchor.assignment_ancestors = Some(Vec::new());
        reg.observe(&[
            fact("root", 1, &[], true),
            anchor.clone(),
            fact("child", 1, &["anchor", "root"], true),
        ]);
        for id in ["root", "anchor", "child"] {
            reg.set_open(id, true);
        }
        assert!(reg.cached_children("root").is_empty());
        assert_eq!(reg.cached_children("anchor"), ["child"]);
        assert_eq!(reg.cached_root("child").as_deref(), Some("anchor"));
        assert_eq!(reg.live_parent("anchor").as_deref(), Some("root"));
        reg.register(
            "anchor".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Inside,
        );
        assert_eq!(reg.cached_parent("anchor"), None);
        reg.observe(&[fact("anchor", 1, &["root"], true)]);
        assert_eq!(reg.cached_parent("anchor").as_deref(), Some("root"));
    }

    #[test]
    fn current_graph_deduplicates_ancestry_and_cuts_cycles_at_first_registration() {
        let mut reg = dropdowns(&["a", "b", "c"]);
        reg.observe(&[
            fact("a", 1, &["a", "b", "b"], true),
            fact("b", 1, &["c"], true),
            fact("c", 1, &["a"], true),
        ]);
        assert_eq!(reg.nodes["a"].trigger_ancestors, ["b"]);
        assert_eq!(reg.cached_parent("a"), None);
        assert_eq!(reg.cached_root("b").as_deref(), Some("a"));
        assert_eq!(reg.cached_children("a"), ["c"]);
        assert_eq!(reg.cached_path_including("b"), ["b", "c", "a"]);
        reg.observe(&[fact("a", 1, &[], true)]);
        assert_eq!(reg.cached_path_including("b"), ["b", "c", "a"]);
    }

    #[test]
    fn current_popover_children_do_not_depend_on_open_presence_or_policy() {
        let mut reg = Registry::new();
        for id in ["parent", "child"] {
            reg.register(id.into(), 1, OverlayKind::Popover, CloseBehavior::Both);
        }
        reg.observe(&[
            fact("parent", 1, &[], true),
            fact("child", 1, &["parent"], true),
        ]);
        assert_eq!(reg.cached_children("parent"), ["child"]);
        assert!(reg.topmost().is_none());
        reg.set_presence("parent", true);
        reg.set_open("child", true);
        reg.set_presence("parent", false);
        reg.set_close_behavior("parent", CloseBehavior::None);
        assert!(reg.is_open("child"));
        assert_eq!(reg.cached_parent("child").as_deref(), Some("parent"));
        reg.observe(&[fact("child", 1, &[], true)]);
        assert!(reg.cached_children("parent").is_empty());
    }

    #[test]
    fn current_paths_exclude_siblings_and_descendants() {
        let mut reg = dropdowns(&["outer", "parent", "sibling", "child"]);
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
    fn dropdown_exclusions_follow_current_logical_and_live_cross_kind_ancestors() {
        let mut reg = dropdowns(&["old", "active", "sibling", "descendant", "live"]);
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
        assert_eq!(reg.cached_parent("active").as_deref(), Some("live"));
        assert_eq!(reg.live_ancestors("active"), ["middle", "live"]);
        assert_eq!(
            reg.dropdown_dismiss_exclusions("active"),
            ["active", "live"]
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
    fn topmost_follows_presentation_order_independent_of_click_policy() {
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
    }

    #[test]
    fn disconnected_visibility_is_not_topmost_and_policy_refresh_preserves_current_facts() {
        let mut reg = dropdowns(&["parent", "child"]);
        reg.observe(&[
            fact("parent", 1, &[], true),
            fact("child", 1, &["parent"], true),
        ]);
        reg.set_open("parent", true);
        reg.set_open("child", true);
        reg.register(
            "child".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Outside,
        );
        assert_eq!(reg.cached_parent("child").as_deref(), Some("parent"));
        assert_eq!(reg.close_behavior("child"), Some(CloseBehavior::Outside));
        let mut disconnected = fact("child", 1, &[], false);
        disconnected.connected = false;
        reg.observe(&[disconnected]);
        assert_eq!(reg.topmost().as_deref(), Some("parent"));
        assert!(!reg.is_connected("child"));
    }
}

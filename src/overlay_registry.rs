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
mod tests;

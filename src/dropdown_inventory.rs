//! Source-host inventory, navigation and activation.
//!
//! Dioxus registers current metadata, overlay ownership and explicit sibling/
//! containing-item order. Rust owns membership, eligibility, navigation and
//! activation. Historical slot algorithms remain pure helpers.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ItemKind {
    Dropdown,
    Select,
    Menu,
    Category,
    NativeOptIn,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub(crate) struct ItemRef(pub(crate) String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InventoryMode {
    ActiveDescendant,
    RovingTabindex,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HostPartKind {
    SelectOption,
    CategoryParent,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostPartFacts {
    pub(crate) kind: HostPartKind,
    pub(crate) capture: String,
    pub(crate) source_focusable: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostRootFact {
    pub(crate) id: String,
    pub(crate) generation: u64,
    pub(crate) is_host: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct HostFacts {
    pub(crate) item: ItemRef,
    pub(crate) kind: ItemKind,
    pub(crate) target: String,
    pub(crate) capture: String,
    #[serde(default)]
    pub(crate) root_path: Vec<HostRootFact>,
    #[serde(default)]
    pub(crate) registration_key: String,
    #[serde(default = "default_focusable")]
    pub(crate) source_focusable: bool,
    #[serde(default)]
    pub(crate) disabled_attribute: bool,
    #[serde(default)]
    pub(crate) hidden_attribute: bool,
    #[serde(default)]
    pub(crate) tabindex_attribute: Option<String>,
    #[serde(default)]
    pub(crate) checked_attribute: bool,
    #[serde(default)]
    pub(crate) visible_focus: bool,
    #[serde(default)]
    pub(crate) roving_opt_in: bool,
    #[serde(default)]
    pub(crate) linked_trigger: Option<String>,
    #[serde(default)]
    pub(crate) parts: Vec<HostPartFacts>,
}
fn default_focusable() -> bool {
    true
}

impl HostFacts {
    pub(crate) fn eligible(&self, mode: InventoryMode) -> bool {
        if self.disabled_attribute || self.hidden_attribute {
            return false;
        }
        match mode {
            InventoryMode::ActiveDescendant => {
                self.kind != ItemKind::NativeOptIn
                    && !self
                        .tabindex_attribute
                        .as_deref()
                        .is_some_and(|value| value.starts_with('-'))
            }
            InventoryMode::RovingTabindex => {
                self.kind != ItemKind::NativeOptIn || self.roving_opt_in
            }
        }
    }

    fn active(&self, mode: InventoryMode) -> bool {
        self.eligible(mode)
            && match mode {
                InventoryMode::ActiveDescendant => self.visible_focus,
                InventoryMode::RovingTabindex => self.tabindex_attribute.as_deref() == Some("0"),
            }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct InventoryFacts {
    pub(crate) items_host: ItemRef,
    pub(crate) hosts: Vec<HostFacts>,
    /// Callback collection: assigned-slot matches, then light descendants.
    pub(crate) navigation_order: Vec<ItemRef>,
    /// queryElements collection: explicit-slot insertion in document order.
    pub(crate) query_order: Vec<ItemRef>,
    /// When present, orders above contain raw direct descendants. Rust expands
    /// assigned roots according to the current mode's selector eligibility.
    #[serde(default)]
    pub(crate) slots: Option<Vec<SlotFacts>>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct SlotAssignment {
    pub(crate) root: Option<ItemRef>,
    pub(crate) descendants: Vec<ItemRef>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct SlotFacts {
    pub(crate) assignments: Vec<SlotAssignment>,
    pub(crate) following: Vec<ItemRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Activation {
    Submenu {
        item: ItemRef,
        trigger: String,
    },
    /// Source calls the configured activation callback even without an active
    /// element. It can prevent default while having no host to click.
    SyntheticHost {
        item: Option<ItemRef>,
    },
    NativeDefault,
    None,
}

pub(crate) struct Inventory {
    mode: InventoryMode,
    hosts: BTreeMap<ItemRef, HostFacts>,
    navigation_order: Vec<ItemRef>,
    query_order: Vec<ItemRef>,
    active_order: Vec<ItemRef>,
}

impl Inventory {
    pub(crate) fn new(mode: InventoryMode, facts: InventoryFacts) -> Self {
        let mut hosts = BTreeMap::new();
        for host in facts.hosts {
            hosts.entry(host.item.clone()).or_insert(host);
        }
        let filter = |order: Vec<ItemRef>| {
            order
                .into_iter()
                .filter(|item| hosts.get(item).is_some_and(|host| host.eligible(mode)))
                .collect()
        };
        let filter_active = |order: Vec<ItemRef>| {
            order
                .into_iter()
                .filter(|item| hosts.get(item).is_some_and(|host| host.active(mode)))
                .collect()
        };
        // Source queryElements runs the active selector independently. An
        // eligible but inactive assigned root must expose active descendants.
        let mut active_order: Vec<ItemRef> = filter_active(facts.query_order.clone());
        let mut navigation_order: Vec<ItemRef> = filter(facts.navigation_order);
        let mut query_order: Vec<ItemRef> = filter(facts.query_order);
        if let Some(slots) = facts.slots {
            let expand = |slot: &SlotFacts| {
                slot.assignments
                    .iter()
                    .flat_map(|assignment| {
                        if let Some(root) = &assignment.root
                            && hosts.get(root).is_some_and(|host| host.eligible(mode))
                        {
                            vec![root.clone()]
                        } else {
                            filter(assignment.descendants.clone())
                        }
                    })
                    .collect::<Vec<_>>()
            };
            let expand_active = |slot: &SlotFacts| {
                slot.assignments
                    .iter()
                    .flat_map(|assignment| {
                        if let Some(root) = &assignment.root
                            && hosts.get(root).is_some_and(|host| host.active(mode))
                        {
                            vec![root.clone()]
                        } else {
                            filter_active(assignment.descendants.clone())
                        }
                    })
                    .collect::<Vec<_>>()
            };
            navigation_order = slots
                .iter()
                .flat_map(&expand)
                .chain(navigation_order)
                .collect();
            for slot in slots.iter().rev() {
                let assigned = expand(slot);
                let at = query_order
                    .iter()
                    .position(|item| slot.following.contains(item))
                    .unwrap_or(query_order.len());
                query_order.splice(at..at, assigned);
                let active = expand_active(slot);
                let at = active_order
                    .iter()
                    .position(|item| slot.following.contains(item))
                    .unwrap_or(active_order.len());
                active_order.splice(at..at, active);
            }
        }
        Self {
            mode,
            hosts,
            navigation_order,
            query_order,
            active_order,
        }
    }

    pub(crate) fn host(&self, item: &ItemRef) -> Option<&HostFacts> {
        self.hosts.get(item)
    }

    #[cfg(test)]
    pub(crate) fn navigation_order(&self) -> &[ItemRef] {
        &self.navigation_order
    }

    #[cfg(test)]
    pub(crate) fn query_order(&self) -> &[ItemRef] {
        &self.query_order
    }

    pub(crate) fn active(&self) -> Option<ItemRef> {
        self.active_order.first().cloned()
    }

    /// Current is a registered Rust key supplied by a typed event handler or
    /// `active()`. A physical origin can be eligible without a painted cue.
    /// Repeated references in the supplied navigation order remain repeated.
    pub(crate) fn navigate(&self, key: &str, current: Option<&ItemRef>) -> Option<ItemRef> {
        let order = &self.navigation_order;
        if order.is_empty() {
            return None;
        }
        let current = current
            .filter(|item| {
                self.hosts
                    .get(*item)
                    .is_some_and(|host| host.eligible(self.mode))
            })
            .and_then(|item| order.iter().position(|candidate| candidate == item));
        let index = match key {
            "Home" => 0,
            "End" => order.len() - 1,
            "ArrowDown" => current.map_or(0, |index| (index + 1) % order.len()),
            "ArrowUp" => current.map_or(order.len() - 1, |index| {
                (index + order.len() - 1) % order.len()
            }),
            _ => return None,
        };
        order.get(index).cloned()
    }

    /// Candidate edge for roving initialization, using query order. This is not
    /// the broader focusFirst/LastDescendant helper used by AD opening.
    #[cfg(test)]
    pub(crate) fn edge(&self, last: bool, focus_checked: bool) -> Option<ItemRef> {
        if focus_checked
            && let Some(item) = self
                .query_order
                .iter()
                .find(|item| self.hosts[*item].checked_attribute)
        {
            return Some(item.clone());
        }
        if last {
            self.query_order.last().cloned()
        } else {
            self.query_order.first().cloned()
        }
    }

    /// Typed handlers may name a current registered origin independently of
    /// the active-marker discovery used when no physical origin is supplied.
    pub(crate) fn activation(
        &self,
        key: &str,
        item_trigger_keys: &[String],
        current: Option<&ItemRef>,
    ) -> Activation {
        let active = current
            .and_then(|item| self.hosts.get(item))
            .filter(|host| host.eligible(self.mode));
        if matches!(key, "ArrowRight" | "Enter" | " ")
            && let Some(host) = active
            && let Some(trigger) = &host.linked_trigger
        {
            return Activation::Submenu {
                item: host.item.clone(),
                trigger: trigger.clone(),
            };
        }
        if !item_trigger_keys.iter().any(|candidate| candidate == key) {
            return Activation::None;
        }
        if active.is_some_and(|host| host.kind == ItemKind::NativeOptIn && host.roving_opt_in) {
            Activation::NativeDefault
        } else {
            Activation::SyntheticHost {
                item: active.map(|host| host.item.clone()),
            }
        }
    }

    pub(crate) fn query_hosts(&self) -> Vec<HostFacts> {
        self.query_order
            .iter()
            .map(|item| self.hosts[item].clone())
            .collect()
    }
}

#[cfg(test)]
mod tests;

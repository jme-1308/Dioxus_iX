//! Source-host inventory, navigation and activation.
//!
//! DOM plumbing supplies the two source query orders and actual-object tokens.
//! Rust owns selector eligibility, navigation and activation.
//! Descriptive registration and current inventory membership are independent:
//! no nearest-overlay or provider-ownership filter belongs in this module.

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

    /// Use the captured active host, not an inferred focused shadow descendant.
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
                    .is_some_and(|host| host.active(self.mode))
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

    pub(crate) fn activation(
        &self,
        key: &str,
        item_trigger_keys: &[String],
        current: Option<&ItemRef>,
    ) -> Activation {
        let active = current
            .and_then(|item| self.hosts.get(item))
            .filter(|host| host.active(self.mode));
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
mod tests {
    use super::*;

    fn item(id: &str, kind: ItemKind, tabindex: Option<&str>) -> HostFacts {
        HostFacts {
            root_path: Vec::new(),
            item: ItemRef(id.into()),
            kind,
            target: format!("public-{id}"),
            capture: id.into(),
            registration_key: format!("key-{id}"),
            source_focusable: true,
            disabled_attribute: false,
            hidden_attribute: false,
            tabindex_attribute: tabindex.map(str::to_string),
            checked_attribute: false,
            visible_focus: false,
            roving_opt_in: kind == ItemKind::NativeOptIn,
            linked_trigger: None,
            parts: Vec::new(),
        }
    }
    fn inventory(
        mode: InventoryMode,
        hosts: Vec<HostFacts>,
        navigation: &[&str],
        query: &[&str],
    ) -> Inventory {
        Inventory::new(
            mode,
            InventoryFacts {
                items_host: ItemRef("actual-host".into()),
                hosts,
                navigation_order: navigation.iter().map(|id| ItemRef((*id).into())).collect(),
                query_order: query.iter().map(|id| ItemRef((*id).into())).collect(),
                slots: None,
            },
        )
    }

    #[test]
    fn literal_eligibility_distinguishes_all_four_ix_hosts_and_native_opt_ins() {
        for kind in [
            ItemKind::Dropdown,
            ItemKind::Select,
            ItemKind::Menu,
            ItemKind::Category,
        ] {
            for (attribute, ad) in [
                (None, true),
                (Some("0"), true),
                (Some("7"), true),
                (Some("00"), true),
                (Some("-1"), false),
                (Some("-invalid"), false),
                (Some(" -1"), true),
            ] {
                let mut host = item("host", kind, attribute);
                assert_eq!(host.eligible(InventoryMode::ActiveDescendant), ad);
                assert!(host.eligible(InventoryMode::RovingTabindex));
                host.disabled_attribute = true;
                assert!(!host.eligible(InventoryMode::ActiveDescendant));
                assert!(!host.eligible(InventoryMode::RovingTabindex));
                host.disabled_attribute = false;
                host.hidden_attribute = true;
                assert!(!host.eligible(InventoryMode::RovingTabindex));
            }
        }
        let mut native = item("native", ItemKind::NativeOptIn, Some("-1"));
        assert!(!native.eligible(InventoryMode::ActiveDescendant));
        assert!(native.eligible(InventoryMode::RovingTabindex));
        native.roving_opt_in = false;
        assert!(!native.eligible(InventoryMode::RovingTabindex));
    }

    #[test]
    fn navigation_and_active_lookup_keep_their_source_orders_and_duplicate_hosts() {
        let mut direct = item("direct", ItemKind::Menu, None);
        direct.visible_focus = true;
        let mut slot = item("slot", ItemKind::Category, None);
        slot.visible_focus = true;
        let inventory = inventory(
            InventoryMode::ActiveDescendant,
            vec![direct, slot],
            &["slot", "direct", "slot", "unknown"],
            &["direct", "slot"],
        );
        assert_eq!(inventory.active(), Some(ItemRef("direct".into())));
        assert_eq!(
            inventory.navigation_order(),
            [
                ItemRef("slot".into()),
                ItemRef("direct".into()),
                ItemRef("slot".into())
            ]
        );
        assert_eq!(
            inventory.query_order(),
            [ItemRef("direct".into()), ItemRef("slot".into())]
        );
        assert_eq!(
            inventory.navigate("ArrowDown", inventory.active().as_ref()),
            Some(ItemRef("slot".into()))
        );
        assert_eq!(
            inventory.navigate("ArrowDown", Some(&ItemRef("slot".into()))),
            Some(ItemRef("direct".into()))
        );
        assert_eq!(
            inventory.navigate("ArrowUp", None),
            Some(ItemRef("slot".into()))
        );
        assert_eq!(
            inventory.navigate("Home", None),
            Some(ItemRef("slot".into()))
        );
        assert_eq!(inventory.edge(false, false), Some(ItemRef("direct".into())));
        assert_eq!(
            inventory.host(&ItemRef("slot".into())).unwrap().kind,
            ItemKind::Category
        );
    }

    #[test]
    fn active_requires_exact_host_marker_and_raw_roving_zero_not_shadow_focus_or_numeric_zero() {
        let mut first = item("first", ItemKind::Dropdown, Some("00"));
        first.visible_focus = true;
        let second = item("second", ItemKind::Select, Some("0"));
        let roving = inventory(
            InventoryMode::RovingTabindex,
            vec![first.clone(), second.clone()],
            &["first", "second"],
            &["first", "second"],
        );
        assert_eq!(roving.active(), Some(second.item.clone()));
        assert_eq!(
            roving.navigate("ArrowDown", Some(&first.item)),
            Some(first.item.clone())
        );
        first.visible_focus = false;
        let ad = inventory(
            InventoryMode::ActiveDescendant,
            vec![first, second],
            &["first", "second"],
            &["first", "second"],
        );
        assert_eq!(ad.active(), None);
        assert_eq!(ad.navigate("ArrowUp", None), Some(ItemRef("second".into())));
        assert_eq!(ad.navigate("unhandled", None), None);
        let empty = inventory(InventoryMode::ActiveDescendant, vec![], &[], &[]);
        assert_eq!(empty.navigate("Home", None), None);
    }

    #[test]
    fn activation_distinguishes_native_default_ix_host_submenu_and_absent_active_callback() {
        let keys = ["ArrowRight", "Enter", " "].map(str::to_string);
        for kind in [
            ItemKind::Dropdown,
            ItemKind::Select,
            ItemKind::Menu,
            ItemKind::Category,
            ItemKind::NativeOptIn,
        ] {
            let mut host = item("host", kind, Some("0"));
            let plain = inventory(
                InventoryMode::RovingTabindex,
                vec![host.clone()],
                &["host"],
                &["host"],
            );
            let expected = if kind == ItemKind::NativeOptIn {
                Activation::NativeDefault
            } else {
                Activation::SyntheticHost {
                    item: Some(host.item.clone()),
                }
            };
            assert_eq!(
                plain.activation("Enter", &keys, plain.active().as_ref()),
                expected
            );
            assert_eq!(
                plain.activation(" ", &keys, plain.active().as_ref()),
                expected
            );
            assert_eq!(
                plain.activation("F2", &keys, plain.active().as_ref()),
                Activation::None
            );
            host.linked_trigger = Some("submenu-root".into());
            let linked = inventory(
                InventoryMode::RovingTabindex,
                vec![host.clone()],
                &["host"],
                &["host"],
            );
            assert_eq!(
                linked.activation("ArrowRight", &[], linked.active().as_ref()),
                Activation::Submenu {
                    item: host.item,
                    trigger: "submenu-root".into()
                }
            );
        }
        let empty = inventory(InventoryMode::ActiveDescendant, vec![], &[], &[]);
        assert_eq!(
            empty.activation("Enter", &keys, None),
            Activation::SyntheticHost { item: None }
        );
    }

    #[test]
    fn raw_slot_branching_uses_mode_eligibility_before_expanding_assigned_descendants() {
        let a = item("a", ItemKind::Dropdown, None);
        let b = item("b", ItemKind::Dropdown, None);
        let native = item("native-assigned-root", ItemKind::NativeOptIn, Some("-1"));
        let leaf = item("leaf", ItemKind::Menu, None);
        let facts = InventoryFacts {
            items_host: ItemRef("host".into()),
            hosts: vec![a, b, native, leaf],
            navigation_order: vec![ItemRef("a".into()), ItemRef("b".into())],
            query_order: vec![ItemRef("a".into()), ItemRef("b".into())],
            slots: Some(vec![SlotFacts {
                assignments: vec![SlotAssignment {
                    root: Some(ItemRef("native-assigned-root".into())),
                    descendants: vec![ItemRef("leaf".into())],
                }],
                following: vec![ItemRef("b".into())],
            }]),
        };
        let ad = Inventory::new(InventoryMode::ActiveDescendant, facts.clone());
        assert_eq!(
            ad.navigation_order(),
            [
                ItemRef("leaf".into()),
                ItemRef("a".into()),
                ItemRef("b".into())
            ]
        );
        assert_eq!(
            ad.query_order(),
            [
                ItemRef("a".into()),
                ItemRef("leaf".into()),
                ItemRef("b".into())
            ]
        );
        let roving = Inventory::new(InventoryMode::RovingTabindex, facts);
        assert_eq!(
            roving.navigation_order(),
            [
                ItemRef("native-assigned-root".into()),
                ItemRef("a".into()),
                ItemRef("b".into())
            ]
        );
        assert_eq!(
            roving.query_order(),
            [
                ItemRef("a".into()),
                ItemRef("native-assigned-root".into()),
                ItemRef("b".into())
            ]
        );
    }

    fn refs(ids: &[&str]) -> Vec<ItemRef> {
        ids.iter().map(|id| ItemRef((*id).into())).collect()
    }

    fn assigned_active_child(mode: InventoryMode) -> InventoryFacts {
        let root = item(
            "root",
            ItemKind::Dropdown,
            match mode {
                InventoryMode::ActiveDescendant => None,
                InventoryMode::RovingTabindex => Some("-1"),
            },
        );
        let mut child = item("child", ItemKind::Select, Some("0"));
        child.visible_focus = true;
        InventoryFacts {
            items_host: ItemRef("actual-items-host".into()),
            hosts: vec![
                item("before", ItemKind::Menu, None),
                root,
                child,
                item("after", ItemKind::Dropdown, None),
            ],
            navigation_order: refs(&["before", "after"]),
            query_order: refs(&["before", "after"]),
            slots: Some(vec![SlotFacts {
                assignments: vec![SlotAssignment {
                    root: Some(ItemRef("root".into())),
                    descendants: refs(&["child"]),
                }],
                following: refs(&["after"]),
            }]),
        }
    }

    #[test]
    fn active_slot_query_exposes_child_for_activation_without_flattening_navigation() {
        for mode in [
            InventoryMode::ActiveDescendant,
            InventoryMode::RovingTabindex,
        ] {
            let inventory = Inventory::new(mode, assigned_active_child(mode));
            let active = inventory.active();
            assert_eq!(active, Some(ItemRef("child".into())));
            assert_eq!(
                inventory.query_order(),
                refs(&["before", "root", "after"]).as_slice()
            );
            assert_eq!(
                inventory.navigation_order(),
                refs(&["root", "before", "after"]).as_slice()
            );
            assert_eq!(
                inventory.activation("Enter", &["Enter".into()], active.as_ref()),
                Activation::SyntheticHost {
                    item: Some(ItemRef("child".into()))
                },
            );
            // The active child is absent from the separate callback query.
            // Source's findIndex returns -1, so Down starts at its first root.
            assert_eq!(
                inventory.navigate("ArrowDown", active.as_ref()),
                Some(ItemRef("root".into()))
            );
        }
    }

    #[test]
    fn active_slot_root_short_circuit_and_own_eligibility_are_query_specific() {
        for mode in [
            InventoryMode::ActiveDescendant,
            InventoryMode::RovingTabindex,
        ] {
            for condition in ["active", "disabled", "hidden", "negative"] {
                let mut facts = assigned_active_child(mode);
                let root = facts
                    .hosts
                    .iter_mut()
                    .find(|host| host.item.0 == "root")
                    .unwrap();
                root.visible_focus = true;
                root.tabindex_attribute = Some("0".into());
                match condition {
                    "disabled" => root.disabled_attribute = true,
                    "hidden" => root.hidden_attribute = true,
                    "negative" => root.tabindex_attribute = Some("-invalid".into()),
                    _ => {}
                }
                let base_root = root.eligible(mode);
                let inventory = Inventory::new(mode, facts);
                assert_eq!(
                    inventory.active(),
                    Some(ItemRef(if condition == "active" {
                        "root".into()
                    } else {
                        "child".into()
                    })),
                    "{mode:?} {condition}"
                );
                assert_eq!(
                    inventory.query_order(),
                    refs(&["before", if base_root { "root" } else { "child" }, "after",])
                        .as_slice(),
                    "{mode:?} {condition}"
                );
            }
            for condition in ["disabled", "hidden", "negative"] {
                let mut facts = assigned_active_child(mode);
                let child = facts
                    .hosts
                    .iter_mut()
                    .find(|host| host.item.0 == "child")
                    .unwrap();
                match condition {
                    "disabled" => child.disabled_attribute = true,
                    "hidden" => child.hidden_attribute = true,
                    _ => child.tabindex_attribute = Some("-1".into()),
                }
                let inventory = Inventory::new(mode, facts);
                assert!(inventory.active().is_none(), "{mode:?} {condition}");
            }
        }
    }

    #[test]
    fn independent_active_slot_splicing_preserves_direct_order_and_duplicate_objects() {
        for mode in [
            InventoryMode::ActiveDescendant,
            InventoryMode::RovingTabindex,
        ] {
            let mut hosts = vec![
                item("a", ItemKind::Menu, Some("0")),
                item("r", ItemKind::Dropdown, None),
                item("x", ItemKind::Select, Some("0")),
                item("s", ItemKind::Dropdown, None),
                item("y", ItemKind::Select, Some("0")),
                item("z", ItemKind::Menu, None),
            ];
            for host in &mut hosts {
                host.visible_focus = matches!(host.item.0.as_str(), "a" | "x" | "y");
            }
            let inventory = Inventory::new(
                mode,
                InventoryFacts {
                    items_host: ItemRef("actual-host".into()),
                    hosts,
                    navigation_order: refs(&["a", "r", "x", "s", "y", "z"]),
                    query_order: refs(&["a", "r", "x", "s", "y", "z"]),
                    slots: Some(vec![
                        SlotFacts {
                            assignments: vec![SlotAssignment {
                                root: Some(ItemRef("r".into())),
                                descendants: refs(&["x"]),
                            }],
                            following: refs(&["r", "x", "s", "y", "z"]),
                        },
                        SlotFacts {
                            assignments: vec![SlotAssignment {
                                root: Some(ItemRef("s".into())),
                                descendants: refs(&["y"]),
                            }],
                            following: refs(&["s", "y", "z"]),
                        },
                    ]),
                },
            );
            assert_eq!(
                inventory.navigation_order(),
                refs(&["r", "s", "a", "r", "x", "s", "y", "z"]).as_slice()
            );
            assert_eq!(
                inventory.query_order(),
                refs(&["a", "r", "r", "x", "s", "s", "y", "z"]).as_slice()
            );
            assert_eq!(inventory.active_order, refs(&["a", "x", "x", "y", "y"]));
            assert_eq!(inventory.active(), Some(ItemRef("a".into())));
            assert_eq!(
                inventory.navigate("ArrowDown", inventory.active().as_ref()),
                Some(ItemRef("r".into()))
            );
        }
    }
}

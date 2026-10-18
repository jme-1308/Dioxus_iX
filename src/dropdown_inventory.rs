//! Source-host inventory and exact roving attribute baselines.
//!
//! DOM plumbing supplies the two source query orders and actual-object tokens.
//! Rust owns selector eligibility, navigation, activation and attribute writes.
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
pub(crate) struct HostFacts {
    pub(crate) item: ItemRef,
    pub(crate) kind: ItemKind,
    pub(crate) target: String,
    pub(crate) capture: String,
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
    pub(crate) items_host: ItemRef,
    hosts: BTreeMap<ItemRef, HostFacts>,
    navigation_order: Vec<ItemRef>,
    query_order: Vec<ItemRef>,
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
            }
        }
        Self {
            mode,
            items_host: facts.items_host,
            hosts,
            navigation_order,
            query_order,
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
        self.query_order
            .iter()
            .find(|item| self.hosts[*item].active(self.mode))
            .cloned()
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct AttributeWrite {
    pub(crate) item: ItemRef,
    pub(crate) tabindex: Option<String>,
}

#[derive(Default)]
pub(crate) struct RovingState {
    // Preserve Map insertion order as well as each exact first attribute value.
    // One actual itemsHost shares a baseline even when multiple roots use it.
    hosts: Vec<(ItemRef, Vec<AttributeWrite>)>,
}

impl RovingState {
    pub(crate) fn retains_token(&self, token: &str) -> bool {
        self.hosts.iter().any(|(host, writes)| {
            host.0 == token || writes.iter().any(|write| write.item.0 == token)
        })
    }
    fn remember(&mut self, host: &ItemRef, candidates: &[&HostFacts]) {
        let index = self.hosts.iter().position(|(item, _)| item == host);
        let baseline = if let Some(index) = index {
            &mut self.hosts[index].1
        } else {
            self.hosts.push((host.clone(), Vec::new()));
            &mut self.hosts.last_mut().unwrap().1
        };
        for candidate in candidates {
            if !baseline.iter().any(|write| write.item == candidate.item) {
                baseline.push(AttributeWrite {
                    item: candidate.item.clone(),
                    tabindex: candidate.tabindex_attribute.clone(),
                });
            }
        }
    }

    pub(crate) fn select(
        &mut self,
        host: &ItemRef,
        query_candidates: &[HostFacts],
        target: &ItemRef,
    ) -> Vec<AttributeWrite> {
        let candidates = query_candidates
            .iter()
            .filter(|candidate| candidate.eligible(InventoryMode::RovingTabindex))
            .collect::<Vec<_>>();
        self.remember(host, &candidates);
        let mut writes = candidates
            .iter()
            .filter(|candidate| candidate.item != *target)
            .map(|candidate| AttributeWrite {
                item: candidate.item.clone(),
                tabindex: Some("-1".into()),
            })
            .collect::<Vec<_>>();
        // Source writes target=0 even if it disappeared from the re-queried
        // current candidates. It is not added to the remembered baseline then.
        writes.push(AttributeWrite {
            item: target.clone(),
            tabindex: Some("0".into()),
        });
        writes
    }

    pub(crate) fn init(
        &mut self,
        host: &ItemRef,
        query_candidates: &[HostFacts],
        last: bool,
        focus_checked: bool,
    ) -> Option<(ItemRef, Vec<AttributeWrite>)> {
        let candidates = query_candidates
            .iter()
            .filter(|candidate| candidate.eligible(InventoryMode::RovingTabindex))
            .collect::<Vec<_>>();
        let edge = if last {
            candidates.last()
        } else {
            candidates.first()
        }?;
        let target = if focus_checked {
            candidates
                .iter()
                .find(|candidate| candidate.checked_attribute)
                .unwrap_or(edge)
        } else {
            edge
        }
        .item
        .clone();
        self.remember(host, &candidates);
        let mut writes = candidates
            .iter()
            .map(|candidate| AttributeWrite {
                item: candidate.item.clone(),
                tabindex: Some("-1".into()),
            })
            .collect::<Vec<_>>();
        writes.push(AttributeWrite {
            item: target.clone(),
            tabindex: Some("0".into()),
        });
        Some((target, writes))
    }

    pub(crate) fn clear(&mut self, host: &ItemRef) -> Vec<AttributeWrite> {
        let Some(index) = self.hosts.iter().position(|(item, _)| item == host) else {
            return Vec::new();
        };
        self.hosts.remove(index).1
    }

    /// Global renderer teardown only. A closing root should clear its tracked
    /// itemsHosts individually, rather than clearing unrelated shared hosts.
    pub(crate) fn clear_all(&mut self) -> Vec<AttributeWrite> {
        std::mem::take(&mut self.hosts)
            .into_iter()
            .flat_map(|(_, writes)| writes)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, kind: ItemKind, tabindex: Option<&str>) -> HostFacts {
        HostFacts {
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
    fn write(id: &str, value: Option<&str>) -> AttributeWrite {
        AttributeWrite {
            item: ItemRef(id.into()),
            tabindex: value.map(str::to_string),
        }
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
        assert_eq!(inventory.items_host, ItemRef("actual-host".into()));
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
    fn roving_init_preserves_query_order_checks_and_exact_original_attributes() {
        let host = ItemRef("shared-host".into());
        let a = item("a", ItemKind::Dropdown, None);
        let mut b = item("b", ItemKind::Select, Some("4"));
        b.checked_attribute = true;
        let c = item("c", ItemKind::Menu, Some("-8"));
        let mut state = RovingState::default();
        let (target, writes) = state.init(&host, &[a, b, c], true, true).unwrap();
        assert_eq!(target, ItemRef("b".into()));
        assert_eq!(
            writes,
            [
                write("a", Some("-1")),
                write("b", Some("-1")),
                write("c", Some("-1")),
                write("b", Some("0"))
            ]
        );
        assert_eq!(
            state.clear(&host),
            [
                write("a", None),
                write("b", Some("4")),
                write("c", Some("-8"))
            ]
        );
        assert!(state.clear(&host).is_empty());
        assert!(state.init(&host, &[], false, false).is_none());
    }

    #[test]
    fn roving_select_restores_first_values_after_consumer_edits_insertion_and_same_id_replacement()
    {
        let host = ItemRef("shared-host".into());
        let mut state = RovingState::default();
        let a = item("actual-a", ItemKind::Dropdown, None);
        let b = item("actual-b", ItemKind::Menu, Some("3"));
        assert_eq!(
            state.select(&host, &[a.clone(), b], &a.item),
            [write("actual-b", Some("-1")), write("actual-a", Some("0"))]
        );
        let mut edited_a = a.clone();
        edited_a.tabindex_attribute = Some("19".into());
        let mut replacement = item("new-actual-b", ItemKind::Menu, Some("-7"));
        replacement.target = "public-actual-b".into();
        let inserted = item("inserted", ItemKind::NativeOptIn, Some("5"));
        state.select(
            &host,
            &[edited_a, replacement.clone(), inserted],
            &replacement.item,
        );
        assert_eq!(
            state.clear(&host),
            [
                write("actual-a", None),
                write("actual-b", Some("3")),
                write("new-actual-b", Some("-7")),
                write("inserted", Some("5"))
            ]
        );
    }

    #[test]
    fn requery_can_remove_selected_target_without_remembering_or_retargeting_it() {
        let host = ItemRef("actual-host".into());
        let a = item("a", ItemKind::Dropdown, None);
        let target = ItemRef("already-selected-removed-object".into());
        let mut state = RovingState::default();
        assert_eq!(
            state.select(&host, &[a], &target),
            [
                write("a", Some("-1")),
                write("already-selected-removed-object", Some("0"))
            ]
        );
        assert_eq!(state.clear(&host), [write("a", None)]);
    }

    #[test]
    fn shared_items_host_keeps_one_first_baseline_and_other_hosts_clear_independently() {
        let first = ItemRef("host-one".into());
        let second = ItemRef("host-two".into());
        let mut state = RovingState::default();
        let a = item("a", ItemKind::Dropdown, Some("2"));
        let mut later_a = a.clone();
        later_a.tabindex_attribute = Some("0".into());
        state.select(&first, &[a.clone(), a], &ItemRef("a".into()));
        state.select(
            &second,
            &[item("b", ItemKind::Menu, None)],
            &ItemRef("b".into()),
        );
        state.select(&first, &[later_a], &ItemRef("a".into()));
        assert_eq!(state.clear(&first), [write("a", Some("2"))]);
        assert_eq!(state.clear_all(), [write("b", None)]);
        assert!(state.clear_all().is_empty());
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
}

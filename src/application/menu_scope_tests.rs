// Scoped Rust Menu/Category registration exercised through typed Dioxus events.
// Mounted handles prove actual focus delivery without native inventory input.

use super::*;

#[test]
fn category_visibility_composes_with_authored_hidden_in_both_presentations() {
    let _guard = crate::test_event_converter_guard();
    for expanded in [true, false] {
        let mut h = MenuHarness::new();
        h.config.borrow_mut().hidden.insert("Two".into());
        h.update();
        if !expanded {
            h.toggle_expanded();
        }
        let category = h.category("Category");
        let ids = ["Child", "Disabled child", "Two"]
            .map(|label| h.attributes[&h.target(label).raw()]["id"].clone());
        let html = h.html();
        for id in &ids {
            assert!(item_opening_tag(&html, id).contains(" hidden"));
        }
        h.click("Category");
        assert!(category.is_open());
        let html = h.html();
        assert!(!item_opening_tag(&html, &ids[0]).contains(" hidden"));
        assert!(!item_opening_tag(&html, &ids[1]).contains(" hidden"));
        assert!(item_opening_tag(&html, &ids[2]).contains(" hidden"));
        h.category_key("Escape", "Category", Some("Child"));
        assert!(!category.is_open());
        let html = h.html();
        for id in &ids {
            assert!(item_opening_tag(&html, id).contains(" hidden"));
        }
        h.config.borrow_mut().hidden.remove("Two");
        h.update();
        h.click("Category");
        assert!(!item_opening_tag(&h.html(), &ids[2]).contains(" hidden"));
    }
}

#[test]
fn root_keyed_order_insert_remove_preserves_primary_bottom_utility_priority() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.config.borrow_mut().root_keys = ["Alpha", "Link", "Omega"]
        .into_iter()
        .map(str::to_string)
        .collect();
    h.update();
    let alpha = h.id("Alpha");
    let link = h.id("Link");
    h.root_key("Home", None, false);
    assert_eq!(h.focuses.borrow().last().copied(), Some(alpha));
    h.config.borrow_mut().root_keys.reverse();
    h.update();
    assert_eq!(h.id("Alpha"), alpha, "a keyed move keeps its Rust owner");
    h.root_key("Home", None, false);
    assert_eq!(h.last(), Some(h.id("Omega")));
    h.config.borrow_mut().root_keys.insert(1, "Inserted".into());
    h.update();
    h.root_key("ArrowDown", Some(h.id("Omega")), false);
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Inserted")));
    h.config
        .borrow_mut()
        .root_keys
        .retain(|label| label != "Link");
    h.update();
    assert!(!h.context.model.borrow().members.contains_key(&link));
    let expected = [
        "Omega",
        "Inserted",
        "Alpha",
        "Footer",
        "Settings",
        "Toggle theme",
        "About & legal information",
    ];
    h.root_key("Home", None, false);
    for (position, label) in expected.iter().enumerate() {
        let id = h.id(label);
        assert_eq!(h.last(), Some(id), "section order at {label}");
        assert_eq!(h.context.attributes(id).1, Some(position + 1));
        assert_eq!(h.context.attributes(id).2, Some(expected.len()));
        if position + 1 < expected.len() {
            h.root_key("ArrowDown", Some(id), false);
        }
    }
    h.root_key("ArrowDown", Some(h.id("About & legal information")), false);
    assert_eq!(
        h.last(),
        Some(h.id("Omega")),
        "utilities wrap back to primary"
    );
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn category_keyed_order_insert_remove_stays_in_its_scoped_membership() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.config.borrow_mut().child_keys = ["Child", "Two"].into_iter().map(str::to_string).collect();
    h.update();
    let category = h.category("Category");
    let child = h.id("Child");
    h.click("Category");
    h.category_key("ArrowDown", "Category", None);
    assert_eq!(h.focuses.borrow().last().copied(), Some(child));
    h.config.borrow_mut().child_keys.reverse();
    h.update();
    assert_eq!(h.id("Child"), child);
    h.config
        .borrow_mut()
        .child_keys
        .insert(1, "Inserted child".into());
    h.update();
    assert_eq!(
        h.category_key("ArrowDown", "Category", Some("Two")),
        (true, false)
    );
    assert_eq!(
        h.focuses.borrow().last().copied(),
        Some(h.id("Inserted child")),
        "inserted child: requested={:?}, items={:?}, children={:?}, controls={:?}, pending_mounts={:?}",
        category.0.model.borrow().requested,
        category.items(false),
        category
            .0
            .model
            .borrow()
            .children
            .iter()
            .map(|(id, child)| (*id, child.order, child.hidden, child.node.is_some()))
            .collect::<Vec<_>>(),
        h.controls,
        h.mounts
    );
    h.config
        .borrow_mut()
        .child_keys
        .retain(|label| label != "Child");
    h.update();
    assert!(!category.0.model.borrow().children.contains_key(&child));
    h.category_key("Tab", "Category", Some("Inserted child"));
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Two")));
    assert!(
        !h.context
            .model
            .borrow()
            .members
            .contains_key(&h.id("Inserted child"))
    );
    assert!(!h.context.model.borrow().members.contains_key(&h.id("Two")));
    assert_eq!(h.config.borrow().activations, 0, "navigation never selects");
}

#[test]
fn collapsed_category_entry_skips_current_hidden_disabled_and_negative_tabindex() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    for reason in ["hidden", "disabled", "negative", "eligible"] {
        {
            let mut config = h.config.borrow_mut();
            config.hidden.clear();
            if reason == "hidden" {
                config.hidden.insert("Child".into());
            }
            config.child_disabled = reason == "disabled";
            config.child_negative = reason == "negative";
        }
        h.update();
        h.category_key("Enter", "Category", None);
        assert!(h.category("Category").is_open());
        let expected = if reason == "eligible" { "Child" } else { "Two" };
        assert_eq!(
            h.focuses.borrow().last().copied(),
            Some(h.id(expected)),
            "{reason}"
        );
        h.category_key("Escape", "Category", Some(expected));
        assert!(!h.category("Category").is_open());
    }
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn retired_category_focus_cannot_adopt_a_replacement_with_the_same_label() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let retired = h.category("Category");
    let child = h.id("Child");
    h.category_key("Enter", "Category", None);
    assert!(retired.is_open());
    h.config.borrow_mut().category_present = false;
    h.update();
    assert!(!retired.0.model.borrow().alive);
    assert!(
        !h.context
            .model
            .borrow()
            .categories
            .contains_key(&retired.0.id)
    );
    h.config.borrow_mut().category_present = true;
    h.update();
    let current = h.category("Category");
    assert_ne!(current.0.id, retired.0.id);
    assert!(!current.is_open());
    let before = h.focuses.borrow().clone();
    h.dom.in_runtime(|| retired.focus(Some(child)));
    h.dom.in_runtime(|| retired.focus(None));
    h.settle();
    assert_eq!(
        *h.focuses.borrow(),
        before,
        "retired mounts cannot focus a new scope"
    );
    assert!(!current.is_open());
    h.category_key("Enter", "Category", None);
    assert!(current.is_open());
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
    assert_eq!(h.config.borrow().activations, 0);
}

// Read-only Menu/Category inventory followed by actual Dioxus interactions.
// Mounted test handles prove delivery and cancellation without native discovery.

use super::*;

#[test]
fn ordinary_category_controls_publish_source_hidden_through_both_presentations() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    let category = h.category("Category");
    let children = ["Child", "Disabled child", "Two"]
        .map(|label| h.attributes[&h.target(label).raw()]["id"].clone());
    for collapsed in [false, true] {
        if collapsed {
            h.toggle_expanded();
        }
        assert!(!category.is_open());
        let html = dioxus_ssr::render(&h.dom);
        for id in &children {
            assert!(item_opening_tag(&html, id).contains(" hidden"));
        }
        h.click("Category");
        assert!(category.is_open());
        let html = dioxus_ssr::render(&h.dom);
        for id in &children {
            assert!(!item_opening_tag(&html, id).contains(" hidden"));
        }
        if collapsed {
            let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
            h.category_event("category-key", "Escape", "Category", None, 1, facts);
        } else {
            h.click("Category");
        }
        assert!(!category.is_open());
        let html = dioxus_ssr::render(&h.dom);
        for id in &children {
            assert!(item_opening_tag(&html, id).contains(" hidden"));
        }
    }
}

#[test]
fn registered_nondirect_category_activation_does_not_expand_menu_candidates() {
    let _guard = crate::test_event_converter_guard();
    for kind in ["category-key", "category-click"] {
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let direct = h.facts(&["Alpha", "Omega", "Footer"]);
        h.observe("ready", "", None, false, direct.clone());
        h.observe("key", "End", None, false, direct.clone());
        let root_last = h.last();
        let root_tabs = h.tabs();
        assert_eq!(root_last, Some(h.id("Footer")));
        assert_eq!(h.context.attributes(category.0.id).1, None);
        assert!(
            h.context
                .model
                .borrow()
                .categories
                .contains_key(&category.0.id)
        );

        let mut children = h.category_facts(&["Child", "Disabled child", "Two"]);
        for child in &mut children {
            child.hidden = true;
        }
        let message = serde_json::json!({
            "owner":category.0.owner,"kind":kind,"key":if kind=="category-key" {"Enter"} else {""},
            "category":category.0.id,"category_parent":true,"sequence":1,
            "category_items":children,"items":[]
        });
        assert_eq!(
            h.raw(message.clone()),
            if kind == "category-click" {
                (false, false)
            } else {
                (true, false)
            }
        );
        assert!(
            category.is_open(),
            "registered own activation remains valid outside the direct list"
        );
        assert_eq!(h.last(), root_last, "no outer-bar input was supplied");
        assert_eq!(h.tabs(), root_tabs);
        assert_eq!(h.context.model.borrow().observed, direct);
        assert_eq!(h.context.attributes(category.0.id).1, None);
        assert_eq!(h.context.attributes(h.id("Alpha")).2, Some(3));
        assert_eq!(h.context.attributes(h.id("Child")).1, None);

        if kind == "category-key" {
            assert_eq!(category.0.model.borrow().entry_focus, Some(h.id("Child")));
            for child in &mut children {
                child.hidden = false;
            }
            assert_eq!(
                h.category_event("category-order", "", "Category", None, 0, children),
                (false, true)
            );
            assert_eq!(
                category.0.model.borrow().requested,
                Some(Some(h.id("Child")))
            );
        } else {
            assert_eq!(
                category.0.model.borrow().entry_focus,
                None,
                "pointer activation does not request keyboard entry"
            );
        }
        assert_eq!(h.config.borrow().activations, 0);
        assert!(category.is_open());
        assert_eq!(h.last(), root_last);
        assert_eq!(h.context.model.borrow().observed, direct);
    }
}

#[test]
fn nondirect_category_child_navigation_and_return_do_not_synthesize_outer_menu_input() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let direct = h.facts(&["Alpha", "Omega"]);
    h.observe("ready", "", None, false, direct.clone());
    h.observe("key", "End", None, false, direct.clone());
    let root_last = h.last();
    let root_tabs = h.tabs();
    let children = h.category_facts(&["Two", "Child", "Disabled child"]);
    assert_eq!(
        h.category_event(
            "category-key",
            "Enter",
            "Category",
            None,
            1,
            children.clone()
        ),
        (true, false)
    );
    assert!(category.is_open());
    assert_eq!(
        category.0.model.borrow().requested,
        Some(Some(h.id("Two"))),
        "entry uses current reported light-descendant order"
    );
    assert_eq!(
        h.category_event(
            "category-key",
            "End",
            "Category",
            Some("Two"),
            2,
            children.clone()
        ),
        (true, false)
    );
    assert_eq!(
        h.focused_category_target("Category"),
        Some(format!(
            "ix-menu-category-child-{}-{}",
            category.0.id,
            h.id("Child")
        ))
    );
    let runtime = category
        .0
        .overlay
        .borrow()
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap();
    assert!(runtime.pending_item_focus_count() == 0);
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
    assert_eq!(h.last(), root_last);
    assert_eq!(h.tabs(), root_tabs);
    assert_eq!(h.context.model.borrow().observed, direct);

    assert_eq!(
        h.category_event(
            "category-key",
            "Enter",
            "Category",
            Some("Child"),
            3,
            children.clone()
        ),
        (true, false)
    );
    assert!(!category.is_open());
    let focused_after_close = h.focuses.borrow().clone();
    let node = category.0.overlay.borrow().as_ref().unwrap().clone();
    let child_target = category.0.model.borrow().children[&h.id("Child")]
        .host_id
        .clone()
        .unwrap();
    assert!(!node.queue_item_focus(
        child_target,
        true,
        Rc::new(|| panic!("closed Category item"))
    ));
    h.settle();
    assert_eq!(*h.focuses.borrow(), focused_after_close);
    assert!(runtime.pending_item_focus_count() == 0);
    assert_eq!(h.config.borrow().activations, 0);
    assert_eq!(h.last(), root_last);
    assert_eq!(h.tabs(), root_tabs);
    assert_eq!(h.context.model.borrow().observed, direct);
    assert!(!category.is_open());
}

#[test]
fn category_scope_inventory_and_old_interactions_do_not_adopt_replacement() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let old_id = category.0.id;
    let old_child = h.id("Child");
    let owner = category.0.owner.clone();
    let children = h.category_facts(&["Child", "Disabled child", "Two"]);
    assert_eq!(
        h.category_event(
            "category-key",
            "Enter",
            "Category",
            None,
            1,
            children.clone()
        ),
        (true, false)
    );
    assert_eq!(
        h.category_event(
            "category-key",
            "Enter",
            "Category",
            Some("Child"),
            2,
            children.clone()
        ),
        (true, false)
    );
    assert!(!category.is_open());
    let reordered = h.category_facts(&["Two", "Child", "Disabled child"]);
    h.category_event("category-order", "", "Category", None, 0, reordered);

    let retained = serde_json::json!({
        "owner":owner,"kind":"category-click","category":old_id,
        "category_parent":true,"child":old_child,"sequence":100,
        "category_items":children,"items":[]
    });
    h.config.borrow_mut().category_present = false;
    h.update();
    assert!(!category.0.model.borrow().alive);
    assert!(!h.context.model.borrow().categories.contains_key(&old_id));
    assert_eq!(h.raw(retained.clone()), (false, true));
    h.config.borrow_mut().category_present = true;
    h.update();
    let other_id = h.id("Second category");
    let replacement = CategoryNavigation(
        h.context
            .model
            .borrow()
            .categories
            .iter()
            .find(|(id, _)| **id != other_id)
            .and_then(|(_, category)| category.upgrade())
            .unwrap(),
    );
    assert_ne!(replacement.0.id, old_id);
    assert_eq!(replacement.0.owner, owner);
    assert!(!replacement.is_open());
    assert_eq!(h.raw(retained.clone()), (false, true));
    assert!(
        !replacement.is_open(),
        "old Category ID cannot activate the new registered instance"
    );
    assert_eq!(
        h.raw(serde_json::json!({
            "owner":owner,"kind":"category-click","category":replacement.0.id,
            "category_parent":true,"sequence":1,"category_items":[],"items":[]
        })),
        (false, false)
    );
    assert!(
        replacement.is_open(),
        "the current registered replacement still accepts its own input"
    );
    assert_eq!(h.raw(retained), (false, true));
    assert!(
        replacement.is_open(),
        "late old click cannot toggle the replacement closed"
    );
}

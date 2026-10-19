// Category native disconnect lifetime and controller-dismissal tests.

use super::*;

#[test]
fn open_category_disconnect_closes_via_controller_state_without_changing_top_menu_order() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let direct = h.facts(&["Alpha", "Category", "Omega", "Footer"]);
    h.observe("ready", "", None, false, direct.clone());
    h.observe("key", "End", None, false, direct.clone());
    let root_last = h.last();
    let root_tabs = h.tabs();
    let root_generation = h.context.model.borrow().generation;
    assert_eq!(root_last, Some(h.id("Footer")));
    assert_eq!(h.context.attributes(category.0.id), (-1, Some(2), Some(4)));

    let children = h.category_facts(&["Child", "Disabled child", "Two"]);
    let child_ids = ["Child", "Disabled child", "Two"]
        .map(|label| h.attributes[&h.target(label).raw()]["id"].clone());
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
        h.category_event(
            "category-key",
            "ArrowDown",
            "Category",
            None,
            2,
            children.clone()
        ),
        (true, false)
    );
    assert_eq!(h.focuses.borrow().last().copied(), Some(h.id("Child")));
    let runtime = category
        .0
        .overlay
        .borrow()
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap();
    assert!(runtime.pending_item_focus_count() == 0);
    let focused = h.focuses.borrow().clone();

    assert_eq!(
        h.raw(serde_json::json!({
            "owner": category.0.owner,
            "kind": "category-disconnect",
            "category": category.0.id,
            "sequence": 3,
            "connected": false,
            "category_items": [],
            "items": h.facts(&["Omega", "Alpha"])
        })),
        (false, true)
    );
    assert!(!category.is_open());
    assert_eq!(*category.0.flyout.peek(), None);
    let html = h.html();
    for id in &child_ids {
        assert!(item_opening_tag(&html, id).contains(" hidden"));
    }
    assert_eq!(h.context.model.borrow().observed, direct);
    assert_eq!(h.last(), root_last);
    assert_eq!(h.tabs(), root_tabs);
    assert_eq!(h.context.model.borrow().generation, root_generation);
    assert_eq!(h.context.attributes(category.0.id), (-1, Some(2), Some(4)));
    assert!(category.0.model.borrow().alive);
    assert!(
        h.context
            .model
            .borrow()
            .categories
            .contains_key(&category.0.id)
    );
    assert!(
        category
            .0
            .overlay
            .borrow()
            .as_ref()
            .unwrap()
            .is_registered()
    );
    assert_eq!(category.0.model.borrow().children.len(), 3);
    assert_eq!(category.0.model.borrow().observed, children);
    assert!(runtime.pending_item_focus_count() == 0);
    assert_eq!(
        *h.focuses.borrow(),
        focused,
        "disconnect does not refocus hidden children"
    );
}

#[test]
fn closed_replayed_wrong_owner_stale_and_dropped_category_disconnect_do_not_close_or_reassign() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let other = h.category("Second category");
    let old_id = category.0.id;
    let other_id = other.0.id;
    let owner = category.0.owner.clone();
    let test = h.focus_boundary("Category");
    h.document_mode("keydown", "Enter", 1);

    h.click("Second category");
    assert!(!category.is_open() && other.is_open());
    assert_eq!(
        h.raw(serde_json::json!({
            "owner": owner,
            "kind": "category-disconnect",
            "category": old_id,
            "sequence": 1,
            "connected": true,
            "active_categories": [old_id, other_id],
            "items": []
        })),
        (false, true)
    );
    assert!(
        other.is_open() && !category.is_open(),
        "closed Category disconnect must not dismiss another open Category"
    );
    assert_eq!(test.execution.started.get(), 0);
    assert!(category.0.model.borrow().return_tasks.is_empty());

    assert_eq!(
        h.raw(serde_json::json!({
            "owner": owner,
            "kind": "category-click",
            "category": old_id,
            "category_parent": true,
            "sequence": 2,
            "category_items": h.category_facts(&["Child", "Disabled child", "Two"]),
            "items": []
        })),
        (false, false)
    );
    assert!(category.is_open() && !other.is_open());

    for rejected in [
        serde_json::json!({
            "owner": "foreign-menu",
            "kind": "category-disconnect",
            "category": old_id,
            "sequence": 3,
            "connected": true,
            "active_categories": [old_id],
            "items": []
        }),
        serde_json::json!({
            "owner": owner,
            "kind": "category-disconnect",
            "category": usize::MAX,
            "sequence": 3,
            "connected": true,
            "active_categories": [old_id],
            "items": []
        }),
        serde_json::json!({
            "owner": owner,
            "kind": "category-disconnect",
            "category": old_id,
            "sequence": 0,
            "connected": true,
            "active_categories": [old_id],
            "items": []
        }),
        serde_json::json!({
            "owner": owner,
            "kind": "category-disconnect",
            "category": old_id,
            "sequence": 1,
            "connected": true,
            "active_categories": [old_id],
            "items": []
        }),
    ] {
        assert_eq!(h.raw(rejected), (false, true));
        assert!(category.is_open());
        // Dioxus clicks do not advance the native lifecycle journal.
        assert_eq!(category.0.model.borrow().newest_sequence, 1);
        assert_eq!(test.execution.started.get(), 0);
    }

    let disconnect_three = serde_json::json!({
        "owner": owner,
        "kind": "category-disconnect",
        "category": old_id,
        "sequence": 3,
        "connected": false,
        "items": []
    });
    assert_eq!(h.raw(disconnect_three.clone()), (false, true));
    assert!(!category.is_open());
    h.hover("Category", "host", "enter", 1, "mouse", &[]);
    assert!(category.is_open());
    assert_eq!(
        h.raw(disconnect_three),
        (false, true),
        "replayed disconnect sequence cannot close a reopened Category"
    );
    assert!(category.is_open());

    h.config.borrow_mut().category_present = false;
    h.update();
    assert!(!category.0.model.borrow().alive);
    assert!(!h.context.model.borrow().categories.contains_key(&old_id));
    h.click("Second category");
    assert!(other.is_open());
    let retained = serde_json::json!({
        "owner": owner,
        "kind": "category-disconnect",
        "category": old_id,
        "sequence": 99,
        "connected": true,
        "active_categories": [old_id, other_id],
        "items": []
    });
    assert_eq!(h.raw(retained.clone()), (false, true));
    assert!(other.is_open());

    h.config.borrow_mut().category_present = true;
    h.update();
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
    assert_eq!(
        h.raw(serde_json::json!({
            "owner": owner,
            "kind": "category-click",
            "category": replacement.0.id,
            "category_parent": true,
            "sequence": 1,
            "category_items": [],
            "items": []
        })),
        (false, false)
    );
    assert!(replacement.is_open() && !other.is_open());
    assert_eq!(h.raw(retained), (false, true));
    assert!(
        replacement.is_open(),
        "dropped Category disconnect must not close or adopt the replacement instance"
    );
    assert_eq!(replacement.0.model.borrow().newest_sequence, 0);
}

#[test]
fn reopened_category_cancels_old_return_while_fresh_return_uses_raw_facts_and_drop_cancels() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let test = h.focus_boundary("Category");
    let owner = category.0.owner.clone();
    let id = category.0.id;

    h.document_mode("keydown", "Tab", 1);
    h.hover("Category", "host", "enter", 1, "mouse", &[]);
    assert!(category.is_open());
    assert_eq!(
        h.raw(serde_json::json!({
            "owner": owner,
            "kind": "category-outside",
            "category": id,
            "sequence": 1,
            "inside": false,
            "outside_cause": "click",
            "active_categories": [id],
            "category_items": [],
            "items": []
        })),
        (false, true)
    );
    assert!(!category.is_open());
    assert_eq!(test.execution.started.get(), 1);
    assert_eq!(
        category
            .0
            .model
            .borrow()
            .return_tasks
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["1"]
    );

    for (hover_seq, doc_mode, payload) in [
        (
            2,
            None,
            serde_json::json!({
                "owner": owner,
                "kind": "category-disconnect",
                "category": id,
                "sequence": 2,
                "connected": false,
                "active_categories": [id],
                "items": []
            }),
        ),
        (
            3,
            None,
            serde_json::json!({
                "owner": owner,
                "kind": "category-disconnect",
                "category": id,
                "sequence": 3,
                "active_categories": [id],
                "items": []
            }),
        ),
        (
            4,
            None,
            serde_json::json!({
                "owner": owner,
                "kind": "category-disconnect",
                "category": id,
                "sequence": 4,
                "connected": true,
                "active_categories": [h.id("Second category")],
                "items": []
            }),
        ),
        (
            5,
            Some(("mousedown", "", 2)),
            serde_json::json!({
                "owner": owner,
                "kind": "category-disconnect",
                "category": id,
                "sequence": 5,
                "connected": true,
                "active_categories": [id],
                "items": []
            }),
        ),
    ] {
        h.hover("Category", "host", "enter", hover_seq, "mouse", &[]);
        assert!(category.is_open());
        if let Some((kind, key, seq)) = doc_mode {
            h.document_mode(kind, key, seq);
        }
        assert_eq!(h.raw(payload), (false, true));
        assert!(!category.is_open());
        assert_eq!(
            test.execution.started.get(),
            1,
            "ineligible disconnect facts must not authorize a fresh return job"
        );
        assert_eq!(
            test.execution.canceled.get(),
            1,
            "reopening cancels the obsolete return before new disconnect facts are checked"
        );
    }

    h.document_mode("keydown", "Tab", 3);
    h.hover("Category", "host", "enter", 6, "mouse", &[]);
    assert!(category.is_open());
    assert_eq!(
        h.raw(serde_json::json!({
            "owner": owner,
            "kind": "category-disconnect",
            "category": id,
            "sequence": 6,
            "connected": true,
            "active_categories": [id],
            "items": []
        })),
        (false, true)
    );
    assert!(!category.is_open());
    assert_eq!(test.execution.started.get(), 2);
    assert_eq!(test.execution.canceled.get(), 1);
    assert_eq!(
        category
            .0
            .model
            .borrow()
            .return_tasks
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["2"]
    );
    h.focuses.borrow_mut().clear();
    h.release_focus_gate(&test.execution);
    assert_eq!(
        h.focuses
            .borrow()
            .iter()
            .filter(|focused| **focused == id)
            .count(),
        1
    );
    assert!(category.0.model.borrow().return_tasks.is_empty());

    test.execution.ready.set(false);
    h.focuses.borrow_mut().clear();
    h.hover("Category", "host", "enter", 7, "mouse", &[]);
    assert!(category.is_open());
    assert_eq!(
        h.raw(serde_json::json!({
            "owner": owner,
            "kind": "category-disconnect",
            "category": id,
            "sequence": 7,
            "connected": true,
            "active_categories": [id],
            "items": []
        })),
        (false, true)
    );
    assert!(!category.is_open());
    assert_eq!(test.execution.started.get(), 3);
    assert_eq!(
        category
            .0
            .model
            .borrow()
            .return_tasks
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ["3"]
    );

    h.config.borrow_mut().category_present = false;
    h.update();
    assert!(!category.0.model.borrow().alive);
    assert_eq!(test.execution.canceled.get(), 2);
    assert!(category.0.model.borrow().return_tasks.is_empty());
    h.release_focus_gate(&test.execution);
    assert!(h.focuses.borrow().is_empty());
}

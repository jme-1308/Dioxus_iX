// Category ownership follows Dioxus pointer/focus events and component lifetime.
use super::*;

#[test]
fn category_pointer_grace_handoff_and_reentry_keep_the_current_flyout() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
    h.category_event("category-order", "", "Category", None, 0, facts);
    let clock = h.hover_clock("Category");
    let focused = h.focuses.borrow().clone();
    h.pointer("Category", "host", "pointerenter", "mouse");
    assert!(category.is_open());
    assert_eq!(*h.focuses.borrow(), focused, "hover does not steal focus");
    h.pointer("Category", "host", "pointerleave", "mouse");
    assert!(category.is_open(), "leaving starts a grace period");
    assert!(category.0.model.borrow().hide_task.is_some());
    h.pointer("Category", "flyout", "pointerenter", "mouse");
    h.advance_hover(&clock, 1_000);
    assert!(category.is_open(), "host-to-flyout handoff cancels grace");
    assert!(category.0.model.borrow().hide_task.is_none());
    h.pointer("Category", "flyout", "pointerleave", "mouse");
    h.pointer("Category", "host", "pointerleave", "mouse");
    h.pointer("Category", "host", "pointerenter", "mouse");
    h.advance_hover(&clock, 1_000);
    assert!(category.is_open(), "reentry supersedes the previous leave");
    h.pointer("Category", "host", "pointerleave", "mouse");
    h.advance_hover(&clock, 1_000);
    assert!(!category.is_open());
    assert_eq!(
        *h.focuses.borrow(),
        focused,
        "pointer dismissal preserves independent focus"
    );
}

#[test]
fn category_touch_leave_keeps_the_presented_flyout_and_expanded_hover_is_inert() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    let category = h.category("Category");
    h.pointer("Category", "host", "pointerenter", "mouse");
    assert!(
        !category.is_open(),
        "expanded panels open through ordinary activation"
    );
    h.toggle_expanded();
    let clock = h.hover_clock("Category");
    h.pointer("Category", "host", "pointerenter", "touch");
    assert!(category.is_open());
    h.pointer("Category", "host", "pointerleave", "touch");
    h.advance_hover(&clock, 1_000);
    assert!(category.is_open());
    assert!(category.0.model.borrow().hide_task.is_none());
    assert_eq!(h.config.borrow().activations, 0);
}

#[test]
fn category_pointer_entry_closes_the_other_category_without_moving_focus() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let first = h.category("Category");
    let second = h.category("Second category");
    let focused = h.focuses.borrow().clone();
    h.pointer("Category", "host", "pointerenter", "mouse");
    let clock = h.hover_clock("Category");
    h.pointer("Category", "host", "pointerleave", "mouse");
    h.pointer("Second category", "host", "pointerenter", "mouse");
    assert!(!first.is_open() && second.is_open());
    h.advance_hover(&clock, 1_000);
    assert!(
        second.is_open(),
        "old owner's grace cannot dismiss the new owner"
    );
    assert_eq!(*h.focuses.borrow(), focused);
}

#[test]
fn category_mode_change_and_drop_cancel_pointer_grace_without_focus_repair() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    let clock = h.hover_clock("Category");
    h.pointer("Category", "host", "pointerenter", "mouse");
    h.pointer("Category", "host", "pointerleave", "mouse");
    h.toggle_expanded();
    let focused = h.focuses.borrow().clone();
    let current = category.is_open();
    h.advance_hover(&clock, 1_000);
    assert_eq!(
        category.is_open(),
        current,
        "an old collapsed leave cannot change the expanded presentation"
    );
    assert!(category.0.model.borrow().hide_task.is_none());
    h.toggle_expanded();
    h.pointer("Category", "host", "pointerenter", "mouse");
    h.pointer("Category", "host", "pointerleave", "mouse");
    let old_id = category.0.id;
    h.config.borrow_mut().category_present = false;
    h.update();
    assert!(!category.0.model.borrow().alive);
    assert!(category.0.model.borrow().hide_task.is_none());
    assert!(!h.context.model.borrow().categories.contains_key(&old_id));
    h.advance_hover(&clock, 1_000);
    h.config.borrow_mut().category_present = true;
    h.update();
    let second_id = h.id("Second category");
    let replacement = h
        .context
        .model
        .borrow()
        .categories
        .iter()
        .find(|(id, _)| **id != second_id)
        .and_then(|(_, category)| category.upgrade())
        .unwrap();
    assert_ne!(replacement.id, old_id);
    assert!(!CategoryNavigation(replacement).is_open());
    assert_eq!(
        *h.focuses.borrow(),
        focused,
        "unmounted parents never receive a repair focus"
    );
}

#[test]
fn category_pending_mounted_parent_return_cancels_on_new_intent_mode_and_drop() {
    let _guard = crate::test_event_converter_guard();
    for cancel in ["reopen", "mode", "drop"] {
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        h.key(h.target("Category"), Key::Enter);
        let parent = MenuTestMount {
            id: h.id("Category"),
            focuses: h.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(&h.dom, "mounted", h.target("Category"), parent.clone());
        h.settle();
        h.focuses.borrow_mut().clear();
        assert_eq!(h.key(h.target("Child"), Key::Escape), (false, false));
        assert!(!category.is_open());
        assert!(
            parent.started.get(),
            "the registered parent focus is genuinely pending"
        );
        match cancel {
            "reopen" => {
                h.key(h.target("Category"), Key::Enter);
            }
            "mode" => h.toggle_expanded(),
            _ => {
                h.config.borrow_mut().category_present = false;
                h.update();
            }
        }
        assert!(
            parent.read_dropped.get(),
            "{cancel} drops the old focus future"
        );
        h.release(&parent);
        assert!(
            !h.focuses.borrow().contains(&parent.id),
            "{cancel} cannot focus the stale return target"
        );
    }
}

#[test]
fn category_grace_returns_only_keyboard_owned_focus_to_the_registered_parent() {
    let _guard = crate::test_event_converter_guard();
    for keyboard in [true, false] {
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        h.key(h.target("Category"), Key::Enter);
        h.focus_control("Child");
        assert!(category.0.model.borrow().focus_inside);
        h.shared_modality("Category", keyboard);
        let clock = h.hover_clock("Category");
        h.focuses.borrow_mut().clear();
        h.pointer("Category", "host", "pointerleave", "mouse");
        h.advance_hover(&clock, 1_000);
        assert!(!category.is_open());
        if keyboard {
            assert_eq!(h.focuses.borrow().as_slice(), &[h.id("Category")]);
        } else {
            assert!(
                h.focuses.borrow().is_empty(),
                "pointer mode preserves the independent focus destination"
            );
        }
    }
}

#[test]
fn category_focusout_handoff_preserves_internal_focus_and_outside_destination() {
    let _guard = crate::test_event_converter_guard();
    let mut h = MenuHarness::new();
    h.toggle_expanded();
    let category = h.category("Category");
    h.key(h.target("Category"), Key::Enter);
    h.focus_control("Child");
    assert!(category.0.model.borrow().focus_inside);
    h.shared_modality("Category", true);
    let focused = h.focuses.borrow().clone();
    category.0.model.borrow_mut().focus_containment = Some(CategoryFocusContainment {
        connected: true,
        inside: true,
    });
    menu_dispatch(
        &h.dom,
        "focusout",
        h.category_surface("Category", "host"),
        (),
    );
    h.settle();
    assert!(
        category.is_open(),
        "moving among controls stays in the current flyout"
    );
    assert!(category.0.model.borrow().focus_inside);
    category.0.model.borrow_mut().focus_containment = Some(CategoryFocusContainment {
        connected: true,
        inside: false,
    });
    menu_dispatch(
        &h.dom,
        "focusout",
        h.category_surface("Category", "host"),
        (),
    );
    h.settle();
    assert!(!category.is_open());
    assert!(!category.0.model.borrow().focus_inside);
    assert_eq!(
        *h.focuses.borrow(),
        focused,
        "the outside destination keeps focus even in keyboard mode"
    );
}

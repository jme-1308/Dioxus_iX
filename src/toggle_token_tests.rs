// Controlled props and rendered native/form adapters only. Native requests,
// source mutation/cancellation, computed motion and AT behavior are separate evidence.
use super::*;
use std::cell::RefCell;

fn attribute<'a>(opening: &'a str, name: &str) -> Option<&'a str> {
    let value = opening.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else if let Some(value) = value.strip_prefix('\'') {
        Some(value.split('\'').next().unwrap())
    } else {
        Some(
            value
                .split(|c: char| c.is_whitespace() || c == '>')
                .next()
                .unwrap(),
        )
    }
}
fn classes(opening: &str) -> Vec<&str> {
    attribute(opening, "class")
        .unwrap_or_default()
        .split_whitespace()
        .collect()
}
fn has_attribute(opening: &str, name: &str) -> bool {
    opening
        .match_indices(&format!(" {name}"))
        .any(|(index, _)| {
            opening[index + name.len() + 1..]
                .chars()
                .next()
                .is_none_or(|c| c.is_whitespace() || c == '=' || c == '>')
        })
}
fn openings(html: &str) -> impl Iterator<Item = &str> {
    html.match_indices('<').filter_map(move |(index, _)| {
        let rest = &html[index..];
        Some(&rest[..=rest.find('>')?])
    })
}
fn opening_with_class<'a>(html: &'a str, marker: &str) -> &'a str {
    openings(html)
        .find(|tag| classes(tag).contains(&marker))
        .unwrap_or_else(|| panic!("missing {marker}: {html}"))
}
fn field<'a>(html: &'a str, marker: &str) -> (&'a str, &'a str) {
    let root = opening_with_class(html, marker);
    assert!(root.starts_with("<span "));
    let rest = &html[html.find(root).unwrap()..];
    let mut depth = 0usize;
    for (index, _) in rest.match_indices('<') {
        let tag = &rest[index..];
        if tag.starts_with("<span ") || tag.starts_with("<span>") {
            depth += 1;
        } else if tag.starts_with("</span>") {
            depth -= 1;
            if depth == 0 {
                return (root, &rest[..index + "</span>".len()]);
            }
        }
    }
    panic!("unclosed toggle field: {rest}");
}
fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_checked_mixed_combination_keeps_switch_role_and_independent_anatomy() {
    let html = render_fixture(|| {
        rsx! {
            for (key, checked, indeterminate) in [("off", false, false), ("on", true, false), ("mixed-off", false, true), ("mixed-on", true, true)] {
                Toggle { class: "state-{key}", checked, indeterminate, label: "Power", text_on: "Enabled", text_off: "Disabled", text_indeterminate: "Unsure" }
            }
        }
    });
    for (key, checked, mixed) in [
        ("off", false, false),
        ("on", true, false),
        ("mixed-off", false, true),
        ("mixed-on", true, true),
    ] {
        let (root, body) = field(&html, &format!("state-{key}"));
        assert!(
            classes(root).contains(&"ix-toggle-native")
                && classes(root).contains(&"ix-toggle-field")
        );
        for node in [root, opening_with_class(body, "ix-toggle-control")] {
            assert_eq!(
                attribute(node, "data-checked"),
                Some(if checked { "true" } else { "false" })
            );
            assert_eq!(
                attribute(node, "data-mixed"),
                Some(if mixed { "true" } else { "false" })
            );
        }
        let control = opening_with_class(body, "ix-toggle-control");
        assert!(control.starts_with("<button "));
        assert_eq!(attribute(control, "type"), Some("button"));
        assert_eq!(attribute(control, "role"), Some("switch"));
        assert_eq!(
            attribute(control, "aria-checked"),
            Some(if mixed {
                "mixed"
            } else if checked {
                "true"
            } else {
                "false"
            })
        );
        assert_eq!(attribute(control, "aria-label"), Some("Power"));
        assert_eq!(attribute(control, "aria-disabled"), Some("false"));
        let switch = opening_with_class(body, "ix-toggle-switch");
        assert_eq!(classes(switch).contains(&"checked"), checked);
        assert_eq!(classes(switch).contains(&"indeterminate"), mixed);
        assert!(body.contains("ix-toggle-wrapper") && body.contains("ix-toggle-slider"));
        assert!(!body.contains("<svg") && !body.contains("ix-toggle-track"));
        let text = opening_with_class(body, "ix-toggle-label");
        assert_eq!(attribute(text, "aria-hidden"), Some("true"));
        assert!(body.contains("ix-type-body"));
        assert!(body.contains(if mixed {
            "Unsure"
        } else if checked {
            "Enabled"
        } else {
            "Disabled"
        }));
    }
}

#[test]
fn accessible_name_precedence_and_hidden_state_text_remain_independent() {
    let html = render_fixture(|| {
        rsx! {
            Toggle { class: "fallback-off" }
            Toggle { class: "fallback-on", checked: true, text_on: "Enabled", text_off: "Disabled" }
            Toggle { class: "stable-label", checked: true, label: "Stable", text_on: "Enabled" }
            Toggle { class: "explicit-name", label: "Stable", aria_label: "Explicit" }
            Toggle { class: "referenced-name", label: "Stable", aria_label: "Explicit", aria_labelledby: "external-name" }
            Toggle { class: "hidden-text", checked: true, hide_text: true, label: "Stable" }
        }
    });
    for (marker, expected) in [
        ("fallback-off", "Off"),
        ("fallback-on", "Enabled"),
        ("stable-label", "Stable"),
        ("explicit-name", "Explicit"),
        ("hidden-text", "Stable"),
    ] {
        let (_, body) = field(&html, marker);
        let control = opening_with_class(body, "ix-toggle-control");
        assert_eq!(attribute(control, "aria-label"), Some(expected));
        assert!(!has_attribute(control, "aria-labelledby"));
        assert_eq!(body.contains("ix-toggle-label"), marker != "hidden-text");
    }
    let (_, referenced) = field(&html, "referenced-name");
    let control = opening_with_class(referenced, "ix-toggle-control");
    assert_eq!(attribute(control, "aria-labelledby"), Some("external-name"));
    assert!(!has_attribute(control, "aria-label"));
    assert!(referenced.contains("ix-toggle-label"));
}

#[derive(Clone)]
struct ToggleState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    calls: Rc<RefCell<Vec<String>>>,
}
fn live_toggle(state: ToggleState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let change_calls = state.calls.clone();
    let blur_calls = state.calls.clone();
    rsx! {
        Toggle {
            class: "live-toggle", checked: current == 1 || current == 3,
            indeterminate: current == 2 || current == 3,
            disabled: current == 2, required: current != 4, hide_text: current == 3,
            label: "Power", aria_label: if current == 3 { Some("Explicit power".to_string()) } else { None },
            text_on: "Running", text_off: "Stopped", text_indeterminate: "Unknown",
            name: if current == 4 { None } else { Some("power".to_string()) },
            value: if current == 4 { "updated" } else { "enabled" },
            state: match current { 1 => FieldState::Valid, 2 => FieldState::Info, 3 => FieldState::Warning, 4 => FieldState::Invalid, _ => FieldState::Default },
            oncheckedchange: move |checked| change_calls.borrow_mut().push(format!("checked:{checked}")),
            onblur: move |_| blur_calls.borrow_mut().push("blur".into()),
        }
        Checkbox { class: "checkbox-peer", label: "Checkbox" }
        Radio { class: "radio-peer", value: "radio", label: "Radio" }
    }
}

#[test]
fn external_toggle_updates_restore_form_mirror_state_and_labels_without_callbacks() {
    let state = ToggleState {
        stage: Rc::new(RefCell::new(None)),
        calls: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_toggle, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in 0..=5 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (root, body) = field(&html, "live-toggle");
        let checked = current == 1 || current == 3;
        let mixed = current == 2 || current == 3;
        let disabled = current == 2;
        let control = opening_with_class(body, "ix-toggle-control");
        let mirror = opening_with_class(body, "ix-check-native");
        assert_eq!(attribute(control, "role"), Some("switch"));
        assert_eq!(
            attribute(control, "aria-checked"),
            Some(if mixed {
                "mixed"
            } else if checked {
                "true"
            } else {
                "false"
            })
        );
        assert_eq!(
            attribute(control, "aria-disabled"),
            Some(if disabled { "true" } else { "false" })
        );
        assert_eq!(
            attribute(control, "aria-required"),
            Some(if current != 4 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(control, "aria-invalid"),
            Some(if current == 4 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-invalid"),
            Some(if current == 4 { "true" } else { "false" })
        );
        assert_eq!(has_attribute(control, "disabled"), disabled);
        assert_eq!(
            attribute(control, "aria-label"),
            Some(if current == 3 {
                "Explicit power"
            } else {
                "Power"
            })
        );
        assert_eq!(attribute(mirror, "type"), Some("checkbox"));
        assert_eq!(
            attribute(mirror, "name"),
            if current == 4 { None } else { Some("power") }
        );
        assert_eq!(
            attribute(mirror, "value"),
            Some(if current == 4 { "updated" } else { "enabled" })
        );
        assert_eq!(has_attribute(mirror, "checked"), checked);
        assert_eq!(has_attribute(mirror, "disabled"), disabled);
        assert_eq!(has_attribute(mirror, "required"), current != 4);
        assert_eq!(attribute(mirror, "aria-hidden"), Some("true"));
        assert_eq!(attribute(mirror, "tabindex"), Some("-1"));
        let expected_class = match current {
            1 => Some("ix-field-valid"),
            2 => Some("ix-field-info"),
            3 => Some("ix-field-warning"),
            4 => Some("ix-field-invalid"),
            _ => None,
        };
        if let Some(expected) = expected_class {
            assert!(classes(root).contains(&expected));
        } else {
            assert!(
                classes(root)
                    .iter()
                    .all(|name| !name.starts_with("ix-field-"))
            );
        }
        assert_eq!(body.contains("ix-toggle-label"), current != 3);
        if current != 3 {
            assert!(body.contains(if mixed {
                "Unknown"
            } else if checked {
                "Running"
            } else {
                "Stopped"
            }));
        }
        assert!(!classes(opening_with_class(&html, "checkbox-peer")).contains(&"ix-toggle-native"));
        assert!(!classes(opening_with_class(&html, "radio-peer")).contains(&"ix-toggle-native"));
        assert!(
            state.calls.borrow().is_empty(),
            "prop updates must not emit user checked/blur callbacks"
        );
    }
}

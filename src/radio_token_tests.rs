// Rendered native controls and controlled prop updates. These tests do not
// certify source cancellable event order, native navigation or computed paint.
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
fn fields(html: &str) -> Vec<(&str, &str)> {
    html.match_indices('<')
        .filter_map(|(index, _)| {
            let rest = &html[index..];
            let root = &rest[..=rest.find('>')?];
            if !classes(root).contains(&"ix-radio-native") {
                return None;
            }
            assert!(root.starts_with("<label "));
            Some((
                root,
                &rest[..rest.find("</label>").unwrap() + "</label>".len()],
            ))
        })
        .collect()
}
fn input(body: &str) -> &str {
    openings(body)
        .find(|tag| tag.starts_with("<input "))
        .unwrap()
}
fn field_for_value<'a>(html: &'a str, value: &str) -> (&'a str, &'a str) {
    fields(html)
        .into_iter()
        .find(|(_, body)| attribute(input(body), "value") == Some(value))
        .unwrap()
}
fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn radio_keeps_native_input_and_mounted_dot_with_optional_typography_label() {
    let html = render_fixture(|| {
        rsx! {
            Radio { class: "selected-radio", name: "choice", value: "first", label: "First", checked: true, required: true }
            Radio { value: "empty", label: "", aria_label: "Unlabelled choice" }
            Radio { value: "disabled", label: "Disabled", disabled: true }
            Checkbox { label: "Checkbox peer" }
            Toggle { label: "Switch peer" }
        }
    });
    assert_eq!(fields(&html).len(), 3);
    for (value, checked, disabled, label) in [
        ("first", true, false, Some("First")),
        ("empty", false, false, None),
        ("disabled", false, true, Some("Disabled")),
    ] {
        let (root, body) = field_for_value(&html, value);
        assert!(classes(root).contains(&"ix-check") && classes(root).contains(&"ix-radio"));
        assert_eq!(
            attribute(root, "data-checked"),
            Some(if checked { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-disabled"),
            Some(if disabled { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-label-less"),
            Some(if label.is_none() { "true" } else { "false" })
        );
        let native = input(body);
        assert_eq!(attribute(native, "type"), Some("radio"));
        assert_eq!(has_attribute(native, "checked"), checked);
        assert_eq!(has_attribute(native, "disabled"), disabled);
        let wrapper = opening_with_class(body, "ix-radio-button");
        assert_eq!(attribute(wrapper, "aria-hidden"), Some("true"));
        assert!(body.contains("ix-check-shape"));
        let dot = opening_with_class(body, "ix-radio-dot");
        // Paint follows the actual native :checked state, including default
        // native selection before a consumer accepts a controlled prop update.
        assert!(attribute(dot, "style").is_none());
        assert_eq!(body.matches("ix-radio-dot").count(), 1);
        assert!(!body.contains("<svg"));
        if let Some(text) = label {
            let typography = opening_with_class(body, "ix-radio-label");
            let names = classes(typography);
            assert!(names.contains(&"ix-typography") && names.contains(&"ix-type-body"));
            assert!(names.contains(&if disabled {
                "ix-text-weak"
            } else {
                "ix-text-std"
            }));
            assert_eq!(attribute(native, "aria-label"), Some(text));
            assert!(body.contains(text));
        } else {
            assert!(!body.contains("ix-radio-label"));
            assert_eq!(attribute(native, "aria-label"), Some("Unlabelled choice"));
        }
    }
    let (root, body) = field_for_value(&html, "first");
    assert!(classes(root).contains(&"selected-radio"));
    assert_eq!(attribute(input(body), "name"), Some("choice"));
    assert!(has_attribute(input(body), "required"));
    assert!(!classes(opening_with_class(&html, "ix-checkbox-native")).contains(&"ix-radio-native"));
    assert!(!classes(opening_with_class(&html, "ix-toggle")).contains(&"ix-radio-native"));
}

#[derive(Clone)]
struct LiveState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    calls: Rc<RefCell<Vec<String>>>,
}
fn live_radio(state: LiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let checked_calls = state.calls.clone();
    let value_calls = state.calls.clone();
    rsx! {
        Radio {
            class: "live-radio", value: if current == 3 { "changed" } else { "original" },
            name: if current == 3 { None } else { Some("selection".to_string()) },
            label: if current == 3 { "" } else if current == 2 { "Unavailable" } else { "Original" },
            aria_label: if current == 3 { Some("Explicit empty-label name".to_string()) } else { None },
            checked: current == 1 || current == 2, disabled: current == 2,
            required: current != 3, state: if current == 3 { FieldState::Invalid } else { FieldState::Default },
            oncheckedchange: move |checked| checked_calls.borrow_mut().push(format!("checked:{checked}")),
            onvaluechange: move |value| value_calls.borrow_mut().push(format!("value:{value}")),
        }
    }
}
fn live_state() -> LiveState {
    LiveState {
        stage: Rc::new(RefCell::new(None)),
        calls: Rc::new(RefCell::new(Vec::new())),
    }
}

#[test]
fn external_radio_updates_restore_native_properties_without_emitting_user_callbacks() {
    let state = live_state();
    let mut dom = VirtualDom::new_with_props(live_radio, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in 0..=4 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let root = opening_with_class(&html, "live-radio");
        let (actual_root, body) = fields(&html).into_iter().next().unwrap();
        assert_eq!(actual_root, root);
        let native = input(body);
        assert_eq!(
            attribute(native, "value"),
            Some(if current == 3 { "changed" } else { "original" })
        );
        assert_eq!(
            attribute(native, "name"),
            if current == 3 {
                None
            } else {
                Some("selection")
            }
        );
        assert_eq!(
            has_attribute(native, "checked"),
            current == 1 || current == 2
        );
        assert_eq!(has_attribute(native, "disabled"), current == 2);
        assert_eq!(has_attribute(native, "required"), current != 3);
        assert_eq!(
            attribute(native, "aria-invalid"),
            Some(if current == 3 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(native, "aria-label"),
            Some(match current {
                2 => "Unavailable",
                3 => "Explicit empty-label name",
                _ => "Original",
            })
        );
        assert_eq!(body.contains("ix-radio-label"), current != 3);
        assert!(body.contains("ix-radio-dot"));
        assert!(
            state.calls.borrow().is_empty(),
            "prop changes are not user change events"
        );
    }
}

fn live_group(state: LiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let calls = state.calls.clone();
    rsx! {
        RadioGroup {
            name: "shared-choice", label: "Group", required: true,
            disabled: current == 1,
            state: match current { 0 | 1 => FieldState::Invalid, 2 => FieldState::Warning, _ => FieldState::Default },
            value: match current { 0 | 1 => "a", 2 => "b", _ => "" },
            options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta").disabled()],
            onvaluechange: move |value| calls.borrow_mut().push(value),
        }
    }
}

#[test]
fn group_disabled_state_and_selection_propagate_without_widening_group_behavior() {
    let state = live_state();
    let mut dom = VirtualDom::new_with_props(live_group, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in 0..=3 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        assert_eq!(fields(&html).len(), 2);
        for value in ["a", "b"] {
            let (root, body) = field_for_value(&html, value);
            let native = input(body);
            let disabled = current == 1 || value == "b";
            let checked = (current <= 1 && value == "a") || (current == 2 && value == "b");
            assert_eq!(attribute(native, "name"), Some("shared-choice"));
            assert!(has_attribute(native, "required"));
            assert_eq!(has_attribute(native, "disabled"), disabled);
            assert_eq!(
                attribute(root, "data-disabled"),
                Some(if disabled { "true" } else { "false" })
            );
            assert_eq!(has_attribute(native, "checked"), checked);
            assert_eq!(
                attribute(native, "aria-invalid"),
                Some(if current <= 1 { "true" } else { "false" })
            );
            let names = classes(root);
            if current <= 1 {
                assert!(names.contains(&"ix-field-invalid"));
            }
            if current == 2 {
                assert!(names.contains(&"ix-field-warning"));
            }
            if current == 3 {
                assert!(
                    !names.contains(&"ix-field-invalid") && !names.contains(&"ix-field-warning")
                );
            }
            let label = opening_with_class(body, "ix-radio-label");
            assert!(classes(label).contains(&if disabled {
                "ix-text-weak"
            } else {
                "ix-text-std"
            }));
        }
        assert!(state.calls.borrow().is_empty());
    }
}

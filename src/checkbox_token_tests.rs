// SSR/live VDOM tests cover independent marks and the Rust control/form adapters.
// They do not establish computed paint, native submission, source auto-toggling,
// or actual checked-change requests.
use super::*;
use std::cell::RefCell;

fn attribute<'a>(opening: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!(" {name}=");
    let value = opening.split_once(&prefix)?.1;
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
        .find(|opening| classes(opening).contains(&marker))
        .unwrap_or_else(|| panic!("missing {marker}: {html}"))
}

fn field<'a>(html: &'a str, marker: &str) -> (&'a str, &'a str) {
    let opening = opening_with_class(html, marker);
    assert!(opening.starts_with("<span "));
    let start = html.find(opening).unwrap();
    let rest = &html[start..];
    let mut depth = 0usize;
    for (index, _) in rest.match_indices('<') {
        let tag = &rest[index..];
        if tag.starts_with("<span ") || tag.starts_with("<span>") {
            depth += 1;
        } else if tag.starts_with("</span>") {
            depth -= 1;
            if depth == 0 {
                return (opening, &rest[..index + "</span>".len()]);
            }
        }
    }
    panic!("unclosed checkbox field: {rest}");
}

fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn checked_and_mixed_are_independent_marks_with_source_svg_anatomy() {
    let html = render_fixture(|| {
        rsx! {
            for (key, checked, indeterminate) in [("off", false, false), ("checked", true, false), ("mixed", false, true), ("both", true, true)] {
                Checkbox { class: "mark-{key}", label: "Check", checked, indeterminate }
            }
        }
    });
    for (key, checked, mixed) in [
        ("off", false, false),
        ("checked", true, false),
        ("mixed", false, true),
        ("both", true, true),
    ] {
        let (root, body) = field(&html, &format!("mark-{key}"));
        assert!(classes(root).contains(&"ix-checkbox-native"));
        assert_eq!(
            attribute(root, "data-checked"),
            Some(if checked { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-mixed"),
            Some(if mixed { "true" } else { "false" })
        );
        let button = opening_with_class(body, "ix-checkbox");
        assert!(button.starts_with("<button "));
        assert_eq!(attribute(button, "role"), Some("checkbox"));
        assert_eq!(
            attribute(button, "aria-checked"),
            Some(if mixed {
                "mixed"
            } else if checked {
                "true"
            } else {
                "false"
            })
        );
        assert!(body.contains("ix-checkbox-button") && body.contains("ix-check-shape"));
        let svg = openings(body).find(|tag| tag.starts_with("<svg ")).unwrap();
        assert_eq!(attribute(svg, "width"), Some("18"));
        assert_eq!(attribute(svg, "height"), Some("18"));
        assert_eq!(attribute(svg, "viewBox"), Some("0 0 18 18"));
        assert_eq!(attribute(svg, "fill"), Some("none"));
        let rectangles: Vec<_> = openings(body)
            .filter(|tag| tag.starts_with("<rect "))
            .collect();
        assert_eq!(rectangles.len(), if mixed { 2 } else { 0 });
        if mixed {
            assert_eq!(attribute(rectangles[0], "width"), Some("18"));
            assert_eq!(attribute(rectangles[0], "height"), Some("18"));
            assert_eq!(attribute(rectangles[0], "fill"), Some("transparent"));
            for (name, value) in [
                ("x", "3"),
                ("y", "8"),
                ("width", "12"),
                ("height", "2"),
                ("fill", "var(--ix-checkbox-checkmark--color)"),
            ] {
                assert_eq!(attribute(rectangles[1], name), Some(value));
            }
        }
        let paths: Vec<_> = openings(body)
            .filter(|tag| tag.starts_with("<path "))
            .collect();
        assert_eq!(paths.len(), usize::from(checked));
        if checked {
            assert_eq!(
                attribute(paths[0], "d"),
                Some("M3.65625 8.15625L8.4375 12.9375L14.625 3.9375")
            );
            assert_eq!(
                attribute(paths[0], "stroke"),
                Some("var(--ix-checkbox-checkmark--color)")
            );
            assert_eq!(attribute(paths[0], "stroke-width"), Some("2"));
        }
    }
}

#[test]
fn validation_metadata_and_checkbox_scope_preserve_peer_radio_and_switch() {
    let html = render_fixture(|| {
        rsx! {
            Checkbox { class: "default-state", label: "Default" }
            Checkbox { class: "valid-state", label: "Valid", state: FieldState::Valid }
            Checkbox { class: "info-state", label: "Info", state: FieldState::Info }
            Checkbox { class: "warning-state", label: "Warning", state: FieldState::Warning }
            Checkbox { class: "invalid-state", label: "Invalid", state: FieldState::Invalid }
            Checkbox { class: "info-invalid", label: "Info invalid", state: FieldState::Info, invalid: true }
            Checkbox { class: "warning-invalid", label: "Warning invalid", state: FieldState::Warning, invalid: true }
            Radio { label: "Radio peer", value: "radio" }
            Toggle { label: "Switch peer" }
        }
    });
    for (marker, state_class, invalid) in [
        ("default-state", None, false),
        ("valid-state", Some("ix-field-valid"), false),
        ("info-state", Some("ix-field-info"), false),
        ("warning-state", Some("ix-field-warning"), false),
        ("invalid-state", Some("ix-field-invalid"), true),
        ("info-invalid", Some("ix-field-info"), true),
        ("warning-invalid", Some("ix-field-warning"), true),
    ] {
        let (root, body) = field(&html, marker);
        if let Some(expected) = state_class {
            assert!(classes(root).contains(&expected));
        }
        assert_eq!(
            attribute(root, "data-invalid"),
            Some(if invalid { "true" } else { "false" })
        );
        assert_eq!(
            attribute(opening_with_class(body, "ix-checkbox"), "aria-invalid"),
            Some(if invalid { "true" } else { "false" })
        );
        let label = opening_with_class(body, "ix-checkbox-label");
        assert!(
            classes(label).contains(&"ix-typography") && classes(label).contains(&"ix-type-body")
        );
        assert!(classes(label).contains(&"ix-text-std"));
    }
    assert_eq!(html.matches("ix-checkbox-native").count(), 7);
    assert!(!classes(opening_with_class(&html, "ix-radio")).contains(&"ix-checkbox-native"));
    assert!(!classes(opening_with_class(&html, "ix-toggle")).contains(&"ix-checkbox-native"));
}

#[derive(Clone)]
struct CheckboxLifecycle(Rc<RefCell<Option<Signal<u8>>>>);

fn live_checkbox(state: CheckboxLifecycle) -> Element {
    let stage = use_signal(|| 0u8);
    *state.0.borrow_mut() = Some(stage);
    let current = stage();
    rsx! {
        Checkbox {
            class: "live-checkbox", checked: current == 1 || current == 2,
            indeterminate: current == 2 || current == 3, disabled: current == 2,
            required: current != 4, invalid: current == 4,
            state: if current == 4 { FieldState::Warning } else { FieldState::Default },
            name: if current == 4 { None } else { Some("consent".to_string()) },
            value: if current == 2 { "updated" } else { "consent" },
            label: if current == 3 { "" } else if current == 2 { "Disabled" } else { "Accept" },
            aria_label: if current == 3 { Some("Accept without label".to_string()) } else { None },
        }
    }
}

#[test]
fn live_checkbox_restores_marks_label_validation_and_native_form_mirror() {
    let state = CheckboxLifecycle(Rc::new(RefCell::new(None)));
    let mut dom = VirtualDom::new_with_props(live_checkbox, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.0.borrow().unwrap();
    for current in 0..=5 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (root, body) = field(&html, "live-checkbox");
        let checked = current == 1 || current == 2;
        let mixed = current == 2 || current == 3;
        let disabled = current == 2;
        let button = opening_with_class(body, "ix-checkbox");
        let input = opening_with_class(body, "ix-check-native");
        assert_eq!(
            attribute(root, "data-checked"),
            Some(if checked { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-mixed"),
            Some(if mixed { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-label-less"),
            Some(if current == 3 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-invalid"),
            Some(if current == 4 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(button, "aria-checked"),
            Some(if mixed {
                "mixed"
            } else if checked {
                "true"
            } else {
                "false"
            })
        );
        assert_eq!(
            attribute(button, "aria-disabled"),
            Some(if disabled { "true" } else { "false" })
        );
        assert_eq!(
            attribute(button, "aria-required"),
            Some(if current != 4 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(button, "aria-invalid"),
            Some(if current == 4 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(button, "aria-label"),
            if current == 3 {
                Some("Accept without label")
            } else {
                None
            }
        );
        assert_eq!(attribute(button, "type"), Some("button"));
        assert!(
            !has_attribute(button, "tabindex"),
            "source inner control has no blocked tabindex override"
        );
        assert_eq!(has_attribute(button, "disabled"), disabled);
        assert_eq!(attribute(input, "type"), Some("checkbox"));
        assert_eq!(
            attribute(input, "name"),
            if current == 4 { None } else { Some("consent") }
        );
        assert_eq!(
            attribute(input, "value"),
            Some(if current == 2 { "updated" } else { "consent" })
        );
        assert_eq!(has_attribute(input, "checked"), checked);
        assert_eq!(has_attribute(input, "disabled"), disabled);
        assert_eq!(has_attribute(input, "required"), current != 4);
        assert_eq!(attribute(input, "aria-hidden"), Some("true"));
        assert_eq!(attribute(input, "tabindex"), Some("-1"));
        let label = opening_with_class(body, "ix-checkbox-label");
        assert!(classes(label).contains(&if disabled {
            "ix-text-weak"
        } else {
            "ix-text-std"
        }));
        if current != 3 {
            assert!(body.contains(if disabled { "Disabled" } else { "Accept" }));
        }
        assert_eq!(
            openings(body)
                .filter(|tag| tag.starts_with("<path "))
                .count(),
            usize::from(checked)
        );
        assert_eq!(
            openings(body)
                .filter(|tag| tag.starts_with("<rect "))
                .count(),
            if mixed { 2 } else { 0 }
        );
    }
}

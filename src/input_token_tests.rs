// Rendering and controlled-prop regressions; native input, focus and computed styles
// are verified by the paired runtime fixture, rather than inferred from SSR.
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
fn part<'a>(html: &'a str, marker: &str) -> (&'a str, &'a str) {
    let (start, opening) = html
        .match_indices('<')
        .find_map(|(index, _)| {
            let rest = &html[index..];
            let opening = &rest[..=rest.find('>')?];
            classes(opening)
                .contains(&marker)
                .then_some((index, opening))
        })
        .unwrap_or_else(|| panic!("missing {marker}: {html}"));
    let tag = opening[1..]
        .split(|c: char| c.is_whitespace() || c == '>')
        .next()
        .unwrap();
    let closing = format!("</{tag}>");
    let rest = &html[start..];
    let mut depth = 0usize;
    for (index, _) in rest.match_indices('<') {
        let current = &rest[index..];
        if current.starts_with(&format!("<{tag} ")) || current.starts_with(&format!("<{tag}>")) {
            depth += 1;
        } else if current.starts_with(&closing) {
            depth -= 1;
            if depth == 0 {
                return (opening, &rest[..index + closing.len()]);
            }
        }
    }
    panic!("unclosed {marker}: {rest}");
}
fn text_content(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text
}
fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn text_fields_render_shared_labels_feedback_and_utf16_counters_without_peer_leakage() {
    let html = render_fixture(|| {
        rsx! {
            for (name, state) in [("default", FieldState::Default), ("valid", FieldState::Valid), ("info", FieldState::Info), ("warning", FieldState::Warning), ("invalid", FieldState::Invalid)] {
                Input { id: "input-{name}", class: "input-{name}", label: "Value", value: "a😀", required: true, max_length: 9, state,
                    helper_text: "  helper  ", valid_text: "  valid  ", info_text: "  info  ", warning_text: "  warning  ", invalid_text: "  invalid  " }
                Textarea { id: "textarea-{name}", class: "textarea-{name}", label: "Notes", value: "a😀", required: true, max_length: 9, state,
                    helper_text: "  helper  ", valid_text: "  valid  ", info_text: "  info  ", warning_text: "  warning  ", invalid_text: "  invalid  " }
            }
            Checkbox { class: "checkbox-peer", label: "Checkbox" }
            Radio { class: "radio-peer", label: "Radio", value: "peer" }
        }
    });
    for kind in ["input", "textarea"] {
        for (state, expected, color) in [
            ("default", "  helper  ", "ix-text-soft"),
            ("valid", "  valid  ", "ix-text-std"),
            ("info", "  info  ", "ix-text-std"),
            ("warning", "  warning  ", "ix-text-std"),
            ("invalid", "  invalid  ", "ix-text-alarm"),
        ] {
            let id = format!("{kind}-{state}");
            let (root, body) = part(&html, &id);
            assert!(classes(root).contains(&"ix-text-field-native"));
            assert!(classes(root).contains(&if kind == "input" {
                "ix-input-native"
            } else {
                "ix-textarea-native"
            }));
            assert_eq!(attribute(root, "data-disabled"), Some("false"));
            assert_eq!(attribute(root, "data-readonly"), Some("false"));
            for marker in [
                "ix-text-field-wrapper",
                "ix-text-field-top",
                "ix-text-field-slot",
                "ix-input-wrap",
                "ix-text-field-bottom",
                "ix-text-field-bottom-right",
            ] {
                part(body, marker);
            }
            let (_, label) = part(body, "ix-text-field-label");
            let native_label = openings(label)
                .find(|opening| opening.starts_with("<label "))
                .unwrap();
            assert_eq!(attribute(native_label, "for"), Some(id.as_str()));
            assert_eq!(
                text_content(label),
                if kind == "input" { "Value*" } else { "Notes*" }
            );
            let (label_type, _) = part(label, "ix-typography");
            assert!(classes(label_type).contains(&"ix-type-body"));
            assert!(classes(label_type).contains(&if state == "invalid" {
                "ix-text-alarm"
            } else {
                "ix-text-soft"
            }));
            let native = openings(body)
                .find(|opening| attribute(opening, "id") == Some(id.as_str()))
                .unwrap();
            assert!(native.starts_with(&format!("<{kind} ")));
            assert!(has_attribute(native, "required"));
            assert_eq!(attribute(native, "maxlength"), Some("9"));
            assert_eq!(
                attribute(native, "aria-invalid"),
                Some(if state == "invalid" { "true" } else { "false" })
            );
            let help = format!("{id}-help");
            assert_eq!(attribute(native, "aria-describedby"), Some(help.as_str()));
            assert_eq!(
                attribute(native, "aria-errormessage"),
                if state == "invalid" {
                    Some(help.as_str())
                } else {
                    None
                }
            );
            let (feedback, feedback_body) = part(body, "ix-text-field-bottom-text");
            assert_eq!(attribute(feedback, "id"), Some(help.as_str()));
            assert!(classes(feedback).contains(&"ix-type-body-paragraph"));
            assert!(classes(feedback).contains(&color));
            assert_eq!(text_content(feedback_body), expected);
            assert_eq!(
                feedback_body.contains("ix-text-field-text-icon"),
                state != "default"
            );
            let (counter, counter_body) = part(body, "ix-text-field-counter");
            assert!(classes(counter).contains(&"ix-type-body"));
            assert!(classes(counter).contains(&"ix-text-soft"));
            assert_eq!(
                text_content(counter_body),
                "3/9",
                "counter uses UTF-16 units"
            );
            if kind == "input" {
                part(body, "ix-input-start");
                part(body, "ix-input-end");
            }
        }
    }
    for peer in ["checkbox-peer", "radio-peer"] {
        assert!(!classes(part(&html, peer).0).contains(&"ix-text-field-native"));
    }
}

#[test]
fn text_field_native_constraints_rows_omission_and_password_adornment_are_preserved() {
    let html = render_fixture(|| {
        rsx! {
            Input { class: "constraints", id: "account", name: "account-name", input_type: "email", value: "a@example.com", placeholder: "Email", aria_label: "Account",
                required: true, min_length: 2, max_length: 48, pattern: ".+@.+", autocomplete: "email",
                start: rsx! { crate::Icon { name: "search", size: 24 } }, end: rsx! { span { "unit" } } }
            Input { class: "password-ready", input_type: "password", readonly: true }
            Input { class: "password-disabled", input_type: "password", disabled: true }
            Input { class: "empty-input", label: "", helper_text: "  ", max_length: 0 }
            Textarea { class: "default-rows", name: "notes", value: "first\nsecond" }
            Textarea { class: "explicit-rows", rows: 7, cols: 19, width: "23rem", height: "11rem", resize: "horizontal", readonly: true }
        }
    });
    let (_, constrained) = part(&html, "constraints");
    let native = openings(constrained)
        .find(|opening| opening.starts_with("<input "))
        .unwrap();
    for (name, value) in [
        ("id", "account"),
        ("name", "account-name"),
        ("type", "email"),
        ("value", "a@example.com"),
        ("placeholder", "Email"),
        ("aria-label", "Account"),
        ("minlength", "2"),
        ("maxlength", "48"),
        ("pattern", ".+@.+"),
        ("autocomplete", "email"),
    ] {
        assert_eq!(attribute(native, name), Some(value));
    }
    assert!(has_attribute(native, "required"));
    assert!(part(constrained, "ix-input-start").1.contains("ix-icon"));
    assert_eq!(text_content(part(constrained, "ix-input-end").1), "unit");
    let (_, password) = part(&html, "password-ready");
    let password_input = openings(password)
        .find(|opening| opening.starts_with("<input "))
        .unwrap();
    assert!(has_attribute(password_input, "readonly"));
    assert_eq!(attribute(password_input, "type"), Some("password"));
    let (eye, eye_body) = part(password, "ix-input-password-eye");
    assert!(eye.starts_with("<button "));
    assert!(classes(eye).contains(&"ix-icon-button-native"));
    assert!(classes(eye).contains(&"ix-icon-size-16"));
    assert!(classes(eye).contains(&"ix-button-tertiary"));
    assert_eq!(attribute(eye, "type"), Some("button"));
    assert_eq!(attribute(eye, "aria-label"), Some("Show password"));
    assert_eq!(attribute(eye, "aria-pressed"), Some("false"));
    assert!(eye_body.contains(crate::icons::icon_svg("eye").unwrap()));
    assert!(!password.contains("ix-password-toggle"));
    assert!(
        !part(&html, "password-disabled")
            .1
            .contains("ix-input-password-eye")
    );
    let (_, empty) = part(&html, "empty-input");
    assert!(!empty.contains("ix-text-field-top"));
    assert!(!empty.contains("ix-text-field-bottom-text"));
    assert!(!empty.contains("ix-text-field-counter"));
    part(empty, "ix-text-field-bottom");
    part(empty, "ix-text-field-bottom-right");
    let (_, default_rows) = part(&html, "default-rows");
    let textarea = openings(default_rows)
        .find(|opening| opening.starts_with("<textarea "))
        .unwrap();
    assert!(
        !has_attribute(textarea, "rows"),
        "omission lets the source CSS choose height"
    );
    assert!(!has_attribute(textarea, "cols"));
    assert!(
        attribute(textarea, "style")
            .unwrap()
            .contains("resize:both")
    );
    assert_eq!(attribute(textarea, "name"), Some("notes"));
    assert_eq!(attribute(textarea, "value"), Some("first\nsecond"));
    let (_, explicit_rows) = part(&html, "explicit-rows");
    let textarea = openings(explicit_rows)
        .find(|opening| opening.starts_with("<textarea "))
        .unwrap();
    assert_eq!(attribute(textarea, "rows"), Some("7"));
    assert_eq!(attribute(textarea, "cols"), Some("19"));
    assert!(has_attribute(textarea, "readonly"));
    let style = attribute(textarea, "style").unwrap();
    for declaration in ["width:23rem", "height:11rem", "resize:horizontal"] {
        assert!(style.contains(declaration));
    }
    assert!(!style.contains("max-width"));
}

#[derive(Clone)]
struct TextFieldLiveState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    calls: Rc<RefCell<Vec<String>>>,
}
fn live_text_fields(state: TextFieldLiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let blocked = current == 1;
    let read_only = current == 2;
    let value = if current == 0 {
        "initial"
    } else if current == 3 {
        "restored"
    } else {
        "a😀"
    };
    let calls = state.calls.clone();
    let commits = state.calls.clone();
    let blurs = state.calls.clone();
    let area_calls = state.calls.clone();
    let area_commits = state.calls.clone();
    rsx! {
        Input { class: "live-input", id: "live-input", name: "entry", value, label: if blocked { None } else { Some("Entry".to_string()) },
            input_type: if current == 3 { "text" } else { "password" }, disabled: blocked, readonly: read_only,
            state: if read_only { FieldState::Invalid } else { FieldState::Default }, invalid_text: "Check entry", helper_text: "Hint",
            max_length: if current == 3 { None } else { Some(20) },
            start: if current == 0 { Some(rsx! { span { "prefix" } }) } else { None },
            end: if current == 3 { Some(rsx! { span { "suffix" } }) } else { None },
            onvaluechange: move |value| calls.borrow_mut().push(format!("edit:{value}")),
            onchange: move |value| commits.borrow_mut().push(format!("commit:{value}")),
            onblur: move |_| blurs.borrow_mut().push("blur".into()),
        }
        Textarea { class: "live-area", id: "live-area", name: "notes", value, disabled: blocked, readonly: read_only,
            rows: if current == 1 { Some(8) } else { None }, cols: if current == 1 { Some(25) } else { None },
            width: if current == 1 { Some("24rem".to_string()) } else { None },
            height: if current == 1 { Some("12rem".to_string()) } else { None },
            resize: if current == 2 { "none" } else { "vertical" },
            onvaluechange: move |value| area_calls.borrow_mut().push(format!("area-edit:{value}")),
            onchange: move |value| area_commits.borrow_mut().push(format!("area-commit:{value}")),
        }
    }
}

#[test]
fn text_fields_restore_live_controlled_values_constraints_and_adornments_without_requests() {
    let state = TextFieldLiveState {
        stage: Rc::new(RefCell::new(None)),
        calls: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_text_fields, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in [0, 1, 2, 3, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let (root, body) = part(&html, "live-input");
        let input = openings(body)
            .find(|opening| opening.starts_with("<input "))
            .unwrap();
        let value = if current == 0 {
            "initial"
        } else if current == 3 {
            "restored"
        } else {
            "a😀"
        };
        assert_eq!(attribute(input, "value"), Some(value));
        assert_eq!(attribute(input, "name"), Some("entry"));
        assert_eq!(
            attribute(root, "data-disabled"),
            Some(if current == 1 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(root, "data-readonly"),
            Some(if current == 2 { "true" } else { "false" })
        );
        assert_eq!(has_attribute(input, "disabled"), current == 1);
        assert_eq!(has_attribute(input, "readonly"), current == 2);
        assert_eq!(
            attribute(input, "type"),
            Some(if current == 3 { "text" } else { "password" })
        );
        assert_eq!(
            attribute(input, "aria-invalid"),
            Some(if current == 2 { "true" } else { "false" })
        );
        assert_eq!(body.contains("ix-text-field-label"), current != 1);
        assert_eq!(
            body.contains("ix-input-password-eye"),
            current != 1 && current != 3
        );
        assert_eq!(
            text_content(part(body, "ix-input-start").1),
            if current == 0 { "prefix" } else { "" }
        );
        assert_eq!(
            part(body, "ix-input-end").1.contains("suffix"),
            current == 3
        );
        assert_eq!(body.contains("ix-text-field-counter"), current != 3);
        if current != 3 {
            assert_eq!(
                text_content(part(body, "ix-text-field-counter").1),
                if current == 0 { "7/20" } else { "3/20" }
            );
        }
        let (_, area_body) = part(&html, "live-area");
        let area = openings(area_body)
            .find(|opening| opening.starts_with("<textarea "))
            .unwrap();
        assert_eq!(
            attribute(area, "rows"),
            if current == 1 { Some("8") } else { None }
        );
        assert_eq!(
            attribute(area, "cols"),
            if current == 1 { Some("25") } else { None }
        );
        assert_eq!(has_attribute(area, "disabled"), current == 1);
        assert_eq!(has_attribute(area, "readonly"), current == 2);
        assert_eq!(attribute(area, "value"), Some(value));
        let style = attribute(area, "style").unwrap();
        assert_eq!(style.contains("width:24rem"), current == 1);
        assert_eq!(style.contains("height:12rem"), current == 1);
        assert!(style.contains(if current == 2 {
            "resize:none"
        } else {
            "resize:vertical"
        }));
        assert!(
            state.calls.borrow().is_empty(),
            "controlled prop updates are not native edits, commits or blur events"
        );
    }
}

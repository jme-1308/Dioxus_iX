// Group rendering and controlled native-option adapters. These tests do not
// prove source descendant validation, focus, tooltip, roving or event parity.
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
fn both_group_kinds_render_required_labels_and_state_specific_inline_feedback() {
    let html = render_fixture(|| {
        rsx! {
            for (name, state) in [("default", FieldState::Default), ("valid", FieldState::Valid), ("info", FieldState::Info), ("warning", FieldState::Warning), ("invalid", FieldState::Invalid)] {
                CheckboxGroup {
                    class: "checks-{name}", label: "Checks", required: true, state,
                    helper_text: "  helper  ", valid_text: "  valid  ", info_text: "  info  ", warning_text: "  warning  ", invalid_text: "  invalid  ",
                    Checkbox { label: "Choice" }
                }
                RadioGroup {
                    class: "radios-{name}", label: "Radios", required: true, state,
                    helper_text: "  helper  ", valid_text: "  valid  ", info_text: "  info  ", warning_text: "  warning  ", invalid_text: "  invalid  ",
                    options: Vec::<SelectOption>::new(),
                }
            }
        }
    });
    for kind in ["checks", "radios"] {
        for (state, expected, color, icon) in [
            ("default", "  helper  ", "ix-text-soft", None),
            ("valid", "  valid  ", "ix-text-std", Some("success")),
            ("info", "  info  ", "ix-text-std", Some("info")),
            ("warning", "  warning  ", "ix-text-std", Some("warning")),
            ("invalid", "  invalid  ", "ix-text-alarm", Some("error")),
        ] {
            let (root, group) = part(&html, &format!("{kind}-{state}"));
            assert!(root.starts_with("<fieldset "));
            assert!(classes(root).contains(&"ix-group-native"));
            assert!(classes(root).contains(&if kind == "checks" {
                "ix-checkbox-group"
            } else {
                "ix-radio-group"
            }));
            for marker in [
                "ix-group-wrapper",
                "ix-group-top",
                "ix-group-slot",
                "ix-group-choices",
                "ix-group-bottom",
                "ix-group-bottom-right",
            ] {
                assert!(group.contains(marker));
            }
            let (label, label_body) = part(group, "ix-group-label");
            assert_eq!(attribute(label, "aria-hidden"), Some("true"));
            let (typography, _) = part(label_body, "ix-typography");
            assert!(classes(typography).contains(&"ix-type-body"));
            assert!(classes(typography).contains(&if state == "invalid" {
                "ix-text-alarm"
            } else {
                "ix-text-soft"
            }));
            assert_eq!(
                text_content(label_body),
                format!("{}*", if kind == "checks" { "Checks" } else { "Radios" })
            );
            assert_eq!(
                attribute(root, "aria-label"),
                Some(if kind == "checks" { "Checks" } else { "Radios" })
            );
            let (wrapper, _) = part(group, "ix-group-wrapper");
            assert_eq!(
                attribute(wrapper, "role"),
                if state == "invalid" {
                    Some("alert")
                } else {
                    None
                }
            );
            assert_eq!(
                attribute(wrapper, "aria-live"),
                if state == "invalid" {
                    Some("polite")
                } else {
                    None
                }
            );
            let (feedback, feedback_body) = part(group, "ix-group-bottom-text");
            assert!(classes(feedback).contains(&"ix-type-body-paragraph"));
            assert!(classes(feedback).contains(&color));
            assert_eq!(
                text_content(feedback_body),
                expected,
                "preserve supplied text whitespace"
            );
            if let Some(name) = icon {
                let (icon, _) = part(feedback_body, "ix-group-text-icon");
                assert!(icon.starts_with("<svg "));
                assert_eq!(attribute(icon, "width"), Some("16"));
                assert_eq!(attribute(icon, "height"), Some("16"));
                assert_eq!(attribute(icon, "aria-hidden"), Some("true"));
                assert!(feedback_body.contains(crate::icons::icon_svg(name).unwrap()));
            } else {
                assert!(!feedback_body.contains("ix-group-text-icon"));
            }
        }
    }
}

#[test]
fn blank_or_inactive_feedback_falls_back_neutrally_and_bottom_containers_stay_mounted() {
    let html = render_fixture(|| {
        rsx! {
            CheckboxGroup { class: "blank-checks", required: true, disabled: true, aria_label: "Named checks", state: FieldState::Invalid, invalid_text: " \t ", helper_text: "  fallback  ", span { "Child" } }
            RadioGroup { class: "blank-radios", required: true, aria_labelledby: "external-radios", state: FieldState::Valid, valid_text: " \n ", helper_text: "  fallback  ", options: Vec::<SelectOption>::new() }
            CheckboxGroup { class: "inactive-text", invalid_text: "Unused error", valid_text: "Unused success", info_text: "Unused info", warning_text: "Unused warning", span { "Child" } }
            RadioGroup { class: "all-blank", state: FieldState::Invalid, invalid_text: "", helper_text: " \t ", options: Vec::<SelectOption>::new() }
        }
    });
    for marker in ["blank-checks", "blank-radios", "inactive-text", "all-blank"] {
        let (root, group) = part(&html, marker);
        assert!(!group.contains("ix-group-top") && !group.contains("ix-group-label"));
        assert!(group.contains("ix-group-bottom") && group.contains("ix-group-bottom-right"));
        assert!(!group.contains("ix-group-text-icon"));
        let (wrapper, _) = part(group, "ix-group-wrapper");
        assert_eq!(
            attribute(wrapper, "role"),
            if marker == "blank-checks" {
                Some("alert")
            } else {
                None
            }
        );
        if marker.starts_with("blank-") {
            let (feedback, body) = part(group, "ix-group-bottom-text");
            assert!(classes(feedback).contains(&"ix-text-soft"));
            assert_eq!(text_content(body), "  fallback  ");
        } else {
            assert!(!group.contains("ix-group-bottom-text"));
        }
        if marker == "blank-checks" {
            assert!(has_attribute(root, "disabled"));
            assert_eq!(attribute(root, "aria-label"), Some("Named checks"));
        }
        if marker == "blank-radios" {
            assert_eq!(attribute(root, "aria-labelledby"), Some("external-radios"));
        }
    }
}

#[derive(Clone)]
struct GroupState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    calls: Rc<RefCell<Vec<String>>>,
}
fn live_groups(state: GroupState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let check_calls = state.calls.clone();
    let radio_calls = state.calls.clone();
    let feedback = match current {
        1 => FieldState::Info,
        2 => FieldState::Invalid,
        3 => FieldState::Warning,
        _ => FieldState::Default,
    };
    let mut options = vec![
        SelectOption::new("a", "Alpha"),
        SelectOption::new("b", "Beta").disabled(),
    ];
    if current == 3 {
        options.push(SelectOption::new("c", "Gamma"));
    }
    rsx! {
        CheckboxGroup {
            class: "live-checks", label: if current == 3 { "" } else { "Features" }, required: current == 1 || current == 2,
            disabled: current == 2, horizontal: current == 1, state: feedback,
            helper_text: "Help", info_text: "Information", invalid_text: "Problem", warning_text: "Caution",
            Checkbox { name: "features", value: "a", checked: current == 1 || current == 2, label: "Feature A", oncheckedchange: move |value| check_calls.borrow_mut().push(format!("check:{value}")) }
            Checkbox { name: "features", value: "b", checked: current == 3, disabled: true, label: "Feature B" }
        }
        RadioGroup {
            class: "live-radios", label: if current == 3 { "" } else { "Selection" }, required: true,
            disabled: current == 2, horizontal: current == 1, state: feedback,
            name: if current == 3 { Some("renamed".to_string()) } else { None },
            value: match current { 1 | 2 => "b", 3 => "c", _ => "a" }, options,
            helper_text: "Help", info_text: "Information", invalid_text: "Problem", warning_text: "Caution",
            onvaluechange: move |value| radio_calls.borrow_mut().push(value),
        }
    }
}
fn inputs<'a>(group: &'a str, kind: &str) -> Vec<&'a str> {
    openings(group)
        .filter(|tag| tag.starts_with("<input ") && attribute(tag, "type") == Some(kind))
        .collect()
}

#[test]
fn live_groups_preserve_native_children_names_and_controlled_options_through_layout_updates() {
    let state = GroupState {
        stage: Rc::new(RefCell::new(None)),
        calls: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_groups, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    let mut original_radio_name = None::<String>;
    for current in 0..=4 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (check_root, checks) = part(&html, "live-checks");
        let (radio_root, radios) = part(&html, "live-radios");
        assert_eq!(has_attribute(check_root, "disabled"), current == 2);
        assert_eq!(has_attribute(radio_root, "disabled"), current == 2);
        for group in [checks, radios] {
            assert_eq!(group.contains("ix-group-label"), current != 3);
            let (choices, _) = part(group, "ix-group-choices");
            assert_eq!(classes(choices).contains(&"ix-group-row"), current == 1);
            let (_, bottom) = part(group, "ix-group-bottom-text");
            assert_eq!(
                text_content(bottom),
                match current {
                    1 => "Information",
                    2 => "Problem",
                    3 => "Caution",
                    _ => "Help",
                }
            );
            assert!(group.contains("ix-group-bottom-right"));
        }
        if current != 3 {
            let (_, label) = part(checks, "ix-group-label");
            assert_eq!(
                text_content(label),
                if current == 1 || current == 2 {
                    "Features*"
                } else {
                    "Features"
                }
            );
        }
        let checkbox_inputs = inputs(checks, "checkbox");
        assert_eq!(checkbox_inputs.len(), 2);
        for native in checkbox_inputs {
            let value = attribute(native, "value").unwrap();
            assert_eq!(attribute(native, "name"), Some("features"));
            assert_eq!(
                has_attribute(native, "checked"),
                if value == "a" {
                    current == 1 || current == 2
                } else {
                    current == 3
                }
            );
            assert!(
                !has_attribute(native, "required"),
                "group marker does not invent child required props"
            );
        }
        let radio_inputs = inputs(radios, "radio");
        assert_eq!(radio_inputs.len(), if current == 3 { 3 } else { 2 });
        if original_radio_name.is_none() {
            original_radio_name = Some(attribute(radio_inputs[0], "name").unwrap().to_string());
        }
        let expected_name = if current == 3 {
            "renamed"
        } else {
            original_radio_name.as_deref().unwrap()
        };
        assert!(!expected_name.is_empty());
        for native in radio_inputs {
            let value = attribute(native, "value").unwrap();
            assert_eq!(attribute(native, "name"), Some(expected_name));
            assert_eq!(
                has_attribute(native, "checked"),
                value
                    == match current {
                        1 | 2 => "b",
                        3 => "c",
                        _ => "a",
                    }
            );
            assert_eq!(
                has_attribute(native, "disabled"),
                current == 2 || value == "b"
            );
            assert!(has_attribute(native, "required"));
        }
        assert!(
            state.calls.borrow().is_empty(),
            "external group/option updates do not emit user callbacks"
        );
    }
}

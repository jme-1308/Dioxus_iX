// Select rendering and live controlled-prop regressions. These checks do not
// establish browser geometry, focus, trusted input, or computed token parity.
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
fn options() -> Vec<SelectOption> {
    vec![
        SelectOption::new("a", "Alpha"),
        SelectOption::new("b", "Beta"),
        SelectOption::new("c", "Gamma").disabled(),
    ]
}
fn native_input(html: &str) -> &str {
    openings(html)
        .find(|opening| {
            opening.starts_with("<input ") && attribute(opening, "type") == Some("text")
        })
        .expect("visible native text input")
}
fn selected_native_values(html: &str) -> Vec<&str> {
    let (_, native) = part(html, "ix-select-native");
    openings(native)
        .filter(|opening| opening.starts_with("<option ") && has_attribute(opening, "selected"))
        .map(|opening| attribute(opening, "value").unwrap())
        .collect()
}

#[test]
fn select_native_fields_preserve_submission_values_and_readonly_disabled_constraints() {
    let html = render_fixture(|| {
        rsx! {
            for (name, disabled, readonly) in [("ready", false, false), ("disabled", true, false), ("readonly", false, true), ("both", true, true)] {
                div { class: "case-{name}",
                    Select { id: "choice-{name}", name: "submission-{name}", value: "b", options: options(),
                        label: "Selection", required: true, disabled, readonly, allow_clear: true }
                }
            }
            div { class: "multiple-case",
                Select { id: "many", name: "selections", multiple: true, values: vec!["a".into(), "c".into()], options: options(), required: true }
            }
        }
    });
    for (name, disabled, readonly) in [
        ("ready", false, false),
        ("disabled", true, false),
        ("readonly", false, true),
        ("both", true, true),
    ] {
        let (_, body) = part(&html, &format!("case-{name}"));
        let input = native_input(body);
        assert_eq!(
            attribute(input, "id"),
            Some(format!("choice-{name}").as_str())
        );
        assert_eq!(attribute(input, "role"), Some("combobox"));
        assert_eq!(attribute(input, "value"), Some("Beta"));
        assert_eq!(attribute(input, "autocomplete"), Some("off"));
        assert_eq!(attribute(input, "aria-autocomplete"), Some("list"));
        assert_eq!(attribute(input, "aria-expanded"), Some("false"));
        assert_eq!(attribute(input, "aria-required"), Some("true"));
        assert_eq!(has_attribute(input, "disabled"), disabled);
        assert_eq!(has_attribute(input, "readonly"), readonly);
        let (native, _) = part(body, "ix-select-native");
        assert!(native.starts_with("<select "));
        assert_eq!(
            attribute(native, "name"),
            Some(format!("submission-{name}").as_str())
        );
        assert_eq!(attribute(native, "tabindex"), Some("-1"));
        assert_eq!(attribute(native, "aria-hidden"), Some("true"));
        assert_eq!(has_attribute(native, "required"), !readonly);
        assert_eq!(has_attribute(native, "disabled"), disabled);
        assert_eq!(selected_native_values(body), ["b"]);
        assert_eq!(body.contains("ix-select-clear"), !disabled && !readonly);
        assert_eq!(body.contains("ix-select-chevron"), !disabled && !readonly);
    }
    let (_, body) = part(&html, "multiple-case");
    let (native, native_body) = part(body, "ix-select-native");
    assert!(has_attribute(native, "multiple"));
    assert_eq!(selected_native_values(body), ["a", "c"]);
    let disabled_option = openings(native_body)
        .find(|opening| opening.starts_with("<option ") && attribute(opening, "value") == Some("c"))
        .unwrap();
    assert!(has_attribute(disabled_option, "disabled"));
    assert!(
        has_attribute(disabled_option, "selected"),
        "source permits a disabled option in external selection"
    );
}

#[test]
fn select_shared_field_and_trigger_layers_contain_real_source_variant_icon_buttons() {
    let html = render_fixture(|| {
        rsx! {
            for (name, state) in [("default", FieldState::Default), ("valid", FieldState::Valid), ("info", FieldState::Info), ("warning", FieldState::Warning), ("invalid", FieldState::Invalid)] {
                div { class: "semantic-{name}",
                    Select { id: "semantic-input-{name}", value: "b", options: options(), label: "Selection", required: true,
                        allow_clear: true, state, helper_text: "Helpful guidance", valid_text: "Accepted", info_text: "Information", warning_text: "Warning", invalid_text: "Invalid" }
                }
            }
            div { class: "empty-selection", Select { id: "empty-select", options: options(), allow_clear: true } }
        }
    });
    for (name, feedback) in [
        ("default", "Helpful guidance"),
        ("valid", "Accepted"),
        ("info", "Information"),
        ("warning", "Warning"),
        ("invalid", "Invalid"),
    ] {
        let (_, body) = part(&html, &format!("semantic-{name}"));
        let (root, _) = part(body, "ix-select");
        assert!(classes(root).contains(&"ix-select-visual"));
        assert!(classes(root).contains(&"ix-text-field-native"));
        for marker in [
            "ix-text-field-wrapper",
            "ix-text-field-top",
            "ix-text-field-slot",
            "ix-text-field-bottom",
            "ix-text-field-bottom-right",
        ] {
            part(body, marker);
        }
        let (_, label) = part(body, "ix-text-field-label");
        assert_eq!(text_content(label), "Selection*");
        assert_eq!(
            attribute(
                openings(label)
                    .find(|tag| tag.starts_with("<label "))
                    .unwrap(),
                "for"
            ),
            Some(format!("semantic-input-{name}").as_str())
        );
        assert_eq!(
            text_content(part(body, "ix-text-field-bottom-text").1),
            feedback
        );
        let (_, control) = part(body, "ix-select-control");
        let (_, container) = part(control, "ix-select-input-container");
        let (_, chips) = part(container, "ix-select-chips");
        let (_, rail) = part(chips, "ix-select-trigger-rail");
        assert!(native_input(rail).contains("role=\"combobox\""));
        assert!(
            !openings(chips).any(|tag| classes(tag).contains(&"ix-select-chip")),
            "single selection has an always mounted chips rail with no selected chips"
        );
        for (host_marker, marker, size, oval, label) in [
            (
                "ix-select-clear-host",
                "ix-select-clear",
                "ix-icon-size-16",
                true,
                "Clear selection",
            ),
            (
                "ix-select-chevron-host",
                "ix-select-chevron",
                "ix-icon-size-20",
                false,
                "Open select dropdown",
            ),
        ] {
            let (host, host_body) = part(rail, host_marker);
            assert!(host.starts_with("<span "));
            assert!(classes(host).contains(&"ix-select-action-host"));
            let (button, button_body) = part(host_body, marker);
            assert!(button.starts_with("<button "));
            for class in [
                "ix-icon-button-native",
                "ix-button-tokenized",
                "ix-button-subtle-tertiary",
                size,
            ] {
                assert!(classes(button).contains(&class));
            }
            assert_eq!(classes(button).contains(&"ix-oval"), oval);
            assert_eq!(attribute(button, "type"), Some("button"));
            assert_eq!(attribute(button, "aria-label"), Some(label));
            assert!(button_body.contains("ix-icon-button-glyph"));
            assert_eq!(text_content(button_body), "");
        }
    }
    let (_, empty) = part(&html, "empty-selection");
    let (_, chips) = part(empty, "ix-select-chips");
    part(chips, "ix-select-trigger-rail");
    assert!(!empty.contains("ix-select-clear-host"));
    assert!(empty.contains("ix-select-chevron-host"));
}

#[test]
fn select_multiple_chips_project_filter_chip_slot_and_tertiary_close_button_layers() {
    let html = render_fixture(|| {
        rsx! {
            for (name, disabled, readonly) in [("ready", false, false), ("disabled", true, false), ("readonly", false, true)] {
                div { class: "chips-{name}",
                    Select { id: "chips-input-{name}", options: options(), multiple: true, values: vec!["a".into(), "b".into()], disabled, readonly }
                }
            }
            div { class: "collapsed-selection",
                Select { options: options(), multiple: true, values: vec!["a".into(), "b".into(), "c".into()], collapse_multiple_selection: true }
            }
        }
    });
    for (name, disabled, readonly) in [
        ("ready", false, false),
        ("disabled", true, false),
        ("readonly", false, true),
    ] {
        let (_, body) = part(&html, &format!("chips-{name}"));
        let (_, chips) = part(body, "ix-select-chips");
        let hosts = openings(chips)
            .filter(|tag| classes(tag).contains(&"ix-select-chip"))
            .collect::<Vec<_>>();
        assert_eq!(hosts.len(), 2);
        assert_eq!(
            openings(chips)
                .filter(|tag| classes(tag).contains(&"ix-select-chip-label"))
                .count(),
            2
        );
        assert!(chips.contains("Alpha") && chips.contains("Beta"));
        assert!(
            chips.find("ix-select-chip").unwrap() < chips.find("ix-select-trigger-rail").unwrap()
        );
        for host in hosts {
            assert_eq!(
                attribute(host, "data-chip-disabled"),
                Some(if disabled || readonly {
                    "true"
                } else {
                    "false"
                })
            );
        }
        assert_eq!(
            chips.contains("ix-select-chip-close-host"),
            !disabled && !readonly
        );
        if !disabled && !readonly {
            let (_, host) = part(chips, "ix-select-chip-close-host");
            let (button, contents) = part(host, "ix-select-chip-remove");
            assert!(button.starts_with("<button "));
            for class in [
                "ix-icon-button-native",
                "ix-button-tokenized",
                "ix-button-tertiary",
                "ix-icon-size-16",
                "ix-oval",
            ] {
                assert!(classes(button).contains(&class));
            }
            assert_eq!(attribute(button, "aria-label"), Some("Remove Alpha"));
            assert!(contents.contains("ix-icon-button-glyph"));
        }
        let input = native_input(chips);
        assert_eq!(attribute(input, "value"), Some(""));
        assert!(classes(input).contains(&"ix-select-trigger-multiple"));
    }
    let (_, body) = part(&html, "collapsed-selection");
    let (_, chips) = part(body, "ix-select-chips");
    assert_eq!(
        openings(chips)
            .filter(|tag| classes(tag).contains(&"ix-select-chip"))
            .count(),
        1
    );
    assert_eq!(
        text_content(part(chips, "ix-select-chip-label").1),
        "All (3)"
    );
}

#[test]
fn select_option_layers_keep_start_check_gutter_and_native_control_semantics() {
    let html = render_fixture(|| {
        rsx! {
            for selected in [false, true] {
                for disabled in [false, true] {
                    for active in [false, true] {
                        div { class: "option-{selected}-{disabled}-{active}",
                            SelectOptionPresentation { id: "option-id-{selected}-{disabled}-{active}",
                                label: "Option label", selected, disabled, active }
                        }
                    }
                }
            }
        }
    });
    for selected in [false, true] {
        for disabled in [false, true] {
            for active in [false, true] {
                let (_, body) = part(&html, &format!("option-{selected}-{disabled}-{active}"));
                let (host, host_body) = part(body, "ix-select-item-host");
                assert_eq!(
                    attribute(host, "data-disabled"),
                    Some(if disabled { "true" } else { "false" })
                );
                let (native_host, native_host_body) = part(host_body, "ix-select-option-host");
                assert_eq!(
                    attribute(native_host, "data-selected"),
                    Some(if selected { "true" } else { "false" })
                );
                assert_eq!(
                    attribute(native_host, "data-disabled"),
                    Some(if disabled { "true" } else { "false" })
                );
                let (button, contents) = part(native_host_body, "ix-select-option");
                assert!(
                    button.starts_with("<button "),
                    "the port's explicit native-button adapter remains usable"
                );
                assert_eq!(attribute(button, "type"), Some("button"));
                assert_eq!(attribute(button, "role"), Some("option"));
                assert_eq!(attribute(button, "tabindex"), Some("-1"));
                assert_eq!(has_attribute(button, "disabled"), disabled);
                assert_eq!(
                    attribute(button, "aria-selected"),
                    Some(if selected { "true" } else { "false" })
                );
                assert_eq!(classes(button).contains(&"ix-select-active"), active);
                let (check, check_contents) = part(contents, "ix-select-choice");
                assert_eq!(attribute(check, "aria-hidden"), Some("true"));
                assert_eq!(check_contents.contains("<svg"), selected);
                if selected {
                    assert!(
                        check_contents.contains(crate::icons::icon_svg("single-check").unwrap())
                    );
                }
                assert_eq!(
                    text_content(part(contents, "ix-select-option-label").1),
                    "Option label"
                );
                assert_eq!(text_content(part(contents, "ix-select-option-end").1), "");
                assert!(
                    contents.find("ix-select-choice").unwrap()
                        < contents.find("ix-select-option-label").unwrap()
                );
                assert!(
                    contents.find("ix-select-option-label").unwrap()
                        < contents.find("ix-select-option-end").unwrap()
                );
            }
        }
    }
}

#[derive(Clone)]
struct SelectLiveState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    requests: Rc<RefCell<Vec<String>>>,
}
fn live_select(state: SelectLiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let single = state.requests.clone();
    let multiple = state.requests.clone();
    let input = state.requests.clone();
    let added = state.requests.clone();
    rsx! {
        Select { id: "live-selection", options: options(), name: "submitted", allow_clear: true,
            value: if current == 1 { "a" } else { "b" }, multiple: current >= 2,
            values: if current >= 2 { vec!["a".into(), "c".into()] } else { vec![] },
            disabled: current == 3, readonly: current == 2, required: current != 0,
            label: if current == 0 { "Selection" } else { "Updated selection" },
            helper_text: if current == 0 { "Initial help" } else { "Updated help" },
            onvaluechange: move |value| single.borrow_mut().push(format!("single:{value}")),
            onvalueschange: move |values| multiple.borrow_mut().push(format!("multiple:{values:?}")),
            oninputchange: move |value| input.borrow_mut().push(format!("input:{value}")),
            onadd: move |value| added.borrow_mut().push(format!("add:{value}")),
        }
    }
}

#[test]
fn select_external_props_update_display_and_form_state_without_emitting_requests() {
    let state = SelectLiveState {
        stage: Rc::new(RefCell::new(None)),
        requests: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_select, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in [0, 1, 2, 3, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let input = native_input(&html);
        assert_eq!(attribute(input, "id"), Some("live-selection"));
        assert_eq!(
            attribute(input, "value"),
            Some(match current {
                0 => "Beta",
                1 => "Alpha",
                _ => "",
            })
        );
        assert_eq!(has_attribute(input, "disabled"), current == 3);
        assert_eq!(has_attribute(input, "readonly"), current == 2);
        assert_eq!(attribute(input, "aria-expanded"), Some("false"));
        assert_eq!(
            selected_native_values(&html),
            if current >= 2 {
                vec!["a", "c"]
            } else if current == 1 {
                vec!["a"]
            } else {
                vec!["b"]
            }
        );
        let (native, _) = part(&html, "ix-select-native");
        assert_eq!(has_attribute(native, "multiple"), current >= 2);
        assert_eq!(
            has_attribute(native, "required"),
            current == 1 || current == 3
        );
        assert_eq!(has_attribute(native, "disabled"), current == 3);
        assert!(html.contains(if current == 0 {
            "Initial help"
        } else {
            "Updated help"
        }));
        assert!(
            state.requests.borrow().is_empty(),
            "external props remain silent controlled updates"
        );
    }
}

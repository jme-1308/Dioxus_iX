// Source state transitions and native rendering adapters. SSR does not prove
// timer delivery, focus, pointer behavior, geometry or computed token paint.
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
fn expanding_search_state_preserves_source_empty_toggle_blur_and_clear_contract() {
    let initial = ExpandingSearchState::default();
    assert!(!initial.expanded && !initial.changed);
    // Input and blur are separate source transitions: typing never expands the
    // field, and a nonempty controlled prop alone never marks it changed.
    for expanded in [false, true] {
        for (changed, value, expected_changed) in [
            (false, "", true),
            (false, "query", true),
            (true, "", false),
            (true, "query", true),
        ] {
            let mut state = ExpandingSearchState { expanded, changed };
            state.input(value);
            assert_eq!(state.expanded, expanded);
            assert_eq!(state.changed, expected_changed);
        }
        for changed in [false, true] {
            let mut state = ExpandingSearchState { expanded, changed };
            state.blur();
            assert_eq!(state.expanded, expanded && changed);
            assert_eq!(state.changed, changed);
            let mut state = ExpandingSearchState { expanded, changed };
            state.clear();
            assert_eq!(
                state.expanded, expanded,
                "clear preserves expansion until an unchanged blur"
            );
            assert!(!state.changed);
            let mut state = ExpandingSearchState { expanded, changed };
            state.expand();
            assert!(state.expanded);
            assert_eq!(state.changed, changed);
        }
    }
    let mut state = ExpandingSearchState::default();
    state.expand();
    state.input("");
    state.blur();
    assert!(
        state.expanded && state.changed,
        "first empty input counts as changed"
    );
    state.input("");
    state.blur();
    assert!(
        !state.expanded && !state.changed,
        "second empty input clears the changed flag"
    );
    state.expand();
    state.input("retained");
    state.blur();
    assert!(state.expanded && state.changed);
    state.clear();
    assert!(state.expanded && !state.changed);
    state.blur();
    assert!(!state.expanded && !state.changed);
}

#[test]
fn expanding_search_initial_markup_retains_native_input_for_presets_and_all_variants() {
    let html = render_fixture(|| {
        rsx! {
            ExpandingSearch { class: "default-search" }
            ExpandingSearch { class: "preset-search", value: "Existing query", full_width: true }
            ExpandingSearch { class: "custom-search", icon: "add", placeholder: "Find equipment",
                aria_label_search_icon_button: "Start equipment search", aria_label_search_input: "Equipment search", aria_label_clear_icon_button: "Remove query" }
            ExpandingSearch { class: "empty-icon-search", icon: "" }
            for variant in [crate::ButtonVariant::Primary, crate::ButtonVariant::Secondary, crate::ButtonVariant::Tertiary,
                crate::ButtonVariant::DangerPrimary, crate::ButtonVariant::DangerSecondary, crate::ButtonVariant::DangerTertiary,
                crate::ButtonVariant::SubtlePrimary, crate::ButtonVariant::SubtleSecondary, crate::ButtonVariant::SubtleTertiary] {
                ExpandingSearch { class: "variant-{variant.as_str()}", variant }
            }
            Input { class: "input-peer" }
        }
    });
    for marker in [
        "default-search",
        "preset-search",
        "custom-search",
        "empty-icon-search",
        "variant-primary",
        "variant-secondary",
        "variant-tertiary",
        "variant-danger-primary",
        "variant-danger-secondary",
        "variant-danger-tertiary",
        "variant-subtle-primary",
        "variant-subtle-secondary",
        "variant-subtle-tertiary",
    ] {
        let (root, body) = part(&html, marker);
        assert!(classes(root).contains(&"ix-expanding-search"));
        assert_eq!(attribute(root, "data-expanded"), Some("false"));
        assert_eq!(attribute(root, "data-changed"), Some("false"));
        assert!(!classes(root).contains(&"expanded"));
        assert_eq!(
            classes(root).contains(&"fullWidth"),
            marker == "preset-search"
        );
        let (host, host_body) = part(body, "ix-search-trigger-host");
        assert!(host.starts_with("<span "));
        assert_eq!(attribute(host, "data-expanded"), Some("false"));
        let (button, button_body) = part(host_body, "ix-search-trigger");
        assert!(button.starts_with("<button "));
        for class in [
            "ix-button-tokenized",
            "ix-icon-button-native",
            "ix-search-button",
            "ix-icon-size-24",
        ] {
            assert!(classes(button).contains(&class));
        }
        let variant = marker.strip_prefix("variant-").unwrap_or("tertiary");
        assert!(classes(button).contains(&format!("ix-button-{variant}").as_str()));
        assert_eq!(attribute(button, "type"), Some("button"));
        assert_eq!(attribute(button, "tabindex"), Some("0"));
        assert_eq!(
            attribute(button, "aria-label"),
            Some(if marker == "custom-search" {
                "Start equipment search"
            } else {
                "Open search"
            })
        );
        assert_eq!(attribute(button, "aria-expanded"), Some("false"));
        assert_eq!(
            button_body.contains("ix-icon-button-glyph"),
            marker != "empty-icon-search"
        );
        assert_eq!(
            text_content(button_body),
            "",
            "search icon button has no invented visible text"
        );
        if marker != "empty-icon-search" {
            assert!(
                button_body.contains(
                    crate::icons::icon_svg(if marker == "custom-search" {
                        "add"
                    } else {
                        "search"
                    })
                    .unwrap()
                )
            );
        }
        let (container, container_body) = part(body, "ix-search-input-container");
        assert_eq!(attribute(container, "data-expanded"), Some("false"));
        let native = openings(container_body)
            .find(|opening| opening.starts_with("<input "))
            .unwrap();
        assert!(classes(native).contains(&"ix-search-input"));
        assert_eq!(attribute(native, "type"), Some("text"));
        assert_eq!(attribute(native, "tabindex"), Some("-1"));
        assert_eq!(
            attribute(native, "aria-label"),
            Some(if marker == "custom-search" {
                "Equipment search"
            } else {
                "Search input"
            })
        );
        assert_eq!(
            attribute(native, "placeholder"),
            Some(if marker == "custom-search" {
                "Find equipment"
            } else {
                "Enter text here"
            })
        );
        assert_eq!(
            attribute(native, "value"),
            Some(if marker == "preset-search" {
                "Existing query"
            } else {
                ""
            })
        );
        assert!(!has_attribute(native, "disabled"));
        assert!(
            !body.contains("ix-search-clear"),
            "initial value does not fabricate changed state or a clear button"
        );
    }
    assert!(!classes(part(&html, "input-peer").0).contains(&"ix-expanding-search"));
}

#[derive(Clone)]
struct SearchLiveState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    requests: Rc<RefCell<Vec<String>>>,
}
fn live_search(state: SearchLiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let changes = state.requests.clone();
    let searches = state.requests.clone();
    rsx! {
        ExpandingSearch { class: "live-search", value: if current == 1 { "external query" } else { "" },
            placeholder: if current == 2 { "Revised placeholder" } else { "Enter text here" }, full_width: current == 1,
            disabled: current == 2, variant: if current == 1 { crate::ButtonVariant::DangerPrimary } else { crate::ButtonVariant::Tertiary },
            icon: match current { 1 => Some("add".to_string()), 2 => Some(String::new()), _ => None },
            aria_label_search_icon_button: if current == 1 { Some("Custom open".to_string()) } else { None },
            aria_label_search_input: if current == 1 { "Custom input" } else { "Search input" },
            aria_label_clear_icon_button: "Custom clear",
            onvaluechange: move |value| changes.borrow_mut().push(format!("change:{value}")),
            onsearch: move |value| searches.borrow_mut().push(format!("submit:{value}")),
        }
    }
}

#[test]
fn expanding_search_external_props_restore_native_attributes_without_changing_source_flags() {
    let state = SearchLiveState {
        stage: Rc::new(RefCell::new(None)),
        requests: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_search, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in [0, 1, 2, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let (root, body) = part(&html, "live-search");
        assert_eq!(attribute(root, "data-expanded"), Some("false"));
        assert_eq!(attribute(root, "data-changed"), Some("false"));
        assert_eq!(classes(root).contains(&"fullWidth"), current == 1);
        let (button, button_body) = part(body, "ix-search-trigger");
        assert_eq!(
            attribute(button, "aria-label"),
            Some(if current == 1 {
                "Custom open"
            } else {
                "Open search"
            })
        );
        assert_eq!(has_attribute(button, "disabled"), current == 2);
        assert_eq!(
            attribute(button, "tabindex"),
            Some(if current == 2 { "-1" } else { "0" })
        );
        assert!(classes(button).contains(&if current == 1 {
            "ix-button-danger-primary"
        } else {
            "ix-button-tertiary"
        }));
        assert_eq!(button_body.contains("ix-icon-button-glyph"), current != 2);
        if current != 2 {
            assert!(button_body.contains(
                crate::icons::icon_svg(if current == 1 { "add" } else { "search" }).unwrap()
            ));
        }
        let (_, container) = part(body, "ix-search-input-container");
        let input = openings(container)
            .find(|opening| opening.starts_with("<input "))
            .unwrap();
        assert_eq!(
            attribute(input, "value"),
            Some(if current == 1 { "external query" } else { "" })
        );
        assert_eq!(attribute(input, "type"), Some("text"));
        assert_eq!(attribute(input, "tabindex"), Some("-1"));
        assert_eq!(
            attribute(input, "aria-label"),
            Some(if current == 1 {
                "Custom input"
            } else {
                "Search input"
            })
        );
        assert_eq!(
            attribute(input, "placeholder"),
            Some(if current == 2 {
                "Revised placeholder"
            } else {
                "Enter text here"
            })
        );
        assert_eq!(has_attribute(input, "disabled"), current == 2);
        assert!(!body.contains("ix-search-clear"));
        assert!(
            state.requests.borrow().is_empty(),
            "prop updates do not emit source value requests or the optional Rust Enter adapter"
        );
    }
}

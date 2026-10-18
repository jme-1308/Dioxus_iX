// Rendered controls and consumer-owned state only. Computed selected styles and
// actual activation/request behavior are verified by the source/native fixtures.
use super::*;

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

fn opening_with_class<'a>(html: &'a str, marker: &str) -> Option<&'a str> {
    html.match_indices('<').find_map(|(index, _)| {
        let rest = &html[index..];
        let opening = &rest[..=rest.find('>')?];
        classes(opening).contains(&marker).then_some(opening)
    })
}

fn button<'a>(html: &'a str, marker: &str) -> (&'a str, &'a str) {
    let opening =
        opening_with_class(html, marker).unwrap_or_else(|| panic!("missing {marker}: {html}"));
    assert!(opening.starts_with("<button "));
    let rest = &html[html.find(opening).unwrap()..];
    let end = rest.find("</button>").unwrap() + "</button>".len();
    (opening, &rest[..end])
}

fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn toggle_variant_conversion_rejects_danger_and_rendering_keeps_icon_toggle_separate() {
    for (button_variant, toggle_variant) in [
        (ButtonVariant::Primary, ToggleButtonVariant::Primary),
        (ButtonVariant::Secondary, ToggleButtonVariant::Secondary),
        (ButtonVariant::Tertiary, ToggleButtonVariant::Tertiary),
        (
            ButtonVariant::SubtlePrimary,
            ToggleButtonVariant::SubtlePrimary,
        ),
        (
            ButtonVariant::SubtleSecondary,
            ToggleButtonVariant::SubtleSecondary,
        ),
        (
            ButtonVariant::SubtleTertiary,
            ToggleButtonVariant::SubtleTertiary,
        ),
    ] {
        let converted: Result<ToggleButtonVariant, ButtonVariant> = button_variant.try_into();
        assert_eq!(converted, Ok(toggle_variant));
    }
    for unsupported in [
        ButtonVariant::DangerPrimary,
        ButtonVariant::DangerSecondary,
        ButtonVariant::DangerTertiary,
    ] {
        let converted: Result<ToggleButtonVariant, ButtonVariant> = unsupported.try_into();
        assert_eq!(converted, Err(unsupported));
    }
    assert_eq!(
        ToggleButtonVariant::default(),
        ToggleButtonVariant::SubtlePrimary
    );
    let html = render_fixture(|| {
        rsx! {
            ToggleButton { class: "default-toggle", aria_label: "Toggle", onpressedchange: |_| {}, "Default" }
            for variant in [
                ToggleButtonVariant::Primary, ToggleButtonVariant::Secondary, ToggleButtonVariant::Tertiary,
                ToggleButtonVariant::SubtlePrimary, ToggleButtonVariant::SubtleSecondary, ToggleButtonVariant::SubtleTertiary,
            ] {
                ToggleButton { variant, class: "variant-{variant.as_str()}", aria_label: "Toggle", onpressedchange: |_| {}, "Toggle" }
            }
            IconToggleButton { icon: "add", label: "Icon toggle", onpressedchange: |_| {} }
        }
    });
    let (default, _) = button(&html, "default-toggle");
    assert!(classes(default).contains(&"ix-button-subtle-primary"));
    assert!(!classes(default).contains(&"ix-selected"));
    for variant in [
        "primary",
        "secondary",
        "tertiary",
        "subtle-primary",
        "subtle-secondary",
        "subtle-tertiary",
    ] {
        let (opening, _) = button(&html, &format!("variant-{variant}"));
        let names = classes(opening);
        assert!(
            names.contains(&"ix-button-tokenized") && names.contains(&"ix-toggle-button-native")
        );
        assert!(names.contains(&format!("ix-button-{variant}").as_str()));
        assert!(
            !names.contains(&"ix-button-standard") && !names.contains(&"ix-icon-button-native")
        );
        assert_eq!(attribute(opening, "type"), Some("button"));
        assert_eq!(attribute(opening, "tabindex"), Some("0"));
        assert_eq!(attribute(opening, "aria-pressed"), Some("false"));
        assert_eq!(attribute(opening, "aria-disabled"), Some("false"));
    }
    let icon_toggle = opening_with_class(&html, "ix-icon-button").unwrap();
    assert!(classes(icon_toggle).contains(&"ix-button-tokenized"));
    assert!(classes(icon_toggle).contains(&"ix-icon-toggle-button-native"));
    assert!(!classes(icon_toggle).contains(&"ix-toggle-button-native"));
    assert_eq!(html.matches("ix-toggle-button-native").count(), 7);
    assert_eq!(html.matches("ix-button-tokenized").count(), 8);
}

#[test]
fn toggle_icons_and_loading_preserve_label_and_only_replace_the_prefix() {
    let html = render_fixture(|| {
        rsx! {
            ToggleButton { class: "no-icons", aria_label: "Toggle", onpressedchange: |_| {}, "Label" }
            ToggleButton { class: "start-icon", icon: "add", aria_label: "Toggle", onpressedchange: |_| {}, "Label" }
            ToggleButton { class: "end-icon", icon_right: "chevron-right", aria_label: "Toggle", onpressedchange: |_| {}, "Label" }
            ToggleButton { class: "both-icons", icon: "add", icon_right: "chevron-right", aria_label: "Toggle", onpressedchange: |_| {}, "Label" }
            ToggleButton { class: "empty-icons", icon: "", icon_right: "", aria_label: "Toggle", onpressedchange: |_| {}, "Label" }
            ToggleButton { class: "loading-icons", pressed: true, loading: true, icon: "add", icon_right: "chevron-right", aria_label: "Toggle", onpressedchange: |_| {}, "Label" }
            ToggleButton { class: "loading-empty", loading: true, icon: "", icon_right: "", aria_label: "Toggle", onpressedchange: |_| {}, "Label" }
        }
    });
    for (marker, count, start, end) in [
        ("no-icons", 0, false, false),
        ("start-icon", 1, true, false),
        ("end-icon", 1, false, true),
        ("both-icons", 2, true, true),
        ("empty-icons", 0, false, false),
        ("loading-icons", 1, false, true),
        ("loading-empty", 0, false, false),
    ] {
        let (opening, body) = button(&html, marker);
        assert!(body.contains("Label"));
        assert_eq!(attribute(opening, "aria-label"), Some("Toggle"));
        assert_eq!(body.matches("ix-toggle-button-icon").count(), count);
        assert_eq!(body.matches("ix-toggle-button-glyph").count(), count);
        assert_eq!(body.contains("ix-button-icon-start"), start);
        assert_eq!(body.contains("ix-button-icon-end"), end);
        if count > 0 {
            let glyph = opening_with_class(body, "ix-toggle-button-glyph").unwrap();
            assert!(glyph.starts_with("<svg "));
            assert_eq!(attribute(glyph, "role"), Some("presentation"));
            assert_eq!(attribute(glyph, "aria-hidden"), Some("true"));
        }
        if marker.starts_with("loading-") {
            assert!(has_attribute(opening, "disabled"));
            assert_eq!(attribute(opening, "aria-busy"), Some("true"));
            assert_eq!(attribute(opening, "tabindex"), Some("-1"));
            let spinner = opening_with_class(body, "ix-spinner").unwrap();
            assert!(classes(spinner).contains(&"ix-spinner-sm"));
            assert!(classes(spinner).contains(&"ix-spinner-no-track"));
            assert_eq!(attribute(spinner, "role"), Some("status"));
            assert!(
                body.find("ix-button-loading").unwrap() < body.find("ix-button-content").unwrap()
            );
        } else {
            assert!(!body.contains("ix-spinner"));
        }
    }
    let (_, both) = button(&html, "both-icons");
    assert!(both.find("ix-button-icon-start").unwrap() < both.find("ix-button-content").unwrap());
    assert!(both.find("ix-button-content").unwrap() < both.find("ix-button-icon-end").unwrap());
    let (loading, body) = button(&html, "loading-icons");
    assert_eq!(attribute(loading, "aria-pressed"), Some("true"));
    assert!(classes(loading).contains(&"ix-selected"));
    assert!(body.find("ix-button-content").unwrap() < body.find("ix-button-icon-end").unwrap());
}

#[derive(Clone)]
struct ToggleLifecycle(Rc<RefCell<Option<Signal<u8>>>>);

fn live_toggle(state: ToggleLifecycle) -> Element {
    let stage = use_signal(|| 0u8);
    *state.0.borrow_mut() = Some(stage);
    let current = stage();
    rsx! {
        ToggleButton {
            class: "live-toggle", pressed: current == 1 || current == 2 || current == 3,
            disabled: current == 2, loading: current == 3,
            icon: if current == 4 { Some(String::new()) } else { Some("add".to_string()) },
            icon_right: if current == 4 { Some(String::new()) } else { Some("chevron-right".to_string()) },
            aria_label: "Current toggle", onpressedchange: |_| {},
            if current == 4 { "External reset" } else { "Stable label" }
        }
    }
}

#[test]
fn consumer_updates_restore_pressed_blocking_icons_and_labels() {
    let state = ToggleLifecycle(Rc::new(RefCell::new(None)));
    let mut dom = VirtualDom::new_with_props(live_toggle, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.0.borrow().unwrap();
    for current in 0..=5 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (opening, body) = button(&html, "live-toggle");
        let pressed = current == 1 || current == 2 || current == 3;
        let blocked = current == 2 || current == 3;
        assert_eq!(classes(opening).contains(&"ix-selected"), pressed);
        assert_eq!(
            attribute(opening, "aria-pressed"),
            Some(if pressed { "true" } else { "false" })
        );
        assert_eq!(
            attribute(opening, "aria-disabled"),
            Some(if blocked { "true" } else { "false" })
        );
        assert_eq!(
            attribute(opening, "aria-busy"),
            Some(if current == 3 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(opening, "tabindex"),
            Some(if blocked { "-1" } else { "0" })
        );
        assert_eq!(has_attribute(opening, "disabled"), blocked);
        assert_eq!(attribute(opening, "aria-label"), Some("Current toggle"));
        assert_eq!(attribute(opening, "type"), Some("button"));
        assert_eq!(
            body.contains("ix-button-icon-start"),
            current != 3 && current != 4
        );
        assert_eq!(body.contains("ix-button-icon-end"), current != 4);
        assert_eq!(body.contains("ix-spinner"), current == 3);
        assert!(body.contains(if current == 4 {
            "External reset"
        } else {
            "Stable label"
        }));
    }
}

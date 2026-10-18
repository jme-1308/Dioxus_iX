// These assertions cover rendered controls and consumer-owned properties.
// Computed colors/dimensions and actual pressed requests use native fixtures.
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
fn icon_toggle_accepts_all_nine_variants_and_keeps_peer_capabilities_distinct() {
    let html = render_fixture(|| {
        rsx! {
            IconToggleButton { icon: "add", label: "Toggle", class: "default-icon-toggle", onpressedchange: |_| {} }
            for variant in [
                ButtonVariant::Primary, ButtonVariant::Secondary, ButtonVariant::Tertiary,
                ButtonVariant::SubtlePrimary, ButtonVariant::SubtleSecondary, ButtonVariant::SubtleTertiary,
                ButtonVariant::DangerPrimary, ButtonVariant::DangerSecondary, ButtonVariant::DangerTertiary,
            ] {
                IconToggleButton { icon: "add", label: "Toggle", variant, pressed: true, class: "variant-{variant.as_str()}", onpressedchange: |_| {} }
            }
            Button { class: "regular-peer", "Regular" }
            IconButton { icon: "add", label: "Icon", class: "icon-peer" }
            ToggleButton { class: "text-toggle-peer", onpressedchange: |_| {}, "Text toggle" }
        }
    });
    let (default, body) = button(&html, "default-icon-toggle");
    let names = classes(default);
    assert!(names.contains(&"ix-button-subtle-primary") && names.contains(&"ix-icon-size-20"));
    assert!(!names.contains(&"ix-selected") && !names.contains(&"ix-oval"));
    assert_eq!(attribute(default, "aria-pressed"), Some("false"));
    assert_eq!(attribute(default, "aria-disabled"), Some("false"));
    assert_eq!(attribute(default, "tabindex"), Some("0"));
    assert_eq!(attribute(default, "type"), Some("button"));
    assert!(!body.contains("ix-button-content"));
    for variant in [
        "primary",
        "secondary",
        "tertiary",
        "subtle-primary",
        "subtle-secondary",
        "subtle-tertiary",
        "danger-primary",
        "danger-secondary",
        "danger-tertiary",
    ] {
        let (opening, _) = button(&html, &format!("variant-{variant}"));
        let names = classes(opening);
        for required in [
            "ix-button-tokenized",
            "ix-icon-button-native",
            "ix-icon-toggle-button-native",
            "ix-button-selectable",
            "ix-selected",
        ] {
            assert!(names.contains(&required), "{variant}: missing {required}");
        }
        assert!(names.contains(&format!("ix-button-{variant}").as_str()));
        assert!(
            !names.contains(&"ix-button-standard") && !names.contains(&"ix-toggle-button-native")
        );
        assert_eq!(attribute(opening, "aria-pressed"), Some("true"));
    }
    for marker in ["regular-peer", "icon-peer"] {
        let (opening, _) = button(&html, marker);
        assert!(!classes(opening).contains(&"ix-button-selectable"));
        assert!(!classes(opening).contains(&"ix-icon-toggle-button-native"));
        assert!(!has_attribute(opening, "aria-pressed"));
    }
    let (text, _) = button(&html, "text-toggle-peer");
    assert!(classes(text).contains(&"ix-button-selectable"));
    assert!(classes(text).contains(&"ix-toggle-button-native"));
    assert!(!classes(text).contains(&"ix-icon-button-native"));
}

#[test]
fn icon_toggle_loading_preserves_name_and_replaces_each_glyph_with_correct_spinner() {
    let html = render_fixture(|| {
        rsx! {
            for size in [ButtonIconSize::Twelve, ButtonIconSize::Sixteen, ButtonIconSize::Twenty, ButtonIconSize::TwentyFour] {
                IconToggleButton { icon: "add", label: "Toggle", size, class: "ready-{size.as_str()}", onpressedchange: |_| {} }
                IconToggleButton { icon: "add", label: "Toggle", size, loading: true, pressed: true, class: "loading-{size.as_str()}", onpressedchange: |_| {} }
            }
            IconToggleButton { icon: "", label: "Empty", size: ButtonIconSize::Twelve, loading: true, class: "empty-12", onpressedchange: |_| {} }
            IconToggleButton { icon: "", label: "Empty", size: ButtonIconSize::Sixteen, loading: true, class: "empty-16", onpressedchange: |_| {} }
            IconToggleButton { icon: "", label: "Empty", class: "empty-ready", onpressedchange: |_| {} }
        }
    });
    for (size, spinner_size, spinner_class) in [
        (12, 12, "ix-spinner-xxs"),
        (16, 16, "ix-spinner-xs"),
        (20, 20, "ix-spinner-sm"),
        (24, 20, "ix-spinner-sm"),
    ] {
        let (_, ready) = button(&html, &format!("ready-{size}"));
        let glyph = opening_with_class(ready, "ix-icon-button-glyph").unwrap();
        assert!(glyph.starts_with("<svg "));
        let size_text = size.to_string();
        assert_eq!(attribute(glyph, "width"), Some(size_text.as_str()));
        assert_eq!(attribute(glyph, "height"), Some(size_text.as_str()));
        assert_eq!(attribute(glyph, "role"), Some("presentation"));
        assert_eq!(attribute(glyph, "aria-hidden"), Some("true"));
        assert_eq!(attribute(glyph, "focusable"), Some("false"));
        assert!(!ready.contains("ix-spinner"));
        let (opening, loading) = button(&html, &format!("loading-{size}"));
        assert!(has_attribute(opening, "disabled"));
        assert_eq!(attribute(opening, "aria-disabled"), Some("true"));
        assert_eq!(attribute(opening, "aria-busy"), Some("true"));
        assert_eq!(attribute(opening, "aria-pressed"), Some("true"));
        assert_eq!(attribute(opening, "aria-label"), Some("Toggle"));
        assert_eq!(attribute(opening, "title"), Some("Toggle"));
        assert_eq!(attribute(opening, "tabindex"), Some("-1"));
        assert!(!loading.contains("ix-icon-button-glyph"));
        let spinner = opening_with_class(loading, "ix-spinner").unwrap();
        assert!(classes(spinner).contains(&spinner_class));
        assert!(classes(spinner).contains(&"ix-spinner-no-track"));
        assert_eq!(attribute(spinner, "role"), Some("status"));
        assert!(
            attribute(spinner, "style")
                .unwrap()
                .contains(&format!("--ix-spinner-size:{spinner_size}px"))
        );
    }
    for marker in ["empty-12", "empty-16"] {
        let (_, body) = button(&html, marker);
        let spinner = opening_with_class(body, "ix-spinner").unwrap();
        assert!(classes(spinner).contains(&"ix-spinner-sm"));
        assert!(!body.contains("ix-icon-button-glyph"));
    }
    let (opening, empty) = button(&html, "empty-ready");
    assert_eq!(attribute(opening, "aria-label"), Some("Empty"));
    assert!(!empty.contains("<svg") && !empty.contains("ix-spinner"));
}

#[derive(Clone)]
struct IconToggleLifecycle(Rc<RefCell<Option<Signal<u8>>>>);

fn live_icon_toggle(state: IconToggleLifecycle) -> Element {
    let stage = use_signal(|| 0u8);
    *state.0.borrow_mut() = Some(stage);
    let current = stage();
    rsx! {
        IconToggleButton {
            class: "live-icon-toggle", icon: if current == 4 { "" } else { "add" },
            label: if current == 1 { "Remove" } else if current == 4 { "Empty" } else { "Add" },
            pressed: current == 1 || current == 2 || current == 4,
            size: match current { 1 => ButtonIconSize::Twelve, 2 => ButtonIconSize::Sixteen, 3 => ButtonIconSize::TwentyFour, _ => ButtonIconSize::Twenty },
            oval: current == 1 || current == 3,
            disabled: current == 1, loading: current == 2,
            onpressedchange: |_| {},
        }
    }
}

#[test]
fn consumer_updates_restore_pressed_size_oval_blocked_state_icon_and_name() {
    let state = IconToggleLifecycle(Rc::new(RefCell::new(None)));
    let mut dom = VirtualDom::new_with_props(live_icon_toggle, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.0.borrow().unwrap();
    for current in 0..=5 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (opening, body) = button(&html, "live-icon-toggle");
        let names = classes(opening);
        let pressed = current == 1 || current == 2 || current == 4;
        let blocked = current == 1 || current == 2;
        let size = match current {
            1 => 12,
            2 => 16,
            3 => 24,
            _ => 20,
        };
        let label = match current {
            1 => "Remove",
            4 => "Empty",
            _ => "Add",
        };
        assert!(names.contains(&format!("ix-icon-size-{size}").as_str()));
        assert_eq!(names.contains(&"ix-selected"), pressed);
        assert_eq!(names.contains(&"ix-oval"), current == 1 || current == 3);
        assert_eq!(has_attribute(opening, "disabled"), blocked);
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
            Some(if current == 2 { "true" } else { "false" })
        );
        assert_eq!(
            attribute(opening, "tabindex"),
            Some(if blocked { "-1" } else { "0" })
        );
        assert_eq!(attribute(opening, "aria-label"), Some(label));
        assert_eq!(attribute(opening, "title"), Some(label));
        assert_eq!(attribute(opening, "type"), Some("button"));
        assert_eq!(
            body.contains("ix-icon-button-glyph"),
            current != 2 && current != 4
        );
        assert_eq!(body.contains("ix-spinner"), current == 2);
        assert!(!body.contains("ix-button-content"));
    }
}

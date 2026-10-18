// SSR and live VDOM assertions cover the native control contract, not computed
// token dimensions, trusted activation, form submission or focus behavior.
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
    let start = html.find(opening).unwrap();
    let rest = &html[start..];
    let end = rest.find("</button>").unwrap() + "</button>".len();
    (opening, &rest[..end])
}

fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn icon_button_defaults_and_variants_retain_distinct_control_markers() {
    let html = render_fixture(|| {
        rsx! {
            IconButton { icon: "add", label: "Add", class: "default-icon" }
            for variant in [
                ButtonVariant::Primary, ButtonVariant::Secondary, ButtonVariant::Tertiary,
                ButtonVariant::SubtlePrimary, ButtonVariant::SubtleSecondary, ButtonVariant::SubtleTertiary,
                ButtonVariant::DangerPrimary, ButtonVariant::DangerSecondary, ButtonVariant::DangerTertiary,
            ] {
                IconButton { icon: "add", label: "Add", variant, class: "variant-{variant.as_str()}" }
            }
            Button { class: "regular-peer", "Regular" }
            ToggleButton { class: "toggle-peer", onpressedchange: |_| {}, "Toggle" }
            IconToggleButton { icon: "add", label: "Icon toggle", onpressedchange: |_| {} }
        }
    });
    let (default, body) = button(&html, "default-icon");
    let names = classes(default);
    assert!(names.contains(&"ix-button-subtle-primary"));
    assert!(names.contains(&"ix-icon-size-20"));
    assert!(!names.contains(&"ix-oval"));
    assert_eq!(attribute(default, "type"), Some("button"));
    assert_eq!(attribute(default, "tabindex"), Some("0"));
    assert_eq!(attribute(default, "aria-disabled"), Some("false"));
    assert_eq!(attribute(default, "aria-label"), Some("Add"));
    assert_eq!(attribute(default, "title"), Some("Add"));
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
        assert!(names.contains(&"ix-button-tokenized") && names.contains(&"ix-icon-button-native"));
        assert!(names.contains(&format!("ix-button-{variant}").as_str()));
        assert!(!names.contains(&"ix-button-standard") && !names.contains(&"ix-selected"));
    }
    let (regular, _) = button(&html, "regular-peer");
    assert!(classes(regular).contains(&"ix-button-tokenized"));
    assert!(classes(regular).contains(&"ix-button-standard"));
    assert!(!classes(regular).contains(&"ix-icon-button-native"));
    let (toggle, _) = button(&html, "toggle-peer");
    assert!(classes(toggle).contains(&"ix-button-tokenized"));
    assert!(classes(toggle).contains(&"ix-toggle-button-native"));
    assert!(!classes(toggle).contains(&"ix-icon-button-native"));
    let icon_toggle = opening_with_class(&html, "ix-icon-toggle-button-native").unwrap();
    assert!(classes(icon_toggle).contains(&"ix-icon-button-native"));
    assert!(classes(icon_toggle).contains(&"ix-button-selectable"));
    assert!(!classes(icon_toggle).contains(&"ix-toggle-button-native"));
    assert_eq!(html.matches("ix-button-tokenized").count(), 13);
    assert_eq!(html.matches("ix-icon-button-native").count(), 11);
}

#[test]
fn loading_spinner_replaces_glyph_and_empty_icon_uses_default_spinner_size() {
    let html = render_fixture(|| {
        rsx! {
            IconButton { icon: "add", label: "Twelve", class: "load-12", size: ButtonIconSize::Twelve, loading: true }
            IconButton { icon: "add", label: "Sixteen", class: "load-16", size: ButtonIconSize::Sixteen, loading: true }
            IconButton { icon: "add", label: "Twenty", class: "load-20", size: ButtonIconSize::Twenty, loading: true }
            IconButton { icon: "add", label: "Twenty-four", class: "load-24", size: ButtonIconSize::TwentyFour, loading: true }
            IconButton { icon: "", label: "Empty twelve", class: "empty-12", size: ButtonIconSize::Twelve, loading: true }
            IconButton { icon: "", label: "Empty sixteen", class: "empty-16", size: ButtonIconSize::Sixteen, loading: true }
            IconButton { icon: "", label: "Empty ready", class: "empty-ready" }
        }
    });
    for (marker, pixels) in [
        ("load-12", 12),
        ("load-16", 16),
        ("load-20", 20),
        ("load-24", 20),
        ("empty-12", 20),
        ("empty-16", 20),
    ] {
        let (opening, body) = button(&html, marker);
        assert!(has_attribute(opening, "disabled"));
        assert_eq!(attribute(opening, "aria-disabled"), Some("true"));
        assert_eq!(attribute(opening, "aria-busy"), Some("true"));
        assert_eq!(attribute(opening, "tabindex"), Some("-1"));
        assert!(body.contains("ix-spinner-no-track"));
        assert!(
            body.contains(&format!("--ix-spinner-size:{pixels}px")),
            "{marker}: {body}"
        );
        assert!(!body.contains("ix-icon-button-glyph"));
        let spinner = opening_with_class(body, "ix-spinner").unwrap();
        let expected_size = match pixels {
            12 => "ix-spinner-xxs",
            16 => "ix-spinner-xs",
            _ => "ix-spinner-sm",
        };
        assert!(classes(spinner).contains(&expected_size));
        assert_eq!(attribute(spinner, "role"), Some("status"));
        assert_eq!(attribute(spinner, "aria-label"), Some("Loading"));
    }
    let (empty, body) = button(&html, "empty-ready");
    assert_eq!(attribute(empty, "aria-label"), Some("Empty ready"));
    assert!(!body.contains("<svg") && !body.contains("ix-spinner"));
}

#[derive(Clone)]
struct IconLifecycle(Rc<RefCell<Option<Signal<u8>>>>);

fn live_icon(state: IconLifecycle) -> Element {
    let stage = use_signal(|| 0u8);
    *state.0.borrow_mut() = Some(stage);
    let current = stage();
    rsx! {
        IconButton {
            class: "live-icon", icon: "add", label: "Submit update", button_type: ButtonType::Submit,
            size: match current { 1 => ButtonIconSize::Twelve, 2 => ButtonIconSize::Sixteen, 3 => ButtonIconSize::TwentyFour, _ => ButtonIconSize::Twenty },
            oval: current == 1 || current == 3,
            disabled: current == 1, loading: current == 2,
            icon_color: if current == 1 || current == 2 { Some("--test-icon-ink".to_string()) } else { None },
        }
    }
}

#[test]
fn live_icon_size_oval_blocked_color_and_glyph_restore_independently() {
    let state = IconLifecycle(Rc::new(RefCell::new(None)));
    let mut dom = VirtualDom::new_with_props(live_icon, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.0.borrow().unwrap();
    for (current, pixels, oval, blocked, loading, color) in [
        (0, 20, false, false, false, false),
        (1, 12, true, true, false, true),
        (2, 16, false, true, true, true),
        (3, 24, true, false, false, false),
        (4, 20, false, false, false, false),
    ] {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (opening, body) = button(&html, "live-icon");
        let names = classes(opening);
        assert!(names.contains(&format!("ix-icon-size-{pixels}").as_str()));
        assert_eq!(names.contains(&"ix-oval"), oval);
        assert_eq!(has_attribute(opening, "disabled"), blocked);
        assert_eq!(
            attribute(opening, "tabindex"),
            Some(if blocked { "-1" } else { "0" })
        );
        assert_eq!(
            attribute(opening, "aria-disabled"),
            Some(if blocked { "true" } else { "false" })
        );
        assert_eq!(
            attribute(opening, "aria-busy"),
            Some(if loading { "true" } else { "false" })
        );
        assert_eq!(attribute(opening, "aria-label"), Some("Submit update"));
        assert_eq!(attribute(opening, "title"), Some("Submit update"));
        assert_eq!(attribute(opening, "type"), Some("submit"));
        let style = attribute(opening, "style")
            .unwrap_or_default()
            .replace(' ', "");
        assert_eq!(
            style.contains("--ix-icon-button-color:var(--test-icon-ink)"),
            color
        );
        if loading {
            assert!(!body.contains("ix-icon-button-glyph"));
            assert!(body.contains("--ix-spinner-size:16px"));
        } else {
            let glyph = opening_with_class(body, "ix-icon-button-glyph").unwrap();
            assert!(glyph.starts_with("<svg "));
            assert_eq!(attribute(glyph, "width"), Some(pixels.to_string().as_str()));
            assert_eq!(
                attribute(glyph, "height"),
                Some(pixels.to_string().as_str())
            );
            assert_eq!(attribute(glyph, "role"), Some("presentation"));
            assert_eq!(attribute(glyph, "aria-hidden"), Some("true"));
            assert_eq!(attribute(glyph, "focusable"), Some("false"));
            assert!(!body.contains("ix-spinner"));
        }
    }
}

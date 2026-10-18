// These tests verify rendered control semantics. Font, color, spacing and
// native input behavior are measured by the separate source/port fixtures.
use super::*;

fn attribute<'a>(opening: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!(" {name}=");
    let value = opening.split_once(&prefix)?.1;
    if let Some(quoted) = value.strip_prefix('"') {
        Some(quoted.split('"').next().unwrap())
    } else if let Some(quoted) = value.strip_prefix('\'') {
        Some(quoted.split('\'').next().unwrap())
    } else {
        Some(
            value
                .split(|character: char| character.is_whitespace() || character == '>')
                .next()
                .unwrap(),
        )
    }
}

fn has_attribute(opening: &str, name: &str) -> bool {
    opening
        .match_indices(&format!(" {name}"))
        .any(|(index, _)| {
            opening[index + name.len() + 1..]
                .chars()
                .next()
                .is_none_or(|character| {
                    character.is_whitespace() || character == '=' || character == '>'
                })
        })
}

fn classes(opening: &str) -> Vec<&str> {
    attribute(opening, "class")
        .unwrap_or_default()
        .split_whitespace()
        .collect()
}

fn marked_control<'a>(html: &'a str, marker: &str) -> (&'a str, &'a str) {
    for (index, _) in html.match_indices('<') {
        let rest = &html[index..];
        let closing = if rest.starts_with("<button ") {
            "</button>"
        } else if rest.starts_with("<a ") {
            "</a>"
        } else {
            continue;
        };
        let opening_end = rest.find('>').unwrap() + 1;
        let opening = &rest[..opening_end];
        if classes(opening).contains(&marker) {
            let end = rest.find(closing).unwrap() + closing.len();
            return (opening, &rest[..end]);
        }
    }
    panic!("missing control with class {marker}: {html}");
}

fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn regular_button_variants_are_scoped_without_changing_other_action_kinds() {
    let html = render_fixture(|| {
        rsx! {
            for variant in [
                ButtonVariant::Primary, ButtonVariant::Secondary, ButtonVariant::Tertiary,
                ButtonVariant::SubtlePrimary, ButtonVariant::SubtleSecondary, ButtonVariant::SubtleTertiary,
                ButtonVariant::DangerPrimary, ButtonVariant::DangerSecondary, ButtonVariant::DangerTertiary,
            ] {
                Button { variant, class: "regular-{variant.as_str()}", "Action" }
            }
            IconButton { icon: "add", label: "Add", class: "icon-only" }
            ToggleButton { class: "toggle-only", pressed: true, onpressedchange: |_| {}, "Toggle" }
            IconToggleButton { icon: "add", label: "Toggle icon", onpressedchange: |_| {} }
            LinkButton { class: "link-only", url: "/details", "Details" }
        }
    });
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
        let (opening, _) = marked_control(&html, &format!("regular-{variant}"));
        let names = classes(opening);
        assert!(names.contains(&"ix-button-standard"));
        assert!(names.contains(&format!("ix-button-{variant}").as_str()));
        assert!(!names.contains(&"ix-selected"));
        assert_eq!(attribute(opening, "type"), Some("button"));
    }
    for marker in ["icon-only", "toggle-only", "link-only"] {
        let (opening, _) = marked_control(&html, marker);
        assert!(!classes(opening).contains(&"ix-button-standard"));
    }
    assert_eq!(html.matches("ix-button-standard").count(), 9);
    let (toggle, _) = marked_control(&html, "toggle-only");
    assert_eq!(attribute(toggle, "aria-pressed"), Some("true"));
}

#[test]
fn loading_replaces_only_the_prefix_and_uses_source_spinner_sizes() {
    let html = render_fixture(|| {
        rsx! {
            Button { class: "ready-empty-icons", icon: "", icon_right: "", "Empty icons" }
            Button { class: "ready-icons", icon: "add", icon_right: "chevron-right", icon_size: ButtonIconSize::Sixteen, "Ready label" }
            Button { class: "loading-no-icon", loading: true, icon_size: ButtonIconSize::Twelve, icon_right: "chevron-right", "No prefix" }
            Button { class: "loading-empty-icon", loading: true, icon: "", icon_size: ButtonIconSize::Twelve, "Empty prefix" }
            Button { class: "loading-12", loading: true, icon: "add", icon_right: "chevron-right", icon_size: ButtonIconSize::Twelve, "Loading twelve" }
            Button { class: "loading-16", loading: true, icon: "add", icon_right: "chevron-right", icon_size: ButtonIconSize::Sixteen, "Loading sixteen" }
            Button { class: "loading-20", loading: true, icon: "add", icon_right: "chevron-right", icon_size: ButtonIconSize::Twenty, "Loading twenty" }
            Button { class: "loading-24", loading: true, icon: "add", icon_right: "chevron-right", icon_size: ButtonIconSize::TwentyFour, "Loading twenty-four" }
        }
    });
    let (_, empty) = marked_control(&html, "ready-empty-icons");
    assert!(!empty.contains("ix-button-icon-start") && !empty.contains("ix-button-icon-end"));
    let (_, ready) = marked_control(&html, "ready-icons");
    assert!(ready.contains("ix-button-icon-start") && ready.contains("ix-button-icon-end"));
    assert!(ready.contains("Ready label"));
    assert!(!ready.contains("ix-spinner"));
    assert_eq!(ready.matches("role=\"presentation\"").count(), 2);
    assert_eq!(ready.matches("aria-hidden=\"true\"").count(), 2);
    for (marker, size) in [
        ("loading-no-icon", 20),
        ("loading-empty-icon", 20),
        ("loading-12", 12),
        ("loading-16", 16),
        ("loading-20", 20),
        ("loading-24", 20),
    ] {
        let (opening, body) = marked_control(&html, marker);
        assert!(has_attribute(opening, "disabled"));
        assert_eq!(attribute(opening, "tabindex"), Some("-1"));
        assert_eq!(attribute(opening, "aria-disabled"), Some("true"));
        assert_eq!(attribute(opening, "aria-busy"), Some("true"));
        assert!(body.contains("ix-button-loading"));
        assert!(!body.contains("ix-button-icon-start"));
        assert!(body.contains("ix-spinner-no-track"));
        assert!(
            body.contains(&format!("--ix-spinner-size:{size}px")),
            "{marker}: {body}"
        );
        assert!(body.contains("role=\"status\"") && body.contains("aria-label=\"Loading\""));
        let spinner = body.find("ix-button-loading").unwrap();
        let label = body.find("ix-button-content").unwrap();
        assert!(spinner < label);
        if marker != "loading-empty-icon" {
            assert!(label < body.find("ix-button-icon-end").unwrap());
        }
    }
}

#[derive(Clone)]
struct AnchorLifecycle(Rc<RefCell<Option<Signal<u8>>>>);

fn anchor_lifecycle(state: AnchorLifecycle) -> Element {
    let stage = use_signal(|| 0u8);
    *state.0.borrow_mut() = Some(stage);
    let current = stage();
    rsx! {
        Button {
            class: "live-control", href: if current == 3 { None } else { Some("/details".to_string()) },
            disabled: current == 1, loading: current == 2,
            button_type: ButtonType::Submit, form: "checkout", target: "_blank", rel: "external",
            aria_label: "Open details", aria_describedby: "details-help", "Details"
        }
    }
}

#[test]
fn anchor_disabled_loading_and_native_button_transitions_preserve_control_attributes() {
    let state = AnchorLifecycle(Rc::new(RefCell::new(None)));
    let mut dom = VirtualDom::new_with_props(anchor_lifecycle, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.0.borrow().unwrap();
    for current in 0..=4 {
        if current > 0 {
            dom.in_scope(ScopeId::ROOT, || stage.set(current));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (opening, body) = marked_control(&html, "live-control");
        let blocked = current == 1 || current == 2;
        assert_eq!(attribute(opening, "aria-label"), Some("Open details"));
        assert_eq!(attribute(opening, "aria-describedby"), Some("details-help"));
        assert_eq!(
            attribute(opening, "aria-disabled"),
            Some(if blocked { "true" } else { "false" })
        );
        assert_eq!(
            attribute(opening, "aria-busy"),
            Some(if current == 2 { "true" } else { "false" })
        );
        assert!(body.contains("Details"));
        if current == 3 {
            assert!(opening.starts_with("<button "));
            assert_eq!(attribute(opening, "type"), Some("submit"));
            assert_eq!(attribute(opening, "form"), Some("checkout"));
            assert!(!has_attribute(opening, "href"));
            assert!(!has_attribute(opening, "disabled"));
        } else {
            assert!(opening.starts_with("<a "));
            assert_eq!(attribute(opening, "role"), Some("button"));
            assert_eq!(
                attribute(opening, "href"),
                if blocked { None } else { Some("/details") }
            );
            assert_eq!(
                attribute(opening, "tabindex"),
                Some(if blocked { "-1" } else { "0" })
            );
            assert_eq!(attribute(opening, "target"), Some("_blank"));
            assert_eq!(attribute(opening, "rel"), Some("external"));
        }
    }
}

#[test]
fn native_button_types_and_accessible_names_do_not_depend_on_visual_variant() {
    let html = render_fixture(|| {
        rsx! {
            Button { class: "submit-control", button_type: ButtonType::Submit, form: "editor", aria_label: "Save document", aria_describedby: "save-help", "Save" }
            Button { class: "reset-control", button_type: ButtonType::Reset, disabled: true, "Reset" }
            Button { class: "default-control", "Default" }
            Button { class: "default-link", href: "/manual", "Manual" }
        }
    });
    let (submit, _) = marked_control(&html, "submit-control");
    assert_eq!(attribute(submit, "type"), Some("submit"));
    assert_eq!(attribute(submit, "form"), Some("editor"));
    assert_eq!(attribute(submit, "aria-label"), Some("Save document"));
    assert_eq!(attribute(submit, "aria-describedby"), Some("save-help"));
    let (reset, _) = marked_control(&html, "reset-control");
    assert_eq!(attribute(reset, "type"), Some("reset"));
    assert!(has_attribute(reset, "disabled"));
    let (default, _) = marked_control(&html, "default-control");
    assert_eq!(attribute(default, "type"), Some("button"));
    assert!(!has_attribute(default, "disabled"));
    let (link, _) = marked_control(&html, "default-link");
    assert_eq!(attribute(link, "target"), Some("_self"));
    assert_eq!(attribute(link, "rel"), Some("noopener noreferrer"));
}

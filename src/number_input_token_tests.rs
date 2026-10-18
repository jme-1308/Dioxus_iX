// NumberInput rendering and controlled-prop regressions. CSS visibility, native
// pointer focus and numeric user requests require the paired runtime fixture.
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
fn number_stepper_request_and_hidden_state_preserve_always_mounted_source_buttons() {
    let html = render_fixture(|| {
        rsx! {
            for (name, disabled, readonly) in [("ready", false, false), ("disabled", true, false), ("readonly", false, true), ("both", true, true)] {
                for shown in [false, true] {
                    NumberInput { class: "number-{name}-{shown}", value: "10", min: 0.0, max: 10.0,
                        show_stepper_buttons: shown, disabled, readonly }
                }
            }
            Input { class: "ordinary-input", input_type: "number" }
        }
    });
    for (name, disabled, readonly) in [
        ("ready", false, false),
        ("disabled", true, false),
        ("readonly", false, true),
        ("both", true, true),
    ] {
        for shown in [false, true] {
            let (root, body) = part(&html, &format!("number-{name}-{shown}"));
            assert!(classes(root).contains(&"ix-input-native"));
            assert!(classes(root).contains(&"ix-number-input-native"));
            assert_eq!(
                classes(root).contains(&"ix-number-has-steppers"),
                shown,
                "requested min-width survives disabled/readonly visibility"
            );
            let native = openings(body)
                .find(|opening| opening.starts_with("<input "))
                .unwrap();
            assert_eq!(attribute(native, "type"), Some("number"));
            assert_eq!(has_attribute(native, "disabled"), disabled);
            assert_eq!(has_attribute(native, "readonly"), readonly);
            let (steppers, buttons) = part(body, "ix-number-steppers");
            assert_eq!(
                attribute(steppers, "data-hidden"),
                Some(if shown && !disabled && !readonly {
                    "false"
                } else {
                    "true"
                })
            );
            assert_eq!(
                openings(buttons)
                    .filter(|opening| opening.starts_with("<button "))
                    .count(),
                2
            );
            let mut positions = Vec::new();
            for (marker, label, icon) in [
                ("ix-number-minus", "decrement number", "minus"),
                ("ix-number-plus", "increment number", "plus"),
            ] {
                let (button, content) = part(buttons, marker);
                assert!(button.starts_with("<button "));
                for class in [
                    "ix-number-stepper",
                    "ix-button-tokenized",
                    "ix-icon-button-native",
                    "ix-button-subtle-tertiary",
                    "ix-icon-size-16",
                ] {
                    assert!(classes(button).contains(&class));
                }
                assert_eq!(attribute(button, "type"), Some("button"));
                assert_eq!(attribute(button, "aria-label"), Some(label));
                assert_eq!(attribute(button, "tabindex"), Some("0"));
                assert!(
                    !has_attribute(button, "disabled"),
                    "source hides the container rather than inventing a bound/readonly button-disabled state"
                );
                assert!(content.contains(crate::icons::icon_svg(icon).unwrap()));
                positions.push(buttons.find(marker).unwrap());
            }
            assert!(positions[0] < positions[1]);
        }
    }
    let (peer, body) = part(&html, "ordinary-input");
    assert!(!classes(peer).contains(&"ix-number-input-native"));
    assert!(!body.contains("ix-number-steppers"));
}

#[test]
fn number_field_forwards_native_constraints_slot_order_and_inline_or_tooltip_feedback() {
    let html = render_fixture(|| {
        rsx! {
            NumberInput { class: "constrained-number", id: "amount", name: "amount-name", value: "12", label: "Amount", required: true,
                min: -2.5, max: 10.0, step: 0.25, placeholder: "Enter amount", aria_label: "Invoice amount", pattern: "[0-9]+",
                text_alignment_end: false, show_stepper_buttons: true, state: FieldState::Warning, warning_text: "  Check bounds  ", helper_text: "Inactive helper",
                start: rsx! { span { class: "number-prefix", "EUR" } },
                end: rsx! { span { class: "number-suffix", "net" } } }
            NumberInput { class: "tooltip-number", id: "rate", value: "", label: "", show_text_as_tooltip: true,
                state: FieldState::Invalid, invalid_text: "  Rate needed  ", show_stepper_buttons: false }
        }
    });
    let (root, body) = part(&html, "constrained-number");
    assert!(classes(root).contains(&"ix-field-warning"));
    let native = openings(body)
        .find(|opening| opening.starts_with("<input "))
        .unwrap();
    for (name, value) in [
        ("id", "amount"),
        ("name", "amount-name"),
        ("type", "number"),
        ("value", "12"),
        ("min", "-2.5"),
        ("max", "10"),
        ("step", "0.25"),
        ("placeholder", "Enter amount"),
        ("aria-label", "Invoice amount"),
        ("pattern", "[0-9]+"),
        ("aria-describedby", "amount-help"),
    ] {
        assert_eq!(attribute(native, name), Some(value));
    }
    assert!(has_attribute(native, "required"));
    assert!(
        attribute(native, "style")
            .unwrap()
            .contains("text-align:start")
    );
    let (_, label) = part(body, "ix-text-field-label");
    assert_eq!(text_content(label), "Amount*");
    assert_eq!(
        attribute(
            openings(label)
                .find(|opening| opening.starts_with("<label "))
                .unwrap(),
            "for"
        ),
        Some("amount")
    );
    assert_eq!(text_content(part(body, "ix-input-start").1), "EUR");
    let (_, end) = part(body, "ix-input-end");
    assert!(end.find("number-suffix").unwrap() < end.find("ix-number-steppers").unwrap());
    assert_eq!(text_content(part(end, "number-suffix").1), "net");
    let (feedback, feedback_body) = part(body, "ix-text-field-bottom-text");
    assert_eq!(attribute(feedback, "id"), Some("amount-help"));
    assert!(classes(feedback).contains(&"ix-type-body-paragraph"));
    assert_eq!(text_content(feedback_body), "  Check bounds  ");
    assert!(!body.contains("Inactive helper"));
    let (_, tooltip) = part(&html, "tooltip-number");
    assert!(!tooltip.contains("ix-text-field-top"));
    let native = openings(tooltip)
        .find(|opening| opening.starts_with("<input "))
        .unwrap();
    assert_eq!(attribute(native, "aria-describedby"), Some("rate-help"));
    assert_eq!(attribute(native, "aria-errormessage"), Some("rate-help"));
    assert_eq!(tooltip.matches("role=\"tooltip\"").count(), 1);
    assert!(tooltip.contains("data-anchor-id=\"rate-input-wrap\""));
    assert_eq!(
        text_content(part(tooltip, "ix-text-field-bottom-text").1),
        "  Rate needed  "
    );
    let (_, bottom) = part(tooltip, "ix-text-field-bottom");
    assert!(!bottom.contains("ix-text-field-bottom-text"));
    assert!(!bottom.contains("ix-text-field-counter"));
    assert_eq!(
        attribute(part(tooltip, "ix-number-steppers").0, "data-hidden"),
        Some("true")
    );
}

#[derive(Clone)]
struct NumberLiveState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    requests: Rc<RefCell<Vec<String>>>,
}
fn live_number(state: NumberLiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let edits = state.requests.clone();
    let numeric = state.requests.clone();
    let commits = state.requests.clone();
    let blurs = state.requests.clone();
    let value = match current {
        1 => "-2.5",
        2 => "1e3",
        3 => "",
        _ => "12",
    };
    rsx! {
        NumberInput { class: "live-number", id: "live-amount", name: "amount", value,
            label: if current == 3 { None } else { Some("Amount".to_string()) },
            min: if current == 2 { None } else { Some(-5.0) }, max: if current == 2 { None } else { Some(20.0) }, step: if current == 1 { 0.5 } else { 1.0 },
            show_stepper_buttons: current != 3, disabled: current == 1, readonly: current == 2, required: current != 3,
            state: if current == 2 { FieldState::Invalid } else { FieldState::Default }, invalid_text: "Check number", helper_text: "Hint",
            show_text_as_tooltip: current == 2, allow_empty_value_change: true,
            end: if current == 3 { Some(rsx! { span { class: "live-unit", "units" } }) } else { None },
            onvaluechange: move |value| edits.borrow_mut().push(format!("edit:{value}")),
            onnumericvaluechange: move |value| numeric.borrow_mut().push(format!("numeric:{value:?}")),
            oncommit: move |value| commits.borrow_mut().push(format!("commit:{value:?}")),
            onblur: move |_| blurs.borrow_mut().push("blur".into()),
        }
    }
}

#[test]
fn number_controlled_updates_restore_draft_constraints_and_stepper_visibility_without_requests() {
    let state = NumberLiveState {
        stage: Rc::new(RefCell::new(None)),
        requests: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_number, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    for current in [0, 1, 2, 3, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        // The NumberInput draft follows its controlled value in a Rust effect.
        // Drain render work without supplying another native input/blur event.
        for _ in 0..3 {
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let (root, body) = part(&html, "live-number");
        let input = openings(body)
            .find(|opening| opening.starts_with("<input "))
            .unwrap();
        assert_eq!(
            attribute(input, "value"),
            Some(match current {
                1 => "-2.5",
                2 => "1e3",
                3 => "",
                _ => "12",
            })
        );
        assert_eq!(attribute(input, "name"), Some("amount"));
        assert_eq!(
            attribute(input, "min"),
            if current == 2 { None } else { Some("-5") }
        );
        assert_eq!(
            attribute(input, "max"),
            if current == 2 { None } else { Some("20") }
        );
        assert_eq!(
            attribute(input, "step"),
            Some(if current == 1 { "0.5" } else { "1" })
        );
        assert_eq!(has_attribute(input, "disabled"), current == 1);
        assert_eq!(has_attribute(input, "readonly"), current == 2);
        assert_eq!(has_attribute(input, "required"), current != 3);
        assert_eq!(
            attribute(input, "aria-invalid"),
            Some(if current == 2 { "true" } else { "false" })
        );
        assert_eq!(
            classes(root).contains(&"ix-number-has-steppers"),
            current != 3
        );
        assert_eq!(body.contains("ix-text-field-label"), current != 3);
        assert_eq!(body.contains("live-unit"), current == 3);
        assert_eq!(body.contains("role=\"tooltip\""), current == 2);
        let (steppers, children) = part(body, "ix-number-steppers");
        assert_eq!(
            attribute(steppers, "data-hidden"),
            Some(if current == 0 { "false" } else { "true" })
        );
        assert_eq!(
            openings(children)
                .filter(|opening| opening.starts_with("<button "))
                .count(),
            2
        );
        assert!(
            state.requests.borrow().is_empty(),
            "external props do not emit edit, numeric, commit or blur requests"
        );
    }
}

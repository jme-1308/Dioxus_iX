// Native markup and controlled projection tests. SVG/native pseudo paint,
// actual focus/drag/tooltip behavior and source cancellation require runtime evidence.
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

fn ratio(opening: &str, property: &str) -> f64 {
    attribute(opening, "style")
        .unwrap()
        .split(';')
        .find_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            (name.trim() == property).then(|| value.trim().parse().unwrap())
        })
        .unwrap_or_else(|| panic!("missing {property}: {opening}"))
}
fn native_input(html: &str) -> &str {
    openings(html)
        .find(|opening| {
            opening.starts_with("<input ") && attribute(opening, "type") == Some("range")
        })
        .unwrap()
}
fn ticks(html: &str) -> Vec<&str> {
    openings(html)
        .filter(|opening| {
            opening.starts_with("<circle ") && classes(opening).contains(&"ix-slider-tick")
        })
        .collect()
}

#[test]
fn slider_projects_ordered_numeric_ticks_and_inclusive_trace_independently_of_native_trace() {
    let html = render_fixture(|| {
        rsx! {
            Slider { class: "ordered-ticks", value: 25.0, trace: true, trace_reference: 75.0,
                markers: vec![75.0, 0.0, 25.0, 25.0, 100.0, -5.0, 105.0, f64::NAN].into_iter().map(|value| SliderMarker { value, label: String::new() }).collect::<Vec<_>>() }
            Slider { class: "equal-reference", value: 50.0, trace: true, trace_reference: 50.0,
                markers: vec![SliderMarker { value: 50.0, label: String::new() }] }
            for (name, reference, trace) in [("low-reference", -50.0, true), ("high-reference", 150.0, true), ("no-trace", 75.0, false)] {
                Slider { class: name, value: 25.0, trace_reference: reference, trace,
                    markers: vec![SliderMarker { value: 25.0, label: String::new() }] }
            }
        }
    });
    let (_, body) = part(&html, "ordered-ticks");
    let input = native_input(body);
    for (name, expected) in [
        ("--value", 0.25),
        ("--trace-reference", 0.75),
        ("--trace-start", 0.25),
        ("--trace-end", 0.75),
    ] {
        assert_eq!(ratio(input, name), expected);
    }
    assert!(classes(input).contains(&"trace"));
    assert!(!classes(input).contains(&"hide-trace-reference"));
    let circles = ticks(body);
    assert_eq!(
        circles
            .iter()
            .map(|circle| ratio(circle, "--tick-value"))
            .collect::<Vec<_>>(),
        vec![0.75, 0.0, 0.25, 0.25, 1.0]
    );
    assert_eq!(
        circles
            .iter()
            .map(|circle| classes(circle).contains(&"ix-slider-tick-active"))
            .collect::<Vec<_>>(),
        vec![true, false, true, true, false]
    );
    assert_eq!(
        circles
            .iter()
            .map(|circle| classes(circle).contains(&"ix-slider-tick-at-min"))
            .collect::<Vec<_>>(),
        vec![false, true, false, false, false]
    );
    assert_eq!(
        circles
            .iter()
            .map(|circle| classes(circle).contains(&"ix-slider-tick-at-max"))
            .collect::<Vec<_>>(),
        vec![false, false, false, false, true]
    );
    for circle in circles {
        assert_eq!(attribute(circle, "cx"), Some("0"));
        assert_eq!(attribute(circle, "cy"), Some("0"));
    }
    let (_, track) = part(body, "ix-slider-track");
    assert!(track.contains("ix-slider-thumb-anchor"));
    assert!(track.contains("ix-slider-ticks"));
    assert!(
        !track.contains("<input"),
        "the native input is a track sibling"
    );
    assert!(!body.contains("<datalist"));
    assert!(!body.contains("ix-slider-marker-captions"));
    let (_, equal) = part(&html, "equal-reference");
    assert!(!classes(native_input(equal)).contains(&"trace"));
    assert!(
        classes(ticks(equal)[0]).contains(&"ix-slider-tick-active"),
        "equal reference still includes its numeric tick"
    );
    for (name, reference) in [("low-reference", 0.0), ("high-reference", 1.0)] {
        let (_, body) = part(&html, name);
        assert_eq!(ratio(native_input(body), "--trace-reference"), reference);
        assert!(classes(native_input(body)).contains(&"hide-trace-reference"));
        assert!(classes(ticks(body)[0]).contains(&"ix-slider-tick-active"));
    }
    let (_, no_trace) = part(&html, "no-trace");
    assert!(!classes(native_input(no_trace)).contains(&"trace"));
    assert!(!classes(native_input(no_trace)).contains(&"hide-trace-reference"));
    assert!(!classes(ticks(no_trace)[0]).contains(&"ix-slider-tick-active"));
}

#[test]
fn slider_shared_field_anatomy_native_attributes_and_optional_adapters_remain_explicit() {
    let html = render_fixture(|| {
        rsx! {
            Slider { class: "plain-slider" }
            for (name, state, disabled) in [("default", FieldState::Default, false), ("invalid", FieldState::Invalid, false), ("valid", FieldState::Valid, false), ("info", FieldState::Info, false), ("warning", FieldState::Warning, true)] {
                Slider { class: "field-{name}", name: "amount", label: "Amount", value: 2.5, min: -5.0, max: 10.0, step: 0.5, state, disabled,
                    label_start: "Low", label_end: "High", helper_text: "Helper", invalid_text: "Invalid", valid_text: "Valid", info_text: "Info", warning_text: "Warning" }
            }
            Slider { class: "caption-slider", value: 30.0, show_value: true,
                markers: vec![SliderMarker { value: 30.0, label: "Target".into() }, SliderMarker { value: 60.0, label: String::new() }, SliderMarker { value: 120.0, label: "Outside".into() }] }
        }
    });
    let (plain, body) = part(&html, "plain-slider");
    assert!(classes(plain).contains(&"ix-slider-field"));
    assert!(classes(plain).contains(&"ix-text-field-native"));
    assert!(!classes(plain).contains(&"ix-field"));
    assert!(!body.contains("ix-text-field-top"));
    assert!(body.contains("ix-text-field-bottom-right"));
    assert!(!body.contains("ix-slider-value-output"));
    assert!(!body.contains("ix-slider-tooltip"));
    assert!(!body.contains("ix-slider-endpoints"));
    assert_eq!(attribute(native_input(body), "aria-label"), Some("Slider"));
    for (name, state, feedback, disabled) in [
        ("default", FieldState::Default, "Helper", false),
        ("invalid", FieldState::Invalid, "Invalid", false),
        ("valid", FieldState::Valid, "Valid", false),
        ("info", FieldState::Info, "Info", false),
        ("warning", FieldState::Warning, "Warning", true),
    ] {
        let (root, body) = part(&html, &format!("field-{name}"));
        if !state.class().is_empty() {
            assert!(classes(root).contains(&state.class()));
        }
        assert_eq!(
            attribute(root, "data-disabled"),
            Some(if disabled { "true" } else { "false" })
        );
        let input = native_input(body);
        for (name, expected) in [
            ("role", "slider"),
            ("name", "amount"),
            ("value", "2.5"),
            ("min", "-5"),
            ("max", "10"),
            ("step", "0.5"),
            ("aria-label", "Amount"),
            ("aria-valuenow", "2.5"),
            ("aria-valuemin", "-5"),
            ("aria-valuemax", "10"),
        ] {
            assert_eq!(attribute(input, name), Some(expected));
        }
        assert_eq!(has_attribute(input, "disabled"), disabled);
        assert_eq!(
            attribute(input, "tabindex"),
            Some(if disabled { "-1" } else { "0" })
        );
        assert_eq!(
            attribute(input, "aria-invalid"),
            Some(if state == FieldState::Invalid {
                "true"
            } else {
                "false"
            })
        );
        let (_, label) = part(body, "ix-text-field-label");
        assert_eq!(text_content(label), "Amount");
        assert_eq!(
            attribute(
                openings(label)
                    .find(|tag| tag.starts_with("<label "))
                    .unwrap(),
                "for"
            ),
            attribute(input, "id")
        );
        assert!(label.contains(if state == FieldState::Invalid {
            "ix-text-alarm"
        } else {
            "ix-text-soft"
        }));
        let (message, message_body) = part(body, "ix-text-field-bottom-text");
        assert_eq!(text_content(message_body), feedback);
        assert!(classes(message).contains(&"ix-type-body-paragraph"));
        assert_eq!(
            attribute(input, "aria-describedby"),
            attribute(message, "id")
        );
        assert_eq!(text_content(part(body, "ix-slider-endpoints").1), "LowHigh");
    }
    let (_, captions) = part(&html, "caption-slider");
    assert_eq!(
        text_content(part(captions, "ix-slider-value-output").1),
        "30"
    );
    assert_eq!(
        text_content(part(captions, "ix-slider-marker-captions").1),
        "Target"
    );
    assert_eq!(ticks(captions).len(), 2);
}

#[derive(Clone)]
struct SliderLiveState {
    stage: Rc<RefCell<Option<Signal<u8>>>>,
    requests: Rc<RefCell<Vec<f64>>>,
}
fn live_slider(state: SliderLiveState) -> Element {
    let stage = use_signal(|| 0u8);
    *state.stage.borrow_mut() = Some(stage);
    let current = stage();
    let requests = state.requests.clone();
    rsx! {
        Slider { class: "live-slider", name: "live-amount", value: if current == 1 { -5.0 } else { 25.0 },
            min: if current == 1 { -10.0 } else { 0.0 }, max: if current == 1 { 10.0 } else { 100.0 },
            step: if current == 1 { 0.25 } else { 1.0 }, trace_reference: if current == 1 { 5.0 } else { 75.0 }, trace: current != 2,
            label: if current == 2 { "" } else { "Amount" }, disabled: current == 1,
            state: if current == 1 { FieldState::Invalid } else { FieldState::Default }, helper_text: "Hint", invalid_text: "Check amount",
            show_value: current == 1, label_start: if current == 2 { None } else { Some("Low".to_string()) }, label_end: if current == 2 { None } else { Some("High".to_string()) },
            markers: if current == 1 { vec![SliderMarker { value: -10.0, label: String::new() }, SliderMarker { value: 5.0, label: "Target".into() }] } else { vec![SliderMarker { value: 0.0, label: String::new() }, SliderMarker { value: 50.0, label: String::new() }, SliderMarker { value: 100.0, label: String::new() }] },
            onvaluechange: move |value| requests.borrow_mut().push(value),
        }
    }
}

#[test]
fn slider_controlled_prop_revisions_restore_geometry_markers_and_feedback_without_requests() {
    let state = SliderLiveState {
        stage: Rc::new(RefCell::new(None)),
        requests: Rc::new(RefCell::new(Vec::new())),
    };
    let mut dom = VirtualDom::new_with_props(live_slider, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.stage.borrow().unwrap();
    let mut original_id = None;
    for current in [0, 1, 2, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let (root, body) = part(&html, "live-slider");
        let input = native_input(body);
        let id = attribute(input, "id").unwrap().to_string();
        assert_eq!(original_id.get_or_insert_with(|| id.clone()), &id);
        assert_eq!(attribute(input, "name"), Some("live-amount"));
        assert_eq!(
            attribute(input, "value"),
            Some(if current == 1 { "-5" } else { "25" })
        );
        assert_eq!(
            attribute(input, "min"),
            Some(if current == 1 { "-10" } else { "0" })
        );
        assert_eq!(
            attribute(input, "max"),
            Some(if current == 1 { "10" } else { "100" })
        );
        assert_eq!(
            attribute(input, "step"),
            Some(if current == 1 { "0.25" } else { "1" })
        );
        assert_eq!(ratio(input, "--value"), 0.25);
        assert_eq!(ratio(input, "--trace-reference"), 0.75);
        assert_eq!(ratio(input, "--trace-start"), 0.25);
        assert_eq!(ratio(input, "--trace-end"), 0.75);
        assert_eq!(classes(input).contains(&"trace"), current != 2);
        assert_eq!(has_attribute(input, "disabled"), current == 1);
        assert_eq!(
            attribute(root, "data-disabled"),
            Some(if current == 1 { "true" } else { "false" })
        );
        assert_eq!(body.contains("ix-slider-value-output"), current == 1);
        assert_eq!(body.contains("ix-slider-endpoints"), current != 2);
        assert_eq!(body.contains("ix-text-field-top"), current != 2);
        assert_eq!(
            text_content(part(body, "ix-text-field-bottom-text").1),
            if current == 1 { "Check amount" } else { "Hint" }
        );
        assert_eq!(
            ticks(body)
                .iter()
                .map(|tick| ratio(tick, "--tick-value"))
                .collect::<Vec<_>>(),
            if current == 1 {
                vec![0.0, 0.75]
            } else {
                vec![0.0, 0.5, 1.0]
            }
        );
        assert_eq!(body.contains("ix-slider-marker-captions"), current == 1);
        assert!(
            state.requests.borrow().is_empty(),
            "external controlled updates must not synthesize value requests"
        );
    }
}

#[test]
fn slider_equal_bounds_and_feedback_announcements_keep_their_actual_targets() {
    let html = render_fixture(|| {
        rsx! {
            Slider { class: "equal-bounds", min: 10.0, max: 10.0, value: 10.0,
                markers: vec![SliderMarker { value: 10.0, label: String::new() }] }
            Slider { class: "invalid-no-message", state: FieldState::Invalid, helper_text: "Hint" }
            Slider { class: "invalid-empty-message", state: FieldState::Invalid, invalid_text: "", helper_text: "Hint" }
            Slider { class: "feedback-anchor ix-invalid", label: "Level", invalid_text: "Check level",
                show_text_as_tooltip: true, label_start: "Low", label_end: "High" }
        }
    });
    let (_, bounded) = part(&html, "equal-bounds");
    assert_eq!(ratio(native_input(bounded), "--value"), 0.0);
    let tick = ticks(bounded)[0];
    assert_eq!(ratio(tick, "--tick-value"), 0.0);
    assert!(classes(tick).contains(&"ix-slider-tick-at-min"));
    assert!(!classes(tick).contains(&"ix-slider-tick-at-max"));
    for name in ["invalid-no-message", "invalid-empty-message"] {
        let (_, body) = part(&html, name);
        let (wrapper, _) = part(body, "ix-text-field-wrapper");
        assert!(!has_attribute(wrapper, "role"));
        assert!(!has_attribute(wrapper, "aria-live"));
        assert_eq!(
            text_content(part(body, "ix-text-field-bottom-text").1),
            "Hint"
        );
    }
    let (root, body) = part(&html, "feedback-anchor");
    assert!(classes(root).contains(&"ix-field-invalid"));
    let (wrapper, _) = part(body, "ix-text-field-wrapper");
    assert_eq!(attribute(wrapper, "role"), Some("alert"));
    let (container, container_body) = part(body, "ix-slider-container");
    assert!(attribute(container, "id").is_some());
    assert!(container_body.contains("ix-slider-endpoints"));
    assert!(!has_attribute(part(body, "ix-slider-control").0, "id"));
    assert_eq!(attribute(native_input(body), "aria-invalid"), Some("true"));
}

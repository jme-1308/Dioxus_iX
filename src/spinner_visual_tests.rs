// Rendering and retained-prop regressions. CSS animation, pseudo-element paint,
// dimensions, and computed tokens require the coordinated browser/native pass.
use super::*;

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
fn render_fixture(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}
fn assert_accessible_host(host: &str, label: &str) {
    assert_eq!(attribute(host, "role"), Some("status"));
    assert_eq!(attribute(host, "aria-busy"), Some("true"));
    assert_eq!(attribute(host, "aria-label"), Some(label));
}

#[test]
fn spinner_defaults_and_accessible_names_wrap_one_real_css_ring() {
    let html = render_fixture(|| {
        rsx! {
            div { class: "default-case", Spinner {} }
            div { class: "label-case", Spinner { class: "application-spinner", label: "Checking records" } }
            div { class: "empty-label-case", Spinner { label: "" } }
        }
    });
    for (case, label) in [
        ("default-case", "Loading"),
        ("label-case", "Checking records"),
        ("empty-label-case", ""),
    ] {
        let (_, body) = part(&html, case);
        let (host, host_body) = part(body, "ix-spinner");
        assert_accessible_host(host, label);
        for marker in ["ix-spinner-md", "ix-spinner-secondary"] {
            assert!(classes(host).contains(&marker));
        }
        assert_eq!(
            classes(host).contains(&"application-spinner"),
            case == "label-case"
        );
        assert!(!classes(host).contains(&"ix-spinner-no-track"));
        let (ring, ring_body) = part(host_body, "ix-spinner-ring");
        assert!(ring.starts_with("<div "));
        assert!(classes(ring).contains(&"md"));
        assert!(!classes(ring).contains(&"primary") && !classes(ring).contains(&"hide-track"));
        assert_eq!(attribute(ring, "aria-hidden"), Some("true"));
        assert_eq!(
            openings(ring_body)
                .filter(|tag| !tag.starts_with("</"))
                .count(),
            1
        );
        assert!(!host_body.contains("<svg") && !host_body.contains("<circle"));
        assert_eq!(
            openings(host_body)
                .filter(|tag| attribute(tag, "role") == Some("status"))
                .count(),
            1
        );
    }
}

#[test]
fn spinner_size_variant_and_track_props_reach_both_compatibility_host_and_source_inner_div() {
    let html = render_fixture(|| {
        rsx! {
            for size in [SpinnerSize::Xxs, SpinnerSize::Xs, SpinnerSize::Sm, SpinnerSize::Md, SpinnerSize::Xxl] {
                for variant in [SpinnerVariant::Secondary, SpinnerVariant::Primary] {
                    for hide_track in [false, true] {
                        div { class: "case-{size.as_str()}-{variant.as_str()}-{hide_track}",
                            Spinner { size, variant, hide_track, label: "Loading {size.as_str()} {variant.as_str()}" }
                        }
                    }
                }
            }
        }
    });
    for (size, token, nominal) in [
        (SpinnerSize::Xxs, "xxs", 12),
        (SpinnerSize::Xs, "xs", 16),
        (SpinnerSize::Sm, "sm", 20),
        (SpinnerSize::Md, "md", 32),
        (SpinnerSize::Xxl, "xxl", 96),
    ] {
        for (variant, primary) in [("secondary", false), ("primary", true)] {
            for hide_track in [false, true] {
                let (_, body) = part(&html, &format!("case-{token}-{variant}-{hide_track}"));
                let (host, host_body) = part(body, "ix-spinner");
                assert_accessible_host(host, &format!("Loading {token} {variant}"));
                assert!(classes(host).contains(&format!("ix-spinner-{token}").as_str()));
                assert!(classes(host).contains(&format!("ix-spinner-{variant}").as_str()));
                assert_eq!(classes(host).contains(&"ix-spinner-no-track"), hide_track);
                assert!(
                    attribute(host, "style")
                        .unwrap()
                        .contains(&format!("--ix-spinner-size:{nominal}px"))
                );
                let (ring, contents) = part(host_body, "ix-spinner-ring");
                assert!(ring.starts_with("<div "));
                assert!(classes(ring).contains(&size.as_str()));
                assert_eq!(classes(ring).contains(&"primary"), primary);
                assert_eq!(classes(ring).contains(&"hide-track"), hide_track);
                assert_eq!(attribute(ring, "aria-hidden"), Some("true"));
                assert!(!contents.contains("<svg") && !contents.contains("<circle"));
            }
        }
    }
}

#[derive(Clone)]
struct LiveSpinner(Rc<RefCell<Option<Signal<u8>>>>);
fn live_spinner(state: LiveSpinner) -> Element {
    let stage = use_signal(|| 0u8);
    *state.0.borrow_mut() = Some(stage);
    let current = stage();
    rsx! {
        Spinner {
            size: match current { 1 => SpinnerSize::Xxs, 2 => SpinnerSize::Xs, 3 => SpinnerSize::Sm, 4 => SpinnerSize::Xxl, _ => SpinnerSize::Md },
            variant: if current % 2 == 1 { SpinnerVariant::Primary } else { SpinnerVariant::Secondary },
            hide_track: current >= 2,
            class: if current == 0 { "initial-spinner" } else { "updated-spinner" },
            label: if current == 0 { "Initial activity" } else if current == 4 { "" } else { "Updated activity" },
        }
    }
}

#[test]
fn spinner_live_props_patch_the_retained_host_and_ring_without_replacing_their_nodes() {
    let state = LiveSpinner(Rc::new(RefCell::new(None)));
    let mut dom = VirtualDom::new_with_props(live_spinner, state.clone());
    dom.rebuild_in_place();
    let mut stage = state.0.borrow().unwrap();
    for current in [0, 1, 2, 3, 4, 0] {
        dom.in_scope(ScopeId::ROOT, || stage.set(current));
        let mutations = dom.render_immediate_to_vec();
        assert!(
            mutations.edits.iter().all(|edit| !matches!(
                edit,
                dioxus::core::Mutation::CreateElement { .. }
                    | dioxus::core::Mutation::CreateText { .. }
                    | dioxus::core::Mutation::Clone
                    | dioxus::core::Mutation::AppendChildren { .. }
                    | dioxus::core::Mutation::ReplaceWith { .. }
                    | dioxus::core::Mutation::InsertAfter { .. }
                    | dioxus::core::Mutation::InsertBefore { .. }
                    | dioxus::core::Mutation::Remove
            )),
            "controlled updates must retain actual host/ring renderer objects: {mutations:?}"
        );
        let html = dioxus_ssr::render(&dom);
        let (host, body) = part(&html, "ix-spinner");
        assert_accessible_host(
            host,
            if current == 0 {
                "Initial activity"
            } else if current == 4 {
                ""
            } else {
                "Updated activity"
            },
        );
        assert_eq!(classes(host).contains(&"initial-spinner"), current == 0);
        assert_eq!(classes(host).contains(&"updated-spinner"), current != 0);
        assert_eq!(classes(host).contains(&"ix-spinner-no-track"), current >= 2);
        let (ring, _) = part(body, "ix-spinner-ring");
        let token = match current {
            1 => "xxs",
            2 => "xs",
            3 => "sm",
            4 => "xxl",
            _ => "md",
        };
        for candidate in ["xxs", "xs", "sm", "md", "xxl"] {
            assert_eq!(classes(ring).contains(&candidate), candidate == token);
            assert_eq!(
                classes(host).contains(&format!("ix-spinner-{candidate}").as_str()),
                candidate == token
            );
        }
        assert_eq!(classes(ring).contains(&"primary"), current % 2 == 1);
        assert_eq!(classes(ring).contains(&"hide-track"), current >= 2);
        assert!(!body.contains("<svg") && !body.contains("<circle"));
    }
}

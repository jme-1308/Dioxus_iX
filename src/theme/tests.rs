use super::*;
use std::{cell::Cell, rc::Rc};

fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn rendered(build: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(build);
    dom.rebuild_in_place();
    settle(&mut dom);
    dioxus_ssr::render(&dom)
}
fn styles(html: &str) -> Vec<&str> {
    html.split("<style>")
        .skip(1)
        .map(|tail| tail.split_once("</style>").expect("closed stylesheet").0)
        .collect()
}
fn scope<'a>(html: &'a str, class: &str) -> &'a str {
    html.split('<')
        .filter_map(|part| part.split_once('>').map(|(tag, _)| tag))
        .find(|tag| {
            tag.starts_with("div ")
                && tag.split_once("class=\"").is_some_and(|(_, tail)| {
                    tail.split('"')
                        .next()
                        .unwrap()
                        .split_ascii_whitespace()
                        .any(|token| token == class)
                })
        })
        .unwrap_or_else(|| panic!("missing theme scope {class}"))
}
fn standalone() -> Element {
    rsx! {IxTheme {"Standalone content"}}
}
#[test]
fn standalone_theme_supplies_one_complete_bundle_and_nested_scopes_reuse_it() {
    let standalone = rendered(standalone);
    let expected = styles(&standalone);
    assert_eq!(expected.len(), 34);
    assert_eq!(expected[0], BASE_STYLES);
    assert_eq!(expected[33], crate::tree::STYLES);
    let nested = rendered(|| {
        rsx! {
            IxTheme {class:"outer",scheme:ColorScheme::Dark,
                IxTheme {class:"inner",scheme:ColorScheme::Light,density:Density::Compact,
                    IxTheme {class:"deepest",style:"--ix-space:11px", "Nested content"}
                }
            }
        }
    });
    assert_eq!(
        styles(&nested),
        expected,
        "descendants share the complete outer bundle"
    );
    assert_eq!(nested.matches("data-ix-theme=\"brand\"").count(), 3);
    assert!(scope(&nested, "outer").contains("data-ix-color-schema=\"dark\""));
    assert!(scope(&nested, "inner").contains("data-ix-color-schema=\"light\""));
    assert!(scope(&nested, "inner").contains("data-ix-density=\"compact\""));
    assert!(scope(&nested, "deepest").contains("--ix-space:11px"));
    assert!(nested.contains("Nested content"));
    assert!(!nested.contains("<script"));
    assert!(!nested.contains("<link"));
}

#[derive(Default)]
struct DraftCapture {
    draft: Cell<Option<Signal<String>>>,
    mounts: Cell<usize>,
}
impl PartialEq for DraftCapture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn ThemeDraft(capture: Rc<DraftCapture>) -> Element {
    use_hook(|| capture.mounts.set(capture.mounts.get() + 1));
    let mut draft = use_signal(|| String::from("Initial preparation"));
    capture.draft.set(Some(draft));
    rsx! {input {id:String::from("theme-private-draft"),value:draft(),oninput:move|event|draft.set(event.value())}}
}
#[derive(Default)]
struct NestedSettings {
    revision: Cell<usize>,
    capture: Rc<DraftCapture>,
}
fn changing_nested_theme(settings: Rc<NestedSettings>) -> Element {
    let revision = settings.revision.get();
    let compact = revision % 2 == 1;
    rsx! {
        IxTheme {class:format!("outer-{revision}"),scheme:if compact {ColorScheme::Light}else{ColorScheme::Dark},
            density:if compact {Density::Compact}else{Density::Normal},style:format!("--ix-space:{}px",8+revision),
            IxTheme {class:format!("inner-{revision}"),scheme:if compact {ColorScheme::Dark}else{ColorScheme::Light},
                density:if compact {Density::Normal}else{Density::Compact},style:format!("--si-sys-color-text-primary:var(--custom-ink-{revision})"),
                ThemeDraft {capture:settings.capture.clone()}
            }
        }
    }
}
#[test]
fn theme_updates_keep_the_outer_bundle_and_nested_child_state() {
    let baseline = rendered(standalone);
    let expected = styles(&baseline);
    let settings = Rc::new(NestedSettings::default());
    let mut dom = VirtualDom::new_with_props(changing_nested_theme, settings.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let mut draft = settings.capture.draft.get().unwrap();
    dom.in_scope(ScopeId::APP, || draft.set("Tools prepared".into()));
    settle(&mut dom);
    for revision in [1, 2, 3] {
        settings.revision.set(revision);
        dom.mark_dirty(ScopeId::APP);
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(
            styles(&html),
            expected,
            "outer rerenders retain the owning bundle"
        );
        let outer = scope(&html, &format!("outer-{revision}"));
        let inner = scope(&html, &format!("inner-{revision}"));
        let (outer_scheme, inner_scheme, outer_density, inner_density) = if revision % 2 == 1 {
            ("light", "dark", "compact", "normal")
        } else {
            ("dark", "light", "normal", "compact")
        };
        assert!(outer.contains(&format!("data-ix-color-schema=\"{outer_scheme}\"")));
        assert!(inner.contains(&format!("data-ix-color-schema=\"{inner_scheme}\"")));
        assert!(outer.contains(&format!("data-ix-density=\"{outer_density}\"")));
        assert!(inner.contains(&format!("data-ix-density=\"{inner_density}\"")));
        assert!(outer.contains(&format!("--ix-space:{}px", 8 + revision)));
        assert!(inner.contains(&format!(
            "--si-sys-color-text-primary:var(--custom-ink-{revision})"
        )));
        assert!(html.contains("value=\"Tools prepared\""));
        assert_eq!(
            settings.capture.mounts.get(),
            1,
            "theme props retain the child component"
        );
    }
}

fn peer_themes(first_visible: Rc<Cell<bool>>) -> Element {
    rsx! {
        section {id:"first-peer",
            if first_visible.get() { IxTheme {class:"first-root",scheme:ColorScheme::Light,"First root"} }
        }
        section {id:"second-peer",
            IxTheme {class:"second-root",scheme:ColorScheme::Dark,
                IxTheme {class:"second-nested",density:Density::Compact,"Second root content"}
            }
        }
    }
}
#[test]
fn independent_peers_retain_own_bundles_after_removal_and_fresh_roots_are_isolated() {
    let baseline = rendered(standalone);
    let expected = styles(&baseline);
    let first_visible = Rc::new(Cell::new(true));
    let mut dom = VirtualDom::new_with_props(peer_themes, first_visible.clone());
    dom.rebuild_in_place();
    settle(&mut dom);
    let initial = dioxus_ssr::render(&dom);
    let two_bundles = styles(&initial);
    assert_eq!(two_bundles.len(), 68);
    assert_eq!(&two_bundles[..34], expected.as_slice());
    assert_eq!(&two_bundles[34..], expected.as_slice());
    first_visible.set(false);
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    let remaining = dioxus_ssr::render(&dom);
    assert_eq!(styles(&remaining), expected);
    assert!(!remaining.contains("class=\"ix-theme first-root\""));
    assert!(scope(&remaining, "second-root").contains("data-ix-color-schema=\"dark\""));
    assert!(remaining.contains("Second root content"));
    first_visible.set(true);
    dom.mark_dirty(ScopeId::APP);
    settle(&mut dom);
    let restored = dioxus_ssr::render(&dom);
    assert_eq!(styles(&restored).len(), 68);
    assert_eq!(&styles(&restored)[..34], expected.as_slice());
    assert_eq!(&styles(&restored)[34..], expected.as_slice());
    // A concurrently live root and a root created after teardown both need
    // complete styles; ownership must not escape through a process global.
    assert_eq!(styles(&rendered(standalone)), expected);
    drop(dom);
    assert_eq!(styles(&rendered(standalone)), expected);
}

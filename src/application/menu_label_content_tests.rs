//! Actual MenuItem rendering and Dioxus child-scope retention for composed labels.
use super::*;

fn item_fragment<'a>(html: &'a str, id: &str, tag: &str) -> &'a str {
    let marker = format!("id=\"{id}\"");
    let position = html.find(&marker).expect("authored item id is rendered");
    let start = html[..position].rfind('<').unwrap();
    let end = html[position..]
        .find(&format!("</{tag}>"))
        .map(|end| position + end + tag.len() + 3)
        .unwrap();
    &html[start..end]
}
fn opening(fragment: &str) -> &str {
    fragment.split('>').next().unwrap()
}

#[test]
fn composed_menu_labels_preserve_semantic_names_native_controls_and_decorations() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            MenuItem {id:"plain-label",label:"Plain name",icon:"home",notifications:3}
            MenuItem {id:"composed-button",label:"Semantic button",icon:"plant",notifications:4,active:true,
                strong {"Visible plant"} span {" station"}
            }
            MenuItem {id:"composed-link",label:"Semantic link",href:"/network",target:"_blank",tooltip_text:"Link tooltip",icon:"network-device",notifications:5,
                em {"Visible network"}
            }
            MenuItem {id:"disabled-composed",label:"Unavailable link",href:"/restricted",disabled:true,
                strong {"Visible restricted"}
            }
            MenuItem {id:"disabled-button",label:"Unavailable button",disabled:true,
                span {"Visible disabled button"}
            }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let plain = item_fragment(&html, "plain-label", "button");
    assert!(plain.contains("<span class=\"ix-menu-label\">Plain name</span>"));
    assert!(opening(plain).contains("aria-label=\"Plain name\""));
    assert!(opening(plain).contains("title=\"Plain name\""));
    assert!(plain.contains("ix-menu-item-icon") && plain.contains("3 notifications"));

    let button = item_fragment(&html, "composed-button", "button");
    assert!(button.starts_with("<button "));
    for expected in [
        "aria-label=\"Semantic button\"",
        "title=\"Semantic button\"",
        "aria-current=\"page\"",
        "ix-menu-item-active",
        "type=\"button\"",
    ] {
        assert!(opening(button).contains(expected), "missing {expected}");
    }
    assert!(button.contains("<strong>Visible plant</strong>") && button.contains(" station"));
    assert!(!button.contains(">Semantic button<"));
    assert!(button.contains("ix-menu-item-icon") && button.contains("4 notifications"));

    let link = item_fragment(&html, "composed-link", "a");
    assert!(link.starts_with("<a "));
    for expected in [
        "aria-label=\"Semantic link\"",
        "title=\"Link tooltip\"",
        "href=\"/network\"",
        "target=\"_blank\"",
        "rel=\"noopener noreferrer\"",
    ] {
        assert!(opening(link).contains(expected), "missing {expected}");
    }
    assert!(link.contains("<em>Visible network</em>"));
    assert!(link.contains("ix-menu-item-icon") && link.contains("5 notifications"));

    let disabled = item_fragment(&html, "disabled-composed", "a");
    assert!(!opening(disabled).contains("href="));
    assert!(opening(disabled).contains("aria-disabled=\"true\""));
    assert!(
        opening(disabled).contains("tabindex=\"-1\"") || opening(disabled).contains("tabindex=-1")
    );
    assert!(opening(disabled).contains("aria-label=\"Unavailable link\""));
    assert!(disabled.contains("<strong>Visible restricted</strong>"));
    let disabled_button = item_fragment(&html, "disabled-button", "button");
    assert!(opening(disabled_button).contains(" disabled"));
    assert!(opening(disabled_button).contains("aria-disabled=\"true\""));
    assert!(disabled_button.contains("Visible disabled button"));
}

#[derive(Default)]
struct LabelContentCapture {
    text: Cell<Option<Signal<String>>>,
    private_text: Cell<Option<Signal<String>>>,
    active: Cell<Option<Signal<bool>>>,
    hidden: Cell<Option<Signal<bool>>>,
    parent_renders: Cell<usize>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
}
impl PartialEq for LabelContentCapture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn ReactiveLabelContent(capture: Rc<LabelContentCapture>) -> Element {
    use_hook(|| capture.mounts.set(capture.mounts.get() + 1));
    use_drop({
        let capture = capture.clone();
        move || capture.drops.set(capture.drops.get() + 1)
    });
    let private_text = use_signal(|| String::from("Initial local content"));
    capture.private_text.set(Some(private_text));
    let text = capture.text.get().unwrap();
    rsx! {strong {"{text}"} span {"{private_text}"}}
}
fn label_content_fixture(capture: Rc<LabelContentCapture>) -> Element {
    let text = use_signal(|| String::from("Initial visible label"));
    let active = use_signal(|| false);
    let hidden = use_signal(|| false);
    capture.text.set(Some(text));
    capture.active.set(Some(active));
    capture.hidden.set(Some(hidden));
    capture.parent_renders.set(capture.parent_renders.get() + 1);
    rsx! {MenuItem {id:"reactive-label",label:"Stable semantic name",active:active(),hidden:hidden(),
        ReactiveLabelContent {capture}
    }}
}
fn settle_label_content(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
#[test]
fn composed_menu_label_child_reacts_independently_and_retains_scope_through_item_states() {
    let capture = Rc::new(LabelContentCapture::default());
    let mut dom = VirtualDom::new_with_props(label_content_fixture, capture.clone());
    dom.rebuild_in_place();
    settle_label_content(&mut dom);
    let renders = capture.parent_renders.get();
    dom.in_scope(ScopeId::APP, || {
        capture
            .text
            .get()
            .unwrap()
            .set("Updated visible label".into());
        capture
            .private_text
            .get()
            .unwrap()
            .set("Retained local content".into());
    });
    settle_label_content(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Updated visible label") && html.contains("Retained local content"));
    assert_eq!(
        capture.parent_renders.get(),
        renders,
        "child signal reads own their subscriptions"
    );
    for (active, hidden) in [(true, false), (true, true), (false, true), (false, false)] {
        dom.in_scope(ScopeId::APP, || {
            capture.active.get().unwrap().set(active);
            capture.hidden.get().unwrap().set(hidden);
        });
        settle_label_content(&mut dom);
        let html = dioxus_ssr::render(&dom);
        let item = item_fragment(&html, "reactive-label", "button");
        assert_eq!(opening(item).contains("ix-menu-item-active"), active);
        assert_eq!(opening(item).contains(" hidden"), hidden);
        assert!(opening(item).contains("aria-label=\"Stable semantic name\""));
        assert!(opening(item).contains("title=\"Stable semantic name\""));
        assert!(item.contains("Updated visible label") && item.contains("Retained local content"));
        assert_eq!(capture.mounts.get(), 1);
        assert_eq!(capture.drops.get(), 0);
    }
    drop(dom);
    assert_eq!(capture.drops.get(), 1);
}

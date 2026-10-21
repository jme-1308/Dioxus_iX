use super::*;
use crate::Content;

fn tag<'a>(html: &'a str, element: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag.starts_with(&format!("{element} ")))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    tag.split_once(&format!(" {name}=\""))?
        .1
        .split_once('"')
        .map(|pair| pair.0)
}
fn defaults() -> Element {
    rsx! { Content { "Page contents" } InfoPage {title_text:"Not found"} }
}
#[test]
fn page_roots_keep_native_semantics_and_default_content_without_extra_wrappers() {
    let mut dom = VirtualDom::new(defaults);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("<main ").count(), 1);
    assert_eq!(html.matches("<section ").count(), 1);
    assert!(
        attr(tag(&html, "main"), "class")
            .unwrap()
            .split_whitespace()
            .eq(["ix-content"])
    );
    assert!(
        attr(tag(&html, "section"), "class")
            .unwrap()
            .split_whitespace()
            .eq(["ix-info-page"])
    );
    assert!(html.contains("Page contents"));
    assert!(html.contains("<h1>Not found</h1>"));
    assert!(html.contains("ix-info-page-image"));
    assert!(!html.contains("ix-info-page-actions"));
}
#[derive(Default)]
struct PageData {
    revision: Cell<usize>,
    epoch: Cell<usize>,
    visible: Cell<bool>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
    deliveries: RefCell<Vec<(usize, usize, Rc<MountedData>)>>,
}
impl PartialEq for PageData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn PrivateDraft(data: Rc<PageData>) -> Element {
    let mut value = use_signal(|| "Initial".to_string());
    use_hook({
        let data = data.clone();
        move || data.mounts.set(data.mounts.get() + 1)
    });
    use_drop(move || data.drops.set(data.drops.get() + 1));
    rsx! {input {value:value(),oninput:move|event|value.set(event.value())}}
}
#[component]
fn PageRoots(data: Rc<PageData>, revision: usize) -> Element {
    let content = data.clone();
    let info = data.clone();
    // Explicit class props compose with required library classes. A raw spread
    // class must not silently replace the root's styling contract.
    let raw_class = || {
        vec![Attribute::new(
            "class",
            "replace-library-class",
            None,
            false,
        )]
    };
    rsx! {
        Content {header:if revision==0 {None} else {Some(rsx!{h2 {"Current editor header {revision}"}})},id:format!("page-{revision}"),class:format!("consumer-main-{revision}"),style:format!("padding:{}px",revision+4),tabindex:"-1",
            aria_label:format!("Editor {revision}"),"data-stage":format!("editing-{revision}"),attributes:raw_class(),
            onmounted:move|event:MountedEvent|content.deliveries.borrow_mut().push((0,revision,event.data())),
            PrivateDraft {data:data.clone()}
        }
        InfoPage {title_text:format!("Missing asset {revision}"),copy_text:format!("Lookup {revision}"),instructions:format!("Choose another {revision}"),
            image:rsx!{span {"Illustration {revision}"}},actions:rsx!{PrivateDraft {data:data.clone()}},
            id:format!("recovery-{revision}"),class:format!("consumer-info-{revision}"),style:format!("margin:{}px",revision+6),tabindex:"-1",
            aria_label:format!("Recovery {revision}"),"data-stage":format!("recovery-{revision}"),attributes:raw_class(),
            onmounted:move|event:MountedEvent|info.deliveries.borrow_mut().push((1,revision,event.data()))
        }
    }
}
fn changing_pages(data: Rc<PageData>) -> Element {
    rsx! {if data.visible.get() {PageRoots {key:"{data.epoch.get()}",data:data.clone(),revision:data.revision.get()}}}
}
fn setup() -> (VirtualDom, Rc<PageData>, dioxus::core::Mutations) {
    let data = Rc::new(PageData::default());
    data.visible.set(true);
    let mut dom = VirtualDom::new_with_props(changing_pages, data.clone());
    let edits = dom.rebuild_to_vec();
    (dom, data, edits)
}
fn refresh(dom: &mut VirtualDom) -> dioxus::core::Mutations {
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec()
}
#[test]
fn page_root_native_attributes_update_without_losing_private_child_drafts() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(FilterTestConverter));
    let (mut dom, data, edits) = setup();
    let inputs = filter_listener_ids_ref(&edits, "input");
    assert_eq!(inputs.len(), 2);
    assert!(!dioxus_ssr::render(&dom).contains("Current editor header"));
    for (input, value) in inputs.into_iter().zip(["Editor draft", "Recovery draft"]) {
        dispatch_filter_event(&dom, "input", input, value.to_string());
    }
    dom.render_immediate_to_vec();
    data.revision.set(1);
    let edits = refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    for (element, id, class, style, label, stage) in [
        (
            "main",
            "page-1",
            "ix-content consumer-main-1",
            "padding:5px",
            "Editor 1",
            "editing-1",
        ),
        (
            "section",
            "recovery-1",
            "ix-info-page consumer-info-1",
            "margin:7px",
            "Recovery 1",
            "recovery-1",
        ),
    ] {
        let root = tag(&html, element);
        for (name, value) in [
            ("id", id),
            ("class", class),
            ("style", style),
            ("tabindex", "-1"),
            ("aria-label", label),
            ("data-stage", stage),
        ] {
            assert_eq!(attr(root, name), Some(value), "{element} {name}");
        }
    }
    assert!(!html.contains("replace-library-class"));
    for expected in [
        "Current editor header 1",
        "Editor draft",
        "Recovery draft",
        "Missing asset 1",
        "Lookup 1",
        "Choose another 1",
        "Illustration 1",
    ] {
        assert!(html.contains(expected), "{expected}");
    }
    assert_eq!(data.mounts.get(), 2);
    assert_eq!(data.drops.get(), 0);
    assert!(filter_listener_ids_ref(&edits, "mounted").is_empty());
    assert!(html.find("Current editor header 1").unwrap() < html.find("Editor draft").unwrap());
    data.revision.set(0);
    refresh(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("Current editor header"));
    assert!(html.contains("Editor draft"));
    assert_eq!(
        (data.mounts.get(), data.drops.get()),
        (2, 0),
        "removing the header preserves the body and action scopes"
    );
}
#[test]
fn page_root_mounts_forward_actual_handles_current_callbacks_and_keyed_cleanup() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(FilterTestConverter));
    let (mut dom, data, edits) = setup();
    let targets = filter_listener_ids_ref(&edits, "mounted");
    assert_eq!(
        targets.len(),
        2,
        "only the actual main and section expose public mounts"
    );
    let backings = [Rc::new(Cell::new(0_usize)), Rc::new(Cell::new(0_usize))];
    for (target, backing) in targets.iter().zip(&backings) {
        dispatch_filter_event(&dom, "mounted", *target, backing.clone());
    }
    assert_eq!(data.deliveries.borrow().len(), 2);
    for (index, revision, node) in data.deliveries.borrow().iter() {
        assert_eq!(*revision, 0);
        assert!(Rc::ptr_eq(
            &node.downcast::<FilterTestMount>().unwrap().0,
            &backings[*index]
        ));
    }
    data.revision.set(1);
    refresh(&mut dom);
    assert_eq!(
        data.deliveries.borrow().len(),
        2,
        "prop updates do not replay mount"
    );
    for (target, backing) in targets.iter().zip(&backings) {
        dispatch_filter_event(&dom, "mounted", *target, backing.clone());
    }
    assert!(
        data.deliveries.borrow()[2..]
            .iter()
            .all(|(_, revision, _)| *revision == 1)
    );
    let old = data.deliveries.borrow()[0].2.clone();
    data.epoch.set(1);
    let edits = refresh(&mut dom);
    let replacements = filter_listener_ids_ref(&edits, "mounted");
    assert_eq!(replacements.len(), 2);
    for target in replacements {
        dispatch_filter_event(&dom, "mounted", target, Rc::new(Cell::new(0_usize)));
    }
    assert_eq!(data.deliveries.borrow().len(), 6);
    assert!(!Rc::ptr_eq(&old, &data.deliveries.borrow()[4].2));
    assert_eq!((data.mounts.get(), data.drops.get()), (4, 2));
    data.visible.set(false);
    refresh(&mut dom);
    assert_eq!(data.drops.get(), 4);
    assert_eq!(data.deliveries.borrow().len(), 6);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("<main"));
    assert!(!html.contains("<section"));
}

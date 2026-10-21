use super::*;

fn layout_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    tag.split_once(&format!(" {name}=\""))?
        .1
        .split_once('"')
        .map(|pair| pair.0)
}
fn layout_tag<'a>(html: &'a str, class: &str) -> &'a str {
    html.split('<')
        .find(|tag| {
            tag.starts_with("div ")
                && layout_attr(tag, "class")
                    .is_some_and(|classes| classes.split_whitespace().any(|value| value == class))
        })
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
#[test]
fn layout_auto_default_ids_are_private_distinct_scopes_and_keep_default_breakpoints() {
    fn siblings() -> Element {
        rsx! { LayoutAuto { LayoutAutoItem { "First" } } LayoutAuto { LayoutAutoItem { colspan:0,"Second" } } }
    }
    let mut dom = VirtualDom::new(siblings);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let roots: Vec<_> = html
        .split('<')
        .filter(|tag| tag.starts_with("div ") && tag.contains("data-ix-layout-scope="))
        .collect();
    assert_eq!(roots.len(), 2);
    let first = layout_attr(roots[0], "data-ix-layout-scope").unwrap();
    let second = layout_attr(roots[1], "data-ix-layout-scope").unwrap();
    assert_ne!(first, second);
    assert_eq!(layout_attr(roots[0], "id"), Some(first));
    assert_eq!(layout_attr(roots[1], "id"), Some(second));
    assert_eq!(html.matches("min-width:48em").count(), 2);
    assert_eq!(html.matches("--ix-colspan:1").count(), 2);
}

#[derive(Default)]
struct LayoutData {
    revision: Cell<usize>,
    visible: Cell<bool>,
    child_mounts: Cell<usize>,
    child_drops: Cell<usize>,
    deliveries: RefCell<Vec<(usize, usize, Rc<MountedData>)>>,
}
impl PartialEq for LayoutData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn LayoutDraft(data: Rc<LayoutData>) -> Element {
    let mut draft = use_signal(|| "Initial layout draft".to_string());
    use_hook({
        let data = data.clone();
        move || data.child_mounts.set(data.child_mounts.get() + 1)
    });
    use_drop(move || data.child_drops.set(data.child_drops.get() + 1));
    rsx! { input {value:draft(),oninput:move |event|draft.set(event.value())} }
}
fn changing_layout(data: Rc<LayoutData>) -> Element {
    let revision = data.revision.get();
    let root = data.clone();
    let item = data.clone();
    rsx! {
        if data.visible.get() {
            LayoutAuto { id:if revision==0 {"consumer:layout[0]"} else {r#"x"}body{display:none}/*"#},
                class:format!("consumer-layout-{revision}"),style:format!("gap:{}px",revision+8),
                layout:vec![AutoLayoutBreakpoint::new("0",1),AutoLayoutBreakpoint::new("40em",revision+2)],
                onmounted:move |event:MountedEvent|root.deliveries.borrow_mut().push((0,revision,event.data())),
                LayoutAutoItem {id:format!("consumer-item-{revision}"),class:format!("consumer-item-{revision}"),colspan:revision+1,
                    onmounted:move |event:MountedEvent|item.deliveries.borrow_mut().push((1,revision,event.data())),
                    LayoutDraft {data:data.clone()}
                }
            }
        }
    }
}
#[test]
fn layout_auto_live_props_retain_native_roots_current_callbacks_and_child_draft() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(FilterTestConverter));
    let data = Rc::new(LayoutData::default());
    data.visible.set(true);
    let mut dom = VirtualDom::new_with_props(changing_layout, data.clone());
    let initial = dom.rebuild_to_vec();
    let mounts = filter_listener_ids_ref(&initial, "mounted");
    assert_eq!(
        mounts.len(),
        2,
        "only the actual layout and item divs mount"
    );
    let backings: Vec<_> = (0..2).map(|_| Rc::new(Cell::new(0_usize))).collect();
    for (&target, backing) in mounts.iter().zip(&backings) {
        dispatch_filter_event(&dom, "mounted", target, backing.clone());
    }
    let first = dioxus_ssr::render(&dom);
    let scope = layout_attr(layout_tag(&first, "ix-layout-auto"), "data-ix-layout-scope")
        .unwrap()
        .to_string();
    let input = filter_listener_ids_ref(&initial, "input")[0];
    dispatch_filter_event(
        &dom,
        "input",
        input,
        String::from("Retained native composition draft"),
    );
    dom.render_immediate_to_vec();
    data.revision.set(1);
    dom.mark_dirty(ScopeId::APP);
    let update = dom.render_immediate_to_vec();
    assert!(filter_listener_ids_ref(&update, "mounted").is_empty());
    let html = dioxus_ssr::render(&dom);
    let root = layout_tag(&html, "ix-layout-auto");
    assert_eq!(
        layout_attr(root, "data-ix-layout-scope"),
        Some(scope.as_str())
    );
    assert_eq!(
        layout_attr(root, "class"),
        Some("ix-layout-auto consumer-layout-1")
    );
    assert_eq!(layout_attr(root, "style"), Some("gap:9px"));
    assert!(root.contains("id=\"x&#34;}body{display:none}/*\""));
    let css = html
        .split_once("<style>")
        .unwrap()
        .1
        .split_once("</style>")
        .unwrap()
        .0;
    assert!(css.contains(&format!("[data-ix-layout-scope=\"{scope}\"]")));
    assert!(css.contains("min-width:40em") && css.contains("--ix-layout-columns:3"));
    assert!(!css.contains("body") && !css.contains("consumer:layout"));
    let item = layout_tag(&html, "ix-layout-auto-item");
    assert_eq!(layout_attr(item, "id"), Some("consumer-item-1"));
    assert_eq!(
        layout_attr(item, "class"),
        Some("ix-layout-auto-item consumer-item-1")
    );
    assert_eq!(layout_attr(item, "style"), Some("--ix-colspan:2"));
    assert!(html.contains("Retained native composition draft"));
    assert_eq!(data.child_mounts.get(), 1);
    assert_eq!(data.child_drops.get(), 0);
    assert_eq!(
        data.deliveries.borrow().len(),
        2,
        "prop changes do not replay mounts"
    );
    for (&target, backing) in mounts.iter().zip(&backings) {
        dispatch_filter_event(&dom, "mounted", target, backing.clone());
    }
    let deliveries = data.deliveries.borrow();
    assert_eq!(deliveries.len(), 4);
    for (delivery, (index, revision, node)) in deliveries.iter().enumerate() {
        assert_eq!(
            *revision,
            delivery / 2,
            "retained native roots invoke current callbacks"
        );
        assert!(Rc::ptr_eq(
            &node.downcast::<FilterTestMount>().unwrap().0,
            &backings[*index]
        ));
    }
    drop(deliveries);
    data.visible.set(false);
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(data.child_drops.get(), 1);
    assert!(!dioxus_ssr::render(&dom).contains("ix-layout-auto"));
}

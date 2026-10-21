use super::*;

fn grid_tag<'a>(html: &'a str, class: &str) -> &'a str {
    html.split('<')
        .find(|tag| tag.starts_with("div ") && tag.contains(&format!("class=\"{class}")))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn grid_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    tag.split_once(&format!(" {name}=\""))?
        .1
        .split_once('"')
        .map(|p| p.0)
}
fn grid_css(html: &str) -> &str {
    html.split_once("<style>")
        .unwrap()
        .1
        .split_once("</style>")
        .unwrap()
        .0
}
fn default_grids() -> Element {
    rsx! {
        CssGrid { GridArea { name:"main", "Default area" } }
        CssGrid { GridArea { name:"a;display:none", "Invalid area" } }
    }
}
#[test]
fn css_grid_default_roots_are_distinct_and_invalid_area_keeps_auto_placement() {
    let mut dom = VirtualDom::new(default_grids);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let roots: Vec<_> = html
        .split('<')
        .filter(|tag| tag.starts_with("div ") && tag.contains("data-ix-grid-scope="))
        .collect();
    assert_eq!(roots.len(), 2);
    let first = grid_attr(roots[0], "data-ix-grid-scope").unwrap();
    let second = grid_attr(roots[1], "data-ix-grid-scope").unwrap();
    assert_ne!(first, second);
    assert_eq!(grid_attr(roots[0], "id"), Some(first));
    assert_eq!(grid_attr(roots[1], "id"), Some(second));
    let areas: Vec<_> = html
        .split('<')
        .filter(|tag| tag.starts_with("div ") && tag.contains("ix-grid-area"))
        .collect();
    assert_eq!(grid_attr(areas[0], "style"), Some("grid-area:main"));
    assert_eq!(grid_attr(areas[1], "style"), None);
    assert!(html.contains("Default area"));
    assert!(html.contains("Invalid area"));
}

#[derive(Default)]
struct GridData {
    revision: Cell<usize>,
    epoch: Cell<usize>,
    visible: Cell<bool>,
    child_mounts: Cell<usize>,
    child_drops: Cell<usize>,
    deliveries: RefCell<Vec<(usize, usize, Rc<MountedData>)>>,
}
impl PartialEq for GridData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn GridDraft(data: Rc<GridData>) -> Element {
    let mut draft = use_signal(|| "Initial draft".to_string());
    use_hook({
        let data = data.clone();
        move || data.child_mounts.set(data.child_mounts.get() + 1)
    });
    use_drop(move || data.child_drops.set(data.child_drops.get() + 1));
    rsx! { input { value:draft(), oninput:move|event|draft.set(event.value()) } }
}
fn changing_grid(data: Rc<GridData>) -> Element {
    let revision = data.revision.get();
    let root = data.clone();
    let area = data.clone();
    let templates = GridTemplates {
        lg: Some(
            GridTemplate::from_rows(if revision == 0 {
                &[&["main", "aside"]]
            } else {
                &[&["updated"], &["aside"]]
            })
            .unwrap(),
        ),
        ..Default::default()
    };
    rsx! {
        if data.visible.get() {
            CssGrid { key:"{data.epoch.get()}",id:if revision == 0 {"grid:initial[1]"} else {r#"x"}body{display:none}/*"#},
                class:format!("consumer-grid-{revision}"),style:format!("gap:{}px",revision+8),templates,
                onmounted:move|event:MountedEvent|root.deliveries.borrow_mut().push((0,revision,event.data())),
                GridArea { id:format!("consumer-area-{revision}"),name:if revision == 0 {"main"} else {"updated"},class:format!("consumer-area-{revision}"),
                    onmounted:move|event:MountedEvent|area.deliveries.borrow_mut().push((1,revision,event.data())),
                    GridDraft { data:data.clone() }
                }
                GridArea { name:"aside", "Context" }
            }
        }
    }
}
fn new_grid_dom() -> (VirtualDom, Rc<GridData>, dioxus::core::Mutations) {
    let data = Rc::new(GridData::default());
    data.visible.set(true);
    let mut dom = VirtualDom::new_with_props(changing_grid, data.clone());
    let mutations = dom.rebuild_to_vec();
    (dom, data, mutations)
}
fn refresh_grid(dom: &mut VirtualDom) -> dioxus::core::Mutations {
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec()
}
#[test]
fn css_grid_live_ids_templates_and_area_props_preserve_private_child_state() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(FilterTestConverter));
    let (mut dom, data, mutations) = new_grid_dom();
    let first = dioxus_ssr::render(&dom);
    let scope = grid_attr(grid_tag(&first, "ix-css-grid"), "data-ix-grid-scope")
        .unwrap()
        .to_string();
    let input = filter_listener_ids_ref(&mutations, "input")[0];
    dispatch_filter_event(&dom, "input", input, "Retained private draft".to_string());
    dom.render_immediate_to_vec();
    data.revision.set(1);
    let updates = refresh_grid(&mut dom);
    let html = dioxus_ssr::render(&dom);
    let root = grid_tag(&html, "ix-css-grid");
    assert_eq!(grid_attr(root, "data-ix-grid-scope"), Some(scope.as_str()));
    assert_eq!(
        grid_attr(root, "class"),
        Some("ix-css-grid consumer-grid-1")
    );
    assert_eq!(grid_attr(root, "style"), Some("gap:9px"));
    assert!(root.contains("id=\"x&#34;}body{display:none}/*\""));
    let css = grid_css(&html);
    assert!(!css.contains("body"));
    assert!(!css.contains("grid:initial"));
    assert!(css.contains(&scope));
    assert_eq!(
        css.matches("grid-template-areas:\"updated\" \"aside\"")
            .count(),
        3
    );
    assert_eq!(
        css.matches("grid-template-columns:repeat(1,minmax(0,1fr))")
            .count(),
        3
    );
    let area = grid_tag(&html, "ix-grid-area");
    assert_eq!(grid_attr(area, "id"), Some("consumer-area-1"));
    assert_eq!(
        grid_attr(area, "class"),
        Some("ix-grid-area consumer-area-1")
    );
    assert_eq!(grid_attr(area, "style"), Some("grid-area:updated"));
    assert!(html.contains("Retained private draft"));
    assert_eq!(data.child_mounts.get(), 1);
    assert_eq!(data.child_drops.get(), 0);
    assert!(
        filter_listener_ids_ref(&updates, "mounted").is_empty(),
        "prop updates do not replace the actual roots"
    );
}

#[test]
fn css_grid_mounted_callbacks_receive_real_roots_current_handlers_and_keyed_lifetimes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(FilterTestConverter));
    let (mut dom, data, mutations) = new_grid_dom();
    let targets = filter_listener_ids_ref(&mutations, "mounted");
    assert_eq!(
        targets.len(),
        3,
        "one grid and its two area divs; no synthetic wrapper mount"
    );
    let mounts: Vec<_> = (0..3).map(|_| Rc::new(Cell::new(0_usize))).collect();
    for (&target, mount) in targets.iter().zip(&mounts) {
        dispatch_filter_event(&dom, "mounted", target, mount.clone());
    }
    assert_eq!(
        data.deliveries.borrow().len(),
        2,
        "only the requested grid/area callbacks deliver"
    );
    for (index, revision, node) in data.deliveries.borrow().iter() {
        assert_eq!(*revision, 0);
        assert!(Rc::ptr_eq(
            &node.downcast::<FilterTestMount>().unwrap().0,
            &mounts[*index]
        ));
    }
    let old = data.deliveries.borrow()[0].2.clone();
    data.revision.set(1);
    refresh_grid(&mut dom);
    assert_eq!(
        data.deliveries.borrow().len(),
        2,
        "ordinary updates never replay mounts"
    );
    for (&target, mount) in targets.iter().take(2).zip(&mounts) {
        dispatch_filter_event(&dom, "mounted", target, mount.clone());
    }
    assert!(
        data.deliveries.borrow()[2..]
            .iter()
            .all(|(_, revision, _)| *revision == 1),
        "the retained div event invokes the current callback"
    );
    data.epoch.set(1);
    let replacement = refresh_grid(&mut dom);
    assert_eq!(data.child_mounts.get(), 2);
    assert_eq!(data.child_drops.get(), 1);
    let replacements = filter_listener_ids_ref(&replacement, "mounted");
    assert_eq!(replacements.len(), 3);
    for target in replacements {
        dispatch_filter_event(&dom, "mounted", target, Rc::new(Cell::new(0_usize)));
    }
    assert_eq!(data.deliveries.borrow().len(), 6);
    assert!(!Rc::ptr_eq(&old, &data.deliveries.borrow()[4].2));
    data.visible.set(false);
    refresh_grid(&mut dom);
    assert_eq!(data.child_drops.get(), 2);
    assert_eq!(
        data.deliveries.borrow().len(),
        6,
        "removal never replays a callback"
    );
    assert!(!dioxus_ssr::render(&dom).contains("ix-css-grid"));
}

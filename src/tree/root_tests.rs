use dioxus::core::{ElementId, Mutations};
use std::cell::Cell;

struct RootCapture {
    model: bool,
    visible: Cell<bool>,
    id: RefCell<Option<String>>,
    revision: Cell<usize>,
    runtimes: RefCell<BTreeMap<String, TreeDom>>,
    deliveries: RefCell<Vec<(usize, Rc<MountedData>)>>,
    mounts: Cell<usize>,
    drops: Cell<usize>,
    selected: RefCell<Vec<String>>,
    controller: Cell<Option<TreeController<String>>>,
}
impl PartialEq for RootCapture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl RootCapture {
    fn new(model: bool) -> Self {
        Self {
            model,
            visible: Cell::new(true),
            id: RefCell::new(None),
            revision: Cell::new(0),
            runtimes: Default::default(),
            deliveries: Default::default(),
            mounts: Cell::new(0),
            drops: Cell::new(0),
            selected: Default::default(),
            controller: Cell::new(None),
        }
    }
}
#[component]
fn RootRow(data: Rc<RootCapture>, row: String) -> Element {
    let mut note = use_signal(|| 0usize);
    use_hook({
        let data = data.clone();
        let row = row.clone();
        move || {
            data.mounts.set(data.mounts.get() + 1);
            data.runtimes
                .borrow_mut()
                .insert(row, consume_context::<TreeDom>());
        }
    });
    use_drop(move || data.drops.set(data.drops.get() + 1));
    rsx! {button {r#type:"button","data-root-note":row,onclick:move|event:MouseEvent|{event.prevent_default();note+=1;},"Note {note}"}}
}
fn root_model() -> TreeModel<String> {
    TreeModel::new(
        "root",
        [
            TreeModelItem::branch("root", String::new(), vec!["a".into(), "b".into()]),
            TreeModelItem::new("a", "Asset A".into()),
            TreeModelItem::new("b", "Asset B".into()),
        ],
    )
    .unwrap()
}
fn root_mounted(data: &RootCapture, revision: usize, event: MountedEvent) {
    let runtime = data.runtimes.borrow()["a"].clone();
    // Mutably borrowing here also proves the internal registration's borrow
    // has ended before consumer code runs.
    let state = runtime.state.borrow_mut();
    assert!(Rc::ptr_eq(
        state.root.as_ref().expect("internal root registered first"),
        &event.data()
    ));
    drop(state);
    data.deliveries.borrow_mut().push((revision, event.data()));
}
fn root_fixture(data: Rc<RootCapture>) -> Element {
    let controller = use_tree_model(root_model);
    data.controller.set(Some(controller));
    let mounted = data.clone();
    let renderer = data.clone();
    let selected = data.clone();
    let revision = data.revision.get();
    let id = data.id.borrow().clone();
    let label = format!("Equipment {revision}");
    let class = format!("consumer-tree-{revision}");
    rsx! {if data.visible.get() {
        if data.model {
            ModelTree {controller,item_label:|value:String|value,id,label,class,
                onmounted:move|event|root_mounted(&mounted,revision,event),
                onselect:move|id|selected.selected.borrow_mut().push(id),
                render_item:move|item:TreeRenderContext<String>|rsx!{RootRow {data:renderer.clone(),row:item.item.id}}
            }
        } else {
            Tree {nodes:vec![TreeNode::new("a","Asset A"),TreeNode::new("b","Asset B")],id,label,class,
                onmounted:move|event|root_mounted(&mounted,revision,event),
                onselect:move|id|selected.selected.borrow_mut().push(id),
                render_item:move|item:TreeItemData|rsx!{RootRow {data:renderer.clone(),row:item.id}}
            }
        }
    }}
}
fn dispatch(dom: &VirtualDom, name: &str, id: ElementId, payload: impl std::any::Any) {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    dom.runtime().handle_event(
        name,
        Event::new(data, !matches!(name, "mounted" | "focus")),
        id,
    );
}
fn mounted_listeners(edits: &Mutations) -> Vec<ElementId> {
    let mut current = None;
    let mut result = Vec::new();
    for edit in &edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = Some(*id)
            }
            dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                result.push(current.unwrap())
            }
            dioxus::core::Mutation::SetAttribute { .. }
            | dioxus::core::Mutation::SetText { .. }
            | dioxus::core::Mutation::NewEventListener { .. }
            | dioxus::core::Mutation::RemoveEventListener { .. } => {}
            _ => current = None,
        }
    }
    result
}
// Index only explicitly registered nodes. Stack movement invalidates the
// current registration so static template attributes cannot be misattributed.
fn collect(edits: Mutations, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    let mut current = None;
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } => {
                attrs.insert(id.raw(), BTreeMap::new());
                current = Some(id);
            }
            dioxus::core::Mutation::PushId { id } => current = Some(id),
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } => {
                if let Some(id) = current {
                    attrs.entry(id.raw()).or_default().insert(name, value);
                }
            }
            dioxus::core::Mutation::CreateElement { .. }
            | dioxus::core::Mutation::CreateText { .. }
            | dioxus::core::Mutation::Child { .. }
            | dioxus::core::Mutation::Pop
            | dioxus::core::Mutation::Clone
            | dioxus::core::Mutation::AppendChildren { .. }
            | dioxus::core::Mutation::ReplaceWith { .. }
            | dioxus::core::Mutation::InsertBefore { .. }
            | dioxus::core::Mutation::InsertAfter { .. }
            | dioxus::core::Mutation::Remove => current = None,
            _ => {}
        }
    }
}
fn target(attrs: &BTreeMap<usize, BTreeMap<String, String>>, name: &str, value: &str) -> ElementId {
    ElementId::from_raw(
        *attrs
            .iter()
            .find(|(_, attrs)| attrs.get(name).is_some_and(|actual| actual == value))
            .unwrap_or_else(|| panic!("missing {name}={value}"))
            .0,
    )
}
fn root_tag(html: &str) -> &str {
    html.split('<')
        .find(|tag| tag.starts_with("ul "))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn exercise_root(model: bool) {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(TreeTestConverter));
    let data = Rc::new(RootCapture::new(model));
    let mut dom = VirtualDom::new_with_props(root_fixture, data.clone());
    let mut attrs = BTreeMap::new();
    collect(dom.rebuild_to_vec(), &mut attrs);
    settle_dom(&mut dom);
    let html = dioxus_ssr::render(&dom);
    let generated = root_tag(&html)
        .split_once("id=\"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .to_string();
    assert!(generated.starts_with("ix-tree-"));
    let root = target(&attrs, "id", &generated);
    let rows = ["a", "b"].map(|id| target(&attrs, "data-tree-node-id", id));
    let note = target(&attrs, "data-root-note", "a");
    let runtime = data.runtimes.borrow()["a"].clone();
    assert!(Rc::ptr_eq(
        &runtime.state,
        &data.runtimes.borrow()["b"].state
    ));
    let focused = Rc::new(RefCell::new(Vec::new()));
    let scrolled = Rc::new(RefCell::new(Vec::new()));
    let backing = TreeMount {
        name: "root".into(),
        focused: focused.clone(),
        scrolled: scrolled.clone(),
    };
    dispatch(&dom, "mounted", root, backing.clone());
    for (index, row) in rows.iter().enumerate() {
        dispatch(
            &dom,
            "mounted",
            *row,
            TreeMount {
                name: ["a", "b"][index].into(),
                focused: focused.clone(),
                scrolled: scrolled.clone(),
            },
        );
    }
    assert_eq!(data.deliveries.borrow().len(), 1);
    let original = data.deliveries.borrow()[0].1.clone();
    assert!(Rc::ptr_eq(
        &original.downcast::<TreeMount>().unwrap().scrolled,
        &scrolled
    ));
    dispatch(&dom, "click", note, ());
    settle_dom(&mut dom);
    dispatch(&dom, "click", rows[1], ());
    settle_dom(&mut dom);
    assert_eq!(data.selected.borrow().as_slice(), ["b"]);
    let original_rows = runtime.state.borrow().rows.clone();
    assert_eq!(original_rows.len(), 2);
    for (revision, id) in [(1, Some("tree }# unusual ID".to_string())), (2, None)] {
        data.revision.set(revision);
        *data.id.borrow_mut() = id.clone();
        dom.mark_dirty(ScopeId::APP);
        let edits = dom.render_immediate_to_vec();
        assert!(
            mounted_listeners(&edits).is_empty(),
            "root metadata must not remount rows or custom content"
        );
        collect(edits, &mut attrs);
        settle_dom(&mut dom);
        let expected = id.as_deref().unwrap_or(&generated);
        assert_eq!(target(&attrs, "id", expected), root);
        assert_eq!(target(&attrs, "data-tree-node-id", "b"), rows[1]);
        assert_eq!(target(&attrs, "data-root-note", "a"), note);
        let html = dioxus_ssr::render(&dom);
        assert!(root_tag(&html).contains(&format!("aria-label=\"Equipment {revision}\"")));
        assert!(root_tag(&html).contains(&format!("consumer-tree-{revision}")));
        assert!(
            html.contains("Note 1"),
            "custom content's private state survives metadata updates"
        );
        let b_tag = html
            .split('<')
            .find(|tag| tag.starts_with("li ") && tag.contains("data-tree-node-id=\"b\""))
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        let selected = b_tag
            .split_once("aria-selected=")
            .unwrap()
            .1
            .split_whitespace()
            .next()
            .unwrap()
            .trim_matches('"');
        assert_eq!(selected, "true", "selection stays with the current row");
        for (id, node) in &original_rows {
            assert!(Rc::ptr_eq(node, &runtime.state.borrow().rows[id]));
        }
    }
    assert!(Rc::ptr_eq(
        runtime.state.borrow().root.as_ref().unwrap(),
        &original
    ));
    dispatch(&dom, "mounted", root, backing);
    assert_eq!(
        data.deliveries.borrow()[1].0,
        2,
        "the current consumer callback receives later native mount delivery"
    );
    assert_eq!((data.mounts.get(), data.drops.get()), (2, 0));
    if model {
        assert!(data.controller.get().unwrap().context()["b"].is_selected);
    }
    data.visible.set(false);
    dom.mark_dirty(ScopeId::APP);
    settle_dom(&mut dom);
    assert_eq!(data.drops.get(), 2);
    assert!(!runtime.state.borrow().alive && runtime.state.borrow().root.is_none());
}
#[test]
fn tree_root_forwards_native_mount_after_registration_and_preserves_rows_on_metadata_updates() {
    exercise_root(false);
}
#[test]
fn model_tree_forwards_root_identity_and_native_mount_without_replacing_owned_rows() {
    exercise_root(true);
}

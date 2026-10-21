use super::*;
type DomCapture = Rc<RefCell<Option<TreeDom>>>;
fn mounted_dom_fixture() -> Element {
    let focused = use_signal(|| None::<String>);
    let viewport = use_signal(|| TreeViewport {
        top: 0.0,
        height: 320.0,
    });
    let runtime = use_hook(|| TreeDom::new(focused, viewport));
    let capture = consume_context::<DomCapture>();
    *capture.borrow_mut() = Some(runtime);
    VNode::empty()
}
#[derive(Clone)]
struct TreeMount {
    name: String,
    focused: Rc<RefCell<Vec<String>>>,
    scrolled: Rc<RefCell<Vec<f64>>>,
}
impl dioxus::html::RenderedElementBacking for TreeMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let target = self.clone();
        Box::pin(async move {
            target.focused.borrow_mut().push(target.name);
            Ok(())
        })
    }
    fn scroll(
        &self,
        point: dioxus::html::geometry::PixelsVector2D,
        _: ScrollBehavior,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        let target = self.clone();
        Box::pin(async move {
            target.scrolled.borrow_mut().push(point.y);
            Ok(())
        })
    }
}
fn settle_dom(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
#[test]
fn delayed_tree_row_mount_obeys_latest_intent_and_removed_target_cancellation() {
    let capture: DomCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(mounted_dom_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let runtime = capture.borrow().clone().unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let scrolls = Rc::new(RefCell::new(Vec::new()));
    let mount = |id: &str| {
        Rc::new(MountedData::new(TreeMount {
            name: id.into(),
            focused: calls.clone(),
            scrolled: scrolls.clone(),
        }))
    };
    runtime.state.borrow_mut().root = Some(mount("root"));
    runtime.reconcile(BTreeSet::from(["a".into(), "b".into()]));
    dom.in_scope(ScopeId::ROOT, || runtime.request(1999, "a".into(), 2000));
    settle_dom(&mut dom);
    assert!(
        calls.borrow().is_empty(),
        "a real target mount is required after scrolling"
    );
    dom.in_scope(ScopeId::ROOT, || runtime.request(0, "b".into(), 2000));
    settle_dom(&mut dom);
    dom.in_scope(ScopeId::ROOT, || runtime.mount("a".into(), mount("a")));
    settle_dom(&mut dom);
    assert!(
        calls.borrow().is_empty(),
        "late old target cannot inherit the newer intent"
    );
    dom.in_scope(ScopeId::ROOT, || runtime.mount("b".into(), mount("b")));
    settle_dom(&mut dom);
    assert_eq!(calls.borrow().as_slice(), ["b"]);
    assert_eq!(scrolls.borrow().as_slice(), [63680.0, 0.0]);
    let a = runtime.state.borrow_mut().rows.remove("a").unwrap();
    dom.in_scope(ScopeId::ROOT, || runtime.request(1999, "a".into(), 2000));
    settle_dom(&mut dom);
    runtime.reconcile(BTreeSet::from(["b".into()]));
    dom.in_scope(ScopeId::ROOT, || runtime.mount("a".into(), a));
    settle_dom(&mut dom);
    assert_eq!(
        calls.borrow().as_slice(),
        ["b"],
        "removed model target cancels pending focus"
    );
    assert!(runtime.state.borrow().pending.is_none());
    runtime.dispose();
}
#[test]
fn tree_blur_and_owner_teardown_cancel_waiting_mount_and_old_drop_keeps_replacement() {
    let capture: DomCapture = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new(mounted_dom_fixture).with_root_context(capture.clone());
    dom.rebuild_in_place();
    let runtime = capture.borrow().clone().unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let scrolls = Rc::new(RefCell::new(Vec::new()));
    let mount = || {
        Rc::new(MountedData::new(TreeMount {
            name: "row".into(),
            focused: calls.clone(),
            scrolled: scrolls.clone(),
        }))
    };
    runtime.state.borrow_mut().root = Some(mount());
    runtime.reconcile(BTreeSet::from(["row".into()]));
    dom.in_scope(ScopeId::ROOT, || runtime.request(0, "row".into(), 1));
    settle_dom(&mut dom);
    assert!(runtime.blur());
    let old = mount();
    let replacement = mount();
    dom.in_scope(ScopeId::ROOT, || {
        runtime.mount("row".into(), old.clone());
        runtime.mount("row".into(), replacement.clone());
    });
    runtime.unmount("row", &old);
    settle_dom(&mut dom);
    assert!(calls.borrow().is_empty());
    assert!(Rc::ptr_eq(
        runtime.state.borrow().rows.get("row").unwrap(),
        &replacement
    ));
    runtime.unmount("row", &replacement);
    dom.in_scope(ScopeId::ROOT, || runtime.request(0, "row".into(), 1));
    settle_dom(&mut dom);
    runtime.dispose();
    dom.in_scope(ScopeId::ROOT, || runtime.mount("row".into(), mount()));
    settle_dom(&mut dom);
    assert!(
        calls.borrow().is_empty(),
        "teardown cannot acquire an outside control's focus"
    );
}
fn flat_sample() -> TreeModel<String> {
    TreeModel::new(
        "root",
        [
            TreeModelItem::branch("root", String::new(), vec!["sample".into()]),
            TreeModelItem::branch("sample", "Sample".into(), vec!["one".into(), "two".into()]),
            TreeModelItem::new("one", "One".into()),
            TreeModelItem::new("two", "Two".into()),
        ],
    )
    .unwrap()
}
#[test]
fn flat_model_rejects_ambiguous_and_recursive_graphs_but_permits_staged_items() {
    let mut model = flat_sample();
    model.items.insert(
        "staged".into(),
        TreeModelItem::new("staged", "Unattached".into()),
    );
    assert!(model.validate().is_ok());
    let context = TreeContext::new();
    let nodes = model_tree_nodes(&model, &context, Clone::clone);
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, "sample");
    assert!(find_node(&nodes, "root", false).is_none());
    assert!(find_node(&nodes, "staged", false).is_none());
    model.items.get_mut("one").unwrap().has_children = true;
    model
        .items
        .get_mut("one")
        .unwrap()
        .children
        .push("sample".into());
    assert!(matches!(
        model.validate(),
        Err(TreeModelError::MultipleParents(_))
    ));
    model.items.get_mut("root").unwrap().children.clear();
    assert!(matches!(model.validate(), Err(TreeModelError::Cycle(_))));
    model.items.get_mut("one").unwrap().children = vec!["absent".into()];
    assert!(matches!(
        model.validate(),
        Err(TreeModelError::MissingChild { .. })
    ));
    assert!(matches!(
        TreeModel::new(
            "root",
            [TreeModelItem::new("root", 1), TreeModelItem::new("root", 2)]
        ),
        Err(TreeModelError::DuplicateId(_))
    ));
}
#[test]
fn failed_flat_transaction_preserves_state_and_queued_refreshes_atomically() {
    let mut store = TreeStore::new(flat_sample());
    store
        .transact(|model| model.append("sample", TreeModelItem::new("three", "Three".into())))
        .unwrap();
    let before = store.clone();
    let result = store.transact(|model| {
        model.items.get_mut("one").unwrap().data = "Should roll back".into();
        model
            .items
            .get_mut("sample")
            .unwrap()
            .children
            .push("missing".into());
        Ok(())
    });
    assert!(matches!(result, Err(TreeModelError::MissingChild { .. })));
    assert_eq!(store, before);
    assert_eq!(store.refresh.revision, 1);
    assert_eq!(store.pending_refreshes.len(), 1);
}
#[test]
fn flat_mutations_keep_callback_order_and_remove_context_for_deleted_subtrees() {
    let mut store = TreeStore::new(flat_sample());
    store.context.get_mut("sample").unwrap().is_expanded = true;
    store.context.get_mut("one").unwrap().is_selected = true;
    store
        .transact(|model| {
            model.items.get_mut("one").unwrap().data = "Renamed".into();
            Ok(())
        })
        .unwrap();
    store
        .transact(|model| model.remove("sample").map(|_| ()))
        .unwrap();
    assert_eq!(
        store
            .pending_refreshes
            .iter()
            .map(|event| event.revision)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        store.pending_refreshes[0].dirty,
        BTreeSet::from(["one".into()])
    );
    assert!(store.pending_refreshes[0].context["one"].is_selected);
    assert_eq!(
        store.pending_refreshes[1]
            .removed
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        ["one", "sample", "two"]
    );
    assert_eq!(store.context.len(), 1);
    assert!(!store.context.values().any(|context| context.is_selected));
    assert!(!store.model.items["root"].has_children);
    assert_eq!(
        store.model.remove("root"),
        Err(TreeModelError::CannotRemoveRoot)
    );
}
#[test]
fn flat_replacement_preserves_surviving_context_and_clamps_hidden_focus_target() {
    let mut store = TreeStore::new(flat_sample());
    store.context.get_mut("sample").unwrap().is_expanded = true;
    store.context.get_mut("one").unwrap().is_selected = true;
    store
        .transact(|model| {
            model.items.get_mut("sample").unwrap().children.reverse();
            model.items.get_mut("one").unwrap().data = "Updated".into();
            Ok(())
        })
        .unwrap();
    assert!(store.context["one"].is_selected);
    let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
    let state = model_tree_state(&store.context);
    let mut visible = vec![];
    visible_nodes(&nodes, &state, None, 1, false, &mut visible);
    assert_eq!(
        visible
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        ["sample", "two", "one"]
    );
    store.context.get_mut("sample").unwrap().is_expanded = false;
    let mut collapsed = vec![];
    visible_nodes(
        &nodes,
        &model_tree_state(&store.context),
        None,
        1,
        false,
        &mut collapsed,
    );
    assert_eq!(collapsed.len(), 1);
    assert!(store.context["one"].is_selected);
    assert_eq!(parent_id(&nodes, "one", None).as_deref(), Some("sample"));
}
#[test]
fn flat_lazy_branch_retains_context_through_loading_and_empty_completion() {
    let mut model = flat_sample();
    model
        .append(
            "sample",
            TreeModelItem::branch("remote", "Remote".into(), vec![]),
        )
        .unwrap();
    let mut store = TreeStore::new(model);
    store.context.get_mut("remote").unwrap().is_expanded = true;
    store
        .transact(|model| {
            model.items.get_mut("remote").unwrap().loading = true;
            Ok(())
        })
        .unwrap();
    let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
    let remote = find_node(&nodes, "remote", false).unwrap().0;
    assert!(remote.lazy && remote.loading && remote.has_children());
    store
        .transact(|model| {
            model.append("remote", TreeModelItem::new("loaded", "Loaded".into()))?;
            model.items.get_mut("remote").unwrap().loading = false;
            Ok(())
        })
        .unwrap();
    let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
    let remote = find_node(&nodes, "remote", false).unwrap().0;
    assert!(!remote.lazy && !remote.loading && remote.has_children());
    assert!(store.context["remote"].is_expanded);
    assert!(store.context.contains_key("loaded"));
    store
        .transact(|model| model.remove("loaded").map(|_| ()))
        .unwrap();
    let nodes = model_tree_nodes(&store.model, &store.context, Clone::clone);
    assert!(!find_node(&nodes, "remote", false).unwrap().0.has_children());
}
fn model_tree_fixture() -> Element {
    let controller = use_tree_model(flat_sample);
    use_hook(move || {
        let mut context = controller.context();
        context.get_mut("sample").unwrap().is_expanded = true;
        context.get_mut("one").unwrap().is_selected = true;
        context.get_mut("two").unwrap().is_disabled = true;
        controller.set_context(context);
    });
    rsx! { ModelTree { controller, item_label: |data: String| data, label: "Flat fixture", render_item: |row: TreeRenderContext<String>| rsx! { span { "data-render-index": row.index, "{row.item.data} ({row.visible_items.len()} visible)" } } } }
}
#[test]
fn flat_renderer_exposes_typed_payload_complete_list_context_and_hidden_root() {
    let mut dom = VirtualDom::new(model_tree_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom).replace('"', "");
    assert_eq!(html.matches("role=treeitem").count(), 3);
    assert!(html.contains("aria-label=Flat fixture"));
    assert!(html.contains("One (3 visible)"));
    assert!(html.contains("data-render-index=2"));
    assert!(html.contains("aria-selected=true"));
    assert!(html.contains("aria-disabled=true"));
    assert!(!html.contains("data-tree-node-id=root"));
}
#[derive(Default)]
struct FlatRuntimeCapture {
    controller: Option<TreeController<String>>,
    events: Vec<TreeRefreshEvent<String>>,
    removed: Vec<String>,
}
fn flat_runtime_fixture() -> Element {
    let controller = use_tree_model(flat_sample);
    let capture = consume_context::<std::rc::Rc<std::cell::RefCell<FlatRuntimeCapture>>>();
    capture.borrow_mut().controller = Some(controller);
    let removed_capture = capture.clone();
    rsx! { ModelTree { controller, item_label: |data: String| data,
        onrefresh: move |event| capture.borrow_mut().events.push(event),
        onremove: move |items: Vec<TreeModelItem<String>>| removed_capture.borrow_mut().removed.extend(items.into_iter().map(|item| item.id)),
    } }
}
#[test]
fn flat_controller_notifies_every_batched_mutation_once_and_refresh_effect_settles() {
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::{
            future::Future,
            pin::pin,
            task::{Context, Poll, Waker},
        };
        let capture = std::rc::Rc::new(std::cell::RefCell::new(FlatRuntimeCapture::default()));
        let mut dom = VirtualDom::new(flat_runtime_fixture).with_root_context(capture.clone());
        dom.rebuild_in_place();
        let controller = capture.borrow().controller.unwrap();
        dom.in_scope(ScopeId::ROOT, || {
            let mut context = controller.context();
            context.get_mut("sample").unwrap().is_expanded = true;
            controller.set_context(context);
            controller
                .append("sample", TreeModelItem::new("three", "Three".into()))
                .unwrap();
            controller
                .update_data("three", |data| *data = "Renamed three".into())
                .unwrap();
            controller.mark_items_as_dirty(["three".into(), "missing".into()]);
            controller.refresh(true);
            controller.remove("one").unwrap();
        });
        dom.render_immediate_to_vec();
        let mut context = Context::from_waker(Waker::noop());
        let mut settled = false;
        for _ in 0..8 {
            let work = { pin!(dom.wait_for_work()).poll(&mut context) };
            match work {
                Poll::Ready(()) => {
                    dom.render_immediate_to_vec();
                }
                Poll::Pending => {
                    settled = true;
                    break;
                }
            }
        }
        assert!(
            settled,
            "refresh effects must settle after draining their notifications"
        );
        let result = capture.borrow();
        assert_eq!(
            result
                .events
                .iter()
                .map(|event| event.revision)
                .collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 6]
        );
        assert_eq!(
            result.events[1]
                .items
                .iter()
                .find(|item| item.id == "three")
                .unwrap()
                .data,
            "Three"
        );
        assert_eq!(
            result.events[2]
                .items
                .iter()
                .find(|item| item.id == "three")
                .unwrap()
                .data,
            "Renamed three"
        );
        assert_eq!(result.events[3].dirty, BTreeSet::from(["three".into()]));
        assert!(result.events[4].force);
        assert_eq!(result.removed, ["one"]);
        assert!(dioxus_ssr::render(&dom).contains("Renamed three"));
        send.send(()).unwrap();
    });
    receive
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("ModelTree callback queue must terminate and deliver every update exactly once");
}
fn sample() -> Vec<TreeNode> {
    vec![TreeNode::branch(
        "sample",
        "Sample",
        vec![
            TreeNode::new("child-1", "Sample Child 1"),
            TreeNode::branch(
                "child-2",
                "Sample Child 2",
                vec![TreeNode::new("grandchild", "Grandchild")],
            ),
        ],
    )]
}
#[test]
fn visible_rows_preserve_hierarchy_and_collapsed_descendant_state() {
    let nodes = sample();
    let mut state = TreeState::default();
    let mut rows = vec![];
    visible_nodes(&nodes, &state, None, 1, false, &mut rows);
    assert_eq!(rows.len(), 1);
    state.set_expanded(&nodes, "sample", true);
    state.set_expanded(&nodes, "child-2", true);
    rows.clear();
    visible_nodes(&nodes, &state, None, 1, false, &mut rows);
    assert_eq!(
        rows.iter()
            .map(|row| (row.id.as_str(), row.level, row.position, row.set_size))
            .collect::<Vec<_>>(),
        vec![
            ("sample", 1, 1, 1),
            ("child-1", 2, 1, 2),
            ("child-2", 2, 2, 2),
            ("grandchild", 3, 1, 1)
        ]
    );
    state.set_expanded(&nodes, "sample", false);
    assert!(state.expanded.contains("child-2"));
    assert_eq!(
        parent_id(&nodes, "grandchild", None),
        Some("child-2".into())
    );
}
#[test]
fn check_cascade_and_ancestor_indeterminate_states_are_consistent() {
    let nodes = sample();
    let mut state = TreeState::default();
    assert!(state.set_checked(&nodes, "child-1", true));
    assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Mixed);
    assert!(!state.checked.contains("sample"));
    state.toggle_checked(&nodes, "sample");
    assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Checked);
    assert_eq!(state.checked.len(), 4);
    state.set_checked(&nodes, "grandchild", false);
    assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Mixed);
    assert!(!state.checked.contains("child-2"));
    state.set_checked(&nodes, "sample", false);
    assert!(state.checked.is_empty());
}
#[test]
fn disabled_subtrees_are_immutable_and_do_not_block_parent_checking() {
    let mut nodes = sample();
    nodes[0].children[1].disabled = true;
    let mut state = TreeState::default();
    state.set_checked(&nodes, "sample", true);
    assert_eq!(state.check_state(&nodes[0]), TreeCheckState::Checked);
    assert!(!state.checked.contains("child-2"));
    assert!(!state.checked.contains("grandchild"));
    assert!(!state.select(&nodes, "grandchild"));
    assert!(!state.set_expanded(&nodes, "child-2", true));
    assert!(!state.set_checked(&nodes, "grandchild", true));
    state.checked.insert("grandchild".into());
    state.set_checked(&nodes, "sample", false);
    assert!(state.checked.contains("grandchild"));
}
#[test]
fn expansion_rejects_leaf_and_missing_nodes_and_handles_lazy_branches() {
    let mut nodes = sample();
    nodes.push(TreeNode {
        lazy: true,
        ..TreeNode::new("lazy", "Lazy branch")
    });
    let mut state = TreeState::default();
    assert!(!state.set_expanded(&nodes, "missing", true));
    assert!(!state.set_expanded(&nodes, "child-1", true));
    assert!(state.set_expanded(&nodes, "lazy", true));
    state.expand_all(&nodes);
    assert_eq!(state.expanded.len(), 3);
    state.collapse_all();
    assert!(state.expanded.is_empty());
}

#[test]
fn window_bounds_dom_work_for_large_scrolls_and_clamped_collapses() {
    let range = tree_window(
        100_000,
        TreeViewport {
            top: 1_600_000.0,
            height: 320.0,
        },
        4,
    );
    assert_eq!(range, 49_996..50_014);
    assert_eq!(range.len(), 18);
    assert_eq!(
        tree_window(
            3,
            TreeViewport {
                top: 1_600_000.0,
                height: 320.0
            },
            4
        ),
        0..3
    );
    assert_eq!(tree_window(0, TreeViewport::default(), 4), 0..0);
    assert_eq!(
        tree_window(
            100,
            TreeViewport {
                top: f64::NAN,
                height: 320.0
            },
            4
        ),
        0..14
    );
}

#[test]
fn keyboard_reveal_reaches_offscreen_end_without_losing_identity_or_hierarchy() {
    let mut nodes = sample();
    nodes[0]
        .children
        .extend((0..2000).map(|i| TreeNode::new(format!("device-{i}"), format!("Device {i}"))));
    let mut state = TreeState::default();
    state.expand_all(&nodes);
    state.select(&nodes, "device-1999");
    let mut rows = vec![];
    visible_nodes(&nodes, &state, None, 1, false, &mut rows);
    let index = rows.iter().position(|row| row.id == "device-1999").unwrap();
    let mut viewport = TreeViewport {
        top: 0.0,
        height: 320.0,
    };
    assert!(!tree_window(rows.len(), viewport, 4).contains(&index));
    viewport.top = reveal_offset(index, viewport, rows.len());
    assert!(tree_window(rows.len(), viewport, 4).contains(&index));
    assert_eq!(viewport.top + viewport.height, rows.len() as f64 * 32.0);
    assert_eq!(
        (
            rows[index].level,
            rows[index].position,
            rows[index].set_size
        ),
        (2, 2002, 2002)
    );
    state.set_expanded(&nodes, "sample", false);
    assert_eq!(state.selected.as_deref(), Some("device-1999"));
    assert_eq!(
        parent_id(&nodes, "device-1999", None).as_deref(),
        Some("sample")
    );
    assert_eq!(reveal_offset(0, viewport, 1), 0.0);
}

fn unwindowed_tree_fixture() -> Element {
    rsx! { Tree { nodes: (0..2000).map(|i| TreeNode::new(format!("device-{i}"), format!("Device {i}"))).collect::<Vec<_>>(), virtualized: false, max_height: 320 } }
}
#[test]
fn consumers_can_explicitly_mount_the_full_visible_tree() {
    let mut dom = VirtualDom::new(unwindowed_tree_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom).replace('"', "");
    assert_eq!(html.matches("role=treeitem").count(), 2000);
    assert!(html.contains("Device 1999"));
    assert!(!html.contains("class=ix-tree-spacer"));
}

fn virtual_tree_fixture() -> Element {
    rsx! { Tree { nodes: (0..2000).map(|i| TreeNode::new(format!("device-{i}"), format!("Device {i}"))).collect::<Vec<_>>(), max_height: 320, label: "Large fixture" } }
}
#[test]
fn large_tree_renders_bounded_rows_with_full_sibling_metadata() {
    let mut dom = VirtualDom::new(virtual_tree_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom).replace('"', "");
    assert_eq!(html.matches("role=treeitem").count(), 14);
    assert!(html.contains("aria-setsize=2000"));
    assert!(html.contains("height:63552px"));
    assert!(!html.contains("Device 1999"));
}

fn accessible_tree_fixture() -> Element {
    let nodes = sample();
    let mut state = TreeState::default();
    state.set_expanded(&nodes, "sample", true);
    state.set_checked(&nodes, "child-1", true);
    state.select(&nodes, "child-1");
    rsx! { Tree { nodes, state, checkboxes: true, label: "Fixture tree" } }
}

#[test]
fn rendered_tree_exposes_hierarchy_selection_mixed_checks_and_hidden_descendants() {
    let mut dom = VirtualDom::new(accessible_tree_fixture);
    dom.rebuild_in_place();
    // Dioxus emits valid unquoted HTML for numeric/bool attribute values.
    let html = dioxus_ssr::render(&dom).replace('"', "");
    assert!(html.contains("role=tree"));
    assert!(html.contains("aria-label=Fixture tree"));
    assert_eq!(html.matches("role=treeitem").count(), 3);
    assert!(html.contains("aria-level=2"), "{html}");
    assert!(html.contains("aria-checked=mixed"));
    assert!(html.contains("aria-selected=true"));
    assert!(html.contains("Sample Child 1"));
    assert!(!html.contains("Grandchild"));
}

struct TreeTestMouse;
impl dioxus::html::InteractionLocation for TreeTestMouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for TreeTestMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for TreeTestMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for TreeTestMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for TreeTestMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct TreeTestKeyboard(Key);
impl dioxus::html::ModifiersInteraction for TreeTestKeyboard {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for TreeTestKeyboard {
    fn key(&self) -> Key {
        self.0.clone()
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Unidentified
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct TreeTestFocus;
impl dioxus::html::HasFocusData for TreeTestFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct TreeTestConverter;
macro_rules! tree_test_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for TreeTestConverter {
                $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                    let _ = event;
                    tree_test_converter!(@data $group, event)
                })*
            }
        };
        (@data Mouse, $event:ident) => { dioxus::html::MouseData::new(TreeTestMouse) };
        (@data Keyboard, $event:ident) => {
            dioxus::html::KeyboardData::new(TreeTestKeyboard(
                $event.downcast::<Key>().unwrap().clone(),
            ))
        };
        (@data Focus, $event:ident) => { dioxus::html::FocusData::new(TreeTestFocus) };
        (@data Mounted, $event:ident) => {
            if let Some(mount) = $event.downcast::<TreeMount>() {
                MountedData::new(mount.clone())
            } else {
                MountedData::new(())
            }
        };
        (@data $other:ident, $event:ident) => { panic!("Unexpected tree test event") };
    }
dioxus::html::with_html_event_groups!(tree_test_converter);

#[derive(Clone, Default)]
struct EmbeddedTreeControlState {
    actions: Rc<RefCell<Vec<String>>>,
    selected: Rc<RefCell<Vec<String>>>,
    toggled: Rc<RefCell<Vec<(String, bool)>>>,
}
impl PartialEq for EmbeddedTreeControlState {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.actions, &other.actions)
    }
}
fn embedded_tree_controls_fixture(state: EmbeddedTreeControlState) -> Element {
    let nodes = sample();
    let mut initial = TreeState::default();
    initial.set_expanded(&nodes, "sample", true);
    let selected = state.selected.clone();
    let toggled = state.toggled.clone();
    let actions = state.actions.clone();
    rsx! {
        Tree {
            nodes,
            default_state: initial,
            toggle_on_item_click: true,
            onselect: move |id| selected.borrow_mut().push(id),
            onexpand: move |toggle: TreeToggle| toggled.borrow_mut().push((toggle.id, toggle.expanded)),
            render_item: move |item: TreeItemData| {
                let click_id = item.id.clone();
                let key_id = item.id.clone();
                let focus_id = item.id.clone();
                let click_actions = actions.clone();
                let key_actions = actions.clone();
                let focus_actions = actions.clone();
                rsx! {
                    // Address a real listener-bearing element, as the
                    // interpreter does when forwarding a label click.
                    span { "data-row-label": item.id.clone(), onclick: move |_| {}, "{item.label}" }
                    button {
                        r#type: "button",
                        "data-row-action": item.id,
                        onclick: move |event: MouseEvent| {
                            event.prevent_default();
                            click_actions.borrow_mut().push(format!("click:{click_id}"));
                        },
                        onkeydown: move |_| {
                            key_actions.borrow_mut().push(format!("key:{key_id}"));
                        },
                        onfocusin: move |_| {
                            focus_actions.borrow_mut().push(format!("focusin:{focus_id}"));
                        },
                        "Action"
                    }
                }
            },
        }
    }
}

#[test]
fn embedded_row_controls_cancel_click_and_contain_keys_and_focusin_while_preserving_row_interactions()
 {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(TreeTestConverter));
    let state = EmbeddedTreeControlState::default();
    let mut dom = VirtualDom::new_with_props(embedded_tree_controls_fixture, state.clone());
    let mut attrs: BTreeMap<usize, BTreeMap<String, String>> = BTreeMap::new();
    let collect = |mutations: dioxus::core::Mutations,
                   attrs: &mut BTreeMap<usize, BTreeMap<String, String>>| {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(text),
                    ..
                } => {
                    attrs.entry(current.raw()).or_default().insert(name, text);
                }
                dioxus::core::Mutation::Remove => {
                    attrs.remove(&current.raw());
                }
                _ => {}
            }
        }
    };
    collect(dom.rebuild_to_vec(), &mut attrs);
    let settle = |dom: &mut VirtualDom, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>| {
        for _ in 0..4 {
            collect(dom.render_immediate_to_vec(), attrs);
            dom.process_events();
        }
    };
    settle(&mut dom, &mut attrs);
    let find_attr = |attrs: &BTreeMap<usize, BTreeMap<String, String>>,
                     attr: &str,
                     expected: &str|
     -> dioxus::core::ElementId {
        let raw = attrs
            .iter()
            .find_map(|(raw, map)| {
                (map.get(attr).map(String::as_str) == Some(expected)).then_some(*raw)
            })
            .unwrap_or_else(|| panic!("missing {attr}={expected}"));
        dioxus::core::ElementId::from_raw(raw)
    };
    let root_id = attrs
        .iter()
        .find_map(|(raw, map)| {
            map.get("id")
                .filter(|id| id.starts_with("ix-tree-"))
                .map(|_| dioxus::core::ElementId::from_raw(*raw))
        })
        .expect("tree root element");
    let focused_rows = Rc::new(RefCell::new(Vec::<String>::new()));
    let scrolled = Rc::new(RefCell::new(Vec::<f64>::new()));
    let dispatch = |dom: &VirtualDom,
                    name: &str,
                    id: dioxus::core::ElementId,
                    payload: Box<dyn std::any::Any>|
     -> (bool, bool) {
        let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(payload));
        let event = Event::new(data, !matches!(name, "mounted" | "focus"));
        dom.runtime().handle_event(name, event.clone(), id);
        (!event.default_action_enabled(), event.propagates())
    };
    let mount_node = |dom: &VirtualDom, id: dioxus::core::ElementId, name: &str| {
        dispatch(
            dom,
            "mounted",
            id,
            Box::new(TreeMount {
                name: name.into(),
                focused: focused_rows.clone(),
                scrolled: scrolled.clone(),
            }),
        );
    };
    mount_node(&dom, root_id, "root");
    for row_id in ["sample", "child-1", "child-2"] {
        mount_node(&dom, find_attr(&attrs, "data-tree-node-id", row_id), row_id);
    }
    settle(&mut dom, &mut attrs);

    let sample_row = find_attr(&attrs, "data-tree-node-id", "sample");
    dispatch(&dom, "focusin", sample_row, Box::new(()));
    settle(&mut dom, &mut attrs);
    assert!(
        focused_rows.borrow().is_empty(),
        "bubbling focusin alone must not authorize row-focus repair"
    );

    let sample_action = find_attr(&attrs, "data-row-action", "sample");
    let (_, focusin_propagates) = dispatch(&dom, "focusin", sample_action, Box::new(()));
    settle(&mut dom, &mut attrs);
    assert!(
        !focusin_propagates,
        "nested control focusin stops at ix-tree-label before reaching tree root"
    );
    assert_eq!(state.actions.borrow().as_slice(), ["focusin:sample"]);
    assert!(
        focused_rows.borrow().is_empty(),
        "nested control focusin must not trigger mounted row focus repair"
    );

    for key in [
        Key::Character(" ".into()),
        Key::Enter,
        Key::ArrowDown,
        Key::ArrowLeft,
    ] {
        let (key_prevented, key_propagates) =
            dispatch(&dom, "keydown", sample_action, Box::new(key));
        assert!(
            !key_prevented,
            "embedded controls keep their native key defaults"
        );
        assert!(
            !key_propagates,
            "nested control keydown stops at ix-tree-label before tree navigation"
        );
    }
    settle(&mut dom, &mut attrs);
    assert_eq!(
        state.actions.borrow().as_slice(),
        [
            "focusin:sample",
            "key:sample",
            "key:sample",
            "key:sample",
            "key:sample"
        ]
    );
    assert!(state.selected.borrow().is_empty());
    assert!(state.toggled.borrow().is_empty());
    assert!(focused_rows.borrow().is_empty());

    let (click_prevented, click_propagates) = dispatch(&dom, "click", sample_action, Box::new(()));
    settle(&mut dom, &mut attrs);
    assert!(click_prevented && click_propagates);
    assert_eq!(
        state.actions.borrow().last().map(String::as_str),
        Some("click:sample")
    );
    assert!(
        state.selected.borrow().is_empty()
            && state.toggled.borrow().is_empty()
            && focused_rows.borrow().is_empty(),
        "prevent_default on a descendant control cancels row selection, toggle and focus"
    );

    let child2_label = find_attr(&attrs, "data-row-label", "child-2");
    let (label_prevented, label_propagates) = dispatch(&dom, "click", child2_label, Box::new(()));
    settle(&mut dom, &mut attrs);
    assert!(!label_prevented && label_propagates);
    assert_eq!(state.selected.borrow().as_slice(), ["child-2"]);
    assert_eq!(
        state.toggled.borrow().as_slice(),
        [("child-2".to_string(), true)]
    );
    assert_eq!(focused_rows.borrow().as_slice(), ["child-2"]);
    assert!(dioxus_ssr::render(&dom).contains("Grandchild"));

    let sample_row = find_attr(&attrs, "data-tree-node-id", "sample");
    dispatch(&dom, "focus", sample_row, Box::new(()));
    let (_, row_focusin_propagates) = dispatch(&dom, "focusin", sample_row, Box::new(()));
    assert!(row_focusin_propagates);
    dispatch(
        &dom,
        "keydown",
        sample_row,
        Box::new(Key::Character(" ".into())),
    );
    settle(&mut dom, &mut attrs);
    assert_eq!(state.selected.borrow().as_slice(), ["child-2", "sample"]);
    dispatch(&dom, "keydown", sample_row, Box::new(Key::ArrowDown));
    settle(&mut dom, &mut attrs);
    assert_eq!(focused_rows.borrow().as_slice(), ["child-2", "child-1"]);
}

mod root_tests {
    use super::*;
    include!("root_tests.rs");
}

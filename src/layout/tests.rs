use super::*;

fn workflow_model_fixture() -> WorkflowContext {
    let context = WorkflowContext::default();
    context.0.borrow_mut().settings.clickable = true;
    for id in ["a", "b", "c"] {
        context.register(id.to_string(), None, Arc::new(|| {}));
    }
    context
}

#[test]
fn workflow_requests_are_cancelable_before_mutation_and_repeat_selected_activation() {
    let context = workflow_model_fixture();
    let mut events = Vec::new();
    assert!(!request_workflow_selection(&context, "c", false, |event| {
        assert_eq!(context.0.borrow().selected.as_deref(), Some("a"));
        assert_eq!((event.index, event.previous), (2, Some(0)));
        event.prevent_default();
    }));
    assert_eq!(context.0.borrow().selected.as_deref(), Some("a"));
    for _ in 0..2 {
        assert!(request_workflow_selection(&context, "c", false, |event| {
            events.push((event.index, event.previous));
        }));
    }
    assert_eq!(events, [(2, Some(0)), (2, Some(2))]);
    assert_eq!(
        context.0.borrow().settings.selected_index,
        0,
        "local selection never mutates the parent prop"
    );
    assert!(!request_workflow_selection(
        &context,
        "b",
        true,
        |_| panic!("disabled event")
    ));
    context.0.borrow_mut().settings.clickable = false;
    assert!(!request_workflow_selection(
        &context,
        "b",
        false,
        |_| panic!("nonclickable event")
    ));
}

#[test]
fn workflow_topology_reconciles_ordinals_positions_signed_indices_and_stale_events() {
    let context = workflow_model_fixture();
    assert_eq!(context.0.borrow().position("a"), "first");
    assert_eq!(context.0.borrow().position("b"), "undefined");
    assert_eq!(context.0.borrow().position("c"), "last");
    assert!(request_workflow_selection(&context, "c", false, |_| {}));
    context.update_order("b", Some(0));
    context.update_order("c", Some(1));
    context.update_order("a", Some(2));
    assert_eq!(
        context.0.borrow().selected.as_deref(),
        Some("b"),
        "topology reapplies parent index"
    );
    assert!(request_workflow_selection(
        &context,
        "a",
        false,
        |event| assert_eq!(event.index, 2)
    ));
    context.unregister("b");
    context.unregister("c");
    assert_eq!(context.0.borrow().position("a"), "single");
    assert_eq!(context.0.borrow().selected.as_deref(), Some("a"));
    for index in [-1, 3] {
        let mut model = context.0.borrow_mut();
        model.settings.selected_index = index;
        model.reset_selection();
        assert!(model.selected.is_none());
    }
    assert!(!request_workflow_selection(
        &context,
        "missing",
        false,
        |_| panic!("unmounted event")
    ));
    context.0.borrow_mut().alive = false;
    context.update_order("a", Some(0));
    assert_eq!(context.0.borrow().order(), ["a"]);
    assert!(!request_workflow_selection(
        &context,
        "a",
        false,
        |_| panic!("stale event")
    ));
}

#[test]
fn workflow_explicit_order_ties_and_static_fallback_are_stable() {
    let context = workflow_model_fixture();
    context.update_order("b", Some(4));
    assert_eq!(context.0.borrow().order(), ["b", "a", "c"]);
    context.update_order("a", Some(4));
    assert_eq!(
        context.0.borrow().order(),
        ["a", "b", "c"],
        "equal explicit positions retain registration order"
    );
    context.register("d".into(), None, Arc::new(|| {}));
    assert_eq!(context.0.borrow().order(), ["a", "b", "c", "d"]);
    context.unregister("a");
    context.register("e".into(), Some(4), Arc::new(|| {}));
    assert_eq!(
        context.0.borrow().order(),
        ["b", "e", "c", "d"],
        "omitted positions retain static registration order after explicit positions"
    );
}

#[derive(Clone, Default)]
struct WorkflowCapture(Rc<RefCell<Option<WorkflowContext>>>);
impl PartialEq for WorkflowCapture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
#[component]
fn CaptureWorkflow(capture: WorkflowCapture, children: Element) -> Element {
    *capture.0.borrow_mut() = Some(consume_context::<WorkflowContext>());
    children
}
#[component]
fn WrappedWorkflowStep(label: String, #[props(default)] order: Option<usize>) -> Element {
    rsx! { WorkflowStep { id: label.clone(), label, order, selected: true, clickable: false, vertical: false } }
}
#[derive(Clone)]
struct WorkflowTestConfig {
    capture: WorkflowCapture,
    index: isize,
    clickable: bool,
    vertical: bool,
    steps: Vec<String>,
}
fn workflow_reactive_fixture(config: Rc<RefCell<WorkflowTestConfig>>) -> Element {
    let config = config.borrow().clone();
    rsx! { WorkflowSteps { id: "test-workflow", selected_index: config.index,
        clickable: config.clickable, vertical: config.vertical,
        CaptureWorkflow { capture: config.capture,
            for (index,label) in config.steps.into_iter().enumerate() { WrappedWorkflowStep { key: "{label}", label,order:Some(index) } }
        }
    } }
}

#[test]
fn workflow_wrapped_descendants_inherit_parent_settings_and_parent_renders_reset_local_selection() {
    let capture = WorkflowCapture::default();
    let config = Rc::new(RefCell::new(WorkflowTestConfig {
        capture: capture.clone(),
        index: 1,
        clickable: true,
        vertical: true,
        steps: vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
    }));
    let mut dom = VirtualDom::new_with_props(workflow_reactive_fixture, config.clone());
    dom.rebuild_in_place();
    // Settle registered children and their explicit order props before
    // inspecting the shared order-dependent presentation.
    dom.render_immediate_to_vec();
    let context = capture.0.borrow().clone().unwrap();
    let original = context.0.borrow().order();
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("role=\"button\"").count(), 3);
    assert_eq!(
        html.matches("tabindex=0").count(),
        3,
        "each enabled step is tabbable"
    );
    assert_eq!(html.matches("data-selected=true").count(), 1);
    assert_eq!(html.matches("data-vertical=true").count(), 4);
    assert_eq!(context.0.borrow().selected.as_ref(), original.get(1));
    dom.in_runtime(|| {
        assert!(request_workflow_selection(
            &context,
            &original[2],
            false,
            |_| {}
        ))
    });
    dom.render_immediate_to_vec();
    assert_eq!(
        context.0.borrow().selected.as_ref(),
        original.get(2),
        "child rerenders preserve local activation"
    );
    config.borrow_mut().vertical = false;
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    assert_eq!(
        context.0.borrow().selected.as_ref(),
        original.get(1),
        "parent render reapplies selected_index"
    );
    assert!(!dioxus_ssr::render(&dom).contains("data-vertical=true"));
    config.borrow_mut().steps = vec!["Gamma".into(), "Alpha".into(), "Beta".into()];
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    dom.render_immediate_to_vec();
    assert_eq!(context.0.borrow().selected.as_ref(), original.first());
    assert_eq!(context.0.borrow().position(&original[2]), "first");
    config.borrow_mut().steps = vec!["Alpha".into()];
    config.borrow_mut().index = 0;
    config.borrow_mut().clickable = false;
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate_to_vec();
    dom.render_immediate_to_vec();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("data-position=\"single\""));
    assert_eq!(html.matches("data-selected=true").count(), 1);
    assert!(!html.contains("role=\"button\""));
    assert_eq!(html.matches("tabindex=-1").count(), 1);
}

#[test]
fn workflow_keyed_reverse_insert_remove_keeps_members_and_reports_current_indices() {
    let capture = WorkflowCapture::default();
    let config = Rc::new(RefCell::new(WorkflowTestConfig {
        capture: capture.clone(),
        index: 1,
        clickable: true,
        vertical: false,
        steps: vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
    }));
    let mut dom = VirtualDom::new_with_props(workflow_reactive_fixture, config.clone());
    dom.rebuild_in_place();
    dom.render_immediate_to_vec();
    let context = capture.0.borrow().clone().unwrap();
    let original = context.0.borrow().order();
    config.borrow_mut().steps.reverse();
    dom.mark_dirty(ScopeId::APP);
    for _ in 0..3 {
        dom.render_immediate_to_vec();
    }
    assert_eq!(
        context.0.borrow().order(),
        [
            original[2].clone(),
            original[1].clone(),
            original[0].clone()
        ]
    );
    assert_eq!(context.0.borrow().position(&original[2]), "first");
    assert_eq!(context.0.borrow().position(&original[0]), "last");
    dom.in_runtime(|| {
        assert!(request_workflow_selection(
            &context,
            &original[2],
            false,
            |event| assert_eq!(event.index, 0)
        ))
    });
    config.borrow_mut().steps.insert(1, "Inserted".into());
    dom.mark_dirty(ScopeId::APP);
    for _ in 0..3 {
        dom.render_immediate_to_vec();
    }
    let inserted = context.0.borrow().order()[1].clone();
    assert_eq!(
        context.0.borrow().selected.as_ref(),
        Some(&inserted),
        "topology reapplies selected_index to the inserted member"
    );
    dom.in_runtime(|| {
        assert!(!request_workflow_selection(
            &context,
            &original[1],
            false,
            |event| {
                assert_eq!((event.index, event.previous), (2, Some(1)));
                event.prevent_default();
            }
        ))
    });
    assert_eq!(context.0.borrow().selected.as_ref(), Some(&inserted));
    config.borrow_mut().steps.remove(0);
    dom.mark_dirty(ScopeId::APP);
    for _ in 0..3 {
        dom.render_immediate_to_vec();
    }
    assert_eq!(
        context.0.borrow().order(),
        [inserted, original[1].clone(), original[0].clone()]
    );
    assert!(!request_workflow_selection(
        &context,
        &original[2],
        false,
        |_| panic!("removed member cannot emit")
    ));
    drop(dom);
    assert!(!context.0.borrow().alive);
    assert!(context.0.borrow().members.is_empty());
    assert!(!request_workflow_selection(
        &context,
        &original[1],
        false,
        |_| panic!("dropped owner cannot emit")
    ));
}

fn nested_workflow_fixture(captures: (WorkflowCapture, WorkflowCapture)) -> Element {
    rsx! {
        WorkflowSteps { vertical: true, clickable: true,
            CaptureWorkflow { capture: captures.0,
                WorkflowStep { label: "Outer first", children: rsx! {
                    WorkflowSteps { vertical: false,
                        CaptureWorkflow { capture: captures.1,
                            WrappedWorkflowStep { label: "Inner only" }
                        }
                    }
                } }
                WrappedWorkflowStep { label: "Outer last" }
            }
        }
    }
}

#[test]
fn nested_workflows_have_separate_registration_and_orientation_boundaries() {
    let outer = WorkflowCapture::default();
    let inner = WorkflowCapture::default();
    let mut dom =
        VirtualDom::new_with_props(nested_workflow_fixture, (outer.clone(), inner.clone()));
    dom.rebuild_in_place();
    dom.render_immediate_to_vec();
    let outer = outer.0.borrow().clone().unwrap();
    let inner = inner.0.borrow().clone().unwrap();
    assert_eq!(outer.0.borrow().order().len(), 2);
    assert_eq!(inner.0.borrow().order().len(), 1);
    assert!(outer.0.borrow().settings.vertical);
    assert!(!inner.0.borrow().settings.vertical);
    assert!(
        !inner.0.borrow().settings.clickable,
        "nearest parent overrides child props"
    );
    assert_eq!(
        inner.0.borrow().position(&inner.0.borrow().order()[0]),
        "single"
    );
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("data-position=\"single\"").count(), 1);
    assert_eq!(html.matches("role=\"button\"").count(), 2);
    assert!(
        !STYLES.contains("] .ix-workflow-line"),
        "connector selectors must stop at their own step"
    );
    assert!(
        !STYLES.contains("] .ix-workflow-label"),
        "orientation selectors must stop at their own step"
    );
}

struct WorkflowMouse;
impl dioxus::html::InteractionLocation for WorkflowMouse {
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
impl dioxus::html::InteractionElementOffset for WorkflowMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for WorkflowMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for WorkflowMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for WorkflowMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct WorkflowEventConverter;
macro_rules! workflow_event_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for WorkflowEventConverter {
                $(fn $convert(&self, _: &dioxus::html::PlatformEventData) -> dioxus::html::$data { workflow_event_converter!(@data $group) })*
            }
        };
        (@data Mouse) => { dioxus::html::MouseData::new(WorkflowMouse) };
        (@data $other:ident) => { panic!("Unexpected non-mouse workflow test event") };
    }
dioxus::html::with_html_event_groups!(workflow_event_converter);

fn nested_workflow_event_fixture(events: Rc<RefCell<Vec<&'static str>>>) -> Element {
    let outer_events = events.clone();
    rsx! {
        WorkflowSteps { clickable: true, onstepselected: move |_| outer_events.borrow_mut().push("outer"),
            WorkflowStep { label: "Outer", children: rsx! {
                WorkflowSteps { clickable: true, onstepselected: move |_| events.borrow_mut().push("inner"),
                    WorkflowStep { label: "Inner" }
                }
            } }
        }
    }
}

#[test]
fn nested_workflow_click_dispatch_does_not_activate_the_enclosing_step() {
    let _event_converter_guard = crate::test_event_converter_guard();
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut dom = VirtualDom::new_with_props(nested_workflow_event_fixture, events.clone());
    dioxus::html::set_event_converter(Box::new(WorkflowEventConverter));
    let mutations = dom.rebuild_to_vec();
    let mut current_id = dioxus::core::ElementId::ROOT;
    let mut inner_step = None;
    for mutation in mutations.edits {
        match mutation {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current_id = id
            }
            dioxus::core::Mutation::NewEventListener { name } if name == "click" => {
                inner_step = Some(current_id)
            }
            _ => {}
        }
    }
    dom.render_immediate_to_vec();
    dom.runtime().handle_event(
        "click",
        Event::new(
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(()))),
            true,
        ),
        inner_step.expect("inner step click target"),
    );
    dom.render_immediate_to_vec();
    assert_eq!(
        *events.borrow(),
        ["inner"],
        "the nested group stops native bubbling before the outer step"
    );
}

fn workflow_icon_fixture() -> Element {
    rsx! {
        WorkflowSteps { clickable: true,
            WorkflowStep { label: "Open" }
            WorkflowStep { label: "Done", status: WorkflowStatus::Done }
            WorkflowStep { label: "Success", status: WorkflowStatus::Success }
            WorkflowStep { label: "Warning", status: WorkflowStatus::Warning }
            WorkflowStep { label: "Error", status: WorkflowStatus::Error, disabled: true }
            WorkflowStep { label: "Duplicate fallback", custom_icon: rsx! { span { "Custom icon" } },
                span { "Slot content" }
            }
        }
    }
}
#[test]
fn workflow_icons_content_slots_and_accessibility_follow_source_contracts() {
    let mut dom = VirtualDom::new(workflow_icon_fixture);
    dom.rebuild_in_place();
    dom.render_immediate_to_vec();
    let html = dioxus_ssr::render(&dom);
    for label in ["Circle dot", "Done", "Success", "Warning", "Error"] {
        assert!(html.contains(&format!("aria-label=\"{label}\"")), "{html}");
    }
    assert_eq!(html.matches("ix-workflow-icon-underlay").count(), 5);
    assert_eq!(html.matches("ix-workflow-status-icon").count(), 5);
    assert_eq!(html.matches("aria-disabled=\"true\"").count(), 1);
    assert_eq!(html.matches("tabindex=0").count(), 5);
    assert_eq!(html.matches("aria-current=\"step\"").count(), 1);
    assert!(html.contains("Custom icon"));
    assert!(html.contains("Slot content"));
    assert!(!html.contains("Duplicate fallback"));
    assert_eq!(WorkflowStatus::Open.icon(false), "circle");
    assert_eq!(WorkflowStatus::Open.icon(true), "circle-dot");
    assert_ne!(
        WorkflowStatus::Done.icon(false),
        WorkflowStatus::Success.icon(false)
    );
}
fn disclosure_slots_fixture() -> Element {
    rsx! {
        Blind { label: "Example", header_actions: rsx! { button { "Header action" } }, "Content" }
        Group { header: "Header text", header_actions: rsx! { button { "Group action" } }, GroupItem { button { "Test" } } }
        EventList { item_height: EventItemHeight::Pixels(60), EventListItem { "Test 1" } }
    }
}
#[test]
fn disclosure_and_custom_group_slots_do_not_nest_controls_or_lose_row_height() {
    let mut dom = VirtualDom::new(disclosure_slots_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("--ix-event-height:60px"));
    assert!(html.contains("Header action"));
    assert!(html.contains("Group action"));
    let mut depth = 0;
    for tag in html.split('<').skip(1) {
        if tag.starts_with("button ") || tag.starts_with("button>") {
            depth += 1;
            assert_eq!(depth, 1, "nested interactive control: {html}");
        } else if tag.starts_with("/button>") {
            depth -= 1;
        }
    }
    assert_eq!(depth, 0);
}
fn card_with_actions_fixture() -> Element {
    rsx! {
        Card {
            aria_label: "Select device",
            CardTitle { actions: rsx! { button { "Inspect device" } }, "Device" }
            CardAccordion { "Device details" }
        }
    }
}
#[test]
fn selectable_card_keeps_primary_and_nested_actions_as_separate_buttons() {
    let mut dom = VirtualDom::new(card_with_actions_fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("<article"));
    assert!(html.contains("aria-label=\"Select device\""));
    let mut depth = 0;
    let mut buttons = 0;
    for tag in html.split('<').skip(1) {
        if tag.starts_with("button ") || tag.starts_with("button>") {
            depth += 1;
            buttons += 1;
            assert_eq!(depth, 1, "card actions must never nest buttons: {html}");
        } else if tag.starts_with("/button>") {
            depth -= 1;
        }
    }
    assert_eq!(depth, 0);
    assert_eq!(
        buttons, 3,
        "primary selection, title action and accordion action"
    );
}
#[test]
fn mounted_tab_targets_follow_stable_keys_after_close_and_reorder() {
    let mut items = vec![
        TabItem::new("a", "A"),
        TabItem::new("b", "B"),
        TabItem::new("c", "C"),
    ];
    // Keyed DOM nodes retain these identities without firing onmounted again.
    let mounted = BTreeMap::from([
        ("a".into(), "DOM A"),
        ("b".into(), "DOM B"),
        ("c".into(), "DOM C"),
    ]);
    assert_eq!(
        mounted_tab(&items, next_tab(&items, 0, true).unwrap(), &mounted),
        Some(&"DOM B")
    );
    items.remove(0);
    assert_eq!(
        mounted_tab(&items, next_tab(&items, 0, true).unwrap(), &mounted),
        Some(&"DOM C")
    );
    items.swap(0, 1);
    assert_eq!(mounted_tab(&items, 0, &mounted), Some(&"DOM C"));
    assert_eq!(
        mounted_tab(&items, next_tab(&items, 0, true).unwrap(), &mounted),
        Some(&"DOM B")
    );
    items.push(TabItem::new("new", "New"));
    assert_eq!(mounted_tab(&items, 2, &mounted), None);
    assert_eq!(mounted_tab(&items, 99, &mounted), None);
}
#[test]
fn tab_overflow_uses_measured_content_and_rejects_invalid_geometry() {
    assert!(!tabs_overflow(340.0, 340.0));
    assert!(tabs_overflow(340.5, 340.0));
    assert!(!tabs_overflow(340.0, 400.0));
    assert!(!tabs_overflow(340.0, 0.0));
    assert!(!tabs_overflow(f64::NAN, 340.0));
    assert!(!tabs_overflow(340.0, f64::INFINITY));
}
#[test]
fn tab_selection_resolves_disabled_removed_and_empty_data() {
    let mut items = vec![
        TabItem::new("a", "A"),
        TabItem::new("b", "B"),
        TabItem::new("c", "C"),
    ];
    items[0].disabled = true;
    assert_eq!(selected_tab(&items, 0), Some(1));
    assert_eq!(selected_tab(&items, 2), Some(2));
    items.remove(2);
    assert_eq!(selected_tab(&items, 2), Some(1));
    items[1].disabled = true;
    assert_eq!(selected_tab(&items, 0), None);
    assert_eq!(selected_tab(&[], 0), None);
}
#[test]
fn explicit_empty_selection_never_falls_back_to_an_enabled_tab() {
    let items = vec![TabItem::new("a", "A"), TabItem::new("b", "B")];
    assert_eq!(TabSelection::Empty.resolve(&items), None);
    assert_eq!(TabSelection::Index(0).resolve(&items), Some(0));
    assert_eq!(TabSelection::default().resolve(&items), Some(0));
}

#[test]
fn tab_change_veto_is_shared_and_blocks_the_accepted_path() {
    let items = vec![TabItem::new("a", "A"), TabItem::new("b", "B")];
    let mut state = Some(0);
    let mut posts = 0;
    if accept_tab_request(&items, state, Some(1), |event| {
        assert_eq!(event.previous, Some(0));
        assert_eq!(event.next, Some(1));
        let retained = event.clone();
        event.prevent_default();
        assert!(retained.default_prevented());
    }) {
        state = Some(1);
        posts += 1;
    }
    assert_eq!(state, Some(0));
    assert_eq!(posts, 0);
    assert!(accept_tab_request(&items, state, None, |event| {
        assert_eq!(event.next, None);
    }));
    assert!(!accept_tab_request(&items, state, None, |event| event.prevent_default()));
}

#[test]
fn invalid_and_disabled_tab_requests_emit_no_before_event() {
    let mut disabled = TabItem::new("disabled", "Disabled");
    disabled.disabled = true;
    let items = vec![TabItem::new("a", "A"), disabled];
    assert!(!accept_tab_request(&items, Some(0), Some(1), |_| panic!(
        "disabled request emitted"
    )));
    assert!(!accept_tab_request(&items, Some(0), Some(99), |_| panic!(
        "unknown request emitted"
    )));
    // Upstream tabClick delivers a change event even for the selected tab.
    assert!(accept_tab_request(&items, Some(0), Some(0), |_| {}));
}

#[test]
fn tab_placement_defaults_to_the_upstream_bottom_edge() {
    assert_eq!(TabPlacement::default(), TabPlacement::Bottom);
    assert_eq!(TabPlacement::Top.class(), "top");
}

#[test]
fn tab_navigation_skips_disabled_and_wraps() {
    let mut items = vec![
        TabItem::new("a", "A"),
        TabItem::new("b", "B"),
        TabItem::new("c", "C"),
    ];
    items[1].disabled = true;
    assert_eq!(next_tab(&items, 0, true), Some(2));
    assert_eq!(next_tab(&items, 2, true), Some(0));
    assert_eq!(next_tab(&items, 0, false), Some(2));
    items.iter_mut().for_each(|item| item.disabled = true);
    assert_eq!(next_tab(&items, 0, true), None);
    assert_eq!(next_tab(&[], 0, false), None);
}
mod tab_workspace_tests {
    use super::*;
    include!("../tab_workspace_tests.rs");
    mod icon_tab_tests {
        use super::*;
        include!("icon_tab_tests.rs");
    }
}

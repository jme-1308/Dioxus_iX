//! Focus-cue protocol regressions through the actual runtime form handlers.
use super::*;
use crate::dropdown_inventory::{HostPartFacts, HostPartKind, ItemRef};
use std::cell::Cell;

struct RawForm(String);
impl dioxus::html::HasFileData for RawForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for RawForm {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
fn event(value: serde_json::Value) -> FormEvent {
    Event::new(
        Rc::new(dioxus::html::FormData::new(RawForm(value.to_string()))),
        true,
    )
}
fn query(runtime: &RuntimeHandle, value: serde_json::Value) -> bool {
    let original = event(value);
    runtime.query(original.clone());
    !original.default_action_enabled()
}

struct Owner {
    node: NodeHandle,
    open: Rc<Cell<bool>>,
    reads: Rc<Cell<usize>>,
    callbacks: Rc<Cell<usize>>,
}
fn owner(runtime: &RuntimeHandle, generation: u64) -> Owner {
    let open = Rc::new(Cell::new(true));
    let reads = Rc::new(Cell::new(0));
    let callbacks = Rc::new(Cell::new(0));
    let read_open = open.clone();
    let read_count = reads.clone();
    let before_count = callbacks.clone();
    let apply_count = callbacks.clone();
    let apply_open = open.clone();
    let key_count = callbacks.clone();
    let node = runtime.register(
        "cue-root".into(),
        generation,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || {
                read_count.set(read_count.get() + 1);
                Some(read_open.get())
            }),
            before: Rc::new(move |_| {
                before_count.set(before_count.get() + 1);
                true
            }),
            apply: Rc::new(move |change| {
                apply_count.set(apply_count.get() + 1);
                apply_open.set(change.open);
            }),
            key: Some(Rc::new(move |_| {
                key_count.set(key_count.get() + 1);
                KeyDecision::default()
            })),
        },
    );
    Owner {
        node,
        open,
        reads,
        callbacks,
    }
}
fn host(key: &str, capture: &str, kind: ItemKind) -> HostFacts {
    HostFacts {
        root_path: Vec::new(),
        item: ItemRef(capture.into()),
        kind,
        target: format!("consumer-{key}"),
        capture: capture.into(),
        registration_key: key.into(),
        source_focusable: true,
        disabled_attribute: false,
        hidden_attribute: false,
        tabindex_attribute: None,
        checked_attribute: false,
        visible_focus: false,
        roving_opt_in: false,
        linked_trigger: None,
        parts: Vec::new(),
    }
}
fn report(
    runtime: &RuntimeHandle,
    sequence: u64,
    generation: u64,
    kind: &str,
    hosts: &[HostFacts],
) -> serde_json::Value {
    let order = hosts
        .iter()
        .map(|host| host.item.clone())
        .collect::<Vec<_>>();
    serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":kind,
        "phase":"bubble","key":"ArrowDown","hosts":["cue-root"],"triggers":["cue-root"],
        "nodes":[{"id":"cue-root","generation":generation,"connected":true,"discovery_ready":true,
            "root_capture":format!("root-object-{generation}"),"trigger_capture":format!("trigger-object-{generation}"),
            "mode":"active-descendant","inside":true,"on_trigger":true,
            "inventory":{"items_host":"actual-items","hosts":hosts,"navigation_order":order,"query_order":order}}]})
}
fn observe(runtime: &RuntimeHandle, sequence: u64, generation: u64, hosts: &[HostFacts]) {
    runtime.input(event(report(runtime, sequence, generation, "key", hosts)));
    assert!(
        runtime.keyboard_mode(),
        "actual admitted key establishes modality"
    );
}
fn focusin(runtime: &RuntimeHandle, sequence: u64, hosts: &[HostFacts], focused: &HostFacts) {
    let mut value = report(runtime, sequence, 1, "item-focusin", hosts);
    value["item_focus_host"] = serde_json::to_value(focused).unwrap();
    runtime.input(event(value));
}
fn cue(runtime: &RuntimeHandle, host: &HostFacts, part: Option<&str>, active: bool) -> bool {
    query(
        runtime,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
        "item_cue":{"key":host.registration_key,"capture":host.capture,"part":part,"active":active}}),
    )
}

#[test]
fn item_cue_queries_project_current_exact_hosts_and_parts_without_consuming_work() {
    let runtime = RuntimeHandle::new();
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let owner = dom.runtime().in_scope(ScopeId::ROOT, || owner(&runtime, 1));
    let mut a = host("a", "actual-a", ItemKind::Select);
    a.parts.push(HostPartFacts {
        kind: HostPartKind::SelectOption,
        capture: "actual-a-inner".into(),
        source_focusable: false,
    });
    let b = host("b", "actual-b", ItemKind::Menu);
    let mut category = host("category", "actual-category", ItemKind::Category);
    category.parts.push(HostPartFacts {
        kind: HostPartKind::CategoryParent,
        capture: "actual-category-parent".into(),
        source_focusable: true,
    });
    let hosts = [a.clone(), b.clone(), category.clone()];
    for host in &hosts {
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
        runtime.register_item_mount(
            &host.registration_key,
            &host.target,
            Rc::new(MountedData::new(())),
        );
    }
    observe(&runtime, 1, 1, &hosts);
    assert!(
        !cue(&runtime, &a, None, false),
        "unknown projection is not a clear command"
    );
    focusin(&runtime, 2, &hosts, &a);
    let delivered = Rc::new(Cell::new(0));
    let completion = delivered.clone();
    assert!(
        owner
            .node
            .queue_completion(Rc::new(move || completion.set(completion.get() + 1)))
    );
    let focus = delivered.clone();
    assert!(
        owner
            .node
            .queue_item_host_focus(&b, Rc::new(move || focus.set(focus.get() + 1)))
    );
    let frames = runtime.pending_completions();
    let jobs = runtime.pending_item_focus_count();
    let reads = owner.reads.get();
    let callbacks = owner.callbacks.get();
    for _ in 0..3 {
        assert!(cue(&runtime, &a, None, true));
        assert!(cue(&runtime, &a, Some("actual-a-inner"), true));
        assert!(!cue(&runtime, &a, None, false));
        assert!(cue(&runtime, &b, None, false));
        assert!(!cue(&runtime, &b, None, true));
        assert!(!cue(&runtime, &a, Some("actual-category-parent"), true));
        assert!(!cue(&runtime, &category, None, false));
        assert!(!cue(&runtime, &category, None, true));
    }
    assert_eq!(
        owner.reads.get(),
        reads,
        "pure offers never invoke registered read callbacks"
    );
    assert_eq!(owner.callbacks.get(), callbacks);
    let mut wrong_owner = serde_json::json!({"owner":"foreign-runtime","sequence":0,
        "item_cue":{"key":a.registration_key,"capture":a.capture,"active":true}});
    assert!(!query(&runtime, wrong_owner.clone()));
    wrong_owner["owner"] = runtime.owner_id().into();
    wrong_owner["item_cue"]["capture"] = "copied-object".into();
    assert!(!query(&runtime, wrong_owner));
    let mut copy = a.clone();
    copy.capture = "copied-object".into();
    copy.item = ItemRef(copy.capture.clone());
    let mut copied_report = report(&runtime, 3, 1, "observe", std::slice::from_ref(&copy));
    copied_report["key"] = "".into();
    runtime.input(event(copied_report));
    let after_observe_reads = owner.reads.get();
    assert!(!cue(&runtime, &copy, None, true));
    assert!(!cue(&runtime, &copy, None, false));
    assert!(
        cue(&runtime, &a, None, true),
        "copy cannot retarget the bound object"
    );
    assert_eq!(
        owner.reads.get(),
        after_observe_reads,
        "pure projection invokes no read callback"
    );
    assert!(after_observe_reads >= reads);
    assert_eq!(owner.callbacks.get(), callbacks);
    assert_eq!(runtime.pending_completions(), frames);
    assert_eq!(runtime.pending_item_focus_count(), jobs);
    assert_eq!(delivered.get(), 0);
    focusin(&runtime, 4, &hosts, &category);
    assert!(cue(&runtime, &a, None, false));
    assert!(cue(
        &runtime,
        &category,
        Some("actual-category-parent"),
        true
    ));
    let mut pointer_clear = report(&runtime, 5, 1, "modality", &hosts);
    pointer_clear["input_kind"] = "mousedown".into();
    runtime.input(event(pointer_clear));
    assert!(cue(
        &runtime,
        &category,
        Some("actual-category-parent"),
        false
    ));
    assert!(cue(&runtime, &a, None, false));
}

#[test]
fn mounted_focus_attempt_publishes_cues_after_await_and_unmount_cancels_the_next_attempt() {
    let runtime = RuntimeHandle::new();
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let owner = dom.runtime().in_scope(ScopeId::ROOT, || owner(&runtime, 1));
    let a = host("a", "actual-a", ItemKind::Dropdown);
    let b = host("b", "actual-b", ItemKind::Menu);
    let hosts = [a.clone(), b.clone()];
    let mock = super::tests::ItemMountMock::default();
    mock.pending.set(true);
    super::tests::install_item_mock(&runtime, &a, mock.clone());
    runtime.register_item_cue(b.registration_key.clone(), b.kind);
    observe(&runtime, 1, 1, &hosts);
    let callbacks = Rc::new(Cell::new(0));
    let count = callbacks.clone();
    assert!(
        owner
            .node
            .queue_item_host_focus(&a, Rc::new(move || count.set(count.get() + 1)))
    );
    super::tests::settle_item_effects(&mut dom);
    assert_eq!(callbacks.get(), 1);
    assert_eq!(runtime.pending_item_focus_count(), 1);
    assert!(
        runtime.0.borrow().item_visual_host.is_none(),
        "focus permission alone does not publish a cue"
    );
    focusin(&runtime, 2, &hosts, &b);
    assert!(cue(&runtime, &b, None, true));
    mock.pending.set(false);
    mock.waker.borrow_mut().take().unwrap().wake();
    super::tests::settle_item_effects(&mut dom);
    assert!(cue(&runtime, &a, None, true));
    assert!(cue(&runtime, &b, None, false));
    assert_eq!(callbacks.get(), 1);
    assert_eq!(runtime.pending_item_focus_count(), 0);
    assert!(
        owner
            .node
            .queue_item_host_focus(&a, Rc::new(|| panic!("unregistered owner")))
    );
    owner.node.unregister();
    super::tests::settle_item_effects(&mut dom);
    assert_eq!(runtime.pending_item_focus_count(), 0);
    assert_eq!(callbacks.get(), 1);
}

fn root_offer(
    runtime: &RuntimeHandle,
    generation: u64,
    field: &str,
    value: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({"owner":runtime.owner_id(),"sequence":0,"item_root_cue":{
        "root":"cue-root","generation":generation,"capture":format!("root-object-{generation}"),
        "trigger_capture":format!("trigger-object-{generation}"),"field":field,"value":value}})
}
#[test]
fn root_cue_queries_read_fresh_mode_open_projection_and_reject_stale_registration() {
    let runtime = RuntimeHandle::new();
    let old = owner(&runtime, 1);
    let host = host("a", "actual-a", ItemKind::Dropdown);
    runtime.register_item_cue(host.registration_key.clone(), host.kind);
    observe(&runtime, 1, 1, std::slice::from_ref(&host));
    focusin(&runtime, 2, std::slice::from_ref(&host), &host);
    let mode_ad = Rc::new(Cell::new(true));
    let active = Rc::new(RefCell::new(Some("consumer-a".to_string())));
    let reader_calls = Rc::new(Cell::new(0));
    let mode = mode_ad.clone();
    let current = active.clone();
    let opened = old.open.clone();
    let count = reader_calls.clone();
    old.node.set_item_visual_reader(Rc::new(move || {
        count.set(count.get() + 1);
        RootVisual {
            active: current.borrow().clone(),
            aria: if mode.get() && opened.get() {
                current.borrow().clone()
            } else {
                None
            },
        }
    }));
    for field in ["active", "aria", "trigger-aria"] {
        assert!(query(
            &runtime,
            root_offer(&runtime, 1, field, Some("consumer-a"))
        ));
        assert!(!query(
            &runtime,
            root_offer(&runtime, 1, field, Some("wrong-id"))
        ));
    }
    let before = reader_calls.get();
    for field in ["capture", "trigger_capture", "generation", "root"] {
        let mut wrong = root_offer(&runtime, 1, "trigger-aria", Some("consumer-a"));
        wrong["item_root_cue"][field] = if field == "generation" {
            serde_json::json!(2)
        } else {
            serde_json::json!("other-object")
        };
        assert!(!query(&runtime, wrong));
    }
    assert_eq!(
        reader_calls.get(),
        before,
        "bad binding cannot invoke projection reader"
    );
    mode_ad.set(false);
    assert!(query(&runtime, root_offer(&runtime, 1, "aria", None)));
    assert!(!query(
        &runtime,
        root_offer(&runtime, 1, "aria", Some("consumer-a"))
    ));
    assert!(query(
        &runtime,
        root_offer(&runtime, 1, "active", Some("consumer-a"))
    ));
    mode_ad.set(true);
    old.open.set(false);
    assert!(query(
        &runtime,
        root_offer(&runtime, 1, "trigger-aria", None)
    ));
    *active.borrow_mut() = Some("fresh-current-target".into());
    assert!(query(
        &runtime,
        root_offer(&runtime, 1, "active", Some("fresh-current-target"))
    ));
    assert!(!query(
        &runtime,
        root_offer(&runtime, 1, "active", Some("consumer-a"))
    ));
    let replacement = owner(&runtime, 2);
    replacement
        .node
        .set_item_visual_reader(Rc::new(|| RootVisual {
            active: None,
            aria: None,
        }));
    assert!(!query(
        &runtime,
        root_offer(&runtime, 1, "active", Some("fresh-current-target"))
    ));
    runtime.input(event(report(
        &runtime,
        3,
        2,
        "observe",
        std::slice::from_ref(&host),
    )));
    assert!(query(&runtime, root_offer(&runtime, 2, "active", None)));
    replacement.node.unregister();
    assert!(!query(&runtime, root_offer(&runtime, 2, "active", None)));
}

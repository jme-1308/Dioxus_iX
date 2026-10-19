//! Rust cue state driven by admitted native observations and mounted focus.
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
#[test]
fn current_host_and_part_cues_are_repeatable_without_consuming_focus_work() {
    let runtime = RuntimeHandle::new();
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let owner = dom.runtime().in_scope(ScopeId::ROOT, || owner(&runtime, 1));
    // The option need not have a native ID/capture: its cue belongs to Select.
    let a = host("a", "actual-a", ItemKind::Select);
    let b = host("b", "actual-b", ItemKind::Menu);
    let mut category = host("category", "actual-category", ItemKind::Category);
    category.source_focusable = false;
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
    assert_eq!(runtime.item_visual_key(), None);
    let observation = |owner: String, path: Vec<String>| {
        serde_json::json!({
            "owner":owner,"sequence":0,
            "item_focus_observation":{"hosts":hosts,"path":path}
        })
    };
    assert!(query(
        &runtime,
        observation(runtime.owner_id(), vec![a.capture.clone()])
    ));
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
        assert_eq!(runtime.item_visual_key().as_deref(), Some("a"));
        assert_eq!(runtime.item_visual_target().as_deref(), Some("consumer-a"));
        assert!(runtime.part_active("a", HostPartKind::SelectOption));
        assert!(!runtime.part_active("b", HostPartKind::SelectOption));
        assert!(!runtime.part_active("category", HostPartKind::CategoryParent));
        assert!(!runtime.part_active("a", HostPartKind::CategoryParent));
    }
    assert_eq!(
        owner.reads.get(),
        reads,
        "cue reads do not invoke application callbacks"
    );
    assert_eq!(owner.callbacks.get(), callbacks);
    assert_eq!(runtime.pending_completions(), frames);
    assert_eq!(runtime.pending_item_focus_count(), jobs);
    assert_eq!(delivered.get(), 0);
    assert!(!query(
        &runtime,
        observation(
            "foreign-runtime".into(),
            vec![category.parts[0].capture.clone()]
        )
    ));
    assert!(!query(
        &runtime,
        observation(runtime.owner_id(), vec!["copied-object".into()])
    ));
    assert_eq!(runtime.item_visual_key().as_deref(), Some("a"));
    assert!(query(
        &runtime,
        observation(
            runtime.owner_id(),
            vec![category.parts[0].capture.clone(), category.capture.clone()]
        )
    ));
    assert_eq!(
        runtime.item_visual_key(),
        None,
        "Category outer host is not an AD active item"
    );
    assert_eq!(runtime.item_visual_target(), None);
    assert!(runtime.part_active("category", HostPartKind::CategoryParent));
    assert!(!runtime.part_active("a", HostPartKind::SelectOption));
    // An ineligible parent cannot retain a cue after the next observation.
    let mut ineligible = category.clone();
    ineligible.parts[0].source_focusable = false;
    assert!(query(
        &runtime,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
        "item_focus_observation":{"hosts":[ineligible],"path":[category.parts[0].capture]}})
    ));
    assert!(!runtime.part_active("category", HostPartKind::CategoryParent));
    focusin(&runtime, 4, &hosts, &a);
    let mut pointer_clear = report(&runtime, 5, 1, "modality", &hosts);
    pointer_clear["input_kind"] = "mousedown".into();
    runtime.input(event(pointer_clear));
    assert_eq!(runtime.item_visual_key(), None);
    assert!(!runtime.part_active("a", HostPartKind::SelectOption));
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
    assert_eq!(runtime.item_visual_key().as_deref(), Some("b"));
    mock.pending.set(false);
    mock.waker.borrow_mut().take().unwrap().wake();
    super::tests::settle_item_effects(&mut dom);
    assert_eq!(runtime.item_visual_key().as_deref(), Some("a"));
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

#[test]
fn unmounted_or_replaced_registered_control_does_not_inherit_the_current_cue() {
    let runtime = RuntimeHandle::new();
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let _owner = dom.runtime().in_scope(ScopeId::ROOT, || owner(&runtime, 1));
    let select = host("select", "actual-select", ItemKind::Select);
    runtime.register_item_cue(select.registration_key.clone(), select.kind);
    runtime.register_item_mount(
        &select.registration_key,
        &select.target,
        Rc::new(MountedData::new(())),
    );
    observe(&runtime, 1, 1, std::slice::from_ref(&select));
    focusin(&runtime, 2, std::slice::from_ref(&select), &select);
    assert!(runtime.part_active("select", HostPartKind::SelectOption));
    runtime.register_item_mount(
        &select.registration_key,
        &select.target,
        Rc::new(MountedData::new(())),
    );
    assert_eq!(runtime.item_visual_key(), None);
    assert!(!runtime.part_active("select", HostPartKind::SelectOption));
    focusin(&runtime, 3, std::slice::from_ref(&select), &select);
    assert!(runtime.part_active("select", HostPartKind::SelectOption));
    runtime.unregister_item_cue(&select.registration_key);
    assert_eq!(runtime.item_visual_key(), None);
    assert_eq!(runtime.item_visual_target(), None);
    assert!(!runtime.part_active("select", HostPartKind::SelectOption));
    runtime.register_item_cue(select.registration_key.clone(), select.kind);
    runtime.register_item_mount(
        &select.registration_key,
        &select.target,
        Rc::new(MountedData::new(())),
    );
    assert_eq!(
        runtime.item_visual_key(),
        None,
        "a reused key and public ID need a new focus attempt"
    );
}

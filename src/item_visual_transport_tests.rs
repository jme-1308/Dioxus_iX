//! Registered Dioxus focus events and mounted-focus cue lifetimes.
use super::*;
use crate::dropdown_inventory::{HostPartKind, ItemRef};
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
    let mut observation = report(runtime, sequence, generation, "observe", hosts);
    observation["document"] = format!("{}-focus-test", runtime.owner_id()).into();
    runtime.input(event(observation));
    let node = NodeHandle {
        runtime: Rc::downgrade(&runtime.0),
        id: "cue-root".into(),
        generation,
    };
    let key = super::tests::typed_keyboard("ArrowDown", Modifiers::empty());
    let facts = node.dioxus_key_facts(&key, true, None, false).unwrap();
    node.dioxus_key(facts);
    assert!(
        runtime.keyboard_mode(),
        "actual admitted key establishes modality"
    );
    assert_eq!(runtime.document_keyboard_mode(), Some(true));
}
fn keyboard(runtime: &RuntimeHandle, sequence: u64) {
    runtime.attach_document(&format!("{}-focus-test", runtime.owner_id()));
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,
        "document":format!("{}-focus-test",runtime.owner_id()),
        "kind":"document","document_event":{"sequence":sequence,"action":"input","input_kind":"keydown","key":"Tab"}}),
    ));
    assert!(runtime.keyboard_mode());
}
#[test]
fn registered_focus_before_inventory_sets_parts_and_late_blur_preserves_newer_cue() {
    let runtime = RuntimeHandle::new();
    let mut dom = VirtualDom::new(|| rsx! { div {} });
    dom.rebuild_in_place();
    let owner = dom.runtime().in_scope(ScopeId::ROOT, || owner(&runtime, 1));
    let sibling = dom.runtime().in_scope(ScopeId::ROOT, || {
        runtime.register(
            "sibling-root".into(),
            1,
            OverlayKind::Dropdown,
            CloseBehavior::Both,
            Callbacks {
                read: Rc::new(|| Some(true)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        )
    });
    keyboard(&runtime, 1);
    let a = host("a", "actual-a", ItemKind::Select);
    let b = host("b", "actual-b", ItemKind::Menu);
    let category = host("category", "actual-category", ItemKind::Category);
    for host in [&a, &b, &category] {
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
        runtime.register_owned_item_control_mount(
            &host.registration_key,
            &host.target,
            &host.target,
            Rc::new(MountedData::new(())),
            Some((owner.node.id.clone(), owner.node.generation)),
        );
    }
    assert!(runtime.0.borrow().inventories.is_empty());
    runtime.focus_registered_item("a", true, Some(HostPartKind::SelectOption));
    assert_eq!(runtime.item_visual_key().as_deref(), Some("a"));
    assert!(
        runtime.item_visual_matches(&a),
        "native capture equality is not cue authority"
    );
    assert!(runtime.item_visual_owned_by(owner.node.id(), owner.node.generation()));
    assert!(!runtime.item_visual_owned_by(sibling.id(), sibling.generation()));
    assert!(!runtime.item_visual_owned_by(owner.node.id(), owner.node.generation() + 1));
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
    }
    assert_eq!(owner.reads.get(), reads);
    assert_eq!(owner.callbacks.get(), callbacks);
    assert_eq!(runtime.pending_completions(), frames);
    assert_eq!(runtime.pending_item_focus_count(), jobs);
    assert_eq!(delivered.get(), 0);
    runtime.focus_registered_item("b", true, None);
    runtime.blur_registered_item("a");
    assert_eq!(runtime.item_visual_key().as_deref(), Some("b"));
    runtime.focus_registered_item("category", true, Some(HostPartKind::CategoryParent));
    runtime.blur_registered_item("b");
    assert_eq!(
        runtime.item_visual_key(),
        None,
        "Category outer never becomes an AD host"
    );
    assert_eq!(runtime.item_visual_target(), None);
    assert!(runtime.part_active("category", HostPartKind::CategoryParent));
    assert!(
        !runtime.item_visual_owned_by(owner.node.id(), owner.node.generation()),
        "Category parent does not create root active-descendant ARIA"
    );
    runtime.blur_registered_item("category");
    assert!(!runtime.part_active("category", HostPartKind::CategoryParent));
    runtime.focus_registered_item("category", false, Some(HostPartKind::CategoryParent));
    assert!(!runtime.part_active("category", HostPartKind::CategoryParent));
    runtime.focus_registered_item("a", true, None);
    runtime.focus_registered_item("unregistered", true, None);
    runtime.blur_registered_item("unregistered");
    assert_eq!(runtime.item_visual_key().as_deref(), Some("a"));
    let mut wrong_kind = a.clone();
    wrong_kind.kind = ItemKind::Dropdown;
    assert!(!runtime.item_visual_matches(&wrong_kind));
    let mut wrong_target = a.clone();
    wrong_target.target = "copied-public-target".into();
    assert!(!runtime.item_visual_matches(&wrong_target));
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"sequence":2,
        "document":format!("{}-focus-test",runtime.owner_id()),
        "kind":"document","document_event":{"sequence":2,"action":"input","input_kind":"mousedown"}}),
    ));
    runtime.focus_registered_item("a", true, None);
    assert_eq!(
        runtime.item_visual_key(),
        None,
        "pointer focus does not recreate a keyboard cue"
    );
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
    super::tests::install_item_mock(&runtime, &b, super::tests::ItemMountMock::default());
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
    runtime.focus_registered_item(&b.registration_key, true, None);
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
    let owner = dom.runtime().in_scope(ScopeId::ROOT, || owner(&runtime, 1));
    let select = host("select", "actual-select", ItemKind::Select);
    runtime.register_item_cue(select.registration_key.clone(), select.kind);
    runtime.register_owned_item_control_mount(
        &select.registration_key,
        &select.target,
        &select.target,
        Rc::new(MountedData::new(())),
        Some((owner.node.id.clone(), owner.node.generation)),
    );
    observe(&runtime, 1, 1, std::slice::from_ref(&select));
    runtime.focus_registered_item(&select.registration_key, true, None);
    assert!(runtime.part_active("select", HostPartKind::SelectOption));
    assert!(runtime.item_visual_owned_by(owner.node.id(), owner.node.generation()));
    runtime.register_owned_item_control_mount(
        &select.registration_key,
        &select.target,
        &select.target,
        Rc::new(MountedData::new(())),
        Some((owner.node.id.clone(), owner.node.generation)),
    );
    assert_eq!(runtime.item_visual_key(), None);
    assert!(!runtime.item_visual_owned_by(owner.node.id(), owner.node.generation()));
    assert!(!runtime.part_active("select", HostPartKind::SelectOption));
    runtime.focus_registered_item(&select.registration_key, true, None);
    assert!(runtime.part_active("select", HostPartKind::SelectOption));
    assert!(runtime.item_visual_owned_by(owner.node.id(), owner.node.generation()));
    runtime.unregister_item_cue(&select.registration_key);
    runtime.focus_registered_item(&select.registration_key, true, None);
    assert_eq!(runtime.item_visual_key(), None);
    assert!(!runtime.item_visual_owned_by(owner.node.id(), owner.node.generation()));
    assert_eq!(runtime.item_visual_target(), None);
    assert!(!runtime.part_active("select", HostPartKind::SelectOption));
    runtime.register_item_cue(select.registration_key.clone(), select.kind);
    runtime.register_owned_item_control_mount(
        &select.registration_key,
        &select.target,
        &select.target,
        Rc::new(MountedData::new(())),
        Some((owner.node.id.clone(), owner.node.generation)),
    );
    assert_eq!(
        runtime.item_visual_key(),
        None,
        "a reused key and public ID need a new focus attempt"
    );
}

//! Opening-focus protocol through registered runtime form-event handlers.
use super::*;
use crate::descendant_focus::{DescendantFacts, DescendantNode};
use crate::dropdown_inventory::ItemRef;
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
    let event = event(value);
    runtime.query(event.clone());
    !event.default_action_enabled()
}

struct Owner {
    node: NodeHandle,
    open: Rc<Cell<bool>>,
    checked: Rc<Cell<bool>>,
    roving: Rc<Cell<bool>>,
    preparations: Rc<Cell<usize>>,
}
fn owner(runtime: &RuntimeHandle, generation: u64) -> Owner {
    let open = Rc::new(Cell::new(true));
    let read = open.clone();
    let apply = open.clone();
    let node = runtime.register(
        "opening-root".into(),
        generation,
        OverlayKind::Dropdown,
        CloseBehavior::Both,
        Callbacks {
            read: Rc::new(move || Some(read.get())),
            before: Rc::new(|_| true),
            apply: Rc::new(move |change| apply.set(change.open)),
            key: Some(Rc::new(|_| KeyDecision::default())),
        },
    );
    let checked = Rc::new(Cell::new(false));
    let roving = Rc::new(Cell::new(false));
    let preparations = Rc::new(Cell::new(0));
    let current_checked = checked.clone();
    let current_roving = roving.clone();
    let count = preparations.clone();
    node.set_descendant_focus_handler(Rc::new(move |_, last, allow_checked| {
        count.set(count.get() + 1);
        if current_roving.get() {
            None
        } else {
            Some(!last && allow_checked && current_checked.get())
        }
    }));
    Owner {
        node,
        open,
        checked,
        roving,
        preparations,
    }
}
fn host(key: &str, capture: &str) -> HostFacts {
    HostFacts {
        item: ItemRef(capture.into()),
        kind: ItemKind::Dropdown,
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
fn inventory(hosts: &[HostFacts]) -> InventoryFacts {
    let order = hosts
        .iter()
        .map(|host| host.item.clone())
        .collect::<Vec<_>>();
    InventoryFacts {
        items_host: ItemRef("actual-items".into()),
        hosts: hosts.to_vec(),
        navigation_order: order.clone(),
        query_order: order,
        slots: None,
    }
}
fn observe(runtime: &RuntimeHandle, generation: u64, sequence: u64, hosts: &[HostFacts]) {
    for host in hosts {
        runtime.register_item_cue(host.registration_key.clone(), host.kind);
    }
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,
        "kind":"key","phase":"bubble","key":"ArrowDown","hosts":["opening-root"],
        "triggers":["opening-root"],"nodes":[{"id":"opening-root","generation":generation,
        "connected":true,"discovery_ready":true,"root_capture":format!("reference-{generation}"),
        "trigger_capture":format!("trigger-{generation}"),"mode":"active-descendant",
        "inside":true,"on_trigger":true,"inventory":inventory(hosts)}]}),
    ));
    assert!(runtime.keyboard_mode());
}
fn focusin(runtime: &RuntimeHandle, generation: u64, sequence: u64, focused: &HostFacts) {
    runtime.input(event(serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,
        "kind":"item-focusin","item_focus_host":focused,"nodes":[{"id":"opening-root",
        "generation":generation,"connected":true,"inventory":inventory(std::slice::from_ref(focused))}]})));
}
fn node(capture: &str, tag: &str, checked: bool) -> DescendantNode {
    DescendantNode {
        capture: capture.into(),
        tag: tag.into(),
        is_html: true,
        source_kind: None,
        tabindex: None,
        href: false,
        contenteditable: false,
        hidden: false,
        disabled: false,
        checked,
    }
}
fn facts(nodes: Vec<DescendantNode>, generation: u64) -> DescendantFacts {
    let light = nodes.iter().map(|node| node.capture.clone()).collect();
    let mut nodes = nodes;
    nodes.push(node(&format!("reference-{generation}"), "div", false));
    DescendantFacts {
        nodes,
        light,
        slots: Vec::new(),
        shadows: Vec::new(),
        fallback: format!("reference-{generation}"),
    }
}
fn job(
    owner: &Owner,
    runtime: &RuntimeHandle,
    version: u64,
    last: bool,
    checked: bool,
) -> DescendantJob {
    assert!(owner.node.queue_descendant_focus(version, last, checked));
    runtime.0.borrow().descendant_focus.unsent().pop().unwrap()
}
fn phase(
    runtime: &RuntimeHandle,
    job: &DescendantJob,
    phase: &str,
    target: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({"owner":runtime.owner_id(),"sequence":0,"descendant_focus":{
        "sequence":job.sequence,"root":job.root,"generation":job.generation,
        "reference":job.reference,"phase":phase,"target":target}})
}
fn arm(runtime: &RuntimeHandle, job: &DescendantJob) {
    assert!(query(runtime, phase(runtime, job, "arm", None)));
}
fn prepare(
    runtime: &RuntimeHandle,
    job: &DescendantJob,
    facts: &DescendantFacts,
    hosts: &[HostFacts],
) -> bool {
    let mut value = phase(runtime, job, "prepare", None);
    value["descendant_focus"]["facts"] = serde_json::to_value(facts).unwrap();
    value["descendant_focus"]["inventory"] = serde_json::to_value(inventory(hosts)).unwrap();
    value["descendant_focus"]["hosts"] = serde_json::to_value(hosts).unwrap();
    query(runtime, value)
}
fn cue(runtime: &RuntimeHandle, host: &HostFacts, active: bool) -> bool {
    query(
        runtime,
        serde_json::json!({"owner":runtime.owner_id(),"sequence":0,
        "item_cue":{"key":host.registration_key,"capture":host.capture,"active":active}}),
    )
}

#[test]
fn descendant_prepare_reads_current_checked_mode_and_preserves_exact_authorization() {
    let runtime = RuntimeHandle::new();
    let owner = owner(&runtime, 1);
    observe(&runtime, 1, 1, &[]);
    let facts = facts(
        vec![
            node("first-button", "button", false),
            node("checked-input", "input", true),
            node("last-button", "button", false),
        ],
        1,
    );
    let first = job(&owner, &runtime, 10, false, true);
    // The setting is read at preparation, after admission.
    owner.checked.set(true);
    let mut wrong = phase(&runtime, &first, "arm", None);
    wrong["descendant_focus"]["reference"] = "copied-reference".into();
    assert!(!query(&runtime, wrong));
    arm(&runtime, &first);
    let mut wrong = phase(&runtime, &first, "prepare", None);
    wrong["owner"] = "foreign-runtime".into();
    assert!(!query(&runtime, wrong));
    assert_eq!(owner.preparations.get(), 0);
    assert!(prepare(&runtime, &first, &facts, &[]));
    assert_eq!(owner.preparations.get(), 1);
    assert!(!prepare(&runtime, &first, &facts, &[]));
    for (field, value) in [
        ("sequence", serde_json::json!(first.sequence + 100)),
        ("root", serde_json::json!("copied-id")),
        ("generation", serde_json::json!(2)),
        ("reference", serde_json::json!("copied-reference")),
        ("target", serde_json::json!("first-button")),
    ] {
        let mut wrong = phase(&runtime, &first, "focus", Some("checked-input"));
        wrong["descendant_focus"][field] = value;
        assert!(!query(&runtime, wrong));
    }
    assert!(query(
        &runtime,
        phase(&runtime, &first, "focus", Some("checked-input"))
    ));
    assert!(!query(
        &runtime,
        phase(&runtime, &first, "focus", Some("checked-input"))
    ));
    assert!(!query(
        &runtime,
        phase(&runtime, &first, "visual", Some("first-button"))
    ));
    assert!(query(
        &runtime,
        phase(&runtime, &first, "visual", Some("checked-input"))
    ));
    assert!(!query(
        &runtime,
        phase(&runtime, &first, "visual", Some("checked-input"))
    ));
    let last = job(&owner, &runtime, 11, true, true);
    arm(&runtime, &last);
    assert!(prepare(&runtime, &last, &facts, &[]));
    assert!(query(
        &runtime,
        phase(&runtime, &last, "focus", Some("last-button"))
    ));
    assert!(query(
        &runtime,
        phase(&runtime, &last, "visual", Some("last-button"))
    ));
    let roving = job(&owner, &runtime, 12, false, true);
    owner.roving.set(true);
    arm(&runtime, &roving);
    assert!(!prepare(&runtime, &roving, &facts, &[]));
    assert!(!query(
        &runtime,
        phase(&runtime, &roving, "focus", Some("checked-input"))
    ));
    assert!(runtime.0.borrow().descendant_focus.unsent().is_empty());
}

#[test]
fn descendant_hide_cancels_unattempted_focus_but_preserves_post_attempt_cue() {
    let runtime = RuntimeHandle::new();
    let owner = owner(&runtime, 1);
    let a = host("a", "actual-a");
    let b = host("b", "actual-b");
    let hosts = [a.clone(), b.clone()];
    observe(&runtime, 1, 1, &hosts);
    focusin(&runtime, 1, 2, &b);
    let mut custom = node(&a.capture, "div", false);
    custom.source_kind = Some(ItemKind::Dropdown);
    let facts = facts(vec![custom], 1);
    let canceled = job(&owner, &runtime, 1, false, true);
    arm(&runtime, &canceled);
    owner.open.set(false);
    assert!(!prepare(&runtime, &canceled, &facts, &hosts));
    owner.open.set(true);
    assert!(!prepare(&runtime, &canceled, &facts, &hosts));
    assert!(cue(&runtime, &b, true));
    let accepted = job(&owner, &runtime, 2, false, true);
    arm(&runtime, &accepted);
    assert!(prepare(&runtime, &accepted, &facts, &hosts));
    assert!(query(
        &runtime,
        phase(&runtime, &accepted, "focus", Some(&a.capture))
    ));
    assert!(
        cue(&runtime, &b, true),
        "permission precedes the native focus attempt"
    );
    assert!(cue(&runtime, &a, false));
    // A native focus listener can hide the overlay before its patched focus returns.
    owner.open.set(false);
    let mut copied = a.clone();
    copied.capture = "copied-object".into();
    copied.item = ItemRef(copied.capture.clone());
    let mut visual = phase(&runtime, &accepted, "visual", Some(&a.capture));
    visual["descendant_focus"]["attempt_path"] = serde_json::json!([copied.capture, a.capture]);
    visual["descendant_focus"]["hosts"] =
        serde_json::to_value([copied.clone(), a.clone()]).unwrap();
    assert!(query(&runtime, visual.clone()));
    assert!(
        cue(&runtime, &a, true),
        "copied public identity cannot retarget the captured attempt"
    );
    assert!(!cue(&runtime, &copied, true));
    assert!(cue(&runtime, &b, false));
    assert!(!query(&runtime, visual));
}

#[test]
fn descendant_jobs_are_independent_until_actual_owner_replacement_or_drop() {
    let runtime = RuntimeHandle::new();
    let old_owner = owner(&runtime, 1);
    observe(&runtime, 1, 1, &[]);
    let facts = facts(
        vec![
            node("first", "button", false),
            node("last", "button", false),
        ],
        1,
    );
    let older = job(&old_owner, &runtime, 9, true, false);
    let newer = job(&old_owner, &runtime, 100, false, false);
    arm(&runtime, &older);
    arm(&runtime, &newer);
    assert!(prepare(&runtime, &newer, &facts, &[]));
    assert!(query(
        &runtime,
        phase(&runtime, &newer, "focus", Some("first"))
    ));
    assert!(query(
        &runtime,
        phase(&runtime, &newer, "visual", Some("first"))
    ));
    runtime.input(event(
        serde_json::json!({"owner":runtime.owner_id(),"sequence":2,
        "kind":"observe","nodes":[{"id":"opening-root","generation":1,
        "connected":true,"discovery_ready":false}]}),
    ));
    assert!(prepare(&runtime, &older, &facts, &[]));
    assert!(query(
        &runtime,
        phase(&runtime, &older, "focus", Some("last"))
    ));
    assert!(query(
        &runtime,
        phase(&runtime, &older, "visual", Some("last"))
    ));
    let replaced = job(&old_owner, &runtime, 101, false, true);
    arm(&runtime, &replaced);
    assert!(prepare(&runtime, &replaced, &facts, &[]));
    assert!(query(
        &runtime,
        phase(&runtime, &replaced, "focus", Some("first"))
    ));
    let replacement = owner(&runtime, 2);
    assert!(!query(
        &runtime,
        phase(&runtime, &replaced, "visual", Some("first"))
    ));
    assert!(!old_owner.node.queue_descendant_focus(102, false, true));
    old_owner.node.unregister();
    observe(&runtime, 2, 3, &[]);
    let dropped = job(&replacement, &runtime, 1, false, true);
    arm(&runtime, &dropped);
    replacement.node.unregister();
    assert!(!query(&runtime, phase(&runtime, &dropped, "prepare", None)));
    assert!(!query(
        &runtime,
        phase(&runtime, &dropped, "focus", Some("first"))
    ));
    assert!(runtime.0.borrow().descendant_focus.unsent().is_empty());
    assert!(runtime.0.borrow().descendant_visual.is_empty());
}

#[test]
fn html_attempt_without_focusin_clears_prior_cue_while_svg_preserves_it() {
    let runtime = RuntimeHandle::new();
    let owner = owner(&runtime, 1);
    let existing = host("existing", "actual-existing");
    observe(&runtime, 1, 1, std::slice::from_ref(&existing));
    focusin(&runtime, 1, 2, &existing);
    let mut svg = node("actual-svg", "svg", false);
    svg.is_html = false;
    svg.tabindex = Some("0".into());
    let svg_facts = facts(vec![svg, node("later-button", "button", false)], 1);
    let svg_job = job(&owner, &runtime, 1, false, true);
    arm(&runtime, &svg_job);
    assert!(prepare(&runtime, &svg_job, &svg_facts, &[]));
    assert!(!query(
        &runtime,
        phase(&runtime, &svg_job, "focus", Some("later-button"))
    ));
    assert!(query(
        &runtime,
        phase(&runtime, &svg_job, "focus", Some("actual-svg"))
    ));
    assert!(cue(&runtime, &existing, true));
    let mut svg_visual = phase(&runtime, &svg_job, "visual", Some("actual-svg"));
    svg_visual["descendant_focus"]["attempt_path"] =
        serde_json::json!(["actual-svg", "reference-1"]);
    assert!(query(&runtime, svg_visual.clone()));
    assert!(cue(&runtime, &existing, true));
    assert!(!query(&runtime, svg_visual));
    // No focusin report occurs for this HTML attempt. The global patched
    // focus method still updates its path even when native focus is a no-op.
    let html_facts = facts(vec![node("native-audio", "audio", false)], 1);
    let html_job = job(&owner, &runtime, 2, false, true);
    arm(&runtime, &html_job);
    assert!(prepare(&runtime, &html_job, &html_facts, &[]));
    assert!(query(
        &runtime,
        phase(&runtime, &html_job, "focus", Some("native-audio"))
    ));
    assert!(
        cue(&runtime, &existing, true),
        "focus permission alone does not clear the cue"
    );
    let mut html_visual = phase(&runtime, &html_job, "visual", Some("native-audio"));
    html_visual["descendant_focus"]["attempt_path"] =
        serde_json::json!(["native-audio", "reference-1"]);
    assert!(query(&runtime, html_visual.clone()));
    assert!(cue(&runtime, &existing, false));
    assert!(!query(&runtime, html_visual));
    assert!(runtime.0.borrow().descendant_focus.unsent().is_empty());
    assert!(runtime.0.borrow().descendant_visual.is_empty());
}

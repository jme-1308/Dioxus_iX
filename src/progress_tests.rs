//! Public rendering regressions; actual paint and controls are checked in the preview.
use super::*;

fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn progress_exposes_normalized_value_and_helper_in_both_presentations() {
    let html = render(|| {
        rsx! {
            ProgressIndicator {
                class: "download", label: "Download", helper_text: "Receiving records",
                min: 100.0, max: 200.0, value: 150.0,
                "50%"
            }
            ProgressIndicator {
                progress_type: ProgressType::Circular, size: ProgressSize::Xl,
                label: "Complete", value: 200.0, "100%"
            }
        }
    });
    assert_eq!(html.matches("role=\"progressbar\"").count(), 2);
    for expected in [
        "aria-label=\"Download\"",
        "aria-description=\"Receiving records\"",
        "aria-valuenow=50",
        "width:50%",
        "aria-valuenow=100",
        "stroke-dashoffset=\"0\"",
        "ix-progress-circle-value",
        "100%",
    ] {
        assert!(html.contains(expected), "missing {expected}: {html}");
    }
}

#[test]
fn progress_tooltip_preserves_description_and_invalid_range_is_empty() {
    let html = render(|| {
        rsx! { ProgressIndicator {
            label: "Import", helper_text: "Check the connection and retry",
            show_text_as_tooltip: true, status: ProgressStatus::Error,
            min: 10.0, max: 10.0, value: 12.0,
        } }
    });
    assert!(!html.contains("title="));
    let root = progress_class_tag(&html, "ix-progress");
    let description = progress_attribute(root, "aria-describedby").unwrap();
    assert_eq!(
        progress_attribute(progress_id_tag(&html, description), "role"),
        Some("tooltip")
    );
    assert_eq!(progress_attribute(root, "tabindex"), Some("-1"));
    assert!(html.contains("aria-description=\"Check the connection and retry\""));
    assert!(html.contains("aria-valuenow=0"));
    assert!(html.contains("width:0%"));
    assert_eq!(
        html.matches("Check the connection and retry</span>")
            .count(),
        1
    );
}

fn progress_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else {
        Some(
            value
                .split(|ch: char| ch.is_whitespace() || ch == '>')
                .next()
                .unwrap(),
        )
    }
}
fn progress_class_tag<'a>(html: &'a str, class: &str) -> &'a str {
    html.split('<')
        .find(|tag| {
            progress_attribute(tag, "class")
                .is_some_and(|classes| classes.split_whitespace().any(|name| name == class))
        })
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn progress_id_tag<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|tag| progress_attribute(tag, "id") == Some(id))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}

#[test]
fn progress_content_only_helpers_stay_inline_and_absent_helpers_add_no_empty_row() {
    let html = render(|| {
        rsx! {
            ProgressIndicator { id:"content-only", show_text_as_tooltip:true, helper_text:"",
                helper_content:rsx! {span{id:"source-helper","Inspect the local transfer"}}
            }
            ProgressIndicator { id:"absent-helper", value:25.0 }
        }
    });
    let description =
        progress_attribute(progress_id_tag(&html, "content-only"), "aria-describedby").unwrap();
    assert!(
        progress_attribute(progress_id_tag(&html, description), "class")
            .unwrap()
            .contains("ix-progress-helper-content")
    );
    assert!(!html.contains("role=\"tooltip\"") && !html.contains("title="));
    assert_eq!(html.matches("Inspect the local transfer").count(), 1);
    assert_eq!(
        html.matches("class=\"ix-progress-helper-content\"").count(),
        1
    );
    assert!(!html.contains("class=\"ix-progress-helper\""));
    assert!(
        progress_attribute(progress_id_tag(&html, "absent-helper"), "aria-describedby").is_none()
    );
}

use std::{cell::Cell, collections::BTreeMap};
#[derive(Default)]
struct ProgressFixture {
    visible: Cell<bool>,
    tooltip: Cell<bool>,
    controls: Cell<bool>,
    composed: Cell<bool>,
    text: RefCell<Option<String>>,
    value: Cell<f64>,
    calls: Cell<usize>,
    helper_mounts: Cell<usize>,
    root_mounts: Cell<usize>,
    mounted: RefCell<Option<Rc<MountedData>>>,
    focuses: Rc<RefCell<Vec<String>>>,
}
impl PartialEq for ProgressFixture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn ProgressPreparation(data: Rc<ProgressFixture>) -> Element {
    use_hook(|| data.helper_mounts.set(data.helper_mounts.get() + 1));
    let mut note = use_signal(|| String::from("Prepare local files"));
    let action = data.clone();
    rsx! {
        input {id:String::from("progress-note"),value:note(),oninput:move|event|note.set(event.value())}
        button {id:String::from("progress-retry"),r#type:"button",onclick:move |_|{
            action.calls.set(action.calls.get()+1);action.value.set(action.value.get()+10.0);
        },"Retry transfer"}
    }
}
fn progress_fixture(data: Rc<ProgressFixture>) -> Element {
    let mounted = data.clone();
    let content = data.composed.get().then(|| {
        if data.controls.get() {
            rsx! {ProgressPreparation{data:data.clone()}}
        } else {
            rsx! {span{id:String::from("progress-summary"),"Seven files remain"}}
        }
    });
    rsx! {
        if data.visible.get() {
            ProgressIndicator {id:"progress-owner",tabindex:0,label:"Transfer",value:data.value.get(),helper_text:data.text.borrow().clone(),
                helper_content:content,show_text_as_tooltip:data.tooltip.get(),
                onmounted:move|event:MountedEvent|{mounted.root_mounts.set(mounted.root_mounts.get()+1);*mounted.mounted.borrow_mut()=Some(event.data());},
                "Transfer value"
            }
        }
    }
}
struct ProgressMouse;
impl dioxus::html::InteractionLocation for ProgressMouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for ProgressMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for ProgressMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for ProgressMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for ProgressMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct ProgressForm(String);
impl dioxus::html::HasFileData for ProgressForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for ProgressForm {
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
struct ProgressFocus;
impl dioxus::html::HasFocusData for ProgressFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
struct ProgressKey(Key);
impl dioxus::html::ModifiersInteraction for ProgressKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for ProgressKey {
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
#[derive(Clone)]
struct ProgressMount(Rc<RefCell<Vec<String>>>);
impl dioxus::html::RenderedElementBacking for ProgressMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        self.0.borrow_mut().push("progress-owner".into());
        Box::pin(async { Ok(()) })
    }
}
struct ProgressConverter;
macro_rules! progress_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{
        impl dioxus::html::HtmlEventConverter for ProgressConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{let _=event;progress_converter!(@data $group,event)})*
        }
    };
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(ProgressMouse)};
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(ProgressFocus)};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new($event.downcast::<ProgressKey>().unwrap().clone())};
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(ProgressForm($event.downcast::<String>().unwrap().clone()))};
    (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<ProgressMount>().unwrap().clone())};
    (@data $other:ident,$event:ident)=>{panic!("Unexpected progress test event")};
}
dioxus::html::with_html_event_groups!(progress_converter);
fn progress_targets(
    edits: &[dioxus::core::Mutation],
    targets: &mut BTreeMap<String, dioxus::core::ElementId>,
) {
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = *id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" => {
                targets.insert(value.clone(), current);
            }
            _ => {}
        }
    }
}
fn progress_setup(
    controls: bool,
) -> (
    VirtualDom,
    Rc<ProgressFixture>,
    BTreeMap<String, dioxus::core::ElementId>,
) {
    dioxus::html::set_event_converter(Box::new(ProgressConverter));
    let data = Rc::new(ProgressFixture::default());
    data.visible.set(true);
    data.composed.set(true);
    data.controls.set(controls);
    data.value.set(25.0);
    *data.text.borrow_mut() = Some("Review connection".into());
    let mut dom = VirtualDom::new_with_props(progress_fixture, data.clone());
    let mut ids = BTreeMap::new();
    progress_targets(&dom.rebuild_to_vec().edits, &mut ids);
    (dom, data, ids)
}
fn progress_refresh(dom: &mut VirtualDom, ids: &mut BTreeMap<String, dioxus::core::ElementId>) {
    dom.mark_dirty(ScopeId::APP);
    for _ in 0..6 {
        progress_targets(&dom.render_immediate_to_vec().edits, ids);
        dom.process_events();
    }
}
fn progress_event(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: impl std::any::Any,
) -> bool {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    let event = Event::new(data, true);
    dom.runtime().handle_event(name, event.clone(), id);
    event.default_action_enabled()
}

#[test]
fn progress_inline_composed_actions_update_controlled_value_and_retain_helper_scope() {
    let _guard = crate::test_event_converter_guard();
    let (mut dom, data, mut ids) = progress_setup(true);
    let html = dioxus_ssr::render(&dom);
    let root = progress_id_tag(&html, "progress-owner");
    let helper_id = progress_attribute(root, "aria-describedby")
        .unwrap()
        .to_string();
    assert_eq!(
        progress_attribute(progress_id_tag(&html, &helper_id), "class"),
        Some("ix-progress-helper-content")
    );
    assert_eq!(
        progress_attribute(
            progress_class_tag(&html, "ix-progress-track"),
            "aria-description"
        ),
        Some("Review connection")
    );
    assert!(
        progress_attribute(
            progress_class_tag(&html, "ix-progress-track"),
            "aria-describedby"
        )
        .is_none()
    );
    let mut bar_depth = 0;
    for tag in html.split('<').skip(1) {
        if progress_attribute(tag, "role") == Some("progressbar") {
            bar_depth = 1;
        } else if bar_depth > 0 && tag.starts_with("div ") {
            bar_depth += 1;
        } else if bar_depth > 0 && tag.starts_with("/div>") {
            bar_depth -= 1;
        }
        if tag.starts_with("button ") || tag.starts_with("input ") {
            assert_eq!(
                bar_depth, 0,
                "helper controls must remain outside the semantic bar"
            );
        }
    }
    progress_event(
        &dom,
        "mounted",
        ids["progress-owner"],
        ProgressMount(data.focuses.clone()),
    );
    assert_eq!(data.root_mounts.get(), 1);
    let node = data.mounted.borrow().clone().unwrap();
    dom.in_scope(ScopeId::APP, || {
        spawn(async move {
            node.set_focus(true).await.unwrap();
        });
    });
    progress_refresh(&mut dom, &mut ids);
    assert_eq!(*data.focuses.borrow(), ["progress-owner"]);
    progress_event(
        &dom,
        "input",
        ids["progress-note"],
        String::from("Retained local file note"),
    );
    assert!(progress_event(&dom, "click", ids["progress-retry"], ()));
    progress_refresh(&mut dom, &mut ids);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(data.calls.get(), 1);
    assert_eq!(data.helper_mounts.get(), 1);
    assert!(html.contains("aria-valuenow=35") || html.contains("aria-valuenow=\"35\""));
    assert_eq!(
        progress_attribute(progress_id_tag(&html, "progress-note"), "value"),
        Some("Retained local file note")
    );
    *data.text.borrow_mut() = Some("Connection recovered".into());
    progress_refresh(&mut dom, &mut ids);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(data.helper_mounts.get(), 1);
    assert_eq!(data.root_mounts.get(), 1);
    assert_eq!(
        progress_attribute(progress_id_tag(&html, "progress-owner"), "aria-describedby"),
        Some(helper_id.as_str())
    );
    assert!(html.contains("Connection recovered") && !html.contains("Review connection"));
}

#[test]
fn progress_shared_tooltip_modes_messages_and_cleanup_keep_one_description_target() {
    let _guard = crate::test_event_converter_guard();
    let (mut dom, data, mut ids) = progress_setup(false);
    let inline = dioxus_ssr::render(&dom);
    let helper_id = progress_attribute(
        progress_id_tag(&inline, "progress-owner"),
        "aria-describedby",
    )
    .unwrap()
    .to_string();
    data.tooltip.set(true);
    progress_refresh(&mut dom, &mut ids);
    let html = dioxus_ssr::render(&dom);
    let panel = progress_id_tag(&html, &helper_id);
    assert_eq!(progress_attribute(panel, "role"), Some("tooltip"));
    assert_eq!(progress_attribute(panel, "popover"), Some("manual"));
    assert_eq!(progress_attribute(panel, "data-interactive"), Some("false"));
    assert_eq!(html.matches("Seven files remain").count(), 1);
    assert_eq!(html.matches(&format!("id=\"{helper_id}\"")).count(), 1);
    assert!(progress_attribute(progress_id_tag(&html, "progress-owner"), "title").is_none());
    assert!(progress_event(&dom, "focusin", ids["progress-owner"], ()));
    assert!(progress_event(
        &dom,
        "keydown",
        ids["progress-owner"],
        ProgressKey(Key::Escape)
    ));
    progress_refresh(&mut dom, &mut ids);
    assert_eq!(
        progress_attribute(
            progress_id_tag(&dioxus_ssr::render(&dom), &helper_id),
            "data-open"
        ),
        Some("false")
    );
    *data.text.borrow_mut() = Some("Inspect network status".into());
    progress_refresh(&mut dom, &mut ids);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        progress_attribute(progress_id_tag(&html, "progress-owner"), "aria-describedby"),
        Some(helper_id.as_str())
    );
    assert!(html.contains("Inspect network status") && !html.contains("Review connection"));
    data.tooltip.set(false);
    progress_refresh(&mut dom, &mut ids);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("role=\"tooltip\""));
    assert_eq!(
        progress_attribute(progress_id_tag(&html, &helper_id), "class"),
        Some("ix-progress-helper-content")
    );
    data.composed.set(false);
    *data.text.borrow_mut() = None;
    progress_refresh(&mut dom, &mut ids);
    let html = dioxus_ssr::render(&dom);
    assert!(
        progress_attribute(progress_id_tag(&html, "progress-owner"), "aria-describedby").is_none()
    );
    assert!(!html.contains(&helper_id));
    data.visible.set(false);
    progress_refresh(&mut dom, &mut ids);
    assert!(!dioxus_ssr::render(&dom).contains("progress-owner"));
}

#[test]
fn progress_observes_geometry_only_while_its_shared_tooltip_exists() {
    use dioxus::core::{ElementId, Mutation};
    use std::collections::BTreeSet;

    // Observe only explicitly registered/selected root mutations. Template or
    // child stack operations must not attribute their listeners to this root.
    fn observation_changes(edits: &[Mutation], root: ElementId) -> BTreeSet<(String, bool)> {
        let mut current = None;
        let mut changes = BTreeSet::new();
        for edit in edits {
            match edit {
                Mutation::SetId { id } | Mutation::PushId { id } => current = Some(*id),
                Mutation::NewEventListener { name } | Mutation::RemoveEventListener { name }
                    if current == Some(root) && matches!(name.as_str(), "resize" | "visible") =>
                {
                    changes.insert((
                        name.clone(),
                        matches!(edit, Mutation::NewEventListener { .. }),
                    ));
                }
                Mutation::SetAttribute { .. }
                | Mutation::SetText { .. }
                | Mutation::NewEventListener { .. }
                | Mutation::RemoveEventListener { .. } => {}
                _ => current = None,
            }
        }
        changes
    }
    fn refresh(
        dom: &mut VirtualDom,
        ids: &mut BTreeMap<String, ElementId>,
        root: ElementId,
    ) -> BTreeSet<(String, bool)> {
        dom.mark_dirty(ScopeId::APP);
        let mut changes = BTreeSet::new();
        for _ in 0..6 {
            let mutations = dom.render_immediate_to_vec();
            changes.extend(observation_changes(&mutations.edits, root));
            progress_targets(&mutations.edits, ids);
            dom.process_events();
        }
        assert_eq!(
            ids["progress-owner"], root,
            "mode changes retain the actual root"
        );
        changes
    }
    let expected = |enabled| {
        BTreeSet::from([
            (String::from("resize"), enabled),
            (String::from("visible"), enabled),
        ])
    };
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ProgressConverter));
    let data = Rc::new(ProgressFixture::default());
    data.visible.set(true);
    let mut dom = VirtualDom::new_with_props(progress_fixture, data.clone());
    let initial = dom.rebuild_to_vec();
    let mut ids = BTreeMap::new();
    // Existing target registration is shared with the interactive regressions.
    progress_targets(&initial.edits, &mut ids);
    let root = ids["progress-owner"];
    assert!(observation_changes(&initial.edits, root).is_empty());
    progress_event(&dom, "mounted", root, ProgressMount(data.focuses.clone()));
    let mounted = data.mounted.borrow().clone().unwrap();
    assert!(refresh(&mut dom, &mut ids, root).is_empty());

    *data.text.borrow_mut() = Some("Inline transfer help".into());
    assert!(refresh(&mut dom, &mut ids, root).is_empty());
    assert!(!dioxus_ssr::render(&dom).contains("role=\"tooltip\""));

    data.tooltip.set(true);
    assert_eq!(refresh(&mut dom, &mut ids, root), expected(true));
    let html = dioxus_ssr::render(&dom);
    let helper = progress_attribute(progress_id_tag(&html, "progress-owner"), "aria-describedby")
        .unwrap()
        .to_string();
    assert_eq!(
        progress_attribute(progress_id_tag(&html, &helper), "role"),
        Some("tooltip")
    );

    data.tooltip.set(false);
    assert_eq!(refresh(&mut dom, &mut ids, root), expected(false));
    assert!(dioxus_ssr::render(&dom).contains("Inline transfer help"));
    data.tooltip.set(true);
    *data.text.borrow_mut() = Some(String::new());
    assert!(refresh(&mut dom, &mut ids, root).is_empty());
    assert!(!dioxus_ssr::render(&dom).contains("role=\"tooltip\""));

    *data.text.borrow_mut() = Some("Current transfer help".into());
    assert_eq!(refresh(&mut dom, &mut ids, root), expected(true));
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        progress_attribute(progress_id_tag(&html, "progress-owner"), "aria-describedby"),
        Some(helper.as_str())
    );
    assert_eq!(
        progress_attribute(progress_id_tag(&html, &helper), "role"),
        Some("tooltip")
    );
    assert!(html.contains("Current transfer help"));
    assert_eq!(data.root_mounts.get(), 1);
    assert!(Rc::ptr_eq(
        &mounted,
        data.mounted.borrow().as_ref().unwrap()
    ));
}

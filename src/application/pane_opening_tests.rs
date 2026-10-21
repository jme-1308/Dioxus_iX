use super::*;
use dioxus::core::{ElementId, Mutation};
use std::collections::BTreeSet;

#[derive(Clone, PartialEq)]
struct Settings {
    expanded: BTreeSet<String>,
    hidden: BTreeSet<String>,
    layout: PaneLayoutOrder,
    floating: bool,
    mobile: bool,
    autofocus: bool,
    topology: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self { expanded: BTreeSet::new(), hidden: BTreeSet::new(), layout: PaneLayoutOrder::FullVertical,
            floating: true, mobile: false, autofocus: true, topology: 0 }
    }
}
#[derive(Default)]
struct Observation {
    initial: Settings,
    settings: Cell<Option<Signal<Settings>>>,
    panes: RefCell<BTreeMap<String, PaneFocusController>>,
    focus_log: Rc<RefCell<Vec<String>>>,
}
impl PartialEq for Observation {
    fn eq(&self, other: &Self) -> bool { std::ptr::eq(self, other) }
}
#[component]
fn Field(name: String, autofocus: bool) -> Element {
    let id = format!("{name}-field");
    let mounted = crate::dropdown::use_focus_mount(id.clone());
    rsx! {input {id,autofocus,onmounted:mounted}}
}
#[component]
fn CapturedPane(data: Rc<Observation>, name: String, expanded: bool, autofocus: bool) -> Element {
    let capture = use_hook(PaneFocusCapture::default);
    use_context_provider(|| capture.clone());
    let named = name.clone();
    use_effect(move || {
        if let Some(focus) = capture.0.borrow().clone() { data.panes.borrow_mut().insert(named.clone(), focus); }
    });
    rsx! {Pane {heading:name.clone(),expanded,hide_on_collapse:true,Field {name,autofocus}}}
}
#[component]
fn Layout(data: Rc<Observation>, settings: Settings, prefix: String) -> Element {
    let pane = |side: &str| {
        let name = format!("{prefix}-{side}");
        (!settings.hidden.contains(&name)).then(|| rsx! {CapturedPane {
            data:data.clone(),name:name.clone(),expanded:settings.expanded.contains(&name),autofocus:settings.autofocus
        }})
    };
    let left = pane("left"); let right = pane("right"); let top = pane("top"); let bottom = pane("bottom");
    rsx! {PaneLayout {left,right,top,bottom,layout:settings.layout,
        variant:if settings.floating {PaneVariant::Floating}else{PaneVariant::Inline},
        if prefix == "a" && settings.topology == 2 {Layout {data,settings,prefix:"b"}}
    }}
}
fn fixture(data: Rc<Observation>) -> Element {
    let settings = use_signal(|| data.initial.clone()); data.settings.set(Some(settings));
    let current = settings();
    let _viewport = crate::application_viewport::use_application_viewport(
        Some(if current.mobile {ApplicationBreakpoint::Small}else{ApplicationBreakpoint::Large}), None,
    );
    rsx! {
        crate::Button {id:"pane-test-opener","Open panes"}
        Layout {data:data.clone(),settings:current.clone(),prefix:"a"}
        if current.topology == 1 {Layout {data,settings:current,prefix:"b"}}
    }
}
#[derive(Clone)]
struct Mount { id: String, log: Rc<RefCell<Vec<String>>> }
impl dioxus::html::RenderedElementBacking for Mount {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn set_focus(&self, focus: bool) -> std::pin::Pin<Box<dyn std::future::Future<Output=dioxus::html::MountedResult<()>>>> {
        let this = self.clone();
        Box::pin(async move { assert!(focus); this.log.borrow_mut().push(this.id); Ok(()) })
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*}) => {
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*}
    };
    (@data Mounted,$event:ident) => {MountedData::new($event.downcast::<Mount>().unwrap().clone())};
    (@data $other:ident,$event:ident) => {panic!("unexpected pane opening event")};
}
dioxus::html::with_html_event_groups!(converter);
struct Harness {
    dom: VirtualDom,
    data: Rc<Observation>,
    ids: BTreeMap<ElementId, String>,
    mounted: Vec<ElementId>,
}
impl Harness {
    fn new(initial: Settings) -> Self {
        let data = Rc::new(Observation {initial,..Default::default()});
        let mut dom = VirtualDom::new_with_props(fixture, data.clone());
        let initial = dom.rebuild_to_vec();
        let mut harness = Self {dom,data,ids:BTreeMap::new(),mounted:Vec::new()};
        harness.collect(initial); harness.settle(); harness
    }
    fn collect(&mut self, changes: dioxus::core::Mutations) {
        let mut current = None;
        for edit in changes.edits {
            match edit {
                Mutation::SetId {id} => {self.ids.remove(&id);current=Some(id);}
                Mutation::PushId {id} => current=Some(id),
                Mutation::SetAttribute {name,value:dioxus::core::AttributeValue::Text(value),..} if name == "id" => {
                    if let Some(id)=current {self.ids.insert(id,value);}
                }
                Mutation::NewEventListener {name} if name == "mounted" => {if let Some(id)=current {self.mounted.push(id);}}
                Mutation::CreateElement {..}|Mutation::CreateText {..}|Mutation::Child {..}|Mutation::Pop|Mutation::Clone
                |Mutation::AppendChildren {..}|Mutation::ReplaceWith {..}|Mutation::InsertBefore {..}|Mutation::InsertAfter {..}|Mutation::Remove => current=None,
                _=>{}
            }
        }
    }
    fn settle(&mut self) {
        for _ in 0..6 {
            let changes=self.dom.render_immediate_to_vec(); self.collect(changes);
            for target in std::mem::take(&mut self.mounted) {
                let Some(id)=self.ids.get(&target).filter(|id|id.ends_with("-field")||id.ends_with("-toggle")||id.as_str()=="pane-test-opener") else {continue};
                let value=Rc::new(dioxus::html::PlatformEventData::new(Box::new(Mount {id:id.clone(),log:self.data.focus_log.clone()}))) as Rc<dyn std::any::Any>;
                self.dom.runtime().handle_event("mounted",Event::new(value,false),target);
            }
            self.dom.process_events();
        }
    }
    fn update(&mut self, change: impl FnOnce(&mut Settings)) {
        self.dom.in_scope(ScopeId::ROOT,||self.data.settings.get().unwrap().with_mut(change)); self.settle();
    }
    fn open(&mut self, names: &[&str]) {
        self.update(|settings|settings.expanded=names.iter().map(|name|(*name).into()).collect());
    }
    fn controller(&self, name: &str) -> PaneFocusController {self.data.panes.borrow()[name].clone()}
    fn facts(&self, name: &str, active: Option<&str>, inside: bool) -> crate::overlay_runtime::ScopedFocusFacts {
        let controller=self.controller(name);
        let toggle=format!("{}-toggle",controller.root_id.strip_suffix("-root").unwrap());
        let field=format!("{name}-field");
        let candidate=|id:&str,autofocus:bool,own:bool|serde_json::from_value(serde_json::json!({
            "target":id,"tab_index":0,"autofocus":autofocus,"hosts":if own {vec![controller.root_id.clone()]}else{Vec::new()}
        })).unwrap();
        crate::overlay_runtime::ScopedFocusFacts {
            observation:crate::overlay_runtime::FocusObservation {
                candidates:vec![candidate(&toggle,false,true),candidate(&field,self.data.settings.get().unwrap().peek().autofocus,true),candidate("pane-test-opener",false,false)],
                active_id:active.unwrap_or_default().into(),active_default:active.is_none(),
            },
            active_hosts:if inside {vec![controller.root_id.clone()]}else{Vec::new()},
            eligible_focus_ids:vec![toggle,field,"pane-test-opener".into()],nested:false,
        }
    }
    fn requested(&self, name: &str, epoch: u64, active: Option<&str>, inside: bool) {
        let controller=self.controller(name); let facts=self.facts(name,active,inside);
        self.dom.in_scope(ScopeId::ROOT,||controller.observe(crate::overlay_runtime::ScopedFocusEvent::Requested {request:epoch,facts}));
    }
    fn report(&mut self, names: &[&str]) {
        for name in names {
            let epoch=self.controller(name).model.borrow().epoch;
            self.requested(name,epoch,Some("pane-test-opener"),false);
        }
        self.settle();
    }
}

#[test]
fn coopening_admits_one_visible_rank_winner_with_its_own_autofocus_or_first_fallback() {
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(Converter));
    for (layout,mobile,winner) in [(PaneLayoutOrder::FullVertical,false,"a-right"),(PaneLayoutOrder::FullHorizontal,false,"a-bottom"),(PaneLayoutOrder::FullVertical,true,"a-bottom"),(PaneLayoutOrder::FullHorizontal,true,"a-bottom")] {
        let mut h=Harness::new(Settings {layout,mobile,..Default::default()});
        h.open(&["a-left","a-right","a-top","a-bottom"]);
        h.report(&["a-bottom","a-left","a-top","a-right"]);
        assert_eq!(*h.data.focus_log.borrow(),[format!("{winner}-field")]);
    }
    let mut h=Harness::new(Settings {autofocus:false,..Default::default()});
    h.open(&["a-left","a-right","a-top","a-bottom"]);h.report(&["a-bottom","a-left","a-top","a-right"]);
    let winner=h.controller("a-right");
    assert_eq!(*h.data.focus_log.borrow(),[format!("{}-toggle",winner.root_id.strip_suffix("-root").unwrap())]);
}

#[test]
fn later_single_opening_uses_its_own_focus_and_layout_changes_alone_do_not_redirect() {
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(Converter));
    let mut h=Harness::new(Settings::default());h.open(&["a-right"]);h.report(&["a-right"]);
    h.data.focus_log.borrow_mut().clear();
    h.open(&["a-right","a-left"]);h.report(&["a-left","a-right"]);
    assert_eq!(*h.data.focus_log.borrow(),["a-left-field"]);
    h.data.focus_log.borrow_mut().clear();
    h.update(|settings|{settings.layout=PaneLayoutOrder::FullHorizontal;settings.mobile=true;});
    h.report(&["a-left","a-right"]);assert!(h.data.focus_log.borrow().is_empty());
}

#[test]
fn independent_and_nested_layouts_each_admit_only_their_own_opening_winner() {
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(Converter));
    for topology in [1,2] {
        let mut h=Harness::new(Settings {topology,..Default::default()});
        h.open(&["a-left","a-right","a-top","a-bottom","b-left","b-right","b-top","b-bottom"]);
        h.report(&["a-bottom","b-left","b-bottom","a-left","a-top","b-top","a-right","b-right"]);
        let mut focused=h.data.focus_log.borrow().clone();focused.sort();
        assert_eq!(focused,["a-right-field","b-right-field"]);
    }
}

#[test]
fn close_variant_and_drop_retire_pending_openings_without_reviving_or_stealing_outside_focus() {
    let _guard=crate::test_event_converter_guard();dioxus::html::set_event_converter(Box::new(Converter));
    let mut h=Harness::new(Settings::default());h.open(&["a-left","a-right","a-top","a-bottom"]);
    let old=h.controller("a-right").model.borrow().epoch;
    h.open(&["a-left","a-top","a-bottom"]);
    h.requested("a-right",old,Some("pane-test-opener"),false);h.report(&["a-bottom","a-top","a-left"]);
    assert_eq!(*h.data.focus_log.borrow(),["a-left-field"]);

    let mut h=Harness::new(Settings::default());h.open(&["a-right"]);
    let old=h.controller("a-right").model.borrow().epoch;
    h.update(|settings|settings.floating=false);h.requested("a-right",old,Some("pane-test-opener"),false);h.settle();
    assert!(h.data.focus_log.borrow().is_empty());

    let mut h=Harness::new(Settings::default());h.open(&["a-right"]);
    let retired=h.controller("a-right");let old=retired.model.borrow().epoch;
    let late=h.facts("a-right",Some("pane-test-opener"),false);
    h.update(|settings|{settings.hidden.insert("a-right".into());});
    h.dom.in_scope(ScopeId::ROOT,||retired.observe(crate::overlay_runtime::ScopedFocusEvent::Requested {request:old,facts:late}));h.settle();
    assert!(!retired.model.borrow().alive);assert!(h.data.focus_log.borrow().is_empty());
    h.update(|settings|settings.hidden.clear());h.report(&["a-right"]);
    assert_eq!(*h.data.focus_log.borrow(),["a-right-field"]);
    h.data.focus_log.borrow_mut().clear();h.open(&[]);
    let current=h.controller("a-right").model.borrow().epoch;
    h.requested("a-right",current,Some("outside-destination"),false);h.settle();
    assert!(h.data.focus_log.borrow().is_empty(),"closing preserves independent outside focus");
    let retained=h.controller("a-right");let log=h.data.focus_log.clone();drop(h);
    assert!(!retained.model.borrow().alive);assert!(log.borrow().is_empty());
}

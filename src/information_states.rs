//! Native recovery actions and accessible equipment reading presentation.
use crate::{Icon, Spinner, SpinnerSize};
use dioxus::prelude::*;

/// Empty state alignment and density.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyStateLayout {
    #[default]
    Large,
    Compact,
    CompactBreak,
}
impl EmptyStateLayout {
    fn class(self) -> &'static str {
        match self {
            Self::Large => "large",
            Self::Compact => "compact",
            Self::CompactBreak => "compact-break",
        }
    }
}

/// Explanation of an empty collection with an optional native recovery action.
/// Disabled or loading actions reject activation. Loading presentation is
/// controlled by the caller; no timer, background task or automatic retry runs.
#[component]
pub fn EmptyState(
    header: String,
    #[props(default)] subheader: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] action: Option<String>,
    #[props(default)] layout: EmptyStateLayout,
    #[props(default)] onaction: EventHandler<()>,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] aria_label_action: Option<String>,
    #[props(default)] action_disabled: bool,
    #[props(default)] action_loading: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let blocked = action_disabled || action_loading;
    rsx! {div{id,class:"ix-empty-state ix-empty-state-{layout.class()} {class}",
        if let Some(icon)=icon{Icon{name:icon,size:if layout==EmptyStateLayout::Large{48}else{32},label:aria_label_icon}}
        div{class:"ix-empty-state-labels",strong{"{header}"}if let Some(text)=subheader{p{"{text}"}}}
        if let Some(text)=action{button{class:"ix-empty-state-action",r#type:"button",disabled:blocked,"aria-label":aria_label_action,"aria-busy":action_loading.to_string(),
            onclick:move|_|{if !blocked{onaction.call(());}},
            if action_loading{span{"aria-hidden":"true",Spinner{size:SpinnerSize::Xs,hide_track:true}}}
            "{text}"
        }}
    }}
}

/// KPI severity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KpiState {
    #[default]
    Neutral,
    Warning,
    Alarm,
}
impl KpiState {
    fn class(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Warning => "warning",
            Self::Alarm => "alarm",
        }
    }
}

/// Compact industrial reading with caller-formatted optional label/value/unit.
/// The native focusable group and title describe the supplied reading, avoiding
/// undefined values and stray separators. An entirely empty reading is named
/// "Measurement". Override `title` or `aria_label` independently when needed.
/// Severity icon labels can be localized without changing state or glyphs.
#[component]
pub fn Kpi(
    #[props(default)] label: String,
    #[props(default)] value: String,
    #[props(default)] unit: Option<String>,
    #[props(default)] state: KpiState,
    #[props(default)] vertical: bool,
    #[props(default="alarm".to_string())] aria_label_alarm_icon: String,
    #[props(default="warning".to_string())] aria_label_warning_icon: String,
    #[props(default)] title: Option<String>,
    #[props(default)] aria_label: Option<String>,
    #[props(default = 0)] tabindex: i32,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let measurement = [value.trim(), unit.as_deref().unwrap_or_default().trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let reading = match (label.trim().is_empty(), measurement.is_empty()) {
        (true, true) => "Measurement".into(),
        (true, false) => measurement,
        (false, true) => label.trim().to_string(),
        (false, false) => format!("{}: {measurement}", label.trim()),
    };
    let title = title.unwrap_or_else(|| reading.clone());
    let aria_label = aria_label
        .filter(|label| !label.trim().is_empty())
        .unwrap_or(reading);
    rsx! {div{id,class:"ix-kpi ix-kpi-{state.class()} {class}","data-vertical":vertical,role:"group",tabindex,title,"aria-label":aria_label,
        div{class:"ix-kpi-label",
            if state!=KpiState::Neutral{Icon{name:if state==KpiState::Alarm{"error"}else{"warning"},size:16,label:if state==KpiState::Alarm{aria_label_alarm_icon}else{aria_label_warning_icon}}}
            "{label}"
        }
        div{class:"ix-kpi-measurement",strong{"{value}"}if let Some(unit)=unit{span{"{unit}"}}}
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        rc::Rc,
    };
    #[derive(Clone)]
    struct Config {
        layout: EmptyStateLayout,
        action: Option<String>,
        disabled: bool,
        loading: bool,
        label: String,
        value: String,
        unit: Option<String>,
        state: KpiState,
        vertical: bool,
        title: Option<String>,
        aria: Option<String>,
        alarm_label: String,
        warning_label: String,
        tabindex: i32,
    }
    impl Default for Config {
        fn default() -> Self {
            Self {
                layout: EmptyStateLayout::Large,
                action: Some("Load readings".into()),
                disabled: false,
                loading: false,
                label: "Motor speed".into(),
                value: "122.6".into(),
                unit: Some("rpm".into()),
                state: KpiState::Neutral,
                vertical: false,
                title: None,
                aria: None,
                alarm_label: "alarm".into(),
                warning_label: "warning".into(),
                tabindex: 0,
            }
        }
    }
    #[derive(Default)]
    struct Fixture {
        config: RefCell<Config>,
        actions: Cell<usize>,
    }
    fn fixture(data: Rc<Fixture>) -> Element {
        let config = data.config.borrow().clone();
        rsx! {
            crate::layout::EmptyState{id:"empty",class:"custom-empty",header:"No readings",subheader:"Load the local equipment readings",icon:"add",aria_label_icon:"Equipment readings unavailable",action:config.action,aria_label_action:"Load equipment readings",action_disabled:config.disabled,action_loading:config.loading,layout:config.layout,onaction:move|_|data.actions.set(data.actions.get()+1)}
            Kpi{id:"reading",class:"custom-reading",label:config.label,value:config.value,unit:config.unit,state:config.state,vertical:config.vertical,title:config.title,aria_label:config.aria,aria_label_alarm_icon:config.alarm_label,aria_label_warning_icon:config.warning_label,tabindex:config.tabindex}
        }
    }
    struct Mouse;
    impl dioxus::html::InteractionLocation for Mouse {
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
    impl dioxus::html::InteractionElementOffset for Mouse {
        fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl dioxus::html::ModifiersInteraction for Mouse {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::PointerInteraction for Mouse {
        fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
            None
        }
        fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
            Default::default()
        }
    }
    impl dioxus::html::HasMouseData for Mouse {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct Converter;
    macro_rules! converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for Converter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data { let _=event;converter!(@data $group,event) })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(Mouse) };
        (@data Mounted,$event:ident) => { MountedData::new(()) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected information-state test event") };
    }
    dioxus::html::with_html_event_groups!(converter);
    #[derive(Default)]
    struct Targets {
        attrs: BTreeMap<usize, BTreeMap<String, String>>,
        mounted: Vec<dioxus::core::ElementId>,
    }
    impl Targets {
        fn collect(&mut self, changes: dioxus::core::Mutations) {
            let mut current = dioxus::core::ElementId::ROOT;
            for edit in changes.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
                    dioxus::core::Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } => {
                        self.attrs
                            .entry(current.raw())
                            .or_default()
                            .insert(name, value);
                    }
                    dioxus::core::Mutation::NewEventListener { name } if name == "mounted" => {
                        self.mounted.push(current)
                    }
                    _ => {}
                }
            }
        }
        fn attr(&self, name: &str, value: &str) -> dioxus::core::ElementId {
            dioxus::core::ElementId::from_raw(
                self.attrs
                    .iter()
                    .find_map(|(id, attrs)| {
                        (attrs.get(name).map(String::as_str) == Some(value)).then_some(*id)
                    })
                    .unwrap_or_else(|| panic!("Missing {name}={value}")),
            )
        }
    }
    fn dispatch(
        dom: &VirtualDom,
        name: &str,
        id: dioxus::core::ElementId,
        data: Box<dyn std::any::Any>,
    ) -> bool {
        let event = Event::new(
            Rc::new(dioxus::html::PlatformEventData::new(data)) as Rc<dyn std::any::Any>,
            name != "mounted",
        );
        dom.runtime().handle_event(name, event.clone(), id);
        event.default_action_enabled()
    }
    fn settle(dom: &mut VirtualDom, targets: &mut Targets) {
        for _ in 0..4 {
            targets.collect(dom.render_immediate_to_vec());
            dom.process_events();
        }
    }
    fn refresh(dom: &mut VirtualDom, targets: &mut Targets) {
        dom.mark_dirty(ScopeId::APP);
        settle(dom, targets);
    }

    fn setup(data: Rc<Fixture>) -> (VirtualDom, Targets) {
        let mut dom = VirtualDom::new_with_props(fixture, data);
        let mut targets = Targets::default();
        targets.collect(dom.rebuild_to_vec());
        settle(&mut dom, &mut targets);
        (dom, targets)
    }
    fn tag<'a>(html: &'a str, name: &str, value: &str) -> &'a str {
        html.split('<')
            .map(|tag| tag.split('>').next().unwrap())
            .find(|tag| tag.contains(&format!("{name}=\"{value}\"")))
            .unwrap_or_else(|| panic!("Missing {name}={value}: {html}"))
    }
    fn attribute_is(tag: &str, name: &str, value: &str) -> bool {
        tag.trim_end_matches('/')
            .split_ascii_whitespace()
            .any(|attribute| {
                attribute == format!("{name}={value}") || attribute == format!("{name}=\"{value}\"")
            })
    }
    #[test]
    fn information_states_empty_native_action_honors_disabled_and_busy_with_live_layout_updates() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let (mut dom, mut targets) = setup(data.clone());
        let button = targets.attr("aria-label", "Load equipment readings");
        // The VDOM does not synthesize HTML button defaults. An unhandled
        // keydown preserves them; an explicit click requests the one action.
        let initial = dioxus_ssr::render(&dom);
        let native = tag(&initial, "aria-label", "Load equipment readings");
        assert!(native.starts_with("button "));
        assert!(native.contains("type=\"button\""));
        assert!(dispatch(&dom, "keydown", button, Box::new(())));
        assert_eq!(data.actions.get(), 0);
        assert!(dispatch(&dom, "click", button, Box::new(())));
        assert_eq!(data.actions.get(), 1);
        data.config.borrow_mut().disabled = true;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(tag(&html, "aria-label", "Load equipment readings").contains("disabled"));
        dispatch(&dom, "click", button, Box::new(()));
        assert_eq!(data.actions.get(), 1);
        data.config.borrow_mut().disabled = false;
        data.config.borrow_mut().loading = true;
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        let action = tag(&html, "aria-label", "Load equipment readings");
        assert!(action.contains("disabled"));
        assert!(attribute_is(action, "aria-busy", "true"));
        assert!(html.contains("ix-spinner"));
        assert!(html.contains("Load readings"));
        dispatch(&dom, "click", button, Box::new(()));
        assert_eq!(data.actions.get(), 1);
        {
            let mut config = data.config.borrow_mut();
            config.loading = false;
            config.layout = EmptyStateLayout::CompactBreak;
            config.action = Some("Retry readings".into());
        }
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(tag(&html, "id", "empty").contains("ix-empty-state-compact-break"));
        assert!(html.contains("Retry readings"));
        assert!(!html.contains("ix-spinner"));
        assert!(attribute_is(
            tag(&html, "aria-label", "Load equipment readings"),
            "aria-busy",
            "false"
        ));
        dispatch(&dom, "click", button, Box::new(()));
        assert_eq!(data.actions.get(), 2);
        data.config.borrow_mut().action = None;
        refresh(&mut dom, &mut targets);
        assert!(!dioxus_ssr::render(&dom).contains("ix-empty-state-action"));
    }
    #[test]
    fn information_states_kpi_derived_names_optional_values_and_localized_severity_are_native() {
        let data = Rc::new(Fixture::default());
        let (mut dom, mut targets) = setup(data.clone());
        let html = dioxus_ssr::render(&dom);
        let reading = tag(&html, "id", "reading");
        assert!(reading.contains("role=\"group\""));
        assert!(attribute_is(reading, "tabindex", "0"));
        assert!(reading.contains("title=\"Motor speed: 122.6 rpm\""));
        assert!(reading.contains("aria-label=\"Motor speed: 122.6 rpm\""));
        {
            let mut config = data.config.borrow_mut();
            config.state = KpiState::Alarm;
            config.alarm_label = "Drehzahlalarm".into();
            config.title = Some("Actual motor speed".into());
            config.aria = Some("Equipment speed".into());
        }
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        let reading = tag(&html, "id", "reading");
        assert!(reading.contains("ix-kpi-alarm"));
        assert!(reading.contains("title=\"Actual motor speed\""));
        assert!(reading.contains("aria-label=\"Equipment speed\""));
        assert!(tag(&html, "aria-label", "Drehzahlalarm").contains("role=\"img\""));
        {
            let mut config = data.config.borrow_mut();
            config.state = KpiState::Warning;
            config.warning_label = "Drehzahlwarnung".into();
            config.vertical = true;
            config.title = None;
            config.aria = None;
            config.label.clear();
            config.value = "7".into();
            config.unit = None;
        }
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(tag(&html, "id", "reading").contains("title=\"7\""));
        assert!(attribute_is(
            tag(&html, "id", "reading"),
            "data-vertical",
            "true"
        ));
        assert!(html.contains("aria-label=\"Drehzahlwarnung\""));
        assert!(!html.contains("Drehzahlalarm"));
        {
            let mut config = data.config.borrow_mut();
            config.state = KpiState::Neutral;
            config.value.clear();
        }
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(tag(&html, "id", "reading").contains("title=\"Measurement\""));
        assert!(!html.contains("undefined"));
        assert!(!html.contains("Drehzahlwarnung"));
    }
    #[test]
    fn information_states_legacy_imports_defaults_and_live_readings_preserve_workflow_callbacks() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(Converter));
        let data = Rc::new(Fixture::default());
        let (mut dom, mut targets) = setup(data.clone());
        {
            let mut config = data.config.borrow_mut();
            config.layout = EmptyStateLayout::Compact;
            config.value = "Nominal".into();
            config.unit = None;
            config.label = "Drive readiness".into();
            config.vertical = true;
            config.tabindex = -1;
        }
        refresh(&mut dom, &mut targets);
        let html = dioxus_ssr::render(&dom);
        assert!(tag(&html, "id", "empty").contains("custom-empty"));
        assert!(tag(&html, "id", "empty").contains("ix-empty-state-compact"));
        let icon = tag(&html, "aria-label", "Equipment readings unavailable");
        assert!(attribute_is(icon, "width", "32"));
        assert!(tag(&html, "id", "reading").contains("custom-reading"));
        assert!(tag(&html, "id", "reading").contains("title=\"Drive readiness: Nominal\""));
        assert!(attribute_is(tag(&html, "id", "reading"), "tabindex", "-1"));
        dispatch(
            &dom,
            "click",
            targets.attr("aria-label", "Load equipment readings"),
            Box::new(()),
        );
        assert_eq!(data.actions.get(), 1);
        let mut legacy = VirtualDom::new(
            || rsx! {crate::layout::EmptyState{header:"Empty",icon:"add"}crate::Kpi{}crate::layout::Kpi{label:"Status",value:"Ready",state:crate::layout::KpiState::Warning}},
        );
        legacy.rebuild_in_place();
        let html = dioxus_ssr::render(&legacy);
        assert!(html.contains("ix-empty-state-large"));
        let icon = html
            .split('<')
            .map(|tag| tag.split('>').next().unwrap())
            .find(|tag| tag.starts_with("svg "))
            .unwrap();
        assert!(attribute_is(icon, "width", "48"));
        assert!(html.contains("title=\"Measurement\""));
        assert!(html.contains("title=\"Status: Ready\""));
        assert!(html.contains("aria-label=\"warning\""));
        assert!(!html.contains("ix-empty-state-action"));
    }
}

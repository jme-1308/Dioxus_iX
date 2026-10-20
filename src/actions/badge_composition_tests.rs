//! Badge content/ARIA composition; native shared-tooltip interaction is verified
//! by the paired gallery workflow instead of inferred from server rendering.
use super::*;
use std::cell::Cell;
fn settle(dom: &mut VirtualDom) {
    for _ in 0..6 {
        dom.render_immediate_to_vec();
        dom.process_events();
    }
}
fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    settle(&mut dom);
    dioxus_ssr::render(&dom)
}
fn opening<'a>(html: &'a str, tag: &str, marker: &str) -> &'a str {
    html.split('<')
        .filter(|part| part.starts_with(&format!("{tag} ")))
        .map(|part| part.split('>').next().unwrap())
        .find(|part| part.contains(marker))
        .unwrap_or_else(|| panic!("missing {tag} {marker}: {html}"))
}
fn attr<'a>(opening: &'a str, name: &str) -> Option<&'a str> {
    let value = opening.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else if let Some(value) = value.strip_prefix('\'') {
        Some(value.split('\'').next().unwrap())
    } else {
        Some(
            value
                .split(|character: char| character.is_whitespace() || character == '>')
                .next()
                .unwrap(),
        )
    }
}
#[test]
fn default_counter_geometry_label_icons_and_status_layers_remain_unchanged() {
    let html = render(|| {
        rsx! {
            Badge {class:"default-count",label:"105.9"}
            Badge {class:"label-icon",badge_type:BadgeType::Label,label:" Ready ",icon:"success",aria_label_icon:"Passed inspection",outline:true,border:true,align_left:true}
            Badge {class:"status-stack",badge_type:BadgeType::StatusIcon,variant:StatusVariant::Primary,aria_label:"Information"}
            Badge {class:"dot-offset",badge_type:BadgeType::Dot,offset_x:2,offset_y:3}
        }
    });
    let default = opening(&html, "span", "default-count");
    assert_eq!(attr(default, "tabindex"), None);
    assert_eq!(attr(default, "role"), None);
    assert!(default.contains("--ix-badge-x:-10px;--ix-badge-y:-10px"));
    assert!(html.contains("class=\"ix-badge-label-text\">99+</span>"));
    assert!(html.contains("class=\"ix-badge-label-text\">Ready</span>"));
    for token in ["ix-outline", "ix-badge-border", "ix-align-start"] {
        assert!(opening(&html, "span", "label-icon").contains(token));
    }
    assert!(html.contains("Passed inspection"));
    assert!(opening(&html, "span", "status-stack").contains("ix-status-info"));
    assert_eq!(html.matches("class=\"ix-badge-status-plate\"").count(), 1);
    assert_eq!(html.matches("class=\"ix-badge-status-glyph\"").count(), 1);
    assert!(opening(&html, "span", "dot-offset").contains("--ix-badge-x:-4px;--ix-badge-y:-3px"));
    assert_eq!(html.matches("role=\"status\"").count(), 4);
    assert!(!html.contains("role=\"tooltip\""));
}
#[test]
fn standalone_help_gates_visible_content_and_promotes_names_and_descriptions_to_focusable_root() {
    let html = render(|| {
        rsx! {
            Badge {class:"custom-help",label:"3",tooltip:" Detailed alerts ",tooltip_from_label:true,aria_label:"Unreviewed alerts",aria_describedby:"existing-note"}
            Badge {class:"derived-help",badge_type:BadgeType::StatusIcon,tooltip_from_label:true,aria_label:"All equipment reviewed",tabindex:-1,role:"img"}
            Badge {class:"no-name",badge_type:BadgeType::Dot,tooltip_from_label:true}
            Badge {class:"invisible-count",label:"invalid",tooltip:"Must stay hidden"}
            Badge {class:"external-only",label:"2",aria_describedby:"external-guidance"}
            Badge {class:"empty-help",label:"6",tooltip:""}
            Badge {class:"icon-help",badge_type:BadgeType::Label,label:"Ready",icon:"success",aria_label_icon:"Inspection passed",tooltip_from_label:true}
            Badge {class:"override-icon-help",badge_type:BadgeType::Label,label:"Ready",icon:"success",aria_label_icon:"Inspection passed",aria_label:"Ready for production",tooltip_from_label:true}
        }
    });
    assert_eq!(html.matches("role=\"tooltip\"").count(), 5);
    assert!(html.contains("Detailed alerts"));
    assert!(!html.contains("Must stay hidden"));
    let custom = opening(&html, "span", "custom-help");
    assert_eq!(attr(custom, "tabindex"), Some("0"), "{custom}");
    assert_eq!(attr(custom, "role"), Some("status"));
    assert_eq!(attr(custom, "aria-label"), Some("Unreviewed alerts"));
    let described = attr(custom, "aria-describedby")
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>();
    assert_eq!(described.len(), 2);
    assert_eq!(described[0], "existing-note");
    assert_eq!(html.matches(&format!("id=\"{}\"", described[1])).count(), 1);
    let derived = opening(&html, "span", "derived-help");
    assert_eq!(attr(derived, "tabindex"), Some("-1"));
    assert_eq!(attr(derived, "role"), Some("img"));
    assert_eq!(attr(derived, "aria-label"), Some("All equipment reviewed"));
    assert_ne!(attr(custom, "id"), attr(derived, "id"));
    assert_eq!(attr(opening(&html, "span", "no-name"), "tabindex"), None);
    assert!(html.contains("aria-describedby=\"external-guidance\""));
    let indicator = opening(&html, "span", "class=\"ix-badge-indicator\"");
    assert_eq!(attr(indicator, "aria-hidden"), Some("true"));
    assert_eq!(attr(indicator, "role"), None);
    let empty_help = opening(&html, "span", "empty-help");
    assert_eq!(attr(empty_help, "tabindex"), Some("0"));
    assert_eq!(attr(empty_help, "aria-label"), Some("6"));
    let empty_panel_id = attr(empty_help, "aria-describedby").unwrap();
    let empty_panel = opening(&html, "div", &format!("id=\"{empty_panel_id}\""));
    assert_eq!(attr(empty_panel, "role"), Some("tooltip"));
    let empty_panel_body = html.split_once(&format!("<{empty_panel}>")).unwrap().1;
    let empty_help_text = empty_panel_body
        .split_once("class=\"ix-tooltip-content\">")
        .unwrap()
        .1
        .split("</div>")
        .next()
        .unwrap();
    assert_eq!(empty_help_text, "6");
    assert_eq!(
        attr(opening(&html, "span", "icon-help"), "aria-label"),
        Some("Inspection passed Ready")
    );
    assert_eq!(
        attr(opening(&html, "span", "override-icon-help"), "aria-label"),
        Some("Ready for production")
    );
    assert!(!html.contains(" title="));
}
#[test]
fn attached_descriptions_are_explicit_and_preserve_anchor_tokens_and_legacy_custom_help() {
    let html = render(|| {
        rsx! {
            span {id:"external-description","Queue instructions"}
            Badge {class:"attached-help",label:"4.9",description_id:"queue-description",description:" Four alerts need review ",tooltip:"Open the review queue",tabindex:0,aria_label:"Ignored badge name",aria_describedby:"ignored-badge-token",role:"alert",
                button {id:"queue-anchor","aria-describedby":"external-description queue-description","Open alerts"}
            }
            Badge {class:"attached-derived-only",label:"2",description_id:"fallback-description",description:" ",tooltip_from_label:true,
                button {id:"fallback-anchor","aria-describedby":"fallback-description","Other alerts"}
            }
            Badge {badge_type:BadgeType::Dot,description_id:"absent-description",button {"Unnamed dot"}}
            Badge {class:"attached-hidden-help",label:"invalid",tooltip:"Anchor help without an indicator",button {id:"hidden-indicator-anchor","Review alerts"}}
        }
    });
    let attached = opening(&html, "span", "attached-help");
    for name in [
        "tabindex",
        "role",
        "aria-label",
        "aria-describedby",
        "title",
    ] {
        assert_eq!(attr(attached, name), None);
    }
    assert_eq!(
        attr(
            opening(&html, "button", "id=\"queue-anchor\""),
            "aria-describedby"
        ),
        Some("external-description queue-description")
    );
    assert_eq!(html.matches("id=\"queue-description\"").count(), 1);
    assert!(opening(&html, "span", "id=\"queue-description\"").contains("class=\"ix-sr-only\""));
    assert!(html.contains("Four alerts need review"));
    let fallback = opening(&html, "span", "id=\"fallback-description\"");
    assert_eq!(attr(fallback, "class"), Some("ix-sr-only"));
    let fallback_text = html
        .split_once(&format!("<{fallback}>"))
        .unwrap()
        .1
        .split("</span>")
        .next()
        .unwrap();
    assert_eq!(fallback_text, "2", "{html}");
    assert!(!html.contains("id=\"absent-description\""));
    assert!(!html.contains("Ignored badge name"));
    assert!(!html.contains("ignored-badge-token"));
    assert_eq!(html.matches("role=\"tooltip\"").count(), 2);
    assert!(html.contains("Open the review queue"));
    assert!(html.contains("Anchor help without an indicator"));
    let hidden = opening(&html, "span", "attached-hidden-help");
    for name in ["tabindex", "role", "aria-label"] {
        assert_eq!(attr(hidden, name), None);
    }
    assert!(!html.contains(" title="));
}
#[derive(Default)]
struct Capture {
    mode: Cell<Option<Signal<u8>>>,
    mounts: Cell<usize>,
}
impl PartialEq for Capture {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn PrivateBadgeDraft(capture: Rc<Capture>) -> Element {
    use_hook(|| capture.mounts.set(capture.mounts.get() + 1));
    let mut draft = use_signal(|| String::from("Initial note"));
    let id = String::from("badge-private-draft");
    rsx! {input{id,value:draft(),"aria-describedby":"existing-description",oninput:move|event|draft.set(event.value())}}
}
fn retained(capture: Rc<Capture>) -> Element {
    let mode = use_signal(|| 0u8);
    capture.mode.set(Some(mode));
    rsx! {Badge {class:"retained-badge",label:format!("{}",100-mode()),badge_type:if mode()==3{BadgeType::Dot}else{BadgeType::Counter},variant:if mode()==1{StatusVariant::Warning}else{StatusVariant::Primary},outline:mode()==2,
        tooltip:if mode()==1||mode()==3{Some(format!("Review guidance {}",mode()))}else{None},tooltip_from_label:true,
        description_id:format!("badge-description-{}",mode()),description:"Review queue description",
        PrivateBadgeDraft{capture:capture.clone()}
    }}
}
struct DraftForm(String);
impl dioxus::html::HasFileData for DraftForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for DraftForm {
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
struct DraftConverter;
macro_rules! draft_converter{
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* })=>{
        impl dioxus::html::HtmlEventConverter for DraftConverter{
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data{let _=event;draft_converter!(@data $group,event)})*
        }
    };
    (@data Form,$event:ident)=>{dioxus::html::FormData::new(DraftForm($event.downcast::<String>().unwrap().clone()))};
    (@data $other:ident,$event:ident)=>{panic!("unexpected badge test event")};
}
dioxus::html::with_html_event_groups!(draft_converter);
fn input_id(changes: dioxus::core::Mutations) -> dioxus::core::ElementId {
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in changes.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" && value == "badge-private-draft" => return current,
            _ => {}
        }
    }
    panic!("dynamic native input ID must be mounted")
}
#[test]
fn private_input_and_stable_tooltip_identity_survive_count_style_help_and_description_changes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(DraftConverter));
    let capture = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(retained, capture.clone());
    let input = input_id(dom.rebuild_to_vec());
    let initial = dioxus_ssr::render(&dom);
    let root = attr(opening(&initial, "span", "retained-badge"), "id")
        .unwrap()
        .to_string();
    let event = Rc::new(dioxus::html::PlatformEventData::new(Box::new(
        String::from("Retained alert note"),
    ))) as Rc<dyn std::any::Any>;
    dom.runtime()
        .handle_event("input", Event::new(event, true), input);
    settle(&mut dom);
    let edited = dioxus_ssr::render(&dom);
    assert_eq!(
        attr(
            opening(&edited, "input", "id=\"badge-private-draft\""),
            "value"
        ),
        Some("Retained alert note"),
        "actual event must edit before prop changes: {edited}"
    );
    let mut panel_id = None;
    for mode in [1, 2, 3, 0, 1] {
        dom.in_scope(ScopeId::APP, || capture.mode.get().unwrap().set(mode));
        settle(&mut dom);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(
            attr(
                opening(&html, "input", "id=\"badge-private-draft\""),
                "value"
            ),
            Some("Retained alert note")
        );
        assert_eq!(
            attr(opening(&html, "span", "retained-badge"), "id"),
            Some(root.as_str())
        );
        assert_eq!(
            html.matches(&format!("id=\"badge-description-{mode}\""))
                .count(),
            1
        );
        assert_eq!(
            attr(
                opening(&html, "input", "id=\"badge-private-draft\""),
                "aria-describedby"
            ),
            Some("existing-description")
        );
        assert_eq!(
            html.matches("role=\"tooltip\"").count(),
            usize::from(mode == 1 || mode == 3)
        );
        if mode == 1 || mode == 3 {
            let current = attr(opening(&html, "div", "role=\"tooltip\""), "id").unwrap();
            if let Some(previous) = panel_id.as_ref() {
                assert_eq!(current, previous);
            } else {
                panel_id = Some(current.to_string());
            }
            assert_ne!(current, format!("badge-description-{mode}"));
        }
        assert_eq!(capture.mounts.get(), 1, "private child must remain mounted");
    }
}

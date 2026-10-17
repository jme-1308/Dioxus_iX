//! Individually reviewable Rust ports of the upstream HTML layout examples.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const SCENARIOS: &[&str] = &[
    "action-card",
    "blind-header-actions",
    "blind-variants",
    "blind",
    "breadcrumb-next-items",
    "breadcrumb-truncate",
    "breadcrumb",
    "card-list",
    "card",
    "content-header-no-back",
    "content-header-text-overflow",
    "content-header-with-slot",
    "content-header",
    "content",
    "empty-state-compact-break",
    "empty-state-compact",
    "empty-state",
    "event-list-compact",
    "event-list-custom-item-height-in-number",
    "event-list-custom-item-height",
    "event-list-filled",
    "event-list-selected",
    "event-list",
    "group-context-menu",
    "group-custom-entry",
    "group-header-suppressed",
    "group",
    "html-table-striped",
    "html-table",
    "key-value-list-striped",
    "key-value-list-with-custom-value",
    "key-value-list-with-icon",
    "key-value-list",
    "key-value-with-custom-value",
    "key-value-with-icon",
    "key-value-with-label-left",
    "key-value",
    "kpi",
    "pagination-advanced",
    "pagination",
    "push-card",
    "tabs-overflow",
    "tabs-rounded",
    "tabs",
    "tile",
    "vertical-tabs-with-avatar",
    "vertical-tabs",
    "workflow-vertical",
    "workflow",
];
const LOREM: &str = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.";

#[component]
pub fn LayoutScenario(name: String) -> Element {
    let content = match name.as_str() {
        "action-card" => rsx! {ActionCardScenario{}},
        "card" => rsx! {CardScenario{}},
        "card-list" => rsx! {CardListScenario{}},
        "push-card" => rsx! {PushCardScenario{}},
        "tile" => rsx! {TileScenario{}},
        "blind" | "blind-variants" | "blind-header-actions" => rsx! {BlindScenario{name}},
        "breadcrumb" | "breadcrumb-truncate" | "breadcrumb-next-items" => {
            rsx! {BreadcrumbScenario{name}}
        }
        "content"
        | "content-header"
        | "content-header-no-back"
        | "content-header-text-overflow"
        | "content-header-with-slot" => rsx! {ContentScenario{name}},
        "empty-state" | "empty-state-compact" | "empty-state-compact-break" => {
            rsx! {EmptyScenario{name}}
        }
        "event-list"
        | "event-list-compact"
        | "event-list-filled"
        | "event-list-selected"
        | "event-list-custom-item-height"
        | "event-list-custom-item-height-in-number" => rsx! {EventScenario{name}},
        "group" | "group-custom-entry" | "group-context-menu" | "group-header-suppressed" => {
            rsx! {GroupScenario{name}}
        }
        "key-value"
        | "key-value-with-icon"
        | "key-value-with-label-left"
        | "key-value-with-custom-value"
        | "key-value-list"
        | "key-value-list-with-icon"
        | "key-value-list-with-custom-value"
        | "key-value-list-striped" => rsx! {KeyValueScenario{name}},
        "kpi" => rsx! {KpiScenario{}},
        "pagination" | "pagination-advanced" => {
            rsx! {PaginationScenario{advanced:name=="pagination-advanced"}}
        }
        "tabs" | "tabs-rounded" | "tabs-overflow" => rsx! {TabsScenario{name}},
        "vertical-tabs" | "vertical-tabs-with-avatar" => {
            rsx! {VerticalScenario{avatar:name=="vertical-tabs-with-avatar"}}
        }
        "workflow" | "workflow-vertical" => {
            rsx! {WorkflowScenario{vertical:name=="workflow-vertical"}}
        }
        "html-table" | "html-table-striped" => {
            rsx! {EquipmentTable{striped:name=="html-table-striped"}}
        }
        _ => rsx! {p{role:"status","No layout scenario named {name}."}},
    };
    rsx! {style{{STYLES}}div{class:"layout-scenario",{content}}}
}
#[component]
fn ActionCardScenario() -> Element {
    let mut scans = use_signal(|| 0usize);
    rsx! {ActionCard{icon:"refresh",heading:"Scan for new devices",subheading:"Secondary text",variant:CardVariant::Outline,onclick:move |_|scans+=1}if scans()>0{p{class:"lx-feedback",role:"status","Device scan requested ({scans})."}}}
}
#[component]
fn CardScenario() -> Element {
    let mut clicks = use_signal(|| 0usize);
    rsx! {Card{variant:CardVariant::Outline,class:"lx-card-example",aria_label:"Number of components",onselect:move |_|clicks+=1,
        CardContent{Icon{name:"capacity"}strong{class:"lx-card-heading","Number of components"}p{class:"lx-card-frameworks","Vanilla JavaScript"br{}"Angular"br{}"Blazor"br{}"React"br{}"Vue.js"}strong{class:"lx-card-number","123"}}
        svg{class:"lx-card-chart",view_box:"0 0 318 86",width:"318",height:"86","aria-hidden":"true",defs{linearGradient{id:"lx-card-fill",x1:"0",y1:"0",x2:"0",y2:"1",stop{offset:"0",stop_color:"var(--ix-color-primary)",stop_opacity:"0.4"}stop{offset:"1",stop_color:"var(--ix-color-primary)",stop_opacity:"0"}}}path{d:"M0 86 L35.4501 66.5689 C38.0359 65.1516 40.982 64.5279 43.9203 64.7756 L73.6615 67.2835 C76.0567 67.4855 78.4653 67.1089 80.6845 66.1855 L105.61 55.8142 C107.436 55.0543 109.394 54.6631 111.372 54.6631 H140.627 C142.105 54.6631 143.575 54.4446 144.989 54.0147 L174.785 44.9575 C176.473 44.4444 178.238 44.2331 180 44.3334 L211.312 46.1153 C212.618 46.1896 213.928 46.0928 215.209 45.8274 L241.928 40.2895 C243.291 40.007 244.608 39.536 245.842 38.8902 L262.578 30.127 C264.06 29.3508 265.662 28.8282 267.317 28.5806 L318 21 V86 Z",fill:"url(#lx-card-fill)",stroke:"var(--ix-color-primary)",stroke_width:"2"}}
    }if clicks()>0{p{class:"lx-feedback",role:"status","Card activated ({clicks})."}}}
}
#[component]
fn PushCardScenario() -> Element {
    rsx! {PushCard{icon:"bulb",notification:"99",heading:"Heading content",subheading:"Subheading",variant:CardVariant::Outline,
        table{class:"ix-table",thead{tr{th{scope:"col","#"}th{scope:"col","First"}th{scope:"col","Last"}th{scope:"col","Handle"}}}tbody{tr{th{scope:"row","1"}td{"Mark"}td{"Otto"}td{"@mdo"}}tr{th{scope:"row","2"}td{"Jacob"}td{"Thornton"}td{"@fat"}}tr{th{scope:"row","3"}td{colspan:"2","Larry the Bird"}td{"@twitter"}}}}
    }}
}
#[component]
fn CardListScenario() -> Element {
    rsx! {div{class:"lx-stack",
        CardList{label:"Stack Layout",show_all_count:12,list_style:CardListStyle::Stack,for (icon,count,variant) in [("rocket","3",CardVariant::Filled),("bulb","1",CardVariant::Warning),("rocket","3",CardVariant::Success)]{PushCard{icon,notification:count,heading:"Heading content",subheading:"Subheading",variant,children:VNode::empty()}}}
        CardList{label:"Flow Layout",show_all_count:12,list_style:CardListStyle::Scroll,for index in 0..5{PushCard{icon:if index==1{"bulb"}else{"rocket"},notification:if index==1{"1"}else{"3"},heading:"Heading content",subheading:"Subheading",variant:if index==1{CardVariant::Warning}else{CardVariant::Filled},children:VNode::empty()}}}
    }}
}
#[component]
fn TileScenario() -> Element {
    let mut event = use_signal(String::new);
    rsx! {div{class:"lx-row",Tile{size:TileSize::Small,"92.8 °C"}Tile{size:TileSize::Medium,header:"Tile header","92.8 °C"}Tile{size:TileSize::Big,header:"Tile header",subheader:"Temperature",header_actions:rsx!{IconButton{icon:"context-menu",label:"Tile actions",variant:ButtonVariant::Tertiary,onclick:move |_|event.set("Tile actions requested".into())}},footer:rsx!{Button{variant:ButtonVariant::Tertiary,icon:"chevron-right-small",onclick:move |_|event.set("Details requested".into()),"Details"}},div{style:"text-align:right","92.8 °C"}}}if !event().is_empty(){p{class:"lx-feedback",role:"status","{event}"}}}
}
#[component]
fn BlindScenario(name: String) -> Element {
    let mut action = use_signal(String::new);
    if name == "blind-variants" {
        return rsx! {div{class:"lx-stack",for (variant,label,copy) in [(CardVariant::Filled,"Example","Filled content"),(CardVariant::Outline,"Outline","Outline content"),(CardVariant::Alarm,"Alarm","Alarm content"),(CardVariant::Critical,"Critical","Critical content"),(CardVariant::Warning,"Warning","Warning content"),(CardVariant::Info,"Info","Info content"),(CardVariant::Success,"Success","Success content"),(CardVariant::Neutral,"Neutral","Neutral content")]{Blind{variant,icon:"bulb",label,sublabel:"sublabel",div{"{copy}"}}}}};
    }
    rsx! {Blind{label:"Example",icon:if name=="blind-header-actions"{Some("info".into())}else{None},header_actions:if name=="blind-header-actions"{Some(rsx!{Dropdown{label:"",icon:"context-menu",aria_label:"Header actions",variant:ButtonVariant::Tertiary,class:"lx-icon-dropdown",DropdownItem{label:"Rename...",icon:"rename",onclick:move |_|action.set("Rename...".into())}DropdownItem{label:"Delete",icon:"trashcan",onclick:move |_|action.set("Delete".into())}}})}else{None},p{class:"lx-lorem","{LOREM}"}}if !action().is_empty(){p{class:"lx-feedback",role:"status","{action} selected"}}}
}
#[component]
fn BreadcrumbScenario(name: String) -> Element {
    let mut target = use_signal(String::new);
    let count = if name == "breadcrumb-truncate" { 5 } else { 3 };
    let items = (1..=count)
        .map(|index| BreadcrumbEntry::new(format!("item-{index}"), format!("Item {index}")))
        .collect();
    rsx! {Breadcrumb{items,visible_item_count:if name=="breadcrumb-truncate"{3}else{9},next_items:if name=="breadcrumb-next-items"{vec![BreadcrumbEntry::new("next-item-1","Next Item 1")]}else{vec![]},onitem:move |item:BreadcrumbEntry|target.set(item.label),onnext:move |item:BreadcrumbEntry|target.set(item.label)}if !target().is_empty(){p{class:"lx-feedback",role:"status","{target} selected"}}}
}
#[component]
fn ContentScenario(name: String) -> Element {
    let mut event = use_signal(String::new);
    if name == "content" {
        return rsx! {Content{ContentHeader{title:"My Content Page"}p{class:"lx-lorem","{LOREM}"}}};
    }
    let ellipsis = name == "content-header-text-overflow";
    let no_back = name == "content-header-no-back";
    rsx! {ContentHeader{title:if ellipsis{"Content title that remains compact when horizontal space is limited"}else{"Content title"},subtitle:if ellipsis{"Supporting context is truncated independently when space is limited"}else{"Subtitle"},ellipsis,show_back:!no_back,onback:move |_|event.set("Back requested".into()),header_content:if name=="content-header-with-slot"{Some(rsx!{Pill{icon:"info","Label"}})}else{None},
        for (index,icon) in ["pen","trashcan","context-menu"].into_iter().enumerate(){if no_back{IconButton{icon,label:format!("Button{}",index+1),variant:ButtonVariant::Tertiary,onclick:move |_|event.set(format!("Button{}",index+1))}}else{Button{variant:ButtonVariant::Tertiary,onclick:move |_|event.set(format!("Button{}",index+1)),"Button{index+1}"}}}
    }if !event().is_empty(){p{class:"lx-feedback",role:"status","{event}"}}}
}
#[component]
fn EmptyScenario(name: String) -> Element {
    let mut requests = use_signal(|| 0usize);
    let layout = match name.as_str() {
        "empty-state-compact" => EmptyStateLayout::Compact,
        "empty-state-compact-break" => EmptyStateLayout::CompactBreak,
        _ => EmptyStateLayout::Large,
    };
    rsx! {EmptyState{layout,header:"No elements available",subheader:"Create an element first",icon:"add",action:"Create element",onaction:move |_|requests+=1}if requests()>0{p{class:"lx-feedback",role:"status","Create element requested ({requests})."}}}
}
#[component]
fn EventScenario(name: String) -> Element {
    let mut selected = use_signal(|| {
        if name == "event-list-selected" {
            Some(1usize)
        } else {
            None
        }
    });
    let custom = name.starts_with("event-list-custom-item-height");
    let count = if custom { 2 } else { 4 };
    let item_height = if name == "event-list-custom-item-height" {
        EventItemHeight::Large
    } else if name == "event-list-custom-item-height-in-number" {
        EventItemHeight::Pixels(60)
    } else {
        EventItemHeight::Small
    };
    rsx! {EventList{compact:name=="event-list-compact",item_height,for index in 0..count{EventListItem{status:if name=="event-list-compact"||name=="event-list-selected"{CardVariant::Primary}else{match index{2=>CardVariant::Alarm,3=>CardVariant::Success,_=>CardVariant::Primary}},filled:name=="event-list-filled",selected:selected()==Some(index),onselect:move |_|selected.set(Some(index)),if custom{"Test {index+1}"}else{"Text {index+1}"}}}}}
}
#[component]
fn GroupScenario(name: String) -> Element {
    let mut item = use_signal(|| None::<usize>);
    let mut group = use_signal(|| false);
    let mut action = use_signal(String::new);
    rsx! {Group{header:"Header text",subheader:"Subheader text",suppress_header_selection:name=="group-header-suppressed",selected:group(),onselect:move |next|{group.set(next);item.set(None);},header_actions:if name=="group-context-menu"{Some(rsx!{Dropdown{label:"",icon:"context-menu",aria_label:"Group actions",variant:ButtonVariant::Tertiary,class:"lx-icon-dropdown",DropdownItem{label:"Item 1",onclick:move |_|action.set("Item 1".into())}DropdownItem{label:"Item 2",onclick:move |_|action.set("Item 2".into())}}})}else{None},
        for index in 0..3{if name=="group-custom-entry"&&index==2{GroupItem{Button{onclick:move |_|action.set("Test".into()),"Test"}}}else{GroupItem{text:format!("Example text {}",index+1),selected:item()==Some(index),onselect:move |_|{let next=if item()==Some(index){None}else{Some(index)};item.set(next);}}}}
    }if !action().is_empty(){p{class:"lx-feedback",role:"status","{action} selected"}}}
}
#[component]
fn KeyValueScenario(name: String) -> Element {
    let mut values = use_signal(|| vec![String::new(); 3]);
    let list = name.starts_with("key-value-list");
    let left = list || name == "key-value-with-label-left";
    let custom = name.ends_with("with-custom-value");
    let icon = name.ends_with("with-icon");
    let entries = rsx! {for index in 0..if list{3}else{1}{KeyValue{label:"Label",label_position:if left{KeyValueLabelPosition::Left}else{KeyValueLabelPosition::Top},icon:if icon{Some("location".into())}else{None},value:if custom{None}else{Some("Value".into())},if custom{input{class:"lx-form-control",r#type:"text",placeholder:"Enter text here","aria-label":format!("Custom value {}",index+1),value:values()[index].clone(),oninput:move |event|values.write()[index]=event.value()}}}}};
    if list {
        rsx! {KeyValueList{striped:name=="key-value-list-striped",{entries}}}
    } else {
        entries
    }
}
#[component]
fn KpiScenario() -> Element {
    rsx! {div{class:"lx-kpis",Kpi{label:"Motor speed",value:"Nominal"}Kpi{label:"Motor speed",value:"122.6",unit:"rpm"}Kpi{label:"Motor speed",value:"122.6",state:KpiState::Alarm}Kpi{label:"Motor speed",value:"122.6",state:KpiState::Warning}Kpi{label:"Motor speed",value:"Nominal",vertical:true}Kpi{label:"Motor speed",value:"122.6",unit:"rpm",state:KpiState::Alarm,vertical:true}}}
}
#[component]
fn PaginationScenario(advanced: bool) -> Element {
    let mut page = use_signal(|| 0usize);
    let mut count = use_signal(|| 15usize);
    rsx! {Pagination{count:100,advanced,selected_page:page(),item_count:count(),onpage:move |next|page.set(next),onitemcount:move |next|{count.set(next);page.set(0);}}}
}
#[component]
fn TabsScenario(name: String) -> Element {
    let rounded = name == "tabs-rounded";
    let overflow = name == "tabs-overflow";
    let items = if rounded {
        [
            ("success", "Success", "success", None, false),
            ("tree", "Tree", "tree", Some(200), false),
            ("maintenance", "Maintenance", "maintenance", None, false),
            ("sound", "Sound", "sound-loud", Some(24), true),
            ("hierarchy", "Hierarchy", "hierarchy", None, false),
            (
                "calendar-settings",
                "Calendar settings",
                "calendar-settings",
                None,
                true,
            ),
        ]
        .into_iter()
        .map(|(key, label, icon, counter, disabled)| TabItem {
            icon: Some(icon.into()),
            counter,
            disabled,
            ..TabItem::new(key, label)
        })
        .collect::<Vec<_>>()
    } else if overflow {
        [
            "Overview",
            "Analytics",
            "Events",
            "Automation",
            "Data Sources",
            "Notifications",
            "History",
            "Settings",
        ]
        .into_iter()
        .map(|label| TabItem::new(label.to_lowercase().replace(' ', "-"), label))
        .collect()
    } else {
        (1..=3)
            .map(|index| TabItem::new(format!("tab-{index}"), format!("Tab {index}")))
            .collect()
    };
    let initial = items
        .first()
        .map(|item| item.key.clone())
        .unwrap_or_default();
    rsx! {div{class:"lx-tabs-example",style:if overflow{Some("max-width:340px")}else{None},
        TabSet{items:items.clone(),default_active_key:initial,rounded,
            if !rounded { for item in items { TabPanel{key:"{item.key}",tab_key:item.key,class:"lx-tab-content","Content {item.label}"} } }
        }
    }}
}
#[component]
fn VerticalScenario(avatar: bool) -> Element {
    let mut selected = use_signal(|| 0usize);
    let mut action = use_signal(String::new);
    rsx! {div{class:"lx-vertical-menu",Menu{start_expanded:true,pinned:true,avatar:if avatar{Some(rsx!{MenuAvatar{top:"john.doe@company.com",bottom:"Administrator",initials:"JD",MenuAvatarItem{icon:"user-profile",label:"User profile...",onclick:move |_|action.set("User profile...".into())}}})}else{None},
        MenuItem{home:true,icon:"home",label:"Home",active:selected()==0,close_on_select:false,onclick:move |_|selected.set(0)}MenuItem{icon:"globe",label:"Normal tab",active:selected()==1,close_on_select:false,onclick:move |_|selected.set(1)}MenuItem{icon:"star",label:"Disabled tab",disabled:true}MenuItem{icon:"star",label:"With other icon",active:selected()==3,close_on_select:false,onclick:move |_|selected.set(3)}
    }}if !action().is_empty(){p{class:"lx-feedback",role:"status","{action} requested"}}}
}
#[component]
fn WorkflowScenario(vertical: bool) -> Element {
    rsx! {WorkflowSteps{vertical,for (index,status) in [WorkflowStatus::Done,WorkflowStatus::Success,WorkflowStatus::Open,WorkflowStatus::Warning,WorkflowStatus::Error].into_iter().enumerate(){WorkflowStep{label:format!("Step {}",index+1),status}}WorkflowStep{label:"Step 6",disabled:true}}}
}
#[component]
fn EquipmentTable(striped: bool) -> Element {
    rsx! {table{class:if striped{"ix-table ix-table-striped"}else{"ix-table"},thead{tr{th{scope:"col","ID"}th{scope:"col","Type"}th{scope:"col","Status"}th{scope:"col","HW Version"}}}tbody{for (id,kind,status,version) in [("1","Equipment","Normal","2.0"),("2","Positioner","Maintenance","1.0"),("3","Pressure sensor","Unknown","N/A")]{tr{th{scope:"row","{id}"}td{"{kind}"}td{"{status}"}td{"{version}"}}}}}}
}

pub const STYLES: &str = r#"
.layout-scenario{width:100%;min-width:0}.lx-stack{display:flex;flex-direction:column;gap:24px}.lx-row{display:flex;flex-wrap:wrap;align-items:flex-start;gap:16px}.lx-feedback{font-size:14px;color:var(--ix-color-text-secondary);margin:12px 0 0}.lx-lorem{margin:0;max-width:75ch;line-height:24px}.lx-card-example{width:318px;height:252px}.lx-card-heading{display:block;margin-top:8px}.lx-card-frameworks{font-size:14px;line-height:20px}.lx-card-number{position:absolute;right:16px;bottom:16px;font-size:40px;line-height:48px;font-weight:600;z-index:1}.lx-card-chart{position:absolute;bottom:0;left:0;pointer-events:none}.lx-icon-dropdown>.ix-dropdown-trigger{font-size:0;width:32px;min-width:32px;padding:4px;gap:0}.lx-icon-dropdown>.ix-dropdown-trigger>.ix-icon:last-child{display:none}.lx-form-control{width:100%;min-width:0;height:32px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface);color:var(--ix-color-text);padding:4px 8px;font:inherit}.lx-form-control::placeholder{color:var(--ix-color-text-secondary)}.lx-form-control:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}.lx-kpis{display:flex;flex-direction:column;gap:16px;max-width:320px}.lx-tab-content{padding-top:16px}.lx-vertical-menu{position:relative;min-height:440px}.lx-vertical-menu>.ix-menu{min-height:440px;position:relative;width:100%;max-width:none}.lx-vertical-menu .ix-menu-toggle{display:none}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scenario_names_are_unique_and_dispatch_is_exhaustive() {
        let names: std::collections::BTreeSet<_> = SCENARIOS.iter().copied().collect();
        assert_eq!(names.len(), SCENARIOS.len());
        for name in SCENARIOS {
            let mut dom = VirtualDom::new_with_props(
                LayoutScenario,
                LayoutScenarioProps {
                    name: (*name).into(),
                },
            );
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            assert!(!html.contains("No layout scenario named"), "{name}");
        }
    }
    #[test]
    fn overflow_fixture_keeps_all_eight_keyed_panels_and_defers_menu_to_measurement() {
        fn fixture() -> Element {
            rsx! { TabsScenario { name: "tabs-overflow" } }
        }
        let mut dom = VirtualDom::new(fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("role=\"tab\"").count(), 8, "{html}");
        assert_eq!(html.matches("role=\"tabpanel\"").count(), 8);
        assert_eq!(html.matches("hidden=true").count(), 7);
        assert!(html.contains("Content Overview"));
        assert!(html.contains("Content Settings"));
        assert!(
            !html.contains("Show all tabs"),
            "SSR cannot know available client width"
        );
        let controls: Vec<_> = html
            .split("aria-controls=\"")
            .skip(1)
            .filter_map(|tail| tail.split('\"').next())
            .collect();
        assert_eq!(controls.len(), 8);
        for panel in controls {
            assert!(
                html.contains(&format!("id=\"{panel}\"")),
                "missing keyed panel {panel}"
            );
        }
    }
    #[test]
    fn selected_event_and_compact_layouts_keep_their_source_states() {
        fn fixture() -> Element {
            rsx! {LayoutScenario{name:"event-list-selected"}}
        }
        let mut dom = VirtualDom::new(fixture);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("aria-pressed=true").count(), 1);
        assert!(html.contains("Text 2"));
    }
}

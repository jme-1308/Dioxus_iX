//! Development-only shared ExpandingSearch fixture; all consumer intent is Rust.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
const CASES: &str = r#"[{"key":"search-s0-collapsed","id":"search-search-s0-collapsed","scopeIndex":0,"pose":"collapsed","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s0-pristine","id":"search-search-s0-pristine","scopeIndex":0,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s0-edited","id":"search-search-s0-edited","scopeIndex":0,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s1-collapsed","id":"search-search-s1-collapsed","scopeIndex":1,"pose":"collapsed","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s1-pristine","id":"search-search-s1-pristine","scopeIndex":1,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s1-edited","id":"search-search-s1-edited","scopeIndex":1,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s2-collapsed","id":"search-search-s2-collapsed","scopeIndex":2,"pose":"collapsed","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s2-pristine","id":"search-search-s2-pristine","scopeIndex":2,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s2-edited","id":"search-search-s2-edited","scopeIndex":2,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s3-collapsed","id":"search-search-s3-collapsed","scopeIndex":3,"pose":"collapsed","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s3-pristine","id":"search-search-s3-pristine","scopeIndex":3,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-s3-edited","id":"search-search-s3-edited","scopeIndex":3,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-primary","id":"search-search-variant-primary","scopeIndex":0,"pose":"collapsed","variant":"primary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-secondary","id":"search-search-variant-secondary","scopeIndex":1,"pose":"collapsed","variant":"secondary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-tertiary","id":"search-search-variant-tertiary","scopeIndex":2,"pose":"collapsed","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-danger-primary","id":"search-search-variant-danger-primary","scopeIndex":3,"pose":"collapsed","variant":"danger-primary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-danger-secondary","id":"search-search-variant-danger-secondary","scopeIndex":0,"pose":"collapsed","variant":"danger-secondary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-danger-tertiary","id":"search-search-variant-danger-tertiary","scopeIndex":1,"pose":"collapsed","variant":"danger-tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-subtle-primary","id":"search-search-variant-subtle-primary","scopeIndex":2,"pose":"collapsed","variant":"subtle-primary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-subtle-secondary","id":"search-search-variant-subtle-secondary","scopeIndex":3,"pose":"collapsed","variant":"subtle-secondary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-variant-subtle-tertiary","id":"search-search-variant-subtle-tertiary","scopeIndex":0,"pose":"collapsed","variant":"subtle-tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s0-pristine","id":"search-search-full-s0-pristine","scopeIndex":0,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s0-edited","id":"search-search-full-s0-edited","scopeIndex":0,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s1-pristine","id":"search-search-full-s1-pristine","scopeIndex":1,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s1-edited","id":"search-search-full-s1-edited","scopeIndex":1,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s2-pristine","id":"search-search-full-s2-pristine","scopeIndex":2,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s2-edited","id":"search-search-full-s2-edited","scopeIndex":2,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s3-pristine","id":"search-search-full-s3-pristine","scopeIndex":3,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-full-s3-edited","id":"search-search-full-s3-edited","scopeIndex":3,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":true,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query"},{"key":"search-custom-icon","id":"search-search-custom-icon","scopeIndex":1,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":"add","ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query","extra":"custom-icon"},{"key":"search-custom-labels","id":"search-search-custom-labels","scopeIndex":2,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":"Search catalog","ariaLabelClearIconButton":"Clear catalog","ariaLabelSearchInput":"Catalog query","parentWidth":360,"alignment":"flex-start","editedValue":"Query","extra":"custom-labels"},{"key":"search-initial-value","id":"search-search-initial-value","scopeIndex":3,"pose":"collapsed","variant":"tertiary","placeholder":"Enter text here","value":"Initial value","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query","extra":"initial-value"},{"key":"search-override","id":"search-search-override","scopeIndex":0,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query","extra":"override","override":true},{"key":"search-narrow-start","id":"search-search-narrow-start","scopeIndex":1,"pose":"pristine","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":200,"alignment":"flex-start","editedValue":"Query","extra":"narrow-start"},{"key":"search-narrow-end","id":"search-search-narrow-end","scopeIndex":2,"pose":"edited","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":200,"alignment":"flex-end","editedValue":"Query","extra":"narrow-end"},{"key":"search-live-props","id":"search-search-live-props","scopeIndex":3,"pose":"edited","variant":"tertiary","placeholder":"Find items","value":"","fullWidth":false,"icon":null,"ariaLabelSearchIconButton":null,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start","editedValue":"Query","extra":"live-props","revised":{"icon":"add","placeholder":"Updated query","fullWidth":true,"value":"External revision","editedValue":"Updated Query","ariaLabelSearchIconButton":"Search updated","ariaLabelClearIconButton":"Clear updated","ariaLabelSearchInput":"Updated input"}}]"#;
fn cases() -> Vec<Value> {
    serde_json::from_str(CASES).expect("authored cases")
}
fn text(case: &Value, key: &str) -> String {
    case[key].as_str().unwrap_or_default().to_string()
}
fn variant(name: &str) -> ButtonVariant {
    match name {
        "primary" => ButtonVariant::Primary,
        "secondary" => ButtonVariant::Secondary,
        "danger-primary" => ButtonVariant::DangerPrimary,
        "danger-secondary" => ButtonVariant::DangerSecondary,
        "danger-tertiary" => ButtonVariant::DangerTertiary,
        "subtle-primary" => ButtonVariant::SubtlePrimary,
        "subtle-secondary" => ButtonVariant::SubtleSecondary,
        "subtle-tertiary" => ButtonVariant::SubtleTertiary,
        _ => ButtonVariant::Tertiary,
    }
}
const COLORS: [&str; 37] = [
    "background-1",
    "background-4",
    "border-2",
    "border-4",
    "border-1",
    "border-information",
    "border-danger",
    "border-warning",
    "text-primary",
    "text-disabled",
    "effects-focus",
    "text-danger",
    "text-secondary",
    "text-accent",
    "text-accent-hover",
    "text-information",
    "text-success",
    "text-warning",
    "background-danger",
    "background-danger-active",
    "background-danger-hover",
    "text-on-danger",
    "border-3",
    "background-accent",
    "background-accent-active",
    "background-accent-hover",
    "text-on-accent",
    "background-accent-secondary",
    "background-accent-secondary-active",
    "background-accent-secondary-hover",
    "border-accent",
    "border-accent-active",
    "border-accent-hover",
    "text-accent-active",
    "background-2",
    "background-selected",
    "background-hover",
];
fn sentinel_style(scope: usize, revision: usize) -> String {
    let mut style = String::new();
    for (index, token) in COLORS.iter().enumerate() {
        style.push_str(&format!(
            "--si-sys-color-{token}:rgb({},{},{});",
            24 + (13 * index + 7 * scope + 3 * revision) % 160,
            32 + (11 * index + 5 * scope + 7 * revision) % 150,
            40 + (7 * index + 3 * scope + 11 * revision) % 140
        ));
    }
    let size = 20 - 4 * (scope % 2) + scope / 2 + 2 * revision;
    for (token, value) in [
        ("spacing-y-10", 1 + scope + revision),
        ("spacing-y-20", 3 + scope + revision),
        ("spacing-y-30", 5 + scope + revision),
        ("spacing-y-40", 8 + scope + 2 * revision),
        ("spacing-x-10", 2 + scope + revision),
        ("spacing-x-20", 4 + scope + revision),
        ("spacing-x-30", 6 + scope + revision),
        ("spacing-x-40", 10 + scope + revision),
        ("spacing-x-60", 14 + scope + 2 * revision),
        ("size-60", size),
        ("size-70", 24),
        ("size-30", 8 + scope + revision),
        ("size-50", 16 + scope + revision),
        ("size-80", 32 - 8 * (scope % 2) + 2 * revision),
        ("size-100", 48 + 2 * scope + 4 * revision),
        ("size-130", 96 + 2 * scope + 4 * revision),
        ("size-40", 12 + scope + revision),
        ("border-width-default", 1 + revision),
        ("border-width-selected", 2 + revision),
        ("border-width-emphasis", 3 + revision),
        ("border-radius-full", 99 + scope + revision),
        ("border-radius-xs", 3 + scope + revision),
        ("focus-ring-offset", 3 + scope + revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("font:400 16px/24px system-ui,sans-serif;color:var(--si-sys-color-text-primary);--si-sys-typography-body:400 {}px/{}px system-ui,sans-serif;--si-sys-typography-body-paragraph:400 {}px/{}px system-ui,sans-serif;--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,15+scope+revision,23+scope+revision,14+scope+revision,20+scope+revision,0.25+0.25*revision as f64));
    for (token, value) in [
        ("size-150", 160 + 8 * scope + 12 * revision),
        ("spacing-x-80", 20 + scope + 2 * revision),
        ("spacing-x-90", 34 + scope + 2 * revision),
        ("spacing-x-100", 38 + scope + 2 * revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    let duration = if revision == 0 { "0s" } else { "0.001s" };
    style.push_str(&format!("--theme-default-time:{duration};--theme-medium-time:{duration};--focus--border-color:var(--si-sys-color-effects-focus);"));
    style
}

fn override_style(r: usize) -> String {
    format!(
        "#search-review .search-component-override{{--ix-expanding-search--container-width:{}px;--ix-expanding-search--height:{}px;--ix-expanding-search-collapsed--width:{}px;--ix-expanding-search--border-radius:{}px;--ix-expanding-search-button-search--max-width:{}px;--ix-expanding-search-button-search--max-height:{}px;--ix-expanding-search-button-search-expanded--margin-left:{}px;--ix-expanding-search-button-clear--margin-right:{}px;--ix-expanding-search-input--padding-left:{}px;--ix-expanding-search-input--padding-right:{}px;--ix-expanding-search--outline-width--focus:{}px;--ix-input--min-height:{}px;--ix-input--border-radius:{}px;}}",
        220 + 10 * r,
        38 + 2 * r,
        34 + 2 * r,
        9 + r,
        30 + 2 * r,
        30 + 2 * r,
        7 + r,
        9 + r,
        41 + 2 * r,
        45 + 2 * r,
        2 + r,
        38 + 2 * r,
        9 + r
    )
}
#[derive(Clone, Copy, PartialEq)]
struct Consumer {
    value: Signal<String>,
    requests: Signal<Vec<Value>>,
    commands: Signal<usize>,
}
impl Consumer {
    fn snapshot(self) -> String {
        json!({"value":(self.value)(),"valueText":(self.value)(),"requests":(self.requests)()})
            .to_string()
    }
    fn accept(mut self, value: String) {
        let before = (self.value)();
        self.value.set(value.clone());
        let mut requests = self.requests.write();
        let sequence = requests.len() + 1;
        requests.push(json!({"sequence":sequence,"value":value,"before":before}));
    }
}
#[component]
pub fn ExpandingSearchReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut supplied = use_signal(|| true);
    let mut extension_value = use_signal(|| "Extension query".to_string());
    let mut extension_requests = use_signal(|| 0_usize);
    let mut pointer = use_signal(|| "collapsed-search".to_string());
    let mut pointer_epoch = use_signal(|| 0_usize);
    let mut consumer = Consumer {
        value: use_signal(String::new),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0),
    };
    let pointer_case = json!({"key":"pointer","id":"search-pointer-sample","scopeIndex":0,"pose":if pointer()=="clear"{"edited"}else if pointer()=="expanded-input"{"pristine"}else{"collapsed"},"variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start"});
    let consumer_case = json!({"key":"consumer","id":"search-consumer","scopeIndex":0,"pose":"collapsed","variant":"tertiary","placeholder":"Enter text here","value":"","fullWidth":false,"ariaLabelClearIconButton":"Clear search","ariaLabelSearchInput":"Search input","parentWidth":360,"alignment":"flex-start"});
    rsx! {style{{STYLES}} style{{override_style(revision())}}
    IxTheme{scheme:ColorScheme::Light,class:"search-review-shell",section{id:"search-review","data-phase":phase(),"data-revision":revision().to_string(),"data-consumer-command":(consumer.commands)().to_string(),onmounted:move |_|{spawn(async move{let query=dioxus::document::eval(OBSERVE);let _=query.join::<Value>().await;});},
    header{h2{"Expanding search"}p{"Compare collapsed, pristine and edited search fields under authored system tokens. The delayed focus and controlled values remain visible in the observations."}}
    section{class:"search-pointer-panel",h3{"Pointer states"}p{"Choose a target, arm it, then press and hold the search button, input or clear button."}
    label{"Pointer case" select{id:"search-pointer-case",value:pointer(),onchange:move|event|pointer.set(event.value()),for name in["collapsed-search","expanded-input","clear"]{option{value:name,"{name}"}}}}
    button{id:"search-pointer-arm",r#type:"button",onclick:move|_|pointer_epoch+=1,"Arm pointer capture"}
    div{id:"search-pointer-target","data-pointer-case":pointer(),"data-arm":pointer_epoch().to_string(),for epoch in [pointer_epoch()]{SearchCase{key:"pointer-{epoch}",case:pointer_case.clone(),revision:0,supplied:true,measured:false}}}
    output{id:"search-pointer-status",role:"status","Awaiting pointer input"}pre{id:"search-pointer-result","aria-hidden":"true"}}
    fieldset{class:"search-review-controls",legend{"Token and prop revisions"}
    for next in["baseline","revised"]{button{id:format!("search-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());},"{next}"}}
    label{input{id:"search-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())}"Supply test tokens"}
    button{id:"search-outside",r#type:"button","Keyboard target"}output{id:"search-status",role:"status","{phase()} — 36 cases"}}
    div{class:"search-review-matrix",for (key,case) in cases().into_iter().map(|case|(text(&case,"key"),case)){SearchCase{key:"{key}",case,revision:revision(),supplied:supplied(),measured:true}}}
    section{class:"search-consumer-panel",h3{"Accepting consumer"}p{"Requests update the Rust value. External values do not expand the field."}
    SearchCase{case:consumer_case,revision:0,supplied:true,measured:false,consumer:Some(consumer)}
    button{id:"search-consumer-reset",r#type:"button",onclick:move|_|{consumer.value.set(String::new());consumer.requests.set(Vec::new());consumer.commands+=1;},"Reset value and requests"}
    button{id:"search-consumer-external",r#type:"button",onclick:move|_|{consumer.value.set("External value".into());consumer.commands+=1;},"Set external value"}
    input{id:"search-consumer-json",r#type:"hidden",value:consumer.snapshot()}}
    section{id:"search-extension",class:"search-fallback",h3{"Rust extension diagnostics"}p{"Disabled and Enter-submit callbacks are separate Rust extensions."}ExpandingSearch{value:extension_value(),onvaluechange:move|value|extension_value.set(value),onsearch:move|_|extension_requests+=1}output{id:"search-extension-count","{extension_requests()}"}div{id:"search-extension-disabled",ExpandingSearch{disabled:true}}}
    section{class:"search-fallback",h3{"Provisional Brand fallback"}p{"No supplied test tokens in this separate preview."}IxTheme{scheme:ColorScheme::Light,ExpandingSearch{}}}
    pre{id:"search-result","aria-hidden":"true"}
    }}}
}
#[component]
fn SearchCase(
    case: Value,
    revision: usize,
    supplied: bool,
    measured: bool,
    #[props(default)] consumer: Option<Consumer>,
) -> Element {
    let mut current = case.clone();
    if revision == 1 {
        if let Some(updates) = case["revised"].as_object() {
            for (key, value) in updates {
                current[key] = value.clone();
            }
        }
    }
    let case = current;
    let scope = case["scopeIndex"].as_u64().unwrap_or(0) as usize;
    let mut edited = use_signal(|| None::<(usize, String)>);
    let mut callbacks = use_signal(Vec::<Value>::new);
    let mut blur_receipts = use_signal(|| 0_usize);
    let value = consumer.map(|model| (model.value)()).unwrap_or_else(|| {
        edited()
            .filter(|(at, _)| *at == revision)
            .map(|(_, value)| value)
            .unwrap_or_else(|| text(&case, "value"))
    });
    let icon = case["icon"].as_str().map(str::to_string);
    let search_label = case["ariaLabelSearchIconButton"]
        .as_str()
        .map(str::to_string);
    let style = if supplied {
        sentinel_style(scope, revision)
    } else {
        String::new()
    };
    let scheme = if scope < 2 {
        ColorScheme::Light
    } else {
        ColorScheme::Dark
    };
    let density = if scope % 2 == 0 {
        Density::Normal
    } else {
        Density::Compact
    };
    let name = text(&case, "key");
    let class = if case["override"] == true {
        "search-component-override"
    } else {
        ""
    };
    rsx! {article{id:text(&case,"id"),class:"search-review-case","data-blur-receipts":blur_receipts().to_string(),onfocusout:move |_|blur_receipts+=1,"data-search-row":measured.to_string(),"data-key":text(&case,"key"),"data-scope":scope.to_string(),"data-revision":revision.to_string(),"data-pose":text(&case,"pose"),"data-case":case.to_string(),"data-public-value":value.clone(),
    h4{class:"search-case-label","{name}"}
    IxTheme{scheme,density,class:"search-case-theme",div{class:"search-sentinel",style,
    div{class:"search-case-parent",style:format!("width:{}px;max-width:100%;display:flex;justify-content:{}",case["parentWidth"].as_u64().unwrap_or(360),case["alignment"].as_str().unwrap_or("flex-start")),
    ExpandingSearch{value:value.clone(),placeholder:text(&case,"placeholder"),full_width:case["fullWidth"]==true,variant:variant(&text(&case,"variant")),icon,class,aria_label_search_icon_button:search_label,aria_label_clear_icon_button:text(&case,"ariaLabelClearIconButton"),aria_label_search_input:text(&case,"ariaLabelSearchInput"),onvaluechange:move|value:String|{edited.set(Some((revision,value.clone())));callbacks.write().push(json!({"value":value}));if let Some(model)=consumer{model.accept(value);}}}
    }}}
    input{r#type:"hidden",class:"search-row-callbacks",value:json!({"requests":callbacks()}).to_string()}
    }}
}
const STYLES: &str = include_str!("styles/expanding_search_review.css");

const OBSERVE: &str = r#"return (()=>{
if(window.ixExpandingSearchReview)return true;
const fields=["display", "width", "height", "minWidth", "boxSizing", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft", "borderTopWidth", "borderTopColor", "borderTopStyle", "borderTopLeftRadius", "borderTopRightRadius", "borderBottomLeftRadius", "borderBottomRightRadius", "backgroundColor", "color", "fontSize", "lineHeight", "fontWeight", "fontFamily", "fontStyle", "letterSpacing", "fontFeatureSettings", "outlineColor", "outlineWidth", "outlineStyle", "outlineOffset", "opacity", "pointerEvents", "justifyContent", "textAlign", "marginLeft", "marginRight", "maxWidth", "minHeight", "maxHeight", "position", "flexShrink", "gap", "top", "right", "bottom", "left", "content", "fill", "stroke", "strokeWidth", "cursor", "visibility", "alignItems", "marginTop", "marginBottom", "borderRightWidth", "borderBottomWidth", "borderLeftWidth", "transform", "transitionProperty", "transitionDuration", "transitionTimingFunction", "transitionDelay", "flexDirection", "wordBreak", "whiteSpace", "overflow", "marginBlockStart", "marginBlockEnd", "marginInlineStart", "rowGap", "columnGap", "flexWrap", "marginInlineEnd", "minInlineSize", "appearance", "textOverflow", "resize", "caretColor", "outline", "userSelect", "zIndex", "insetInlineStart", "alignSelf", "borderBottomStyle", "borderRightStyle", "borderLeftStyle"],objects=new WeakMap();let nextObject=0,sequence=0;const journal=[],errors=[];
const object=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++nextObject);return objects.get(node);};
const log=(kind,details={})=>journal.push({sequence:++sequence,kind,time:performance.now(),...details});
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const frames=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const wait=async(predicate,label)=>{const until=performance.now()+15000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const metrics=(node,origin,pseudo=null)=>{if(!(node instanceof Element))return null;const css=getComputedStyle(node,pseudo),rect=node.getBoundingClientRect(),o=origin.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,css[key]])),rect:{x:rect.x,y:rect.y,width:rect.width,height:rect.height,relativeX:rect.x-o.x,relativeY:rect.y-o.y},pseudo};};
const hostFor=wrap=>wrap?.querySelector('.ix-expanding-search');const inputFor=wrap=>hostFor(wrap)?.querySelector('.ix-search-input');
const capture=wrap=>{const host=hostFor(wrap),control=inputFor(wrap);if(!host||!control)throw Error('Missing actual search '+wrap?.id);const search=host.querySelector('.ix-search-trigger-host'),searchControl=host.querySelector('.ix-search-trigger'),clear=host.querySelector('.ix-search-clear-host'),clearControl=host.querySelector('.ix-search-clear'),part=node=>metrics(node,host);return{id:wrap.id,key:wrap.dataset.key,scopeIndex:Number(wrap.dataset.scope),revision:Number(wrap.dataset.revision),pose:wrap.dataset.pose,props:JSON.parse(wrap.dataset.case),host:part(host),parent:part(host.parentElement),container:part(host.querySelector('.ix-search-input-container')),control:part(control),placeholder:metrics(control,host,'::placeholder'),search:part(search),searchControl:part(searchControl),searchGlyph:part(searchControl?.querySelector('.ix-icon-button-glyph')),clear:part(clear),clearControl:part(clearControl),clearGlyph:part(clearControl?.querySelector('.ix-icon-button-glyph')),resolvedAliases:Object.fromEntries(["--ix-expanding-search-button-search--outline-color--focus", "--ix-expanding-search-button--border--active", "--ix-expanding-search-input-container--transition-duration", "--ix-expanding-search-button--transition-duration", "--ix-expanding-search--border-radius", "--ix-expanding-search--container-width", "--ix-expanding-search--height", "--ix-expanding-search-collapsed--width", "--ix-expanding-search-button-search--max-width", "--ix-expanding-search-button-search--max-height", "--ix-expanding-search-button-search-expanded--margin-left", "--ix-expanding-search-button-search-active--width", "--ix-expanding-search-button-search-active--height", "--ix-expanding-search-button-clear--margin-right", "--ix-expanding-search-input--padding-left", "--ix-expanding-search-input--padding-right", "--ix-expanding-search--outline-width--focus", "--focus--border-color", "--ix-input--min-height", "--ix-input--padding", "--ix-input--border-width"].map(key=>[key,getComputedStyle(host).getPropertyValue(key)])),nativeFacts:{clearPresent:!!clear,hostValue:wrap.dataset.publicValue,focused:document.activeElement===control,focusVisible:control.matches(':focus-visible'),hover:control.matches(':hover'),active:control.matches(':active'),controlTabIndex:control.tabIndex,searchControlTabIndex:searchControl?.tabIndex??null,clearControlTabIndex:clearControl?.tabIndex??null,inputAriaLabel:control.getAttribute('aria-label'),searchAriaLabel:searchControl?.getAttribute('aria-label')??null,clearAriaLabel:clearControl?.getAttribute('aria-label')??null,hostFocus:host.matches(':focus'),searchHostFocus:search?.matches(':focus')??false,searchHostFocusVisible:search?.matches(':focus-visible')??false,searchHostFocused:search?.classList.contains('ix-focused')??false,searchControlFocus:document.activeElement===searchControl,searchControlFocusVisible:searchControl?.matches(':focus-visible')??false,clearHostFocus:clear?.matches(':focus')??null,clearHostFocusVisible:clear?.matches(':focus-visible')??null,clearHostFocused:clear?.classList.contains('ix-focused')??null,clearControlFocus:clearControl?document.activeElement===clearControl:null,clearControlFocusVisible:clearControl?.matches(':focus-visible')??null,activeObject:object(document.activeElement),activeTag:document.activeElement?.tagName,controlObject:object(control),hostObject:object(host),expanded:host.dataset.expanded==='true',changed:host.dataset.changed==='true',value:control.value,type:control.type,placeholder:control.placeholder,inputTabIndex:control.tabIndex,searchTabIndex:searchControl?.tabIndex??null,clearTabIndex:clearControl?.tabIndex??null,inputFocused:document.activeElement===control,searchFocused:document.activeElement===searchControl,clearFocused:document.activeElement===clearControl,inputFocusVisible:control.matches(':focus-visible'),searchFocusVisible:searchControl?.matches(':focus-visible')??false,clearFocusVisible:clearControl?.matches(':focus-visible')??false,inputHover:control.matches(':hover'),inputActive:control.matches(':active'),hostHover:host.matches(':hover'),hostActive:host.matches(':active'),searchHover:search?.matches(':hover')??false,searchActive:search?.matches(':active')??false,clearHover:clear?.matches(':hover')??false,clearActive:clear?.matches(':active')??false,ariaLabelSearch:searchControl?.getAttribute('aria-label')??null,ariaLabelClear:clearControl?.getAttribute('aria-label')??null,ariaLabelInput:control.getAttribute('aria-label'),searchAriaExpanded:searchControl?.getAttribute('aria-expanded')??null,disabled:control.disabled},callbacks:JSON.parse(wrap.querySelector('.search-row-callbacks').value)};};
const originalClick=node=>{const events=[];let event=null;const record=site=>e=>{if(e.composedPath()[0]!==node)return;event=e;events.push({site,defaultPrevented:e.defaultPrevented,trusted:e.isTrusted,targetTag:e.target.tagName});};const a=record('document-capture'),b=record('window-bubble');document.addEventListener('click',a,true);window.addEventListener('click',b);node.click();document.removeEventListener('click',a,true);window.removeEventListener('click',b);return{type:'click',eventObserved:!!event,defaultPrevented:event?.defaultPrevented??null,events};};
const nativeInput=(wrap,value)=>{const control=inputFor(wrap);control.value=value;const event=new InputEvent('input',{bubbles:true,composed:true,cancelable:false,inputType:'insertText',data:value});const dispatched=control.dispatchEvent(event);log('original-input',{id:wrap.id,object:object(control),value,defaultPrevented:event.defaultPrevented});return{type:'input',eventObserved:true,defaultPrevented:event.defaultPrevented,dispatched,value,trusted:event.isTrusted};};
const requestCount=wrap=>JSON.parse(wrap.querySelector('.search-row-callbacks').value).requests.length;
const requestReceipt=async(wrap,before)=>{await wait(()=>requestCount(wrap)>before,'actual Rust callback receipt '+wrap.id);log('rust-callback-commit',{id:wrap.id,before,after:requestCount(wrap)});};
const blurReceipt=async(wrap,target)=>{const active=inputFor(wrap)===document.activeElement,before=Number(wrap.dataset.blurReceipts);target.focus();if(active)await wait(()=>Number(wrap.dataset.blurReceipts)>before,'actual Rust focusout receipt '+wrap.id);log('rust-focusout-commit',{id:wrap.id,before,after:Number(wrap.dataset.blurReceipts),inputWasActive:active});};
const valueReceipt=async wrap=>{await wait(()=>inputFor(wrap)?.value===wrap.dataset.publicValue,'actual Rust public value/native receipt '+wrap.id);log('public-native-value-commit',{id:wrap.id,rustValue:wrap.dataset.publicValue,nativeValue:inputFor(wrap).value,object:object(inputFor(wrap))});};
for(const type of ['focusin','focusout'])document.addEventListener(type,event=>{const host=event.target?.closest?.('.ix-expanding-search');if(host)log(type,{object:object(event.target),tag:event.target.tagName,host:object(host),active:object(document.activeElement)});},true);
const expand=wrap=>{const original=originalClick(hostFor(wrap).querySelector('.ix-search-trigger'));log('search-click-return-0ms',{id:wrap.id,input:object(inputFor(wrap)),focused:document.activeElement===inputFor(wrap),original});return original;};
const timer=async wrap=>{await sleep(300);log('search-focus-timer-observation',{id:wrap.id,input:object(inputFor(wrap)),focused:document.activeElement===inputFor(wrap),elapsedNominalMs:300});};
const pointerRoot=()=>document.getElementById('search-pointer-target');
let pointerRows=[];const pointerSnapshot=(phase,event)=>{const wrap=pointerRoot()?.querySelector('.search-review-case');if(!wrap)return;const row=capture(wrap),target=event.target;pointerRows.push({phase,event:{type:event.type,trusted:event.isTrusted,button:event.button,clientX:event.clientX,clientY:event.clientY,target:object(target),targetTag:target.tagName,defaultPrevented:event.defaultPrevented},row});const output=document.getElementById('search-pointer-result');if(output)output.textContent=JSON.stringify({case:pointerRoot().dataset.pointerCase,records:pointerRows});document.getElementById('search-pointer-status').textContent=phase;};
for(const type of ['pointerdown','pointerup'])document.addEventListener(type,event=>{if(!pointerRoot()?.contains(event.target))return;if(type==='pointerdown')requestAnimationFrame(()=>pointerSnapshot('pointerdown-frame',event));else requestAnimationFrame(()=>requestAnimationFrame(()=>pointerSnapshot('pointerup-second-frame',event)));},true);
const arm=async()=>{const root=pointerRoot(),old=inputFor(root.querySelector('.search-review-case'));await wait(()=>inputFor(root.querySelector('.search-review-case'))!==old,'fresh Rust pointer sample');const wrap=root.querySelector('.search-review-case'),key=root.dataset.pointerCase;pointerRows=[];if(key!=='collapsed-search'){expand(wrap);if(key==='clear')nativeInput(wrap,'Pointer Query');await timer(wrap);await valueReceipt(wrap);}await frames();root.dataset.ready='true';document.getElementById('search-pointer-status').textContent='ready';return capture(wrap);};
window.addEventListener('click',event=>{if(event.target?.closest?.('#search-pointer-arm')){pointerRoot().dataset.ready='false';arm().catch(error=>{errors.push(String(error));document.getElementById('search-pointer-status').textContent=String(error);});}},true);
window.ixExpandingSearchReview={capture,hostFor,inputFor,frames,wait,sleep,log,journal,errors,object,originalClick,nativeInput,valueReceipt,requestCount,requestReceipt,blurReceipt,expand,timer,rows:()=>Array.from(document.querySelectorAll('[data-search-row="true"]')),pointer:()=>{const root=pointerRoot(),wrap=root?.querySelector('.search-review-case');if(!wrap)return null;const row=capture(wrap);return{case:root.dataset.pointerCase,ready:root.dataset.ready==='true',controlRect:row.control.rect,searchRect:row.searchControl.rect,clearRect:row.clearControl?.rect??null,row,records:pointerRows};}};
return true;
})()"#;

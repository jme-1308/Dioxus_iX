//! Shared dev-only Slider fixture. Consumer values and revisions stay in Rust.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
const CASES: &str = r#"[{"key":"slider-s0-default","id":"slider-slider-s0-default","scopeIndex":0,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":35,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s0-invalid","id":"slider-slider-s0-invalid","scopeIndex":0,"state":"invalid","condition":"rest","min":0,"max":100,"step":1,"value":35,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s0-valid","id":"slider-slider-s0-valid","scopeIndex":0,"state":"valid","condition":"rest","min":0,"max":100,"step":1,"value":35,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s0-info","id":"slider-slider-s0-info","scopeIndex":0,"state":"info","condition":"rest","min":0,"max":100,"step":1,"value":35,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s0-warning","id":"slider-slider-s0-warning","scopeIndex":0,"state":"warning","condition":"rest","min":0,"max":100,"step":1,"value":35,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s1-default","id":"slider-slider-s1-default","scopeIndex":1,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s1-invalid","id":"slider-slider-s1-invalid","scopeIndex":1,"state":"invalid","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s1-valid","id":"slider-slider-s1-valid","scopeIndex":1,"state":"valid","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s1-info","id":"slider-slider-s1-info","scopeIndex":1,"state":"info","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s1-warning","id":"slider-slider-s1-warning","scopeIndex":1,"state":"warning","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s2-default","id":"slider-slider-s2-default","scopeIndex":2,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":45,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s2-invalid","id":"slider-slider-s2-invalid","scopeIndex":2,"state":"invalid","condition":"rest","min":0,"max":100,"step":1,"value":45,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s2-valid","id":"slider-slider-s2-valid","scopeIndex":2,"state":"valid","condition":"rest","min":0,"max":100,"step":1,"value":45,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s2-info","id":"slider-slider-s2-info","scopeIndex":2,"state":"info","condition":"rest","min":0,"max":100,"step":1,"value":45,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s2-warning","id":"slider-slider-s2-warning","scopeIndex":2,"state":"warning","condition":"rest","min":0,"max":100,"step":1,"value":45,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s3-default","id":"slider-slider-s3-default","scopeIndex":3,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s3-invalid","id":"slider-slider-s3-invalid","scopeIndex":3,"state":"invalid","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s3-valid","id":"slider-slider-s3-valid","scopeIndex":3,"state":"valid","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s3-info","id":"slider-slider-s3-info","scopeIndex":3,"state":"info","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-s3-warning","id":"slider-slider-s3-warning","scopeIndex":3,"state":"warning","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start"},{"key":"slider-plain","id":"slider-slider-plain","scopeIndex":0,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":0,"trace":false,"traceReference":0,"marker":null,"label":null,"helperText":null,"invalidText":null,"validText":null,"infoText":null,"warningText":null,"labelStart":null,"labelEnd":null,"width":320,"alignment":"flex-start","extra":"plain"},{"key":"slider-min-value","id":"slider-slider-min-value","scopeIndex":1,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":0,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"min-value"},{"key":"slider-max-value","id":"slider-slider-max-value","scopeIndex":2,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":100,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"max-value"},{"key":"slider-no-trace","id":"slider-slider-no-trace","scopeIndex":3,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":false,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"no-trace"},{"key":"slider-equal-reference","id":"slider-slider-equal-reference","scopeIndex":0,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"equal-reference"},{"key":"slider-low-reference","id":"slider-slider-low-reference","scopeIndex":1,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":-20,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"low-reference"},{"key":"slider-high-reference","id":"slider-slider-high-reference","scopeIndex":2,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":65,"trace":true,"traceReference":120,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"high-reference"},{"key":"slider-negative-range","id":"slider-slider-negative-range","scopeIndex":3,"state":"default","condition":"rest","min":-100,"max":100,"step":1,"value":-35,"trace":true,"traceReference":0,"marker":[-100,-50,0,50,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"−100","labelEnd":"100","width":320,"alignment":"flex-start","extra":"negative-range"},{"key":"slider-fractional","id":"slider-slider-fractional","scopeIndex":0,"state":"default","condition":"rest","min":0,"max":1,"step":0.1,"value":0.3,"trace":true,"traceReference":0.5,"marker":[0,0.25,0.5,0.75,1],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"1","width":320,"alignment":"flex-start","extra":"fractional"},{"key":"slider-outside-markers","id":"slider-slider-outside-markers","scopeIndex":1,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":50,"marker":[-50,0,25,100,150,25],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"outside-markers"},{"key":"slider-labels","id":"slider-slider-labels","scopeIndex":2,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":45,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"A longer field label","helperText":"Endpoint labels are actual supplied elements","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"Minimum level","labelEnd":"Maximum level","width":320,"alignment":"flex-start","extra":"labels"},{"key":"slider-override","id":"slider-slider-override","scopeIndex":3,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"override","override":true},{"key":"slider-narrow-start","id":"slider-slider-narrow-start","scopeIndex":0,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":35,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"Low level","labelEnd":"High level","width":200,"alignment":"flex-start","extra":"narrow-start"},{"key":"slider-narrow-end","id":"slider-slider-narrow-end","scopeIndex":1,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":40,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"Low level","labelEnd":"High level","width":200,"alignment":"flex-end","extra":"narrow-end"},{"key":"slider-live-props","id":"slider-slider-live-props","scopeIndex":2,"state":"default","condition":"rest","min":0,"max":100,"step":1,"value":45,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"live-props","revised":{"min":-20,"max":80,"step":2,"value":26,"traceReference":10,"marker":[-20,0,20,40,80],"label":"Updated level","state":"warning","warningText":"Updated warning","labelStart":"−20","labelEnd":"80"}},{"key":"slider-focus","id":"slider-slider-focus","scopeIndex":3,"state":"default","condition":"focus","min":0,"max":100,"step":1,"value":50,"trace":true,"traceReference":50,"marker":[0,25,50,75,100],"label":"Level","helperText":"Helpful guidance","invalidText":"Level invalid","validText":"Level accepted","infoText":"Level information","warningText":"Level warning","labelStart":"0","labelEnd":"100","width":320,"alignment":"flex-start","extra":"focus"}]"#;
fn cases() -> Vec<Value> {
    serde_json::from_str(CASES).expect("authored slider cases")
}
fn text(case: &Value, key: &str) -> String {
    case[key].as_str().unwrap_or_default().to_string()
}
fn optional(case: &Value, key: &str) -> Option<String> {
    case[key].as_str().map(str::to_string)
}
fn number(case: &Value, key: &str) -> f64 {
    case[key].as_f64().unwrap_or_default()
}
fn state(name: &str) -> FieldState {
    match name {
        "invalid" => FieldState::Invalid,
        "valid" => FieldState::Valid,
        "info" => FieldState::Info,
        "warning" => FieldState::Warning,
        _ => FieldState::Default,
    }
}
const COLORS: [&str; 41] = [
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
    "background-information",
    "background-success",
    "background-warning",
    "border-success",
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
        ("size-70", 24 - 4 * (scope % 2) + scope / 2 + 2 * revision),
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
        ("size-10", 2 + scope + revision),
        ("size-20", 4 + scope + revision),
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
        "#slider-review .slider-component-override{{--ix-slider--min-height:{}px;--ix-slider--height:{}px;--ix-slider--thumb-size:{}px;--ix-slider-track--height:{}px;--ix-slider-ticks--tick-size:{}px;--ix-slider-label--margin-top:{}px;--ix-slider-reference--width:{}px;--ix-slider-reference--height:{}px;--ix-slider-track--border-radius:{}px;--ix-slider-trace--border-radius:{}px;--ix-slider-thumb--border-radius:{}px;--ix-slider--outline-width--focus:3px;--ix-slider--outline-offset:5px;}}",
        40 + 2 * r,
        28 + 2 * r,
        18 + 2 * r,
        5 + r,
        7 + r,
        11 + r,
        3 + r,
        20 + 2 * r,
        7 + r,
        7 + r,
        23 + r
    )
}
#[derive(Clone, Copy, PartialEq)]
struct Consumer {
    value: Signal<f64>,
    disabled: Signal<bool>,
    requests: Signal<Vec<Value>>,
    commands: Signal<usize>,
}
impl Consumer {
    fn snapshot(self) -> String {
        json!({"value":(self.value)(),"disabled":(self.disabled)(),"requests":(self.requests)()})
            .to_string()
    }
    fn accept(mut self, value: f64) {
        let before = (self.value)();
        self.value.set(value);
        let mut requests = self.requests.write();
        let sequence = requests.len() + 1;
        requests.push(json!({"sequence":sequence,"value":value,"before":before}));
    }
    fn command(mut self) {
        self.commands += 1;
    }
    fn reset(mut self) {
        self.value.set(35.0);
        self.disabled.set(false);
        self.requests.set(Vec::new());
        self.command();
    }
}
#[component]
pub fn SliderReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut supplied = use_signal(|| true);
    let mut pointer = use_signal(|| "rest".to_string());
    let mut pointer_epoch = use_signal(|| 0_usize);
    let mut consumer = Consumer {
        value: use_signal(|| 35.0),
        disabled: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0),
    };
    let mut pointer_case = cases()[0].clone();
    pointer_case["id"] = json!("slider-pointer-sample");
    pointer_case["key"] = json!("pointer");
    pointer_case["value"] = json!(50);
    pointer_case["traceReference"] = json!(0);
    pointer_case["state"] = json!(if pointer() == "invalid" {
        "invalid"
    } else {
        "default"
    });
    pointer_case["disabled"] = json!(pointer() == "disabled");
    let mut consumer_case = cases()[0].clone();
    consumer_case["id"] = json!("slider-consumer");
    consumer_case["key"] = json!("consumer");
    rsx! {style{{STYLES}}style{{override_style(revision())}}
    IxTheme{scheme:ColorScheme::Light,class:"slider-review-shell",section{id:"slider-review","data-phase":phase(),"data-revision":revision().to_string(),"data-consumer-command":(consumer.commands)().to_string(),onmounted:move |_|{spawn(async move{let query=dioxus::document::eval(OBSERVE);let _=query.join::<Value>().await;});},
    header{h2{"Slider track, thumb and ticks"}p{"Compare native range controls and semantic state tokens. The transparent anchor and native thumb remain separate observations."}}
    section{class:"slider-pointer-panel",h3{"Pointer and keyboard states"}p{"Choose a state, arm it, then press and hold the native thumb or use its arrow keys."}
    label{"Pointer case" select{id:"slider-pointer-case",value:pointer(),onchange:move|event|pointer.set(event.value()),for name in["rest","invalid","disabled"]{option{value:name,"{name}"}}}}
    button{id:"slider-pointer-arm",r#type:"button",onclick:move|_|pointer_epoch+=1,"Arm pointer capture"}
    div{id:"slider-pointer-target","data-pointer-case":pointer(),"data-arm":pointer_epoch().to_string(),for epoch in[pointer_epoch()]{SliderCase{key:"pointer-{epoch}",case:pointer_case.clone(),revision:0,supplied:true,measured:false}}}
    output{id:"slider-pointer-status",role:"status","Awaiting native input"}pre{id:"slider-pointer-result","aria-hidden":"true"}}
    fieldset{class:"slider-review-controls",legend{"Token and prop revisions"}
    for next in["baseline","revised"]{button{id:format!("slider-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.into());},"{next}"}}
    label{input{id:"slider-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())}"Supply test tokens"}
    button{id:"slider-outside",r#type:"button","Keyboard target"}output{id:"slider-status",role:"status","{phase()} — 36 cases"}}
    div{class:"slider-review-matrix",for (key,case) in cases().into_iter().map(|case|(text(&case,"key"),case)){SliderCase{key:"{key}",case,revision:revision(),supplied:supplied(),measured:true}}}
    section{class:"slider-consumer-panel",h3{"Accepting consumer"}p{"The Rust consumer accepts one callback per native input event. Source duplicate events are retained separately."}
    form{id:"slider-consumer-form",onsubmit:move|event|event.prevent_default(),SliderCase{case:consumer_case,revision:0,supplied:true,measured:false,consumer:Some(consumer)}}
    button{id:"slider-consumer-reset",r#type:"button",onclick:move|_|consumer.reset(),"Reset"}
    button{id:"slider-consumer-external",r#type:"button",onclick:move|_|{consumer.value.set(45.0);consumer.command();},"Set 45 externally"}
    button{id:"slider-consumer-disable",r#type:"button",onclick:move|_|{consumer.disabled.set(true);consumer.command();},"Disable"}
    button{id:"slider-consumer-enable",r#type:"button",onclick:move|_|{consumer.disabled.set(false);consumer.command();},"Enable"}
    input{id:"slider-consumer-json",r#type:"hidden",value:consumer.snapshot()}}
    section{class:"slider-fallback",h3{"Provisional Brand fallback"}p{"No supplied system tokens in this separate preview."}IxTheme{scheme:ColorScheme::Light,Slider{label:"Level",value:35.0,trace:true,trace_reference:50.0,label_start:Some("Low".to_string()),label_end:Some("High".to_string())}}}
    pre{id:"slider-result","aria-hidden":"true"}
    }}}
}
#[component]
fn SliderCase(
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
    let mut edited = use_signal(|| None::<(usize, f64)>);
    let mut callbacks = use_signal(Vec::<Value>::new);
    let mut focus_in = use_signal(|| 0_usize);
    let mut focus_out = use_signal(|| 0_usize);
    let value = consumer.map(|model| (model.value)()).unwrap_or_else(|| {
        edited()
            .filter(|(at, _)| *at == revision)
            .map(|(_, value)| value)
            .unwrap_or_else(|| number(&case, "value"))
    });
    let disabled = consumer
        .map(|model| (model.disabled)())
        .unwrap_or(case["disabled"] == true);
    let markers = case["marker"]
        .as_array()
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_f64)
                .map(|value| SliderMarker {
                    value,
                    label: String::new(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
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
    let density = if scope.is_multiple_of(2) {
        Density::Normal
    } else {
        Density::Compact
    };
    let name = text(&case, "key");
    let class = if case["override"] == true {
        "slider-component-override"
    } else {
        ""
    };
    rsx! {article{id:text(&case,"id"),class:"slider-review-case","data-slider-row":measured.to_string(),"data-key":name.clone(),"data-scope":scope.to_string(),"data-revision":revision.to_string(),"data-condition":text(&case,"condition"),"data-case":case.to_string(),"data-public-value":value.to_string(),"data-callback-count":callbacks().len().to_string(),"data-focus-in":focus_in().to_string(),"data-focus-out":focus_out().to_string(),onfocusin:move |_|focus_in+=1,onfocusout:move |_|focus_out+=1,
    h4{class:"slider-case-label","{name}"}
    IxTheme{scheme,density,class:"slider-case-theme",div{class:"slider-sentinel",style,
    div{class:"slider-case-parent",style:format!("width:{}px;max-width:100%;display:flex;justify-content:{}",case["width"].as_u64().unwrap_or(320),case["alignment"].as_str().unwrap_or("flex-start")),
    Slider{value,name:consumer.map(|_|"level".to_string()),min:number(&case,"min"),max:number(&case,"max"),step:number(&case,"step"),label:text(&case,"label"),disabled,trace:case["trace"]==true,trace_reference:number(&case,"traceReference"),markers,show_value:false,class,label_start:optional(&case,"labelStart"),label_end:optional(&case,"labelEnd"),helper_text:optional(&case,"helperText"),invalid_text:optional(&case,"invalidText"),valid_text:optional(&case,"validText"),info_text:optional(&case,"infoText"),warning_text:optional(&case,"warningText"),state:state(&text(&case,"state")),onvaluechange:move|value:f64|{edited.set(Some((revision,value)));let mut rows=callbacks.write();let sequence=rows.len()+1;rows.push(json!({"sequence":sequence,"value":value}));drop(rows);if let Some(model)=consumer{model.accept(value);}}}
    }}}
    input{r#type:"hidden",class:"slider-row-callbacks",value:json!({"requests":callbacks()}).to_string()}
    }}
}
const STYLES: &str = include_str!("styles/slider_review.css");

const OBSERVE: &str = r#"return (()=>{
if(window.ixSliderReview)return {installed:false};
const fields=["display", "width", "height", "minWidth", "boxSizing", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft", "borderTopWidth", "borderTopColor", "borderTopStyle", "borderTopLeftRadius", "borderTopRightRadius", "borderBottomLeftRadius", "borderBottomRightRadius", "backgroundColor", "color", "fontSize", "lineHeight", "fontWeight", "fontFamily", "fontStyle", "letterSpacing", "fontFeatureSettings", "outlineColor", "outlineWidth", "outlineStyle", "outlineOffset", "opacity", "pointerEvents", "justifyContent", "textAlign", "marginLeft", "marginRight", "maxWidth", "minHeight", "maxHeight", "position", "flexShrink", "gap", "top", "right", "bottom", "left", "content", "fill", "stroke", "strokeWidth", "cursor", "visibility", "alignItems", "marginTop", "marginBottom", "borderRightWidth", "borderBottomWidth", "borderLeftWidth", "transform", "transitionProperty", "transitionDuration", "transitionTimingFunction", "transitionDelay", "flexDirection", "wordBreak", "whiteSpace", "overflow", "marginBlockStart", "marginBlockEnd", "marginInlineStart", "rowGap", "columnGap", "flexWrap", "marginInlineEnd", "minInlineSize", "appearance", "textOverflow", "resize", "caretColor", "outline", "userSelect", "zIndex", "insetInlineStart", "alignSelf", "borderBottomStyle", "borderRightStyle", "borderLeftStyle", "r", "cx", "cy", "overflowX", "overflowY"],aliases=["--ix-slider-thumb--background--active", "--ix-slider-thumb--background--disabled", "--ix-slider-thumb--background--hover", "--ix-slider-thumb--background--info", "--ix-slider-thumb--background--invalid--active", "--ix-slider-thumb--background--invalid--hover", "--ix-slider-thumb--background--invalid", "--ix-slider-thumb--background--success", "--ix-slider-thumb--background--warning", "--ix-slider-thumb--background", "--ix-slider-thumb--outline-color--focus", "--ix-slider-trace--background--disabled", "--ix-slider-trace--background--info", "--ix-slider-trace--background--invalid", "--ix-slider-trace--background--success", "--ix-slider-trace--background--warning", "--ix-slider-trace--background", "--ix-slider-trace-marker--background--disabled", "--ix-slider-trace-marker--background--info", "--ix-slider-trace-marker--background--invalid", "--ix-slider-trace-marker--background--success", "--ix-slider-trace-marker--background--warning", "--ix-slider-trace-marker--background", "--ix-slider-track--background--disabled", "--ix-slider-track--background", "--ix-slider-track-marker--background--disabled", "--ix-slider-track-marker--background", "--ix-slider-thumb--transition-duration", "--ix-slider--min-height", "--ix-slider--thumb-size", "--ix-slider--height", "--ix-slider-label--margin-top", "--ix-slider-input--margin-top", "--ix-slider-trace--height", "--ix-slider-trace--border-radius", "--ix-slider-reference--width", "--ix-slider-reference--height", "--ix-slider-input--track-height", "--ix-slider--z-index", "--ix-slider--outline-width--focus", "--ix-slider--outline-offset", "--ix-slider-ticks--tick-size", "--ix-slider-track--height", "--ix-slider-track--border-radius", "--ix-slider-thumb--border-radius"],objects=new WeakMap(),journal=[],errors=[];let nextObject=0,sequence=0;
const object=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++nextObject);return objects.get(node);};
const log=(kind,details={})=>journal.push({sequence:++sequence,kind,time:performance.now(),...details});
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const wait=async(predicate,label)=>{const until=performance.now()+15000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const frames=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const metrics=(node,origin,pseudo=null)=>{if(!(node instanceof Element))return null;let css;try{css=getComputedStyle(node,pseudo);}catch(error){return{object:object(node),pseudo,error:String(error)};}const rect=node.getBoundingClientRect(),o=origin.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,css[key]])),rect:{x:rect.x,y:rect.y,width:rect.width,height:rect.height,relativeX:rect.x-o.x,relativeY:rect.y-o.y},pseudo};};
const hostFor=wrap=>wrap?.querySelector('.ix-slider-field');const inputFor=wrap=>hostFor(wrap)?.querySelector('input.ix-slider');
const capture=wrap=>{const host=hostFor(wrap),control=inputFor(wrap);if(!host||!control)throw Error('Missing actual Slider '+wrap?.id);const find=selector=>host.querySelector(selector),part=node=>metrics(node,host),endpoint=find('.ix-slider-endpoints'),feedback=find('.ix-text-field-bottom-text');const circles=Array.from(host.querySelectorAll('.ix-slider-tick'),(node,index)=>({...part(node),index,fraction:getComputedStyle(node).getPropertyValue('--tick-value'),active:node.classList.contains('ix-slider-tick-active'),atMin:node.classList.contains('ix-slider-tick-at-min'),atMax:node.classList.contains('ix-slider-tick-at-max'),tickValue:getComputedStyle(node).getPropertyValue('--tick-value')}));return{id:wrap.id,key:wrap.dataset.key,scopeIndex:Number(wrap.dataset.scope),revision:Number(wrap.dataset.revision),condition:wrap.dataset.condition,props:JSON.parse(wrap.dataset.case),host:part(host),parent:part(host.parentElement),wrapper:part(find('.ix-text-field-wrapper')),top:part(find('.ix-text-field-top')),labelHost:part(find('.ix-text-field-label')),label:part(find('.ix-text-field-label .ix-typography')),labelText:find('.ix-text-field-label')?.textContent??null,slot:part(find('.ix-text-field-slot')),bottom:part(find('.ix-text-field-bottom')),feedback:part(feedback),feedbackText:feedback?Array.from(feedback.childNodes).filter(node=>node.nodeType===Node.TEXT_NODE).map(node=>node.textContent).join(''):null,icon:part(find('.ix-text-field-text-icon')),right:part(find('.ix-text-field-bottom-right')),container:part(find('.ix-slider-container')),slider:part(find('.ix-slider-control')),track:part(find('.ix-slider-track')),anchor:part(find('.ix-slider-thumb-anchor')),ticks:part(find('.ix-slider-ticks')),circles,endpoints:part(endpoint),endpointStart:part(endpoint?.children[0]),endpointEnd:part(endpoint?.children[1]),startChild:part(endpoint?.children[0]?.firstElementChild),endChild:part(endpoint?.children[1]?.firstElementChild),startText:endpoint?.children[0]?.textContent??null,endText:endpoint?.children[1]?.textContent??null,tooltipRaw:part(find('.ix-slider-tooltip')),control:part(control),before:metrics(control,host,'::before'),after:metrics(control,host,'::after'),thumbPseudo:metrics(control,host,'::-webkit-slider-thumb'),trackPseudo:metrics(control,host,'::-webkit-slider-runnable-track'),mozThumbPseudo:metrics(control,host,'::-moz-range-thumb'),mozTrackPseudo:metrics(control,host,'::-moz-range-track'),resolvedAliases:Object.fromEntries(aliases.map(key=>[key,getComputedStyle(host).getPropertyValue(key)])),resolvedLocal:Object.fromEntries(['--thumb-size','--trace-reference-color','--trace-color','--tick-color','--tick-color--active','--track-color'].map(key=>[key,getComputedStyle(host).getPropertyValue(key)])),nativeFacts:{hostDisabled:host.dataset.disabled==='true',tabIndex:control.tabIndex,hideReference:control.classList.contains('hide-trace-reference'),type:control.type,value:control.value,valueAsNumber:control.valueAsNumber,hostValue:Number(wrap.dataset.publicValue),min:control.min,max:control.max,step:control.step,disabled:control.disabled,controlTabIndex:control.tabIndex,focused:document.activeElement===control,focusVisible:control.matches(':focus-visible'),hover:control.matches(':hover'),active:control.matches(':active'),hostHover:host.matches(':hover'),hostActive:host.matches(':active'),ariaLabel:control.getAttribute('aria-label'),ariaValueNow:control.getAttribute('aria-valuenow'),ariaValueMin:control.getAttribute('aria-valuemin'),ariaValueMax:control.getAttribute('aria-valuemax'),ariaInvalid:control.getAttribute('aria-invalid'),role:control.getAttribute('role'),list:control.getAttribute('list'),trace:control.classList.contains('trace'),hideTraceReference:control.classList.contains('hide-trace-reference'),valueFraction:control.style.getPropertyValue('--value'),traceReference:control.style.getPropertyValue('--trace-reference'),traceStart:control.style.getPropertyValue('--trace-start'),traceEnd:control.style.getPropertyValue('--trace-end'),activeObject:object(document.activeElement),activeTag:document.activeElement?.tagName,hostObject:object(host),controlObject:object(control)},callbacks:JSON.parse(wrap.querySelector('.slider-row-callbacks').value)};};
const requestCount=wrap=>Number(wrap.dataset.callbackCount);
const requestReceipt=async(wrap,before)=>{await wait(()=>requestCount(wrap)>before,'actual Rust callback receipt '+wrap.id);log('rust-callback-commit',{id:wrap.id,before,after:requestCount(wrap)});};
const valueReceipt=async wrap=>{await wait(()=>Number(inputFor(wrap)?.value)===Number(wrap.dataset.publicValue),'current Rust value/native receipt '+wrap.id);log('public-native-value-commit',{id:wrap.id,object:object(inputFor(wrap)),rustValue:wrap.dataset.publicValue,nativeValue:inputFor(wrap).value});};
const focusReceipt=async wrap=>{const control=inputFor(wrap),wasActive=document.activeElement===control,before=Number(wrap.dataset.focusIn);control.focus();if(!wasActive&&document.activeElement===control)await wait(()=>Number(wrap.dataset.focusIn)>before,'actual Rust focusin receipt '+wrap.id);log('rust-focusin-commit',{id:wrap.id,before,after:Number(wrap.dataset.focusIn),focused:document.activeElement===control});};
const blurReceipt=async target=>{const previous=document.activeElement,wrap=previous?.closest?.('.slider-review-case'),before=Number(wrap?.dataset.focusOut??0);target.focus();if(wrap&&previous!==document.activeElement)await wait(()=>Number(wrap.dataset.focusOut)>before,'actual Rust focusout receipt '+wrap.id);log('rust-focusout-commit',{id:wrap?.id??null,before,after:Number(wrap?.dataset.focusOut??0)});};
const nativeInput=(wrap,value)=>{const control=inputFor(wrap);control.value=String(value);const event=new InputEvent('input',{bubbles:true,composed:true,cancelable:false,inputType:'insertText',data:null});const dispatched=control.dispatchEvent(event);log('original-native-input',{id:wrap.id,value:control.value,object:object(control),defaultPrevented:event.defaultPrevented});return{type:'input',dispatched,eventObserved:true,defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,value:control.value,target:object(control)};};
for(const type of ['focusin','focusout','keydown','keyup'])document.addEventListener(type,event=>{const wrap=event.target?.closest?.('.slider-review-case');if(wrap)log('native-'+type,{id:wrap.id,key:event.key??null,target:object(event.target),defaultPrevented:event.defaultPrevented,trusted:event.isTrusted});},true);
const panel=()=>document.getElementById('slider-pointer-target');let observations=[];
const take=(phase,event,original)=>{const wrap=panel()?.querySelector('.slider-review-case');if(!wrap)return;observations.push({case:panel().dataset.pointerCase,arm:Number(panel().dataset.arm),phase,original:{type:event.type,key:event.key??null,trusted:event.isTrusted,pointerType:event.pointerType??null,button:event.button??null,clientX:event.clientX??null,clientY:event.clientY??null,target:object(original),targetTag:original?.tagName??null,defaultPrevented:event.defaultPrevented},row:capture(wrap)});document.getElementById('slider-pointer-result').textContent=JSON.stringify({case:panel().dataset.pointerCase,observations});document.getElementById('slider-pointer-status').textContent=observations.length+' actual input observations';};
for(const type of ['pointerdown','pointerup','keydown'])document.addEventListener(type,event=>{if(!panel()?.contains(event.target))return;const original=event.composedPath()[0];if(type==='pointerdown')requestAnimationFrame(()=>take('pointerdown-frame',event,original));else if(type==='pointerup')requestAnimationFrame(()=>requestAnimationFrame(()=>take('pointerup-second-frame',event,original)));else requestAnimationFrame(()=>requestAnimationFrame(()=>take('keydown-second-frame',event,original)));},true);
window.addEventListener('click',event=>{if(!event.target?.closest?.('#slider-pointer-arm'))return;const root=panel(),old=inputFor(root.querySelector('.slider-review-case'));root.dataset.ready='false';(async()=>{await wait(()=>inputFor(root.querySelector('.slider-review-case'))!==old,'fresh keyed Rust pointer input');const wrap=root.querySelector('.slider-review-case');await valueReceipt(wrap);await frames();root.dataset.ready='true';document.getElementById('slider-pointer-status').textContent='ready';log('pointer-armed',{case:root.dataset.pointerCase,arm:Number(root.dataset.arm),input:object(inputFor(wrap))});})().catch(error=>{errors.push(String(error));document.getElementById('slider-pointer-status').textContent=String(error);});},true);
window.ixSliderReview={capture,hostFor,inputFor,metrics,object,frames,wait,sleep,log,journal,errors,requestCount,requestReceipt,valueReceipt,focusReceipt,blurReceipt,nativeInput,rows:()=>Array.from(document.querySelectorAll('[data-slider-row="true"]')),pointer:()=>{const root=panel(),wrap=root?.querySelector('.slider-review-case');if(!wrap)return null;const row=capture(wrap),rect=row.anchor.rect;return{case:root.dataset.pointerCase,ready:root.dataset.ready==='true',arm:Number(root.dataset.arm),controlRect:row.control.rect,thumbPoint:{x:rect.x+rect.width/2,y:rect.y+rect.height/2,source:'actual transparent anchor rectangle'},row,observations};}};
window.addEventListener('error',event=>errors.push(event.message));window.addEventListener('unhandledrejection',event=>errors.push(String(event.reason)));
return{installed:true};
})()
"#;

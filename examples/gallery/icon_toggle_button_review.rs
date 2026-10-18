//! Shared controlled IconToggleButton gallery/native token fixture.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

pub const STATES: [&str; 6] = ["rest", "hover", "active", "disabled", "loading", "focus"];
pub const BASE_CORE: [(&str, ButtonVariant, ButtonIconSize, bool); 16] = [
    (
        "primary-12-square",
        ButtonVariant::Primary,
        ButtonIconSize::Twelve,
        false,
    ),
    (
        "primary-12-oval",
        ButtonVariant::Primary,
        ButtonIconSize::Twelve,
        true,
    ),
    (
        "primary-16-square",
        ButtonVariant::Primary,
        ButtonIconSize::Sixteen,
        false,
    ),
    (
        "primary-16-oval",
        ButtonVariant::Primary,
        ButtonIconSize::Sixteen,
        true,
    ),
    (
        "primary-20-square",
        ButtonVariant::Primary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "primary-20-oval",
        ButtonVariant::Primary,
        ButtonIconSize::Twenty,
        true,
    ),
    (
        "primary-24-square",
        ButtonVariant::Primary,
        ButtonIconSize::TwentyFour,
        false,
    ),
    (
        "primary-24-oval",
        ButtonVariant::Primary,
        ButtonIconSize::TwentyFour,
        true,
    ),
    (
        "secondary-20-square",
        ButtonVariant::Secondary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "tertiary-20-square",
        ButtonVariant::Tertiary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "subtle-primary-20-square",
        ButtonVariant::SubtlePrimary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "subtle-secondary-20-square",
        ButtonVariant::SubtleSecondary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "subtle-tertiary-20-square",
        ButtonVariant::SubtleTertiary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "danger-primary-20-square",
        ButtonVariant::DangerPrimary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "danger-secondary-20-square",
        ButtonVariant::DangerSecondary,
        ButtonIconSize::Twenty,
        false,
    ),
    (
        "danger-tertiary-20-square",
        ButtonVariant::DangerTertiary,
        ButtonIconSize::Twenty,
        false,
    ),
];
const COLORS: [&str; 28] = [
    "effects-focus",
    "background-danger",
    "background-danger-active",
    "background-1",
    "background-danger-hover",
    "text-on-danger",
    "text-disabled",
    "text-danger",
    "border-danger",
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
    "text-accent",
    "text-accent-active",
    "text-accent-hover",
    "background-2",
    "background-selected",
    "background-hover",
    "text-primary",
    "border-2",
];
pub fn sentinel_style(scope: usize, revision: usize) -> String {
    let mut style = String::new();
    for (index, token) in COLORS.iter().enumerate() {
        let red = 24 + (13 * index + 7 * scope + 3 * revision) % 160;
        let green = 32 + (11 * index + 5 * scope + 7 * revision) % 150;
        let blue = 40 + (7 * index + 3 * scope + 11 * revision) % 140;
        style.push_str(&format!(
            "--si-sys-color-{token}:rgb({red},{green},{blue});"
        ));
    }
    for (token, value) in [
        ("size-40", 12 - 4 * (scope % 2) + scope / 2 + 2 * revision),
        ("size-50", 16 - 4 * (scope % 2) + scope / 2 + 2 * revision),
        ("size-60", 20 - 4 * (scope % 2) + scope / 2 + 2 * revision),
        ("size-70", 24 - 4 * (scope % 2) + scope / 2 + 2 * revision),
        ("size-80", 32 - 8 * (scope % 2) + scope / 2 + 2 * revision),
        ("size-120", 64 + 2 * scope + 4 * revision),
        ("size-130", 40),
        ("spacing-x-40", 10 + scope + revision),
        ("spacing-x-20", 4 + scope + revision),
        ("border-radius-xs", 3 + scope + revision),
        ("border-radius-full", 99 + scope + revision),
        ("border-width-default", 1 + revision),
        ("border-width-selected", 2 + revision),
        ("border-width-emphasis", 3 + revision),
        ("focus-ring-offset", 3 + revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,0.25+0.25*revision as f64));
    style.push_str(&format!(
        "--icon-review-ink:rgb(183,{},91);",
        71 + 3 * revision
    ));
    style.push_str("color:var(--si-sys-color-text-primary);");
    style
}
#[derive(Clone, Copy, PartialEq)]
struct Controlled {
    pressed: Signal<bool>,
    accept: Signal<bool>,
    disabled: Signal<bool>,
    loading: Signal<bool>,
    requests: Signal<Vec<Value>>,
    commands: Signal<usize>,
}
impl Controlled {
    fn command(mut self) {
        self.commands += 1;
    }
    fn reset(mut self) {
        self.pressed.set(false);
        self.accept.set(false);
        self.disabled.set(false);
        self.loading.set(false);
        self.requests.set(Vec::new());
        self.command();
    }
    fn request(mut self, value: bool) {
        let before = (self.pressed)();
        let accept = (self.accept)();
        if accept {
            self.pressed.set(value);
        }
        let after = (self.pressed)();
        let mut requests = self.requests.write();
        let sequence = requests.len() + 1;
        requests.push(json!({"sequence":sequence,"before":before,"value":value,"accept":accept,"after":after}));
    }
    fn snapshot(self) -> String {
        json!({"pressed":(self.pressed)(),"accept":(self.accept)(),"disabled":(self.disabled)(),"loading":(self.loading)(),"requests":(self.requests)()}).to_string()
    }
}

fn override_rules(revision: usize) -> String {
    let mut rules = String::new();
    for (key, token, base) in [
        ("override12", "extra-small", 23),
        ("override16", "small", 33),
        ("override24", "large", 44),
    ] {
        rules.push_str(&format!("#icon-toggle-review .icon-toggle-{key}{{--ix-icon-button-{token}--size:{}px;--ix-button-border-radius-left:{}px;--ix-button-border-radius-right:{}px;--ix-icon-button-color:rgb(73,119,161);}}",base+2*revision,2+revision,9+revision));
    }
    rules.push_str("#icon-toggle-review .icon-toggle-icon-color{--ix-icon-button-color:rgb(17,71,131)}#icon-toggle-review .icon-toggle-danger-pressed-override{--ix-button-danger-secondary--background--pressed:rgb(39,83,127);--ix-button-danger-secondary--color--pressed:rgb(247,249,251)}");
    rules
}

#[component]
pub fn IconToggleButtonReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut state = use_signal(|| "rest".to_string());
    let mut supplied = use_signal(|| true);
    let mut controlled = Controlled {
        pressed: use_signal(|| false),
        accept: use_signal(|| false),
        disabled: use_signal(|| false),
        loading: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    rsx! {
        style {{STYLES}}
        style {{override_rules(revision())}}
        IxTheme {scheme:ColorScheme::Light,class:"icon-toggle-review-shell",
            section {id:"icon-toggle-review","data-phase":phase(),"data-revision":revision().to_string(),"data-state":state(),"data-supplied":supplied().to_string(),"data-consumer-command":(controlled.commands)().to_string(),
                header {class:"icon-toggle-intro",
                    h2 {"Icon toggle button selected states"}
                    p {"Compare pressed and unpressed icon controls across sizes, shapes and nine variants. Supplied system-token values are diagnostic integration inputs, not Brand measurements or accessibility claims."}
                }
                fieldset {class:"icon-toggle-controls",
                    legend {"Shared preview controls"}
                    label {"Visual state",select {id:"icon-toggle-state",value:state(),onchange:move|event|state.set(event.value()),for value in STATES{option {value,selected:state()==value,"{value}"}}}}
                    label {class:"icon-toggle-check",input {id:"icon-toggle-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
                    for next in ["baseline","revised","restored"]{button {id:format!("icon-toggle-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.to_string());state.set("rest".into());supplied.set(true);},match next{"baseline"=>"Baseline","revised"=>"Revise tokens",_=>"Restore tokens"}}}
                    output {id:"icon-toggle-status",role:"status","aria-live":"polite","{phase()} / {state()} — 128 controls, pressed and unpressed"}
                    button {id:"icon-toggle-outside",r#type:"button","Keyboard target"}
                }
                div {class:"icon-toggle-matrix",
                    for(scope,index,scheme,density)in[("light-normal",0,ColorScheme::Light,Density::Normal),("light-compact",1,ColorScheme::Light,Density::Compact),("dark-normal",2,ColorScheme::Dark,Density::Normal),("dark-compact",3,ColorScheme::Dark,Density::Compact)]{
                        section {id:format!("icon-toggle-scope-{scope}"),"data-icon-toggle-scope":scope,"data-scope-index":index.to_string(),"data-revision":revision().to_string(),
                            h3 {class:"icon-toggle-scope-title","{scheme.as_str()} / {density.as_str()}"}
                            IxTheme {scheme,density,style:if supplied(){sentinel_style(index,revision())}else{String::new()},
                                div {class:"icon-toggle-samples",for(base,variant,size,oval)in BASE_CORE{for pressed in[false,true]{
                                    Sample {row_id:format!("icon-toggle-{scope}-{base}-{}",if pressed{"on"}else{"off"}),kind:"core",sample_key:format!("{base}-{}",if pressed{"on"}else{"off"}),variant,size,oval,pressed,state:state()}
                                }}}
                            }
                        }
                    }
                }
                section {class:"icon-toggle-extras","data-icon-toggle-scope":"light-normal","data-scope-index":"0","data-revision":revision().to_string(),
                    h3 {"Component overrides and nested scope"}
                    IxTheme {scheme:ColorScheme::Light,style:if supplied(){sentinel_style(0,revision())}else{String::new()},
                        div {class:"icon-toggle-extra-grid",
                            for key in["default","override12","override16","override24","icon-color","empty-loading12","empty-ready","host-active-selected","danger-pressed-override"]{
                                Sample {row_id:format!("icon-toggle-extra-{key}"),kind:"extra",sample_key:key,
                                    variant:match key{"icon-color"=>ButtonVariant::Secondary,"danger-pressed-override"=>ButtonVariant::DangerSecondary,_=>ButtonVariant::Primary},
                                    size:match key{"override12"|"empty-loading12"=>ButtonIconSize::Twelve,"override16"=>ButtonIconSize::Sixteen,"override24"=>ButtonIconSize::TwentyFour,_=>ButtonIconSize::Twenty},
                                    oval:key=="override12",pressed:!matches!(key,"default"|"empty-ready")}
                            }
                            Sample {row_id:"icon-toggle-extra-controlled",kind:"extra",sample_key:"controlled",controlled:Some(controlled)}
                            div {"data-icon-toggle-scope":"nested","data-scope-index":"3","data-revision":"0",
                                IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),Sample {row_id:"icon-toggle-extra-nested",kind:"extra",sample_key:"nested",variant:ButtonVariant::Secondary,pressed:true}}
                            }
                        }
                    }
                }
                fieldset {class:"icon-toggle-controls icon-toggle-consumer-controls",
                    legend {"Controlled request consumer"}
                    p {"The consumer chooses whether to accept each requested pressed value."}
                    button {id:"icon-toggle-accept",r#type:"button",onclick:move|_|{controlled.accept.set(true);controlled.command();},"Accept requests"}
                    button {id:"icon-toggle-reject",r#type:"button",onclick:move|_|{controlled.accept.set(false);controlled.command();},"Reject requests"}
                    button {id:"icon-toggle-external-true",r#type:"button",onclick:move|_|{controlled.pressed.set(true);controlled.command();},"Set pressed externally"}
                    button {id:"icon-toggle-external-false",r#type:"button",onclick:move|_|{controlled.pressed.set(false);controlled.command();},"Set unpressed externally"}
                    button {id:"icon-toggle-reset",r#type:"button",onclick:move|_|controlled.reset(),"Reset controlled example"}
                    label {class:"icon-toggle-check",input {id:"icon-toggle-controlled-disabled",r#type:"checkbox",checked:(controlled.disabled)(),onchange:move|event|{controlled.disabled.set(event.checked());controlled.command();}} "Disable controlled button"}
                    label {class:"icon-toggle-check",input {id:"icon-toggle-controlled-loading",r#type:"checkbox",checked:(controlled.loading)(),onchange:move|event|{controlled.loading.set(event.checked());controlled.command();}} "Show controlled loading"}
                    output {role:"status","Mode: " if (controlled.accept)(){"accept"}else{"reject"} "; value: " if (controlled.pressed)(){"pressed"}else{"unpressed"} "; {(controlled.requests)().len()} requests"}
                    input {id:"icon-toggle-controlled-json",r#type:"hidden",value:controlled.snapshot()}
                }
                section {id:"icon-toggle-brand-fallback",class:"icon-toggle-fallback",
                    h3 {"Existing Brand fallback"}
                    p {"No test tokens are supplied here. Sizes and selected colors remain a bounded public-source fallback."}
                    for density in[Density::Normal,Density::Compact]{h4 {"{density.as_str()}"}
                        IxTheme {scheme:ColorScheme::Light,density,
                            div {class:"icon-toggle-fallback-row",for size in[ButtonIconSize::Twelve,ButtonIconSize::Sixteen,ButtonIconSize::Twenty,ButtonIconSize::TwentyFour]{for oval in[false,true]{for pressed in[false,true]{
                                div {class:"icon-toggle-fallback-sample",span {class:"icon-toggle-label","{size.as_str()} / " if oval{"oval"}else{"square"} " / " if pressed{"pressed"}else{"unpressed"}} IconToggleButton {icon:"add",label:"Toggle",variant:ButtonVariant::Primary,size,oval,pressed,onpressedchange:move|_|{}}}
                            }}}}
                            div {class:"icon-toggle-fallback-row",span {class:"icon-toggle-label","Loading / 16"} IconToggleButton {icon:"add",label:"Toggle",size:ButtonIconSize::Sixteen,loading:true,onpressedchange:move|_|{}}}
                        }
                    }
                }
                pre {id:"icon-toggle-result","aria-hidden":"true",class:"icon-toggle-result"}
            }
        }
    }
}

#[component]
fn Sample(
    row_id: String,
    kind: String,
    sample_key: String,
    #[props(default=ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default)] size: ButtonIconSize,
    #[props(default)] oval: bool,
    #[props(default)] pressed: bool,
    #[props(default="rest".to_string())] state: String,
    #[props(default)] controlled: Option<Controlled>,
) -> Element {
    let actual_pressed = controlled.map(|model| (model.pressed)()).unwrap_or(pressed);
    let disabled = controlled
        .map(|model| (model.disabled)())
        .unwrap_or(state == "disabled");
    let loading = controlled
        .map(|model| (model.loading)())
        .unwrap_or(state == "loading" || sample_key == "empty-loading12");
    let default = sample_key == "default";
    let icon = if matches!(sample_key.as_str(), "empty-loading12" | "empty-ready") {
        ""
    } else {
        "add"
    };
    let class = if sample_key.starts_with("override") {
        format!("icon-toggle-{sample_key}")
    } else {
        match sample_key.as_str() {
            "icon-color" => "icon-toggle-icon-color",
            "danger-pressed-override" => "icon-toggle-danger-pressed-override",
            "host-active-selected" => "active hover",
            _ => match state.as_str() {
                "hover" => "hover",
                "active" => "ix-toggle-active",
                "disabled" | "loading" => "hover ix-toggle-active",
                _ => "",
            },
        }
        .to_string()
    };
    let request = move |value: bool| {
        if let Some(model) = controlled {
            model.request(value);
        }
    };
    rsx! {div {id:row_id,"data-icon-toggle-row":kind,"data-key":sample_key.clone(),"data-variant":if default{"subtle-primary"}else{variant.as_str()},"data-size":size.as_str(),"data-oval":oval.to_string(),"data-pressed":actual_pressed.to_string(),"data-disabled":disabled.to_string(),"data-loading":loading.to_string(),class:"icon-toggle-sample",
        code {class:"icon-toggle-label","{sample_key} / " if actual_pressed{"pressed"}else{"unpressed"}}
        if default {IconToggleButton {icon,label:"Toggle",disabled,loading,class,onpressedchange:request}}
        else {IconToggleButton {icon,label:"Toggle",variant,size,oval,pressed:actual_pressed,disabled,loading,class,onpressedchange:request}}
    }}
}

pub const STYLES: &str = r#"
#icon-toggle-review .ix-theme{min-height:0}.icon-toggle-review-shell #icon-toggle-review{box-sizing:border-box;max-width:1440px;min-width:0;padding:clamp(16px,3vw,32px);margin:0 auto}.icon-toggle-intro h2{font:600 28px/1.25 system-ui,sans-serif;margin:0 0 12px}.icon-toggle-intro p,.icon-toggle-fallback p{max-width:72ch;line-height:1.5;margin:0 0 24px}.icon-toggle-controls{display:flex;align-items:end;flex-wrap:wrap;gap:12px 16px;min-width:0;padding:16px;margin:0 0 32px;border:1px solid #718391}.icon-toggle-controls legend{font-weight:600;padding:0 6px}.icon-toggle-controls label{display:grid;gap:6px}.icon-toggle-controls>.icon-toggle-check{display:flex;align-items:center;gap:8px;min-height:36px}.icon-toggle-controls>button,.icon-toggle-controls select{font:inherit;min-height:36px;padding:6px 10px;color:#253d52;background:#fff;border:1px solid #718391;border-radius:4px}.icon-toggle-controls>button:hover{background:#eaf0f4}.icon-toggle-controls :is(button,select,input):focus-visible{outline:2px solid #125c8c;outline-offset:3px}.icon-toggle-controls output,.icon-toggle-consumer-controls>p{flex-basis:100%;font-size:13px;color:#253d52}.icon-toggle-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}.icon-toggle-matrix>section{min-width:0;border-top:1px solid #a6b4bf}.icon-toggle-scope-title{font:600 20px/1.3 system-ui,sans-serif;margin:0;padding:16px;color:#253d52;background:#fff}.icon-toggle-samples{padding:0 12px 12px;display:grid;gap:4px;background:#fff}.icon-toggle-sample{display:flex;align-items:center;justify-content:space-between;gap:16px;min-width:0;padding:8px 4px;border-bottom:1px solid #dbe2e7;background:#fff}.icon-toggle-label{font:12px/1.5 ui-monospace,monospace;color:#253d52;background:#fff;overflow-wrap:anywhere}.icon-toggle-extras,.icon-toggle-fallback{margin-top:36px}.icon-toggle-extras>h3,.icon-toggle-fallback>h3{font-size:22px;line-height:1.3;margin:0 0 16px;color:#253d52}.icon-toggle-extra-grid{padding:12px;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:0 24px;background:#fff}.icon-toggle-consumer-controls{margin-top:24px}.icon-toggle-consumer-controls>p{margin:0}.icon-toggle-fallback-row{display:flex;align-items:end;gap:16px;flex-wrap:wrap;padding:16px;background:#fff}.icon-toggle-fallback-sample{display:grid;gap:8px;justify-items:center}.icon-toggle-fallback h4{color:#253d52}.icon-toggle-result{max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px}.icon-toggle-review-shell ::selection{background:#bedbe9;color:#132e41}@media(max-width:760px){.icon-toggle-matrix,.icon-toggle-extra-grid{grid-template-columns:minmax(0,1fr)}.icon-toggle-sample{gap:12px}}
"#;

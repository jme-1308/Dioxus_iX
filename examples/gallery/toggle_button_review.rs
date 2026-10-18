//! Shared controlled ToggleButton gallery/native semantic-token fixture.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};

pub const VARIANTS: [(&str, ToggleButtonVariant); 6] = [
    ("primary", ToggleButtonVariant::Primary),
    ("secondary", ToggleButtonVariant::Secondary),
    ("tertiary", ToggleButtonVariant::Tertiary),
    ("subtle-primary", ToggleButtonVariant::SubtlePrimary),
    ("subtle-secondary", ToggleButtonVariant::SubtleSecondary),
    ("subtle-tertiary", ToggleButtonVariant::SubtleTertiary),
];
pub const STATES: [&str; 6] = ["rest", "hover", "active", "disabled", "loading", "focus"];
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
        ("size-80", 32 - 8 * (scope % 2) + 4 * revision),
        ("size-120", 64 + 2 * scope + 4 * revision),
        ("spacing-x-40", 10 + scope + revision),
        ("spacing-x-20", 4 + scope + revision),
        ("border-radius-xs", 3 + scope + revision),
        ("border-width-default", 1 + revision),
        ("border-width-selected", 2 + revision),
        ("border-width-emphasis", 3 + revision),
        ("focus-ring-offset", 3 + revision),
        ("size-40", 12),
        ("size-50", 16),
        ("size-60", 20 + scope + revision),
        ("size-130", 40),
        ("icon-lg", 22 + 2 * scope + 2 * revision),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,0.25+0.25*revision as f64));
    style
}
fn override_rules(revision: usize) -> String {
    format!(
        "#toggle-review .toggle-component-override{{--ix-toggle-button--height:{}px;--ix-toggle-button--min-width:{}px;--ix-toggle-button-icon-right--icon-size:{}px;--ix-button-padding:2px 18px;--ix-button-border-radius-left:{}px;--ix-button-border-radius-right:{}px;--ix-button-secondary--background--pressed:rgb(39,83,127);--ix-button-secondary--color--pressed:rgb(247,249,251);}}",
        44 + 4 * revision,
        110 + 4 * revision,
        30 + 2 * revision,
        2 + revision,
        9 + revision
    )
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

#[component]
pub fn ToggleButtonReview() -> Element {
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
        IxTheme {scheme:ColorScheme::Light,class:"toggle-review-shell",
            section {id:"toggle-review","data-phase":phase(),"data-revision":revision().to_string(),"data-state":state(),"data-supplied":supplied().to_string(),"data-consumer-command":(controlled.commands)().to_string(),
                header {class:"toggle-review-intro",
                    h2 {"Toggle button selected states"}
                    p {"Compare pressed and unpressed controls across six variants and four theme scopes. Test tokens exercise integration; their colors and measurements are not Brand or accessibility claims."}
                }
                fieldset {class:"toggle-review-controls",
                    legend {"Shared preview controls"}
                    label {"Visual state",select {id:"toggle-state",value:state(),onchange:move|event|state.set(event.value()),for value in STATES{option {value,selected:state()==value,"{value}"}}}}
                    label {class:"toggle-review-check",input {id:"toggle-supplied",r#type:"checkbox",checked:supplied(),onchange:move|event|supplied.set(event.checked())} "Supply test tokens"}
                    for next in ["baseline","revised","restored"]{button {id:format!("toggle-{next}"),r#type:"button",onclick:move|_|{revision.set(usize::from(next=="revised"));phase.set(next.to_string());state.set("rest".into());supplied.set(true);},match next{"baseline"=>"Baseline","revised"=>"Revise tokens",_=>"Restore tokens"}}}
                    output {id:"toggle-status",role:"status","aria-live":"polite","{phase()} / {state()} — 48 controls, pressed and unpressed"}
                    button {id:"toggle-outside",r#type:"button","Keyboard target"}
                }
                div {class:"toggle-review-matrix",
                    for(scope,index,scheme,density)in[("light-normal",0,ColorScheme::Light,Density::Normal),("light-compact",1,ColorScheme::Light,Density::Compact),("dark-normal",2,ColorScheme::Dark,Density::Normal),("dark-compact",3,ColorScheme::Dark,Density::Compact)]{
                        section {id:format!("toggle-scope-{scope}"),"data-toggle-scope":scope,"data-scope-index":index.to_string(),"data-revision":revision().to_string(),
                            h3 {class:"toggle-review-scope-title","{scheme.as_str()} / {density.as_str()}"}
                            IxTheme {scheme,density,style:if supplied(){sentinel_style(index,revision())}else{String::new()},
                                div {class:"toggle-review-samples",for(name,variant)in VARIANTS{for pressed in[false,true]{
                                    Sample {row_id:format!("toggle-{scope}-{name}-{}",if pressed{"on"}else{"off"}),kind:"core",sample_key:format!("{name}-{}",if pressed{"on"}else{"off"}),variant,pressed,state:state()}
                                }}}
                            }
                        }
                    }
                }
                section {class:"toggle-review-extras","data-toggle-scope":"light-normal","data-scope-index":"0","data-revision":revision().to_string(),
                    h3 {"Icons, component overrides and nested scope"}
                    IxTheme {scheme:ColorScheme::Light,style:if supplied(){sentinel_style(0,revision())}else{String::new()},
                        div {class:"toggle-review-extra-grid",
                            for key in["default","icon-start","icon-end","icon-both","loading-icons","empty-icons","component-override","host-active-selected"]{
                                Sample {row_id:format!("toggle-extra-{key}"),kind:"extra",sample_key:key,variant:if key=="component-override"{ToggleButtonVariant::Secondary}else{ToggleButtonVariant::Primary},pressed:matches!(key,"loading-icons"|"component-override"|"host-active-selected")}
                            }
                            div {"data-toggle-scope":"nested","data-scope-index":"3","data-revision":"0",
                                IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),Sample {row_id:"toggle-extra-nested",kind:"extra",sample_key:"nested",variant:ToggleButtonVariant::Secondary,pressed:true}}
                            }
                            Sample {row_id:"toggle-extra-controlled",kind:"extra",sample_key:"controlled",controlled:Some(controlled)}
                        }
                    }
                }
                fieldset {class:"toggle-review-controls toggle-consumer-controls",
                    legend {"Controlled request consumer"}
                    p {"Requests do not change the pressed value until this consumer accepts them."}
                    button {id:"toggle-accept",r#type:"button",onclick:move|_|{controlled.accept.set(true);controlled.command();},"Accept requests"}
                    button {id:"toggle-reject",r#type:"button",onclick:move|_|{controlled.accept.set(false);controlled.command();},"Reject requests"}
                    button {id:"toggle-external-true",r#type:"button",onclick:move|_|{controlled.pressed.set(true);controlled.command();},"Set pressed externally"}
                    button {id:"toggle-external-false",r#type:"button",onclick:move|_|{controlled.pressed.set(false);controlled.command();},"Set unpressed externally"}
                    button {id:"toggle-reset",r#type:"button",onclick:move|_|controlled.reset(),"Reset controlled example"}
                    label {class:"toggle-review-check",input {id:"toggle-controlled-disabled",r#type:"checkbox",checked:(controlled.disabled)(),onchange:move|event|{controlled.disabled.set(event.checked());controlled.command();}} "Disable controlled button"}
                    label {class:"toggle-review-check",input {id:"toggle-controlled-loading",r#type:"checkbox",checked:(controlled.loading)(),onchange:move|event|{controlled.loading.set(event.checked());controlled.command();}} "Show controlled loading"}
                    output {role:"status","Mode: " if (controlled.accept)(){"accept"}else{"reject"} "; value: " if (controlled.pressed)(){"pressed"}else{"unpressed"} "; {(controlled.requests)().len()} requests"}
                    input {id:"toggle-controlled-json",r#type:"hidden",value:controlled.snapshot()}
                }
                section {id:"toggle-brand-fallback",class:"toggle-review-fallback",
                    h3 {"Existing Brand fallback"}
                    p {"No test tokens are supplied here. Toggle values stay controlled; the remaining Brand metrics are provisional."}
                    for density in[Density::Normal,Density::Compact]{h4 {"{density.as_str()}"}
                        IxTheme {scheme:ColorScheme::Light,density,
                            div {class:"toggle-review-fallback-row",for(name,variant)in VARIANTS{for pressed in[false,true]{
                                div {class:"toggle-review-fallback-sample",span {class:"toggle-review-label","{name} / " if pressed{"pressed"}else{"unpressed"}} ToggleButton {variant,pressed,aria_label:"Toggle",onpressedchange:move|_|{},"Toggle"}}
                            }}}
                        }
                    }
                }
                pre {id:"toggle-result","aria-hidden":"true",class:"toggle-review-result"}
            }
        }
    }
}

#[component]
fn Sample(
    row_id: String,
    kind: String,
    sample_key: String,
    #[props(default=ToggleButtonVariant::Primary)] variant: ToggleButtonVariant,
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
        .unwrap_or(state == "loading" || sample_key == "loading-icons");
    let core = kind == "core";
    let default = sample_key == "default";
    let prefix = core
        || matches!(
            sample_key.as_str(),
            "icon-start" | "icon-both" | "loading-icons" | "component-override"
        );
    let suffix = matches!(
        sample_key.as_str(),
        "icon-end" | "icon-both" | "loading-icons" | "component-override"
    );
    let icon = if sample_key == "empty-icons" {
        Some(String::new())
    } else {
        prefix.then(|| "add".to_string())
    };
    let icon_right = if sample_key == "empty-icons" {
        Some(String::new())
    } else {
        suffix.then(|| "add".to_string())
    };
    let class = if sample_key == "component-override" {
        "toggle-component-override"
    } else if sample_key == "host-active-selected" {
        "active hover"
    } else {
        match state.as_str() {
            "hover" => "hover",
            "active" => "ix-toggle-active",
            "disabled" | "loading" => "hover ix-toggle-active",
            _ => "",
        }
    };
    let request = move |value: bool| {
        if let Some(model) = controlled {
            model.request(value);
        }
    };
    rsx! {div {id:row_id,"data-toggle-row":kind,"data-key":sample_key.clone(),"data-variant":if default{"subtle-primary"}else{variant.as_str()},"data-pressed":actual_pressed.to_string(),"data-disabled":disabled.to_string(),"data-loading":loading.to_string(),class:"toggle-review-sample",
        code {class:"toggle-review-label","{sample_key} / " if actual_pressed{"pressed"}else{"unpressed"}}
        if default {ToggleButton {aria_label:"Toggle",disabled,loading,icon,icon_right,class,onpressedchange:request,"Toggle"}}
        else {ToggleButton {variant,pressed:actual_pressed,aria_label:"Toggle",disabled,loading,icon,icon_right,class,onpressedchange:request,"Toggle"}}
    }}
}

pub const STYLES: &str = r#"
#toggle-review .ix-theme{min-height:0}.toggle-review-shell #toggle-review{box-sizing:border-box;max-width:1440px;min-width:0;padding:clamp(16px,3vw,32px);margin:0 auto}.toggle-review-intro h2{font:600 28px/1.25 system-ui,sans-serif;margin:0 0 12px}.toggle-review-intro p,.toggle-review-fallback p{max-width:72ch;line-height:1.5;margin:0 0 24px}.toggle-review-controls{display:flex;align-items:end;flex-wrap:wrap;gap:12px 16px;min-width:0;padding:16px;margin:0 0 32px;border:1px solid #718391}.toggle-review-controls legend{font-weight:600;padding:0 6px}.toggle-review-controls label{display:grid;gap:6px}.toggle-review-controls>.toggle-review-check{display:flex;align-items:center;gap:8px;min-height:36px}.toggle-review-controls>button,.toggle-review-controls select{font:inherit;min-height:36px;padding:6px 10px;color:#253d52;background:#fff;border:1px solid #718391;border-radius:4px}.toggle-review-controls>button:hover{background:#eaf0f4}.toggle-review-controls :is(button,select,input):focus-visible{outline:2px solid #125c8c;outline-offset:3px}.toggle-review-controls output,.toggle-consumer-controls>p{flex-basis:100%;font-size:13px;color:#253d52}.toggle-review-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}.toggle-review-matrix>section{min-width:0;border-top:1px solid #a6b4bf}.toggle-review-scope-title{font:600 20px/1.3 system-ui,sans-serif;margin:0;padding:16px;color:#253d52;background:#fff}.toggle-review-samples{padding:0 12px 12px;display:grid;gap:4px;background:#fff}.toggle-review-sample{display:flex;align-items:center;justify-content:space-between;gap:16px;min-width:0;padding:8px 4px;border-bottom:1px solid #dbe2e7;background:#fff}.toggle-review-label{font:12px/1.5 ui-monospace,monospace;color:#253d52;background:#fff;overflow-wrap:anywhere}.toggle-review-extras,.toggle-review-fallback{margin-top:36px}.toggle-review-extras>h3,.toggle-review-fallback>h3{font-size:22px;line-height:1.3;margin:0 0 16px;color:#253d52}.toggle-review-extra-grid{padding:12px;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:0 24px;background:#fff}.toggle-consumer-controls{margin-top:24px}.toggle-consumer-controls>p{margin:0}.toggle-review-fallback-row{display:flex;align-items:end;gap:16px;flex-wrap:wrap;padding:16px;background:#fff}.toggle-review-fallback-sample{display:grid;gap:8px;justify-items:center}.toggle-review-fallback h4{color:#253d52}.toggle-review-result{max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px}.toggle-review-shell ::selection{background:#bedbe9;color:#132e41}@media(max-width:760px){.toggle-review-matrix,.toggle-review-extra-grid{grid-template-columns:minmax(0,1fr)}.toggle-review-sample{gap:12px}}
"#;

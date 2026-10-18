//! Shared IconButton gallery/native system-token fixture.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const STATES: [&str; 6] = ["rest", "hover", "active", "disabled", "loading", "focus"];
pub const CORE: [(&str, ButtonVariant, ButtonIconSize, bool); 16] = [
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
    style.push_str(&format!("--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;--icon-review-ink:rgb(183,{},91);",14+scope+revision,20+scope+revision,0.25+0.25*revision as f64,71+3*revision));
    style
}
fn override_rules(revision: usize) -> String {
    let mut rules = String::new();
    for (key, token, base) in [
        ("override12", "extra-small", 23),
        ("override16", "small", 33),
        ("override20", "large", 44),
    ] {
        rules.push_str(&format!("#icon-button-review .icon-button-{key}{{--ix-icon-button-{token}--size:{}px;--ix-button-border-radius-left:{}px;--ix-button-border-radius-right:{}px;--ix-icon-button-color:rgb(73,119,161);}}",base+2*revision,2+revision,9+revision));
    }
    rules
}

#[component]
pub fn IconButtonReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut state = use_signal(|| "rest".to_string());
    let mut supplied = use_signal(|| true);
    let activations = use_signal(|| 0_usize);
    rsx! {
        style {{STYLES}}
        style {{override_rules(revision())}}
        IxTheme {scheme:ColorScheme::Light,class:"icon-button-review-shell",
            section {id:"icon-button-review","data-phase":phase(),"data-revision":revision().to_string(),"data-state":state(),"data-supplied":supplied().to_string(),"data-activations":activations().to_string(),
                header {class:"icon-review-intro",
                    h2 {"Icon button sizing and states"}
                    p {"Compare four glyph sizes, square and oval controls, and all nine variants. Supplied test tokens deliberately differ by scope; these are integration inputs, not Brand color or accessibility claims."}
                }
                fieldset {class:"icon-review-controls",
                    legend {"Shared preview controls"}
                    label {"Visual state"
                        select {id:"icon-button-state",value:state(),onchange:move |event|state.set(event.value()),for value in STATES {option {value,selected:state()==value,"{value}"}}}
                    }
                    label {class:"icon-review-check",input {id:"icon-button-supplied",r#type:"checkbox",checked:supplied(),onchange:move |event|supplied.set(event.checked())} "Supply test tokens"}
                    for next in ["baseline","revised","restored"] {button {id:format!("icon-button-{next}"),r#type:"button",onclick:move |_|{revision.set(usize::from(next=="revised"));phase.set(next.to_string());state.set("rest".into());supplied.set(true);},match next {"baseline"=>"Baseline","revised"=>"Revise tokens",_=>"Restore"}}}
                    output {id:"icon-button-status",role:"status","aria-live":"polite","{phase()} / {state()} — 64 preview controls; {activations()} activations"}
                    button {id:"icon-button-outside",r#type:"button","Keyboard target"}
                }
                div {class:"icon-review-matrix",
                    for (scope,index,scheme,density) in [("light-normal",0,ColorScheme::Light,Density::Normal),("light-compact",1,ColorScheme::Light,Density::Compact),("dark-normal",2,ColorScheme::Dark,Density::Normal),("dark-compact",3,ColorScheme::Dark,Density::Compact)] {
                        section {id:format!("icon-button-scope-{scope}"),"data-icon-scope":scope,"data-scope-index":index.to_string(),"data-revision":revision().to_string(),"data-scheme":scheme.as_str(),"data-density":density.as_str(),
                            h3 {class:"icon-review-scope-title","{scheme.as_str()} / {density.as_str()}"}
                            IxTheme {scheme,density,style:if supplied(){sentinel_style(index,revision())}else{String::new()},
                                div {class:"icon-review-samples",for (key,variant,size,oval) in CORE {
                                    Sample {row_id:format!("icon-button-{scope}-{key}"),kind:"core",sample_key:key,variant,size,oval,state:state(),activations}
                                }}
                            }
                        }
                    }
                }
                section {id:"icon-button-extras",class:"icon-review-extras","data-icon-scope":"light-normal","data-scope-index":"0","data-revision":revision().to_string(),
                    h3 {"Component overrides and nested scope"}
                    IxTheme {scheme:ColorScheme::Light,style:if supplied(){sentinel_style(0,revision())}else{String::new()},
                        div {class:"icon-review-extra-grid",
                            for key in ["default","override12","override16","override20","icon-color","loading-empty","host-active"] {
                                Sample {row_id:format!("icon-button-extra-{key}"),kind:"extra",sample_key:key,size:match key{"override12"|"loading-empty"=>ButtonIconSize::Twelve,"override16"=>ButtonIconSize::Sixteen,_=>ButtonIconSize::Twenty},activations}
                            }
                            div {"data-icon-scope":"nested","data-scope-index":"3","data-revision":"0",
                                IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),Sample {row_id:"icon-button-extra-nested",kind:"extra",sample_key:"nested",activations}}
                            }
                        }
                    }
                }
                section {id:"icon-button-brand-fallback",class:"icon-review-fallback",
                    h3 {"Existing Brand fallback"}
                    p {"No test tokens are supplied here. Fallback sizing and theme values remain a bounded public-source approximation."}
                    for density in [Density::Normal,Density::Compact] {
                        h4 {"{density.as_str()}"}
                        IxTheme {scheme:ColorScheme::Light,density,
                            div {class:"icon-review-fallback-row",
                                for size in [ButtonIconSize::Twelve,ButtonIconSize::Sixteen,ButtonIconSize::Twenty,ButtonIconSize::TwentyFour] {
                                    for oval in [false,true] {div {class:"icon-review-fallback-sample",span {class:"icon-review-label","{size.as_str()} / {oval}"} IconButton {icon:"add",label:"Add",size,oval}}}
                                }
                                div {class:"icon-review-fallback-sample",span {class:"icon-review-label","Loading / 16"} IconButton {icon:"add",label:"Adding",size:ButtonIconSize::Sixteen,loading:true}}
                            }
                        }
                    }
                }
                pre {id:"icon-button-result","aria-hidden":"true",class:"icon-review-result"}
            }
        }
    }
}

#[component]
fn Sample(
    row_id: String,
    kind: String,
    sample_key: String,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonIconSize,
    #[props(default)] oval: bool,
    #[props(default="rest".to_string())] state: String,
    mut activations: Signal<usize>,
) -> Element {
    let default = sample_key == "default";
    let loading = state == "loading" || sample_key == "loading-empty";
    let icon = if sample_key == "loading-empty" {
        ""
    } else {
        "add"
    };
    let icon_color = (sample_key == "icon-color").then(|| "--icon-review-ink".to_string());
    let class = if sample_key.starts_with("override") {
        format!("icon-button-{sample_key}")
    } else if sample_key == "host-active" {
        "active".to_string()
    } else {
        match state.as_str() {
            "hover" => "hover",
            "active" => "active",
            "disabled" | "loading" => "hover active",
            _ => "",
        }
        .to_string()
    };
    rsx! {div {id:row_id,"data-icon-row":kind,"data-key":sample_key.clone(),"data-variant":if default{"subtle-primary"}else{variant.as_str()},"data-size":size.as_str(),"data-oval":oval.to_string(),"data-icon-color":icon_color.clone(),"data-loading":loading.to_string(),"data-disabled":(state=="disabled").to_string(),class:"icon-review-sample",
        code {class:"icon-review-label","{sample_key}"}
        if default {IconButton {icon,label:"Add",disabled:state=="disabled",loading,oval,class,icon_color:icon_color.clone(),onclick:move |_|activations+=1}}
        else {IconButton {icon,label:"Add",variant,size,oval,disabled:state=="disabled",loading,class,icon_color:icon_color.clone(),onclick:move |_|activations+=1}}
    }}
}

pub const STYLES: &str = r#"
#icon-button-review .ix-theme{min-height:0}
.icon-button-review-shell #icon-button-review{box-sizing:border-box;max-width:1440px;min-width:0;padding:clamp(16px,3vw,32px);margin:0 auto}.icon-review-intro h2{font:600 28px/1.25 system-ui,sans-serif;margin:0 0 12px}.icon-review-intro p,.icon-review-fallback p{max-width:72ch;line-height:1.5;margin:0 0 24px}.icon-review-controls{display:flex;align-items:end;flex-wrap:wrap;gap:12px 16px;min-width:0;padding:16px;margin:0 0 32px;border:1px solid var(--ix-color-border)}.icon-review-controls legend{font-weight:600;padding:0 6px}.icon-review-controls label{display:grid;gap:6px}.icon-review-controls>.icon-review-check{display:flex;align-items:center;gap:8px;min-height:36px}.icon-review-controls>button,.icon-review-controls select{font:inherit;min-height:36px;padding:6px 10px;color:#253d52;background:#fff;border:1px solid #718391;border-radius:4px}.icon-review-controls>button:hover{background:#eaf0f4}.icon-review-controls :is(button,select,input):focus-visible{outline:2px solid #125c8c;outline-offset:3px}.icon-review-controls output{flex-basis:100%;font-size:13px;color:#253d52}.icon-review-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}.icon-review-matrix>section{min-width:0;border-top:1px solid #a6b4bf}.icon-review-scope-title{font:600 20px/1.3 system-ui,sans-serif;margin:0;padding:16px;color:#253d52;background:#fff}.icon-review-samples{padding:0 12px 12px;display:grid;gap:4px;background:#fff}.icon-review-sample{display:flex;align-items:center;justify-content:space-between;gap:16px;min-width:0;padding:8px 4px;border-bottom:1px solid #dbe2e7;background:#fff}.icon-review-label{font:12px/1.5 ui-monospace,monospace;color:#253d52;background:#fff;overflow-wrap:anywhere}.icon-review-extras,.icon-review-fallback{margin-top:36px}.icon-review-extras>h3,.icon-review-fallback>h3{font-size:22px;line-height:1.3;margin:0 0 16px;color:#253d52}.icon-review-extra-grid{padding:12px;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:0 24px;background:#fff}.icon-review-fallback-row{display:flex;align-items:end;gap:16px;flex-wrap:wrap;padding:16px;background:#fff}.icon-review-fallback-sample{display:grid;gap:8px;justify-items:center}.icon-review-fallback h4{color:#253d52}.icon-review-result{max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px}.icon-button-review-shell ::selection{background:#bedbe9;color:#132e41}@media(max-width:760px){.icon-review-matrix,.icon-review-extra-grid{grid-template-columns:minmax(0,1fr)}.icon-review-sample{gap:12px}}
"#;

//! Shared regular-Button gallery/native semantic-token fixture.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const VARIANTS: [(&str, ButtonVariant); 9] = [
    ("primary", ButtonVariant::Primary),
    ("secondary", ButtonVariant::Secondary),
    ("tertiary", ButtonVariant::Tertiary),
    ("subtle-primary", ButtonVariant::SubtlePrimary),
    ("subtle-secondary", ButtonVariant::SubtleSecondary),
    ("subtle-tertiary", ButtonVariant::SubtleTertiary),
    ("danger-primary", ButtonVariant::DangerPrimary),
    ("danger-secondary", ButtonVariant::DangerSecondary),
    ("danger-tertiary", ButtonVariant::DangerTertiary),
];
pub const STATES: [&str; 6] = ["rest", "hover", "active", "disabled", "loading", "focus"];
const COLOR_TOKENS: [&str; 28] = [
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
    for (index, token) in COLOR_TOKENS.iter().enumerate() {
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
        ("focus-ring-offset", 3 + revision),
        ("border-width-selected", 2 + revision),
        ("border-width-emphasis", 3 + revision),
        ("size-40", 12),
        ("size-50", 16),
        ("size-60", 20),
        ("size-130", 40),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",14+scope+revision,20+scope+revision,0.25+0.25*revision as f64));
    style
}
fn override_rules(revision: usize) -> String {
    format!(
        "#button-review .button-component-override{{--ix-button--height:{}px;--ix-button-padding:2px 18px;--ix-button-border-radius-left:{}px;--ix-button-border-radius-right:{}px;--ix-button-primary--background:rgb(31,81,123);--ix-button-primary--color:rgb(255,255,255);--ix-button-primary--border-color:rgb(11,41,73);}}",
        40 + 4 * revision,
        2 + revision,
        9 + revision
    )
}

#[component]
pub fn ButtonReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut state = use_signal(|| "rest".to_string());
    let mut supplied = use_signal(|| true);
    let activations = use_signal(|| 0_usize);
    let consumer_prevented = use_signal(|| None::<bool>);
    rsx! {
        style {{STYLES}}
        style {{override_rules(revision())}}
        IxTheme {scheme:ColorScheme::Light,class:"button-review-shell",
            section {id:"button-review","data-phase":phase(),"data-revision":revision().to_string(),"data-state":state(),"data-supplied":supplied().to_string(),"data-activations":activations().to_string(),"data-consumer-prevented":consumer_prevented().map(|value|value.to_string()),
                header {class:"button-review-intro",
                    h2 {"Button sizing and states"}
                    p {"Compare all regular Button variants across four theme scopes. Test colors and measurements are deliberately distinct system-token inputs, not Brand metrics. Hover and active previews use the shipped visual aliases."}
                }
                fieldset {class:"button-review-controls",
                    legend {"Shared preview controls"}
                    label {"Visual state"
                        select {id:"button-state",value:state(),onchange:move |event|state.set(event.value()),
                            for value in STATES {option {value,selected:state()==value,"{value}"}}
                        }
                    }
                    label {class:"button-review-check",input {id:"button-supplied",r#type:"checkbox",checked:supplied(),onchange:move |event|supplied.set(event.checked())} "Supply test tokens"}
                    for next in ["baseline","revised","restored"] {button {id:format!("button-{next}"),r#type:"button",onclick:move |_|{revision.set(usize::from(next=="revised"));phase.set(next.to_string());state.set("rest".into());supplied.set(true);},match next {"baseline"=>"Baseline","revised"=>"Revise tokens",_=>"Restore"}}}
                    output {id:"button-status","aria-live":"polite","{phase()} · {state()} · {activations()} activations"}
                    button {id:"button-outside",r#type:"button","Keyboard target"}
                }
                div {class:"button-scope-matrix",
                    for (scope,index,scheme,density) in [("light-normal",0,ColorScheme::Light,Density::Normal),("light-compact",1,ColorScheme::Light,Density::Compact),("dark-normal",2,ColorScheme::Dark,Density::Normal),("dark-compact",3,ColorScheme::Dark,Density::Compact)] {
                        section {id:format!("button-scope-{scope}"),"data-button-scope":scope,"data-scope-index":index.to_string(),"data-revision":revision().to_string(),"data-scheme":scheme.as_str(),"data-density":density.as_str(),
                            IxTheme {scheme,density,style:if supplied(){sentinel_style(index,revision())}else{String::new()},
                                h3 {class:"button-scope-title","{scheme.as_str()} / {density.as_str()}"}
                                div {class:"button-scope-content",
                                    for (name,variant) in VARIANTS {ButtonSample {row_id:format!("button-{scope}-{name}"),kind:"core",sample_key:name,variant,state:state(),activations,consumer_prevented}}
                                }
                            }
                        }
                    }
                }
                section {class:"button-extras",id:"button-extras","data-button-scope":"light-normal","data-scope-index":"0","data-revision":revision().to_string(),
                    h3 {"Icons, content and component overrides"}
                    IxTheme {scheme:ColorScheme::Light,style:if supplied(){sentinel_style(0,revision())}else{String::new()},
                        div {class:"button-extra-grid",
                            for key in ["anchor","icon-start","icon-end","icon-both","alignment-start","loading12","loading16","loading20","loading24","component-override","host-active"] {
                                ButtonSample {row_id:format!("button-extra-{key}"),kind:"extra",sample_key:key,activations,consumer_prevented}
                            }
                            div {"data-button-scope":"nested","data-scope-index":"3","data-revision":"0",
                                IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),
                                    ButtonSample {row_id:"button-extra-nested",kind:"extra",sample_key:"nested",activations,consumer_prevented}
                                }
                            }
                        }
                    }
                }
                section {class:"button-fallback",id:"button-brand-fallback",
                    h3 {"Existing Brand fallback"}
                    p {"No test tokens are supplied here. Normal and compact sizing use the current public fallback; complete Brand state values remain provisional."}
                    for density in [Density::Normal,Density::Compact] {
                        IxTheme {scheme:ColorScheme::Light,density,
                            h4 {"{density.as_str()}"}
                            div {class:"button-fallback-row",for (name,variant) in VARIANTS {Button {variant,"{name}"}}}
                        }
                    }
                }
                div {id:"button-token-anchor"}
                pre {id:"button-result","aria-label":"Button token native observations"}
            }
        }
    }
}

#[component]
fn ButtonSample(
    row_id: String,
    kind: String,
    sample_key: String,
    #[props(default)] variant: ButtonVariant,
    #[props(default="rest".to_string())] state: String,
    mut activations: Signal<usize>,
    mut consumer_prevented: Signal<Option<bool>>,
) -> Element {
    let extra = kind == "extra";
    let loading = state == "loading" || sample_key.starts_with("loading");
    let icon_size = match sample_key.as_str() {
        "loading12" => ButtonIconSize::Twelve,
        "loading16" => ButtonIconSize::Sixteen,
        "loading24" => ButtonIconSize::TwentyFour,
        _ => ButtonIconSize::Twenty,
    };
    let icon = (matches!(
        sample_key.as_str(),
        "icon-start" | "icon-both" | "alignment-start"
    ) || sample_key.starts_with("loading"))
    .then(|| "add".to_string());
    let icon_right =
        matches!(sample_key.as_str(), "icon-end" | "icon-both").then(|| "add".to_string());
    let href = (sample_key == "anchor").then(|| "#button-token-anchor".to_string());
    let anchor = href.is_some();
    let class = if sample_key == "component-override" {
        "button-component-override"
    } else if sample_key == "host-active" {
        "active"
    } else {
        match state.as_str() {
            "hover" => "hover",
            "active" => "active",
            "disabled" | "loading" => "hover active",
            _ => "",
        }
    };
    rsx! {div {id:row_id,"data-button-row":kind.clone(),"data-key":sample_key.clone(),"data-variant":variant.as_str(),"data-state":if extra{if loading{"loading"}else{"rest"}}else{state.as_str()},class:"button-sample",
        code {class:"button-sample-label","{sample_key}"}
        Button {variant,disabled:state=="disabled",loading,icon_size,icon,icon_right,href,class,rel:anchor.then(||"nofollow".to_string()),
            alignment:if sample_key=="alignment-start"{ButtonAlignment::Start}else{ButtonAlignment::Center},
            onclick:move |event:MouseEvent|{if anchor{event.prevent_default();consumer_prevented.set(Some(!event.default_action_enabled()));}activations+=1;},
            "Action"
        }
    }}
}

pub const STYLES: &str = include_str!("styles/button_review.css");

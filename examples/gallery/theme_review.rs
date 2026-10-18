//! Shared gallery/native fixture for externally supplied semantic typography.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const FORMATS: [(&str, TypographyFormat); 26] = [
    ("body", TypographyFormat::Body),
    ("body-sbold", TypographyFormat::BodySbold),
    ("body-paragraph", TypographyFormat::BodyParagraph),
    ("body-paragraph-sbold", TypographyFormat::BodyParagraphSbold),
    ("body-sm", TypographyFormat::BodySm),
    ("body-sm-sbold", TypographyFormat::BodySmSbold),
    ("body-lg", TypographyFormat::BodyLg),
    ("body-lg-sbold", TypographyFormat::BodyLgSbold),
    ("display", TypographyFormat::Display),
    ("display-sbold", TypographyFormat::DisplaySbold),
    ("display-lg", TypographyFormat::DisplayLg),
    ("display-lg-sbold", TypographyFormat::DisplayLgSbold),
    ("display-xl", TypographyFormat::DisplayXl),
    ("display-xl-sbold", TypographyFormat::DisplayXlSbold),
    ("display-xxl", TypographyFormat::DisplayXxl),
    ("display-xxl-sbold", TypographyFormat::DisplayXxlSbold),
    ("h6", TypographyFormat::H6),
    ("h5", TypographyFormat::H5),
    ("h5-bold", TypographyFormat::H5Bold),
    ("h4", TypographyFormat::H4),
    ("h3", TypographyFormat::H3),
    ("h2", TypographyFormat::H2),
    ("h1", TypographyFormat::H1),
    ("code", TypographyFormat::Code),
    ("code-sm", TypographyFormat::CodeSm),
    ("code-lg", TypographyFormat::CodeLg),
];
pub const COLORS: [(&str, TypographyColor); 16] = [
    ("alarm", TypographyColor::Alarm),
    ("alarm-contrast", TypographyColor::AlarmContrast),
    ("contrast", TypographyColor::Contrast),
    ("critical-contrast", TypographyColor::CriticalContrast),
    ("info-contrast", TypographyColor::InfoContrast),
    ("inv-contrast", TypographyColor::InvContrast),
    ("inv-soft", TypographyColor::InvSoft),
    ("inv-std", TypographyColor::InvStd),
    ("inv-weak", TypographyColor::InvWeak),
    ("neutral-contrast", TypographyColor::NeutralContrast),
    ("primary-contrast", TypographyColor::PrimaryContrast),
    ("soft", TypographyColor::Soft),
    ("std", TypographyColor::Std),
    ("success-contrast", TypographyColor::SuccessContrast),
    ("warning-contrast", TypographyColor::WarningContrast),
    ("weak", TypographyColor::Weak),
];
const COLOR_TOKENS: [&str; 12] = [
    "text-danger",
    "text-on-danger",
    "text-primary",
    "text-on-critical",
    "text-on-information",
    "text-inverse",
    "text-on-neutral",
    "text-on-accent",
    "text-secondary",
    "text-on-success",
    "text-on-warning",
    "text-disabled",
];

/// Deliberately distinct, developer-authored system-font inputs. These values
/// demonstrate scope plumbing; they are not Siemens Brand metrics or assets.
pub fn sentinel_style(scope: usize, revision: usize) -> String {
    let mut style = String::new();
    for (index, (name, _)) in FORMATS.iter().enumerate() {
        let size = 12 + 2 * (index % 8) + scope + 2 * revision;
        let weight = if name.contains("sbold") || *name == "h5-bold" {
            600
        } else {
            400
        };
        let family = if name.starts_with("code") {
            "ui-monospace, monospace"
        } else if revision == 0 {
            "system-ui, sans-serif"
        } else {
            "serif"
        };
        style.push_str(&format!(
            "--si-sys-typography-{name}:normal {weight} {size}px/{}px {family};",
            size + 8
        ));
    }
    for (index, token) in COLOR_TOKENS.iter().enumerate() {
        let red = 24 + 11 * index + 5 * scope + 3 * revision;
        let green = 32 + 7 * index + 3 * scope + 5 * revision;
        let blue = 48 + 5 * index + 7 * scope + 2 * revision;
        style.push_str(&format!(
            "--si-sys-color-{token}:rgb({red},{green},{blue});"
        ));
    }
    style.push_str(&format!(
        "--si-ref-typography-letter-spacing-normal:{}px;--si-ref-typography-font-weight-bold:{};",
        0.25 + 0.25 * revision as f64,
        700 + 100 * revision
    ));
    style
}

#[component]
pub fn ThemeReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut dark = use_signal(|| false);
    let mut compact = use_signal(|| false);
    let mut supplied = use_signal(|| true);
    let mut nested = use_signal(|| true);
    let mut reactive = use_signal(|| false);
    let mut phase = use_signal(|| "baseline".to_string());
    rsx! {
        style { {STYLES} }
        IxTheme {scheme:ColorScheme::Light,class:"typography-review-shell",
            section {id:"typography-review","data-theme-phase":phase(),"data-theme-revision":revision().to_string(),"data-theme-supplied":supplied().to_string(),
                header {class:"typo-intro",
                    h2 {"Typography and theme scopes"}
                    p {"Compare semantic text styles across light, dark, normal and compact scopes. Supplied test tokens use system fonts and intentionally distinct values; they do not represent Brand typography metrics."}
                }
                fieldset {class:"typo-controls",
                    legend {"Primary scope"}
                    label {"Color scheme"
                        select {id:"theme-scheme",value:if dark(){"dark"}else{"light"},onchange:move |event|dark.set(event.value()=="dark"),option {value:"light","Light"} option {value:"dark","Dark"}}
                    }
                    label {"Density"
                        select {id:"theme-density",value:if compact(){"compact"}else{"normal"},onchange:move |event|compact.set(event.value()=="compact"),option {value:"normal","Normal"} option {value:"compact","Compact"}}
                    }
                    label {class:"typo-check",input {id:"theme-tokens",r#type:"checkbox",checked:supplied(),onchange:move |event|supplied.set(event.checked())} "Supply test tokens"}
                    label {class:"typo-check",input {id:"theme-nested",r#type:"checkbox",checked:nested(),onchange:move |event|nested.set(event.checked())} "Show isolated nested scope"}
                    div {class:"typo-phase-controls",
                        for next in ["baseline","revised","reactive","restored"] {
                            button {id:format!("typography-{next}"),r#type:"button",onclick:move |_| {
                                revision.set(usize::from(matches!(next,"revised"|"reactive")));
                                dark.set(next=="reactive"); compact.set(next=="reactive");
                                supplied.set(true);nested.set(true);reactive.set(next=="reactive");phase.set(next.to_string());
                            },match next {"baseline"=>"Baseline","revised"=>"Revise tokens","reactive"=>"Change text properties",_=>"Restore"}}
                        }
                    }
                    output {id:"typography-status","aria-live":"polite","Token revision {revision()}; phase {phase()}"}
                }
                div {class:"typo-scope-matrix",
                    ThemeScope {scope:"light-normal",scope_index:usize::from(dark())*2+usize::from(compact()),scheme:if dark(){ColorScheme::Dark}else{ColorScheme::Light},density:if compact(){Density::Compact}else{Density::Normal},revision:revision(),supplied:supplied(),reactive:reactive(),nested:nested()}
                    ThemeScope {scope:"light-compact",scope_index:1,scheme:ColorScheme::Light,density:Density::Compact,revision:revision(),supplied:supplied(),reactive:reactive()}
                    ThemeScope {scope:"dark-normal",scope_index:2,scheme:ColorScheme::Dark,density:Density::Normal,revision:revision(),supplied:supplied(),reactive:reactive()}
                    ThemeScope {scope:"dark-compact",scope_index:3,scheme:ColorScheme::Dark,density:Density::Compact,revision:revision(),supplied:supplied(),reactive:reactive()}
                }
                section {class:"typo-fallback",id:"typography-brand-fallback",
                    h3 {"Existing Brand fallback"}
                    p {"No test tokens are supplied to this scope. Its current fallback metrics remain provisional."}
                    IxTheme {scheme:if dark(){ColorScheme::Dark}else{ColorScheme::Light},density:if compact(){Density::Compact}else{Density::Normal},
                        div {class:"typo-fallback-content",
                            Typography {format:TypographyFormat::H2,"Review operating conditions"}
                            Typography {"Status, instructions and supporting text stay readable when supplied semantic tokens are absent."}
                            Typography {text_color:TypographyColor::Soft,format:TypographyFormat::BodySm,"Secondary information"}
                            Button {"Review settings"}
                        }
                    }
                }
                pre {id:"typography-result",class:"typo-result","aria-label":"Typography native observations"}
            }
        }
    }
}

#[component]
fn ThemeScope(
    scope: String,
    scope_index: usize,
    scheme: ColorScheme,
    density: Density,
    revision: usize,
    supplied: bool,
    reactive: bool,
    #[props(default)] nested: bool,
) -> Element {
    let style = if supplied {
        sentinel_style(scope_index, revision)
    } else {
        String::new()
    };
    rsx! {section {id:format!("typo-scope-{scope}"),"data-theme-scope":scope.clone(),"data-scope-index":scope_index.to_string(),"data-revision":revision.to_string(),"data-scheme":scheme.as_str(),"data-density":density.as_str(),"data-supplied":supplied.to_string(),class:"typo-scope",
        IxTheme {scheme,density,style,
            h3 {class:"typo-scope-title","{scheme.as_str()} / {density.as_str()}"}
            div {class:"typo-proof-surface",
                for (name, format) in FORMATS {Sample {scope:scope.clone(),kind:"format",sample_key:name,format}}
                h4 {"Semantic colors"}
                for (name, color) in COLORS {Sample {scope:scope.clone(),kind:"color",sample_key:name,text_color:Some(color)}}
                h4 {"Inheritance and text properties"}
                Sample {scope:scope.clone(),kind:"extra",sample_key:"inherited"}
                Sample {scope:scope.clone(),kind:"extra",sample_key:"bold",bold:true}
                Sample {scope:scope.clone(),kind:"extra",sample_key:"underline",text_decoration:TypographyDecoration::Underline}
                Sample {scope:scope.clone(),kind:"extra",sample_key:"line-through",text_decoration:TypographyDecoration::LineThrough}
                Sample {scope:scope.clone(),kind:"extra",sample_key:"default-format",default_format:true}
                Sample {scope:scope.clone(),kind:"extra",sample_key:"reactive",format:if reactive{TypographyFormat::H2}else{TypographyFormat::Body},text_color:if reactive{Some(TypographyColor::WarningContrast)}else{None},bold:reactive,text_decoration:if reactive{TypographyDecoration::Underline}else{TypographyDecoration::None}}
                if nested {section {"data-theme-scope":"nested-isolated","data-scope-index":"3","data-revision":"0","data-scheme":"dark","data-density":"compact","data-supplied":"true",class:"typo-nested",
                    h4 {"Nested scope — fixed tokens"}
                    IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,style:sentinel_style(3,0),
                        div {class:"typo-proof-surface",Sample {scope:"nested-isolated",kind:"nested",sample_key:"body",row_id:Some("typo-nested-isolated".to_string()),text_color:Some(TypographyColor::Soft)}}
                    }
                }}
            }
        }
    }}
}

#[component]
fn Sample(
    scope: String,
    kind: String,
    sample_key: String,
    #[props(default)] format: TypographyFormat,
    #[props(default)] text_color: Option<TypographyColor>,
    #[props(default)] bold: bool,
    #[props(default)] text_decoration: TypographyDecoration,
    #[props(default)] default_format: bool,
    #[props(default)] row_id: Option<String>,
) -> Element {
    let id = row_id.unwrap_or_else(|| format!("typo-{scope}-{kind}-{sample_key}"));
    let content = format!("Aa 0123 — {sample_key}");
    rsx! {div {id,"data-theme-row":"","data-theme-kind":kind.clone(),"data-theme-key":sample_key.clone(),"data-theme-format":format.as_str(),"data-theme-color":text_color.map(|value|value.as_str()),"data-theme-bold":bold.to_string(),"data-theme-decoration":text_decoration.as_str(),class:"typo-sample",
        code {class:"typo-sample-label","{sample_key}"}
        if default_format {Typography {text_color,bold,text_decoration,"{content}"}}
        else {Typography {format,text_color,bold,text_decoration,"{content}"}}
    }}
}

pub const STYLES: &str = r#"
.typography-review-shell{min-width:0}.typography-review-shell #typography-review{padding:clamp(16px,3vw,32px);max-width:1440px;margin:0 auto;min-width:0}.typography-review-shell .typo-intro h2{font:600 28px/1.25 system-ui,sans-serif;margin:0 0 12px}.typography-review-shell .typo-intro p{max-width:72ch;margin:0 0 24px;line-height:1.5}.typography-review-shell .typo-controls{display:flex;align-items:end;gap:16px;flex-wrap:wrap;padding:16px;margin:0 0 32px;border:1px solid var(--ix-color-border);min-width:0}.typography-review-shell .typo-controls legend{font-weight:600;padding:0 6px}.typography-review-shell .typo-controls label{display:grid;gap:6px;font-size:14px}.typography-review-shell .typo-controls select,.typography-review-shell .typo-phase-controls button{font:inherit;min-height:36px;border:1px solid var(--ix-color-border);background:var(--ix-color-surface);color:var(--ix-color-text);border-radius:var(--ix-radius);padding:6px 10px}.typography-review-shell .typo-controls .typo-check{display:flex;align-items:center;gap:8px;min-height:36px}.typography-review-shell .typo-phase-controls{display:flex;gap:8px;flex-wrap:wrap}.typography-review-shell .typo-controls output{flex-basis:100%;font-size:13px;color:var(--ix-color-text-secondary)}.typography-review-shell .typo-controls :is(button,select,input):focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.typography-review-shell .typo-phase-controls button:hover{background:var(--ix-color-surface-2)}.typography-review-shell .typo-scope-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:32px 24px}.typography-review-shell .typo-scope{min-width:0;border-top:1px solid var(--ix-color-border)}.typography-review-shell .typo-scope-title{font:600 20px/1.3 system-ui,sans-serif;margin:0;padding:16px}.typography-review-shell .typo-proof-surface{background:#fff;color:rgb(37,83,119);padding:12px;min-width:0}.typography-review-shell .typo-proof-surface h4{font:600 17px/1.3 system-ui,sans-serif;color:#253d52;margin:28px 0 12px}.typography-review-shell .typo-sample{display:grid;grid-template-columns:minmax(100px,32%) minmax(0,1fr);gap:12px;align-items:baseline;padding:8px 0;border-bottom:1px solid #e1e7eb;overflow-wrap:anywhere}.typography-review-shell .typo-sample-label{font:12px/1.5 ui-monospace,monospace;color:#435a6b}.typography-review-shell .typo-nested{margin-top:24px}.typography-review-shell .typo-fallback{margin-top:40px}.typography-review-shell .typo-fallback h3{font-size:22px}.typography-review-shell .typo-fallback-content{display:grid;gap:16px;padding:24px}.typography-review-shell .typo-result{white-space:pre-wrap;overflow-wrap:anywhere;max-height:200px;overflow:auto;font-size:12px}.typography-review-shell ::selection{background:#bedbe9;color:#132e41}@media(max-width:760px){.typography-review-shell .typo-scope-matrix{grid-template-columns:minmax(0,1fr)}.typography-review-shell .typo-sample{grid-template-columns:minmax(90px,30%) minmax(0,1fr)}}
"#;

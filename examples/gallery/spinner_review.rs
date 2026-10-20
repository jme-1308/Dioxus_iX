//! Shared development-only Spinner recipe fixture. Props and consumer state remain Rust.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::Value;
const SPEC: &str = r####"{"revision":"8bf599bc68365d7752e12832e71b625883b93d7a","fixture":"Pinned CSS Spinner recipe under authored system tokens; system fonts, no full Brand metrics","revisions":[{"id":"baseline","revision":0},{"id":"revised","revision":1}],"cases":[{"key":"spinner-s0-xxs-secondary","id":"spinner-spinner-s0-xxs-secondary","scopeIndex":0,"size":"xxs","variant":"secondary","hideTrack":false},{"key":"spinner-s0-xxs-primary","id":"spinner-spinner-s0-xxs-primary","scopeIndex":0,"size":"xxs","variant":"primary","hideTrack":false},{"key":"spinner-s0-xs-secondary","id":"spinner-spinner-s0-xs-secondary","scopeIndex":0,"size":"xs","variant":"secondary","hideTrack":false},{"key":"spinner-s0-xs-primary","id":"spinner-spinner-s0-xs-primary","scopeIndex":0,"size":"xs","variant":"primary","hideTrack":false},{"key":"spinner-s0-sm-secondary","id":"spinner-spinner-s0-sm-secondary","scopeIndex":0,"size":"sm","variant":"secondary","hideTrack":false},{"key":"spinner-s0-sm-primary","id":"spinner-spinner-s0-sm-primary","scopeIndex":0,"size":"sm","variant":"primary","hideTrack":false},{"key":"spinner-s0-md-secondary","id":"spinner-spinner-s0-md-secondary","scopeIndex":0,"size":"md","variant":"secondary","hideTrack":false},{"key":"spinner-s0-md-primary","id":"spinner-spinner-s0-md-primary","scopeIndex":0,"size":"md","variant":"primary","hideTrack":false},{"key":"spinner-s0-xxl-secondary","id":"spinner-spinner-s0-xxl-secondary","scopeIndex":0,"size":"xxl","variant":"secondary","hideTrack":false},{"key":"spinner-s0-xxl-primary","id":"spinner-spinner-s0-xxl-primary","scopeIndex":0,"size":"xxl","variant":"primary","hideTrack":false},{"key":"spinner-s1-xxs-secondary","id":"spinner-spinner-s1-xxs-secondary","scopeIndex":1,"size":"xxs","variant":"secondary","hideTrack":false},{"key":"spinner-s1-xxs-primary","id":"spinner-spinner-s1-xxs-primary","scopeIndex":1,"size":"xxs","variant":"primary","hideTrack":false},{"key":"spinner-s1-xs-secondary","id":"spinner-spinner-s1-xs-secondary","scopeIndex":1,"size":"xs","variant":"secondary","hideTrack":false},{"key":"spinner-s1-xs-primary","id":"spinner-spinner-s1-xs-primary","scopeIndex":1,"size":"xs","variant":"primary","hideTrack":false},{"key":"spinner-s1-sm-secondary","id":"spinner-spinner-s1-sm-secondary","scopeIndex":1,"size":"sm","variant":"secondary","hideTrack":false},{"key":"spinner-s1-sm-primary","id":"spinner-spinner-s1-sm-primary","scopeIndex":1,"size":"sm","variant":"primary","hideTrack":false},{"key":"spinner-s1-md-secondary","id":"spinner-spinner-s1-md-secondary","scopeIndex":1,"size":"md","variant":"secondary","hideTrack":false},{"key":"spinner-s1-md-primary","id":"spinner-spinner-s1-md-primary","scopeIndex":1,"size":"md","variant":"primary","hideTrack":false},{"key":"spinner-s1-xxl-secondary","id":"spinner-spinner-s1-xxl-secondary","scopeIndex":1,"size":"xxl","variant":"secondary","hideTrack":false},{"key":"spinner-s1-xxl-primary","id":"spinner-spinner-s1-xxl-primary","scopeIndex":1,"size":"xxl","variant":"primary","hideTrack":false},{"key":"spinner-s2-xxs-secondary","id":"spinner-spinner-s2-xxs-secondary","scopeIndex":2,"size":"xxs","variant":"secondary","hideTrack":false},{"key":"spinner-s2-xxs-primary","id":"spinner-spinner-s2-xxs-primary","scopeIndex":2,"size":"xxs","variant":"primary","hideTrack":false},{"key":"spinner-s2-xs-secondary","id":"spinner-spinner-s2-xs-secondary","scopeIndex":2,"size":"xs","variant":"secondary","hideTrack":false},{"key":"spinner-s2-xs-primary","id":"spinner-spinner-s2-xs-primary","scopeIndex":2,"size":"xs","variant":"primary","hideTrack":false},{"key":"spinner-s2-sm-secondary","id":"spinner-spinner-s2-sm-secondary","scopeIndex":2,"size":"sm","variant":"secondary","hideTrack":false},{"key":"spinner-s2-sm-primary","id":"spinner-spinner-s2-sm-primary","scopeIndex":2,"size":"sm","variant":"primary","hideTrack":false},{"key":"spinner-s2-md-secondary","id":"spinner-spinner-s2-md-secondary","scopeIndex":2,"size":"md","variant":"secondary","hideTrack":false},{"key":"spinner-s2-md-primary","id":"spinner-spinner-s2-md-primary","scopeIndex":2,"size":"md","variant":"primary","hideTrack":false},{"key":"spinner-s2-xxl-secondary","id":"spinner-spinner-s2-xxl-secondary","scopeIndex":2,"size":"xxl","variant":"secondary","hideTrack":false},{"key":"spinner-s2-xxl-primary","id":"spinner-spinner-s2-xxl-primary","scopeIndex":2,"size":"xxl","variant":"primary","hideTrack":false},{"key":"spinner-s3-xxs-secondary","id":"spinner-spinner-s3-xxs-secondary","scopeIndex":3,"size":"xxs","variant":"secondary","hideTrack":false},{"key":"spinner-s3-xxs-primary","id":"spinner-spinner-s3-xxs-primary","scopeIndex":3,"size":"xxs","variant":"primary","hideTrack":false},{"key":"spinner-s3-xs-secondary","id":"spinner-spinner-s3-xs-secondary","scopeIndex":3,"size":"xs","variant":"secondary","hideTrack":false},{"key":"spinner-s3-xs-primary","id":"spinner-spinner-s3-xs-primary","scopeIndex":3,"size":"xs","variant":"primary","hideTrack":false},{"key":"spinner-s3-sm-secondary","id":"spinner-spinner-s3-sm-secondary","scopeIndex":3,"size":"sm","variant":"secondary","hideTrack":false},{"key":"spinner-s3-sm-primary","id":"spinner-spinner-s3-sm-primary","scopeIndex":3,"size":"sm","variant":"primary","hideTrack":false},{"key":"spinner-s3-md-secondary","id":"spinner-spinner-s3-md-secondary","scopeIndex":3,"size":"md","variant":"secondary","hideTrack":false},{"key":"spinner-s3-md-primary","id":"spinner-spinner-s3-md-primary","scopeIndex":3,"size":"md","variant":"primary","hideTrack":false},{"key":"spinner-s3-xxl-secondary","id":"spinner-spinner-s3-xxl-secondary","scopeIndex":3,"size":"xxl","variant":"secondary","hideTrack":false},{"key":"spinner-s3-xxl-primary","id":"spinner-spinner-s3-xxl-primary","scopeIndex":3,"size":"xxl","variant":"primary","hideTrack":false},{"key":"spinner-hidden-xxs-secondary","id":"spinner-spinner-hidden-xxs-secondary","scopeIndex":0,"size":"xxs","variant":"secondary","hideTrack":true},{"key":"spinner-hidden-xxs-primary","id":"spinner-spinner-hidden-xxs-primary","scopeIndex":1,"size":"xxs","variant":"primary","hideTrack":true},{"key":"spinner-hidden-xs-secondary","id":"spinner-spinner-hidden-xs-secondary","scopeIndex":2,"size":"xs","variant":"secondary","hideTrack":true},{"key":"spinner-hidden-xs-primary","id":"spinner-spinner-hidden-xs-primary","scopeIndex":3,"size":"xs","variant":"primary","hideTrack":true},{"key":"spinner-hidden-sm-secondary","id":"spinner-spinner-hidden-sm-secondary","scopeIndex":0,"size":"sm","variant":"secondary","hideTrack":true},{"key":"spinner-hidden-sm-primary","id":"spinner-spinner-hidden-sm-primary","scopeIndex":1,"size":"sm","variant":"primary","hideTrack":true},{"key":"spinner-hidden-md-secondary","id":"spinner-spinner-hidden-md-secondary","scopeIndex":2,"size":"md","variant":"secondary","hideTrack":true},{"key":"spinner-hidden-md-primary","id":"spinner-spinner-hidden-md-primary","scopeIndex":3,"size":"md","variant":"primary","hideTrack":true},{"key":"spinner-hidden-xxl-secondary","id":"spinner-spinner-hidden-xxl-secondary","scopeIndex":0,"size":"xxl","variant":"secondary","hideTrack":true},{"key":"spinner-hidden-xxl-primary","id":"spinner-spinner-hidden-xxl-primary","scopeIndex":1,"size":"xxl","variant":"primary","hideTrack":true},{"key":"spinner-default","id":"spinner-spinner-default","scopeIndex":0,"size":"md","variant":"secondary","hideTrack":false,"omitted":true},{"key":"spinner-label","id":"spinner-spinner-label","scopeIndex":1,"size":"md","variant":"secondary","hideTrack":false,"label":"Preparing records"},{"key":"spinner-empty-label","id":"spinner-spinner-empty-label","scopeIndex":2,"size":"md","variant":"secondary","hideTrack":false,"label":""},{"key":"spinner-override","id":"spinner-spinner-override","scopeIndex":3,"size":"md","variant":"primary","hideTrack":false,"override":true},{"key":"spinner-button-color","id":"spinner-spinner-button-color","scopeIndex":0,"size":"md","variant":"secondary","hideTrack":false,"buttonColor":true},{"key":"spinner-live-nested","id":"spinner-spinner-live-nested","scopeIndex":2,"size":"md","variant":"secondary","hideTrack":false,"live":true,"nested":true}],"consumers":[{"key":"consumer-s0-button-20","id":"spinner-consumer-s0-button-20","scopeIndex":0,"type":"button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"primary"},{"key":"consumer-s0-icon-button-20","id":"spinner-consumer-s0-icon-button-20","scopeIndex":0,"type":"icon-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"primary"},{"key":"consumer-s0-toggle-button-20","id":"spinner-consumer-s0-toggle-button-20","scopeIndex":0,"type":"toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"subtle-primary"},{"key":"consumer-s0-icon-toggle-button-20","id":"spinner-consumer-s0-icon-toggle-button-20","scopeIndex":0,"type":"icon-toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"subtle-primary"},{"key":"consumer-s0-upload-loading","id":"spinner-consumer-s0-upload-loading","scopeIndex":0,"type":"upload","state":"loading","disabled":false,"loadingText":"Checking records…","width":520,"expectedSize":"md","expectedVariant":"primary","label":""},{"key":"consumer-s0-upload-disabled","id":"spinner-consumer-s0-upload-disabled","scopeIndex":0,"type":"upload","state":"loading","disabled":true,"loadingText":"Checking records…","width":520,"expectedSize":null,"expectedVariant":"primary","label":""},{"key":"consumer-s1-button-20","id":"spinner-consumer-s1-button-20","scopeIndex":1,"type":"button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"primary"},{"key":"consumer-s1-icon-button-20","id":"spinner-consumer-s1-icon-button-20","scopeIndex":1,"type":"icon-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"primary"},{"key":"consumer-s1-toggle-button-20","id":"spinner-consumer-s1-toggle-button-20","scopeIndex":1,"type":"toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"subtle-primary"},{"key":"consumer-s1-icon-toggle-button-20","id":"spinner-consumer-s1-icon-toggle-button-20","scopeIndex":1,"type":"icon-toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"subtle-primary"},{"key":"consumer-s1-upload-loading","id":"spinner-consumer-s1-upload-loading","scopeIndex":1,"type":"upload","state":"loading","disabled":false,"loadingText":"Checking records…","width":520,"expectedSize":"md","expectedVariant":"primary","label":""},{"key":"consumer-s1-upload-disabled","id":"spinner-consumer-s1-upload-disabled","scopeIndex":1,"type":"upload","state":"loading","disabled":true,"loadingText":"Checking records…","width":520,"expectedSize":null,"expectedVariant":"primary","label":""},{"key":"consumer-s2-button-20","id":"spinner-consumer-s2-button-20","scopeIndex":2,"type":"button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"primary"},{"key":"consumer-s2-icon-button-20","id":"spinner-consumer-s2-icon-button-20","scopeIndex":2,"type":"icon-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"primary"},{"key":"consumer-s2-toggle-button-20","id":"spinner-consumer-s2-toggle-button-20","scopeIndex":2,"type":"toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"subtle-primary"},{"key":"consumer-s2-icon-toggle-button-20","id":"spinner-consumer-s2-icon-toggle-button-20","scopeIndex":2,"type":"icon-toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"subtle-primary"},{"key":"consumer-s2-upload-loading","id":"spinner-consumer-s2-upload-loading","scopeIndex":2,"type":"upload","state":"loading","disabled":false,"loadingText":"Checking records…","width":520,"expectedSize":"md","expectedVariant":"primary","label":""},{"key":"consumer-s2-upload-disabled","id":"spinner-consumer-s2-upload-disabled","scopeIndex":2,"type":"upload","state":"loading","disabled":true,"loadingText":"Checking records…","width":520,"expectedSize":null,"expectedVariant":"primary","label":""},{"key":"consumer-s3-button-20","id":"spinner-consumer-s3-button-20","scopeIndex":3,"type":"button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"primary"},{"key":"consumer-s3-icon-button-20","id":"spinner-consumer-s3-icon-button-20","scopeIndex":3,"type":"icon-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"primary"},{"key":"consumer-s3-toggle-button-20","id":"spinner-consumer-s3-toggle-button-20","scopeIndex":3,"type":"toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"subtle-primary"},{"key":"consumer-s3-icon-toggle-button-20","id":"spinner-consumer-s3-icon-toggle-button-20","scopeIndex":3,"type":"icon-toggle-button","icon":"add","iconSize":"20","expectedSize":"sm","loading":true,"disabled":false,"label":"","variant":"subtle-primary"},{"key":"consumer-s3-upload-loading","id":"spinner-consumer-s3-upload-loading","scopeIndex":3,"type":"upload","state":"loading","disabled":false,"loadingText":"Checking records…","width":520,"expectedSize":"md","expectedVariant":"primary","label":""},{"key":"consumer-s3-upload-disabled","id":"spinner-consumer-s3-upload-disabled","scopeIndex":3,"type":"upload","state":"loading","disabled":true,"loadingText":"Checking records…","width":520,"expectedSize":null,"expectedVariant":"primary","label":""},{"key":"consumer-s0-button-12","id":"spinner-consumer-s0-button-12","scopeIndex":0,"type":"button","icon":"add","iconSize":"12","expectedSize":"xxs","loading":true,"disabled":false,"label":"Load records","variant":"primary"},{"key":"consumer-s0-button-16","id":"spinner-consumer-s0-button-16","scopeIndex":0,"type":"button","icon":"add","iconSize":"16","expectedSize":"xs","loading":true,"disabled":false,"label":"Load records","variant":"primary"},{"key":"consumer-s0-icon-button-12","id":"spinner-consumer-s0-icon-button-12","scopeIndex":0,"type":"icon-button","icon":"add","iconSize":"12","expectedSize":"xxs","loading":true,"disabled":false,"label":"","variant":"primary"},{"key":"consumer-s0-icon-button-16","id":"spinner-consumer-s0-icon-button-16","scopeIndex":0,"type":"icon-button","icon":"add","iconSize":"16","expectedSize":"xs","loading":true,"disabled":false,"label":"","variant":"primary"},{"key":"consumer-s0-icon-toggle-button-12","id":"spinner-consumer-s0-icon-toggle-button-12","scopeIndex":0,"type":"icon-toggle-button","icon":"add","iconSize":"12","expectedSize":"xxs","loading":true,"disabled":false,"label":"","variant":"subtle-primary"},{"key":"consumer-s0-icon-toggle-button-16","id":"spinner-consumer-s0-icon-toggle-button-16","scopeIndex":0,"type":"icon-toggle-button","icon":"add","iconSize":"16","expectedSize":"xs","loading":true,"disabled":false,"label":"","variant":"subtle-primary"},{"key":"consumer-s0-button-empty12","id":"spinner-consumer-s0-button-empty12","scopeIndex":0,"type":"button","icon":"","iconSize":"12","expectedSize":"sm","loading":true,"disabled":false,"label":"Load records","variant":"primary"}],"parts":["host","ring","before","after"],"aliases":["--ix-spinner--border-color","--ix-spinner-track--border-color","--ix-spinner-primary--border-color","--ix-spinner-primary-track--border-color","--ix-spinner-track--border-width-default","--ix-spinner-track--border-width-selected","--ix-spinner-track--border-width-emphasis","--ix-spinner-medium--size","--ix-spinner-extra-extra-small--size","--ix-spinner-extra-small--size","--ix-spinner-small--size","--ix-spinner-large--size","--ix-spinner--rotation-duration","--ix-spinner--mask-duration"],"computedSchema":["display","boxSizing","position","width","height","minWidth","minHeight","maxWidth","maxHeight","padding","margin","borderTopWidth","borderTopStyle","borderTopColor","borderRadius","color","backgroundColor","font","letterSpacing","lineHeight","overflow","whiteSpace","content","inset","transform","clipPath","animationName","animationDuration","animationTimingFunction","animationIterationCount"],"motionFractions":[0.0,0.125,0.25,0.375,0.5,0.625,0.75,0.875,1.0],"motionKeys":["spinner-s0-md-secondary","spinner-s1-xxs-primary","spinner-hidden-xxl-primary","spinner-override"],"consumerComputedSchema":["display","boxSizing","width","height","minWidth","minHeight","padding","margin","color","backgroundColor","font","lineHeight","alignItems","justifyContent","borderTopWidth","borderTopColor"],"api":{"source":"window.ixSpinnerReview","readiness":"#spinner-status[data-ready=true]","run":"#spinner-run","output":"#spinner-result","result":"phases[{id,revision,rows:[112total],consumers}],motion[{key,samples:[{fraction,row,animations}]}],running[{key,before,after,advanced}],errors,journal"},"colorTokens":["text-primary","text-disabled","text-danger","text-success","border-3","border-accent-hover","border-accent","border-accent-active","background-accent-secondary","background-accent-secondary-hover","background-accent-secondary-active","text-accent","text-accent-hover","text-accent-active","effects-focus","border-2","border-4","background-1"],"palettes":{"light":[[32,41,50],[100,111,121],[160,25,55],[22,105,60],[126,140,154],[0,92,151],[0,109,161],[0,70,120],[230,242,250],[210,232,246],[190,218,238],[0,83,137],[0,66,111],[0,50,85],[0,93,165],[110,126,142],[189,201,211],[238,243,247]],"dark":[[238,244,249],[150,164,177],[255,151,165],[119,226,165],[136,155,172],[111,207,255],[83,189,239],[152,224,255],[33,66,91],[42,82,110],[54,101,132],[135,214,255],[168,229,255],[200,240,255],[135,214,255],[163,184,200],[99,119,138],[42,56,69]]},"sampleFont":"400 16px/24px system-ui,sans-serif","parentWidth":160}"####;
fn spec() -> Value {
    serde_json::from_str(SPEC).expect("authored Spinner specification")
}
fn text(case: &Value, key: &str) -> String {
    case[key].as_str().unwrap_or_default().to_string()
}
fn size(value: &str) -> SpinnerSize {
    match value {
        "xxs" => SpinnerSize::Xxs,
        "xs" => SpinnerSize::Xs,
        "sm" => SpinnerSize::Sm,
        "xxl" => SpinnerSize::Xxl,
        _ => SpinnerSize::Md,
    }
}
fn variant(value: &str) -> SpinnerVariant {
    if value == "primary" {
        SpinnerVariant::Primary
    } else {
        SpinnerVariant::Secondary
    }
}
fn icon_size(value: &str) -> ButtonIconSize {
    match value {
        "12" => ButtonIconSize::Twelve,
        "16" => ButtonIconSize::Sixteen,
        _ => ButtonIconSize::Twenty,
    }
}
#[component]
pub fn SpinnerReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let config = spec();
    rsx! {
        style {{STYLES}}style {{override_style(revision())}}
        IxTheme {scheme:ColorScheme::Light,class:"spinner-review-shell",
            section {id:"spinner-review","data-revision":revision().to_string(),"data-phase":phase(),
                onmounted:move |_|{spawn(async move{let task=dioxus::document::eval(OBSERVE);let _=task.join::<Value>().await;});},
                header {h2 {"Spinner sizes, ink and motion"} p {"The actual CSS ring appears in fifty-six standalone cases and thirty-one loading consumers under four authored token scopes. Two revisions retain the same elements. System fonts and diagnostic tokens qualify this comparison."}}
                section {class:"spinner-controls",h3 {"Token and prop revisions"}
                    for next in ["baseline","revised"] {button {id:format!("spinner-{next}"),r#type:"button",onclick:move |_|{revision.set(usize::from(next=="revised"));phase.set(next.into());},"{next}"}}
                    button {id:"spinner-motion-run",r#type:"button",onclick:move |_|{spawn(async move{let task=dioxus::document::eval("return await window.ixSpinnerReview.motion();");let _=task.join::<Value>().await;});},"Seek nine motion phases"}
                    output {id:"spinner-status",role:"status","data-ready":"true","{phase()} — fifty-six standalone cases"}
                }
                div {class:"spinner-matrix",for case in config["cases"].as_array().expect("cases").iter().cloned(){SpinnerCase {key:"{text(&case,\"key\")}",case,revision:revision(),measured:true}}}
                section {class:"spinner-consumer-section",h3 {"Loading consumers"}p {"Actual Button, IconButton, ToggleButton, IconToggleButton and Upload preserve their loading size, label spacing, ink and blocked input."}
                    div {class:"spinner-consumer-matrix",for case in config["consumers"].as_array().expect("consumers").iter().cloned(){SpinnerConsumer {key:"{text(&case,\"key\")}",case,revision:revision()}}}
                }
                section {class:"spinner-fallback-panel",h3 {"Provisional Brand fallback"}p {"These live rings use public fallback sizing without diagnostic system tokens. Full Brand metrics and wall-clock frame pacing remain unverified."}
                    IxTheme {scheme:ColorScheme::Light,density:Density::Normal,class:"spinner-fallback-preview",h4 {"Light · normal"}div {class:"spinner-fallback-row",Spinner {size:SpinnerSize::Xxs}Spinner {size:SpinnerSize::Md,variant:SpinnerVariant::Primary}Spinner {size:SpinnerSize::Xxl,hide_track:true}}}
                    IxTheme {scheme:ColorScheme::Dark,density:Density::Compact,class:"spinner-fallback-preview",h4 {"Dark · compact"}div {class:"spinner-fallback-row",Spinner {size:SpinnerSize::Xs,variant:SpinnerVariant::Primary}Spinner {size:SpinnerSize::Sm,hide_track:true}Spinner {size:SpinnerSize::Xxl}}}
                }
                input {id:"spinner-spec",r#type:"hidden",value:SPEC}pre {id:"spinner-result",hidden:true,"aria-hidden":"true"}pre {id:"spinner-motion-result",hidden:true,"aria-hidden":"true"}
            }
        }
    }
}
#[component]
fn SpinnerCase(case: Value, revision: usize, measured: bool) -> Element {
    let case = current_props(case, revision);
    let scope = case["scopeIndex"].as_u64().unwrap_or_default() as usize;
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
    let class = if case["override"] == true {
        "spinner-component-override"
    } else if case["buttonColor"] == true {
        "spinner-button-color"
    } else {
        ""
    };
    let label = case["label"].as_str().unwrap_or("Loading").to_string();
    rsx! {article {id:text(&case,"id"),class:"spinner-review-case","data-spinner-row":measured.to_string(),"data-key":text(&case,"key"),"data-scope":scope.to_string(),"data-revision":revision.to_string(),"data-case":case.to_string(),
        h4 {class:"spinner-case-label","{text(&case,\"key\")}"}
        IxTheme {scheme,density,class:"spinner-case-theme",div {class:"spinner-sentinel",style:sentinel_style(scope,revision),
            div {class:"spinner-parent",style:parent_style(&case,revision),
                if case["nested"]==true {div {class:"spinner-nested",style:nested_style(revision),Spinner {class,size:size(&text(&case,"size")),variant:variant(&text(&case,"variant")),hide_track:case["hideTrack"]==true,label}}}
                else if case["omitted"]==true {Spinner {class}}
                else {Spinner {class,size:size(&text(&case,"size")),variant:variant(&text(&case,"variant")),hide_track:case["hideTrack"]==true,label}}
            }
        }}
    }}
}
#[component]
fn SpinnerConsumer(case: Value, revision: usize) -> Element {
    let scope = case["scopeIndex"].as_u64().unwrap_or_default() as usize;
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
    let kind = text(&case, "type");
    let label = text(&case, "label");
    let icon = text(&case, "icon");
    let icon_size = icon_size(&text(&case, "iconSize"));
    rsx! {article {id:text(&case,"id"),class:"spinner-consumer-case","data-spinner-consumer":"true","data-key":text(&case,"key"),"data-scope":scope.to_string(),"data-revision":revision.to_string(),"data-case":case.to_string(),
        h4 {class:"spinner-case-label","{text(&case,\"key\")}"}
        IxTheme {scheme,density,class:"spinner-case-theme",div {class:"spinner-sentinel",style:sentinel_style(scope,revision),
            div {class:"spinner-consumer-parent",style:consumer_parent_style(&case),
                if kind=="button" {Button {variant:ButtonVariant::Primary,loading:true,icon:Some(icon),icon_size,"{label}"}}
                else if kind=="icon-button" {IconButton {variant:ButtonVariant::Primary,loading:true,icon,size:icon_size,label:"Load records"}}
                else if kind=="toggle-button" {ToggleButton {variant:ToggleButtonVariant::SubtlePrimary,loading:true,icon:Some(icon),onpressedchange:move |_|{},"{label}"}}
                else if kind=="icon-toggle-button" {IconToggleButton {variant:ButtonVariant::SubtlePrimary,loading:true,icon,size:icon_size,label:"Load records",onpressedchange:move |_|{}}}
                else {Upload {state:UploadState::Loading,disabled:case["disabled"]==true,loading_text:Some(text(&case,"loadingText"))}}
            }
        }}
    }}
}
const STYLES: &str = include_str!("styles/spinner_review.css");
fn current_props(mut case: Value, revision: usize) -> Value {
    if case["live"] == true && revision == 1 {
        for (key, value) in [
            ("size", serde_json::json!("xxl")),
            ("variant", serde_json::json!("primary")),
            ("hideTrack", serde_json::json!(true)),
            ("label", serde_json::json!("Revised nested loading")),
        ] {
            case[key] = value;
        }
    }
    case
}
fn sentinel_style(s: usize, r: usize) -> String {
    let config = spec();
    let compact = s % 2;
    let dark = s / 2;
    let palette = &config["palettes"][if dark == 0 { "light" } else { "dark" }];
    let mut style = String::new();
    for (index, name) in config["colorTokens"]
        .as_array()
        .expect("colors")
        .iter()
        .enumerate()
    {
        let values = palette[index]
            .as_array()
            .expect("RGB")
            .iter()
            .map(|v| (v.as_u64().unwrap_or_default() as usize + compact + 3 * r).min(255))
            .collect::<Vec<_>>();
        style.push_str(&format!(
            "--si-sys-color-{}:rgb({}, {}, {});",
            name.as_str().expect("color"),
            values[0],
            values[1],
            values[2]
        ));
    }
    for (name, value) in [
        ("size-40", 12 - 4 * compact + 2 * dark + r),
        ("size-50", 16 - 4 * compact + 2 * dark + r),
        ("size-60", 20 - 4 * compact + 2 * dark + r),
        ("size-70", 24 - 4 * compact + 2 * dark + 2 * r),
        ("size-80", 32 - 8 * compact + 2 * dark + 2 * r),
        ("size-110", 64 + 2 * s + 4 * r),
        ("size-120", 80 + 2 * s + 4 * r),
        ("size-130", 96 - 16 * compact + 4 * dark + 4 * r),
        ("icon-lg", 32 + s + 2 * r),
        ("spacing-x-10", 2 + s + r),
        ("spacing-y-10", 1 + s + r),
        ("spacing-x-20", 4 + s + r),
        ("spacing-x-40", 10 + s + r),
        ("spacing-y-60", 14 + s + 2 * r),
        ("spacing-x-60", 14 + s + 2 * r),
        ("border-width-default", 1 + r),
        ("border-width-selected", 2 + r),
        ("border-width-emphasis", 4 + r),
        ("border-radius-xs", 3 + s + r),
        ("border-radius-sm", 4 + s + r),
        ("focus-ring-offset", 3 + s + r),
    ] {
        style.push_str(&format!("--si-sys-sizing-{name}:{value}px;"));
    }
    style.push_str(&format!("font:{};color:var(--si-sys-color-text-primary);--si-sys-typography-body:400 {}px/{}px system-ui,sans-serif;--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;",config["sampleFont"].as_str().expect("sample font"),14+s+r,20+s+r,14+s+r,20+s+r,0.25+0.25*r as f64));
    style
}
fn override_style(r: usize) -> String {
    format!(
        "#spinner-review .spinner-component-override{{--ix-spinner-medium--size:{}px;--ix-spinner-track--border-width-selected:{}px;--ix-spinner-primary--border-color:{};--ix-spinner-primary-track--border-color:{};--ix-spinner--rotation-duration:{}s;--ix-spinner--mask-duration:{}s;}}#spinner-review .spinner-button-color{{--ix-button-color:{};}}",
        45 + 3 * r,
        5 + r,
        if r == 0 {
            "rgb(113, 220, 181)"
        } else {
            "rgb(121, 231, 192)"
        },
        if r == 0 {
            "rgb(72, 104, 126)"
        } else {
            "rgb(83, 115, 137)"
        },
        1.6 + 0.2 * r as f64,
        2.4 + 0.3 * r as f64,
        if r == 0 {
            "rgb(91, 41, 141)"
        } else {
            "rgb(101, 51, 151)"
        }
    )
}
fn parent_style(_case: &Value, _revision: usize) -> String {
    format!("width:{}px", spec()["parentWidth"].as_u64().unwrap_or(160))
}
fn consumer_parent_style(_case: &Value) -> String {
    "width:520px".to_string()
}
fn nested_style(_revision: usize) -> String {
    String::new()
}
const OBSERVE: &str = r#"return (()=>{
if(window.ixSpinnerReview)return{installed:false};
const spec=JSON.parse(document.getElementById('spinner-spec').value);
const items=[],consumerItems=[],journal=[],errors=[],objects=new WeakMap();let objectSeq=0,seq=0,revision=0,ready=false,busy=false;const byId=id=>document.getElementById('spinner-'+id),oid=n=>{if(!objects.has(n))objects.set(n,++objectSeq);return objects.get(n)},log=(kind,data={})=>journal.push({seq:++seq,kind,time:performance.now(),revision,...data});
function ringOf(host){return host?.querySelector(':scope > .ix-spinner-ring')??null}
function actualAnimations(host){const ring=ringOf(host);return ring?[...ring.getAnimations({subtree:true})]:[]}
function animationKind(animation){const keys=animation.effect?.getKeyframes()??[];return keys.some(k=>'clipPath'in k)?'mask':keys.some(k=>'transform'in k)?'rotation':'other'}
function safeNumber(v){return typeof v==='number'&&!Number.isFinite(v)?String(v):v}
function animationFacts(a){const effect=a.effect,timing=effect?.getTiming(),computed=effect?.getComputedTiming(),target=effect?.target;return{kind:animationKind(a),name:a.animationName??null,playState:a.playState,pending:a.pending,currentTime:safeNumber(a.currentTime),startTime:safeNumber(a.startTime),playbackRate:a.playbackRate,target:{tag:target?.tagName??null,id:target?.id??null,pseudo:effect?.pseudoElement??target?.type??null},timing:timing?Object.fromEntries(Object.entries(timing).map(([k,v])=>[k,safeNumber(v)])):null,computed:computed?Object.fromEntries(Object.entries(computed).map(([k,v])=>[k,safeNumber(v)])):null,keyframes:effect?.getKeyframes()??[]}}
function metric(n,origin,pseudo=null,schema=spec.computedSchema){if(!(n instanceof Element))return null;const cs=getComputedStyle(n,pseudo),r=pseudo?null:n.getBoundingClientRect(),rr=origin.getBoundingClientRect();return{object:oid(n),tag:n.tagName,id:n.id,class:n.getAttribute('class'),textContent:n.textContent,attrs:pseudo?{}:Object.fromEntries([...n.attributes].map(a=>[a.name,a.value])),pseudo,computed:Object.fromEntries(schema.map(k=>[k,cs[k]])),nativeLayout:pseudo?null:{width:n.offsetWidth??null,height:n.offsetHeight??null,left:n.offsetLeft??null,top:n.offsetTop??null},rect:r?{x:r.x,y:r.y,width:r.width,height:r.height,relativeX:r.x-rr.x,relativeY:r.y-rr.y}:null}}
function captureSpinner(host,descriptor,props,sample,origin=host){if(!host)return null;const ring=ringOf(host);return{key:descriptor.key,id:sample.id,scopeIndex:props.scopeIndex,revision:Number(sample.dataset.revision),props,host:metric(host,origin),ring:metric(ring,origin),before:metric(ring,origin,'::before'),after:metric(ring,origin,'::after'),resolvedAliases:Object.fromEntries(spec.aliases.map(k=>[k,getComputedStyle(host).getPropertyValue(k)])),nativeFacts:{role:host.getAttribute('role'),ariaLabel:host.getAttribute('aria-label'),ariaBusy:host.getAttribute('aria-busy'),size:['xxs','xs','sm','md','xxl'].find(size=>ring?.classList.contains(size))??null,variant:ring?.classList.contains('primary')?'primary':'secondary',hideTrack:ring?.classList.contains('hide-track')??false},animations:actualAnimations(host).map(animationFacts).sort((a,b)=>a.kind.localeCompare(b.kind))}}
function capture(item){return captureSpinner(item.host,item.descriptor,item.props,item.sample)}
function consumerParts(item){const h=item.host;if(item.props.type==='upload'){return{control:h?.querySelector('.ix-upload-area'),label:h?.querySelector('.ix-upload-text'),spinner:h?.querySelector('.ix-spinner'),action:h?.querySelector('.ix-upload-button-host button'),disabled:h?.querySelector('input[type=file]')?.disabled}}return{control:h,label:h?.querySelector('.ix-button-content'),spinner:h?.querySelector('.ix-spinner'),action:null,disabled:h?.disabled}}
function captureConsumer(item){const parts=consumerParts(item),origin=parts.control??item.host;return{key:item.descriptor.key,scopeIndex:item.props.scopeIndex,revision:Number(item.sample.dataset.revision),props:item.props,nativeHost:metric(item.host,origin,null,spec.consumerComputedSchema),spinnerWrapper:metric(parts.spinner?.closest('.ix-button-loading,.ix-upload-spinner-host')??parts.spinner,origin,null,spec.consumerComputedSchema),control:metric(parts.control,origin,null,spec.consumerComputedSchema),label:metric(parts.label,origin,null,spec.consumerComputedSchema),labelText:parts.label?.textContent??null,disabled:parts.disabled,action:metric(parts.action,origin,null,spec.consumerComputedSchema),spinner:captureSpinner(parts.spinner,{...item.descriptor,key:item.descriptor.key+'-spinner'},{...item.props,size:item.props.expectedSize,variant:item.props.expectedVariant??'secondary',hideTrack:item.props.type!=='upload'},item.sample,origin)}}
async function settle(label){await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{label,index,timestamp});resolve()})))}
async function animationPause(host,fraction=0){const animations=actualAnimations(host);for(const a of animations){a.pause();a.currentTime=Number(a.effect.getTiming().duration)*fraction}await Promise.all(animations.map(a=>a.ready));return animations}
function allSpinnerHosts(){return [...items.map(i=>i.host),...consumerItems.map(i=>consumerParts(i).spinner).filter(Boolean)]}
async function pauseAll(fraction=0){await Promise.all(allSpinnerHosts().map(host=>animationPause(host,fraction)));await settle('native-animation-pause-'+fraction)}
async function runningFacts(){const hosts=items.filter(i=>spec.motionKeys.includes(i.descriptor.key));for(const item of hosts)for(const a of actualAnimations(item.host)){a.currentTime=0;a.play()}await Promise.all(hosts.flatMap(item=>actualAnimations(item.host).map(a=>a.ready)));const before=hosts.map(item=>({key:item.descriptor.key,animations:actualAnimations(item.host).map(animationFacts)}));await settle('running-advance');const after=hosts.map(item=>({key:item.descriptor.key,animations:actualAnimations(item.host).map(animationFacts)}));return before.map((row,index)=>({key:row.key,before:row.animations,after:after[index].animations,advanced:row.animations.every(a=>{const b=after[index].animations.find(x=>x.kind===a.kind);return typeof a.currentTime==='number'&&typeof b?.currentTime==='number'&&b.currentTime>a.currentTime&&b.playState==='running'})}))}
async function motion(){const rows=[];for(const key of spec.motionKeys){const item=items.find(item=>item.descriptor.key===key),samples=[];for(const fraction of spec.motionFractions){await animationPause(item.host,fraction);await settle('native-animation-seek-'+key+'-'+fraction);samples.push({fraction,row:capture(item)})}rows.push({key,samples})}await pauseAll(0);return rows}

const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const wait=async(predicate,label)=>{const end=performance.now()+30000;while(performance.now()<end){if(predicate())return;await sleep(5)}throw Error('Public Spinner receipt timed out: '+label)};
const standalone=()=>Array.from(document.querySelectorAll('#spinner-review [data-spinner-row="true"]'));
const consumerRows=()=>Array.from(document.querySelectorAll('#spinner-review [data-spinner-consumer="true"]'));
const itemFor=(sample,consumer=false)=>({sample,descriptor:JSON.parse(sample.dataset.case),get props(){return JSON.parse(sample.dataset.case)},get host(){return sample.querySelector(consumer?'.spinner-consumer-parent > .ix-button,.spinner-consumer-parent > .ix-upload':'.spinner-parent .ix-spinner')}});
items.push(...standalone().map(sample=>itemFor(sample)));consumerItems.push(...consumerRows().map(sample=>itemFor(sample,true)));
const actualStandaloneReceipt=item=>{const h=item.host,p=item.props,ring=ringOf(h);return !!h&&!!ring&&['xxs','xs','sm','md','xxl'].find(size=>ring.classList.contains(size))===p.size&&ring.classList.contains('primary')===(p.variant==='primary')&&ring.classList.contains('hide-track')===!!p.hideTrack&&h.getAttribute('role')==='status'&&h.getAttribute('aria-busy')==='true'&&h.getAttribute('aria-label')===(p.label??'Loading')};
const actualConsumerReceipt=item=>{const p=item.props,parts=consumerParts(item),h=parts.spinner,ring=ringOf(h);if(!parts.control||parts.disabled!==true)return false;if(p.expectedSize===null)return !h;return !!h&&!!ring&&ring.classList.contains(p.expectedSize)&&ring.classList.contains('primary')===(p.expectedVariant==='primary')&&ring.classList.contains('hide-track')===(p.type!=='upload')};
const publicReceipt=async expectedRevision=>{await wait(()=>Number(byId('review').dataset.revision)===expectedRevision&&items.length===56&&consumerItems.length===31&&items.every(item=>Number(item.sample.dataset.revision)===expectedRevision&&actualStandaloneReceipt(item))&&consumerItems.every(item=>Number(item.sample.dataset.revision)===expectedRevision&&actualConsumerReceipt(item)),'actual revision/ring classes/name/native blocked controls');log('rust-public-native-commit',{revision:expectedRevision,rows:items.map(item=>({key:item.descriptor.key,revision:Number(item.sample.dataset.revision),nativeFacts:capture(item).nativeFacts})),consumers:consumerItems.map(item=>({key:item.descriptor.key,revision:Number(item.sample.dataset.revision),disabled:consumerParts(item).disabled,spinnerPresent:!!consumerParts(item).spinner}))})};
async function setRevision(id){const selected=spec.revisions.find(value=>value.id===id);if(!selected)throw Error('Unknown Spinner revision '+id);byId(id).click();await publicReceipt(selected.revision);revision=Number(byId('review').dataset.revision);await settle('rust-public-render-'+id);return revision}
async function run(){if(busy)return;busy=true;const phases=[],running=[];try{for(const rev of spec.revisions){await setRevision(rev.id);running.push({id:rev.id,revision:rev.revision,rows:await runningFacts()});await pauseAll(0);phases.push({id:rev.id,revision:rev.revision,rows:items.map(capture),consumers:consumerItems.map(captureConsumer)})}const motionRows=await motion();const result={ok:errors.length===0&&running.every(phase=>phase.rows.every(row=>row.advanced)),revision:spec.revision,fixture:spec.fixture,phases,motion:motionRows,running,errors:errors.slice(),journal:journal.slice(),limits:['Authored diagnostic tokens and system fonts; no full Brand certification','Native CSSAnimations are paused at zero for settled comparison after running advancement observations','Nine native seeks inspect declared motion; no wall-clock frame pacing or physical AT certification','Source custom hosts versus native Rust host markup and ARIA wrappers retained raw']};byId('result').textContent=JSON.stringify(result);byId('status').dataset.run='done';byId('status').textContent='112 standalone and 62 consumer Spinner observations';return result}catch(error){errors.push(String(error));const result={ok:false,error:String(error),phases,running,errors:errors.slice()};byId('result').textContent=JSON.stringify(result);byId('status').dataset.run='done';return result}finally{busy=false}}
window.addEventListener('error',event=>errors.push(event.message));window.addEventListener('unhandledrejection',event=>errors.push(String(event.reason)));
window.ixSpinnerReview={spec,run,setRevision,motion:async()=>{const rows=await motion();byId('motion-result').textContent=JSON.stringify(rows);return rows},capture:()=>items.map(capture),consumers:()=>consumerItems.map(captureConsumer),errors,journal};
ready=true;byId('status').dataset.ready='true';return{installed:true};
})()"#;

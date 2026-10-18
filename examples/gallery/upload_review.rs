//! Shared development-only Upload fixture. Application state and file reads stay in Rust.
use dioxus::html::FileData;
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
const SPEC: &str = r####"{"revision":"8bf599bc68365d7752e12832e71b625883b93d7a","fixture":"Developer-authored system-token Upload adapter; system fonts; not full Brand metrics","revisions":[{"id":"baseline","revision":0},{"id":"revised","revision":1}],"cases":[{"key":"upload-s0-select","id":"upload-upload-s0-select","scopeIndex":0,"state":"select","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s0-loading","id":"upload-upload-s0-loading","scopeIndex":0,"state":"loading","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s0-failed","id":"upload-upload-s0-failed","scopeIndex":0,"state":"failed","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s0-success","id":"upload-upload-s0-success","scopeIndex":0,"state":"success","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s1-select","id":"upload-upload-s1-select","scopeIndex":1,"state":"select","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s1-loading","id":"upload-upload-s1-loading","scopeIndex":1,"state":"loading","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s1-failed","id":"upload-upload-s1-failed","scopeIndex":1,"state":"failed","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s1-success","id":"upload-upload-s1-success","scopeIndex":1,"state":"success","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s2-select","id":"upload-upload-s2-select","scopeIndex":2,"state":"select","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s2-loading","id":"upload-upload-s2-loading","scopeIndex":2,"state":"loading","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s2-failed","id":"upload-upload-s2-failed","scopeIndex":2,"state":"failed","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s2-success","id":"upload-upload-s2-success","scopeIndex":2,"state":"success","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s3-select","id":"upload-upload-s3-select","scopeIndex":3,"state":"select","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s3-loading","id":"upload-upload-s3-loading","scopeIndex":3,"state":"loading","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s3-failed","id":"upload-upload-s3-failed","scopeIndex":3,"state":"failed","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-s3-success","id":"upload-upload-s3-success","scopeIndex":3,"state":"success","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-disabled-select","id":"upload-upload-disabled-select","scopeIndex":0,"state":"select","disabled":true,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-disabled-loading","id":"upload-upload-disabled-loading","scopeIndex":1,"state":"loading","disabled":true,"multiple":false,"directory":false,"multiline":false,"width":520},{"key":"upload-directory","id":"upload-upload-directory","scopeIndex":2,"state":"select","disabled":false,"multiple":false,"directory":true,"multiline":false,"width":520},{"key":"upload-multiple","id":"upload-upload-multiple","scopeIndex":3,"state":"select","disabled":false,"multiple":true,"directory":false,"multiline":false,"width":520,"accept":".txt,text/plain"},{"key":"upload-multiline","id":"upload-upload-multiline","scopeIndex":0,"state":"select","disabled":false,"multiple":false,"directory":false,"multiline":true,"width":520,"selectFileText":"Drop the selected engineering records here to review the complete document set before uploading."},{"key":"upload-narrow","id":"upload-upload-narrow","scopeIndex":1,"state":"select","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":260,"selectFileText":"Drop the selected engineering records here to review the complete document set before uploading."},{"key":"upload-override","id":"upload-upload-override","scopeIndex":2,"state":"success","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520,"override":true},{"key":"upload-live-props","id":"upload-upload-live-props","scopeIndex":3,"state":"select","disabled":false,"multiple":false,"directory":false,"multiline":false,"width":520,"live":true,"selectFileText":"Initial review files","i18nUploadFile":"Choose records…"}],"colorTokens":["text-primary","text-disabled","text-danger","text-success","border-3","border-accent-hover","border-accent","border-accent-active","background-accent-secondary","background-accent-secondary-hover","background-accent-secondary-active","text-accent","text-accent-hover","text-accent-active","effects-focus","border-2","border-4"],"aliases":["--ix-upload-error-icon--color","--ix-upload-success-icon--color","--ix-upload--border-color","--ix-upload--border-color--dragover","--ix-upload-text--color","--ix-upload-text--color--checking","--ix-upload-text--color--disabled","--ix-upload--border-radius","--ix-upload--min-height","--ix-upload--height","--ix-upload-file-upload-area--padding","--ix-upload-file-upload-area--margin-inline-start","--ix-upload-loader--margin","--ix-upload-loader--width","--ix-upload-loader--height","--ix-upload-filename--margin-bottom","--ix-upload-state--margin-inline-end","--ix-upload-icon--icon-size","--ix-upload-icon--margin-inline-end","--ix-upload-file-upload-area--border-width","--ix-upload-glyph--margin-block-start","--ix-upload-loader--font-size","--ix-upload-loader--animation-duration"],"buttonAliases":["--ix-button--height","--ix-button--min-width","--ix-button--padding","--ix-button--border-width","--ix-button--border-radius","--ix-button--outline-width--focus","--ix-button--outline-color--focus","--ix-button--focus--outline-offset","--ix-button-secondary--background","--ix-button-secondary--background--hover","--ix-button-secondary--background--active","--ix-button-secondary--background--disabled","--ix-button-secondary--border-color","--ix-button-secondary--border-color--hover","--ix-button-secondary--border-color--active","--ix-button-secondary--border-color--disabled","--ix-button-secondary--color","--ix-button-secondary--color--hover","--ix-button-secondary--color--active","--ix-button-secondary--color--disabled"],"parts":["host","parent","area","state","text","actions","input","buttonHost","button","buttonLabel","icon","spinnerHost","spinner","spinnerBefore","spinnerAfter"],"computedSchema":["display","boxSizing","position","width","height","minWidth","minHeight","padding","margin","borderTopWidth","borderTopStyle","borderTopColor","borderRadius","color","backgroundColor","font","letterSpacing","lineHeight","overflow","whiteSpace","textOverflow","alignItems","justifyContent","flexShrink","outline","outlineOffset","pointerEvents"],"consumer":{"key":"consumer","id":"upload-consumer","scopeIndex":0,"state":"select","disabled":false,"multiple":true,"directory":false,"multiline":false,"width":520,"accept":".txt,text/plain"},"pointerCases":[{"id":"enabled","state":"select","disabled":false},{"id":"loading","state":"loading","disabled":false},{"id":"disabled","state":"select","disabled":true}],"api":{"readiness":"#upload-status[data-ready=true]","output":"#upload-result","run":"#upload-run","source":"window.ixUploadReview","schema":"phases[{id,revision,rows:[{id,key,scopeIndex,revision,props,...parts,resolvedAliases,buttonAliases,nativeFacts,textContent,buttonText}]}]"}}"####;
fn spec() -> Value {
    serde_json::from_str(SPEC).expect("authored Upload review specification")
}
fn text(case: &Value, key: &str) -> String {
    case[key].as_str().unwrap_or_default().to_string()
}
fn optional(case: &Value, key: &str) -> Option<String> {
    case[key].as_str().map(str::to_string)
}
fn state(name: &str) -> UploadState {
    match name {
        "loading" => UploadState::Loading,
        "failed" => UploadState::Failed,
        "success" => UploadState::Success,
        _ => UploadState::SelectFile,
    }
}
fn state_name(value: UploadState) -> &'static str {
    match value {
        UploadState::SelectFile => "select",
        UploadState::Loading => "loading",
        UploadState::Failed => "failed",
        UploadState::Success => "success",
    }
}
fn current_props(mut case: Value, revision: usize) -> Value {
    if revision == 1 && case["live"] == true {
        for (key, value) in [
            ("state", json!("loading")),
            ("directory", json!(true)),
            ("multiple", json!(true)),
            ("disabled", json!(false)),
            ("loadingText", json!("Reviewing revised folder…")),
            ("i18nUploadFile", json!("Choose revised folder…")),
        ] {
            case[key] = value;
        }
    }
    case
}
fn sentinel_style(scope: usize, revision: usize) -> String {
    let mut style = String::new();
    for (index, token) in spec()["colorTokens"]
        .as_array()
        .expect("colors")
        .iter()
        .enumerate()
    {
        style.push_str(&format!(
            "--si-sys-color-{}:rgb({},{},{});",
            token.as_str().expect("color"),
            24 + (13 * index + 7 * scope + 3 * revision) % 160,
            32 + (11 * index + 5 * scope + 7 * revision) % 150,
            40 + (7 * index + 3 * scope + 11 * revision) % 140
        ));
    }
    let s = scope;
    let r = revision;
    for (token, value) in [
        ("size-30", 8 + s + r),
        ("size-40", 12 + s + r),
        ("size-10", 2 + s + r),
        ("size-20", 4 + s + r),
        ("size-50", 16 + s + r),
        ("size-60", 20 - 4 * (s % 2) + s / 2 + 2 * r),
        ("size-70", 24 - 4 * (s % 2) + s / 2 + 2 * r),
        ("size-80", 32 - 8 * (s % 2) + 2 * r),
        ("size-100", 48 + 2 * s + 4 * r),
        ("size-110", 64 + 2 * s + 4 * r),
        ("size-120", 80 + 2 * s + 4 * r),
        ("size-130", 96 + 2 * s + 4 * r),
        ("icon-lg", 32 + s + 2 * r),
        ("spacing-x-10", 2 + s + r),
        ("spacing-y-10", 1 + s + r),
        ("spacing-x-20", 4 + s + r),
        ("spacing-x-40", 10 + s + r),
        ("spacing-y-60", 14 + s + 2 * r),
        ("spacing-x-60", 14 + s + 2 * r),
        ("border-width-default", 1 + r),
        ("border-width-selected", 2 + r),
        ("border-width-emphasis", 3 + r),
        ("border-radius-xs", 3 + s + r),
        ("border-radius-sm", 4 + s + r),
        ("focus-ring-offset", 3 + s + r),
    ] {
        style.push_str(&format!("--si-sys-sizing-{token}:{value}px;"));
    }
    style.push_str(&format!("font:400 16px/24px system-ui,sans-serif;color:var(--si-sys-color-text-primary);--si-sys-typography-body:400 {}px/{}px system-ui,sans-serif;--si-sys-typography-body-sbold:600 {}px/{}px system-ui,sans-serif;--si-ref-typography-letter-spacing-normal:{}px;--focus--border-color:var(--si-sys-color-effects-focus);--theme-default-time:0s;--theme-medium-time:0s;",14+s+r,20+s+r,14+s+r,20+s+r,0.25+0.25*r as f64));
    style
}
fn override_style(r: usize) -> String {
    format!(
        "#upload-review .upload-component-override{{--ix-upload--height:{}px;--ix-upload--min-height:{}px;--ix-upload-file-upload-area--padding:{}px {}px;--ix-upload--border-radius:{}px;--ix-upload-icon--icon-size:{}px;--ix-upload-file-upload-area--margin-inline-start:{}px;--ix-upload-state--margin-inline-end:{}px;--ix-upload-icon--margin-inline-end:{}px;}}",
        78 + 4 * r,
        78 + 4 * r,
        9 + r,
        17 + r,
        11 + r,
        27 + 2 * r,
        19 + r,
        13 + r,
        13 + r
    )
}
#[derive(Clone, Copy, PartialEq)]
struct Consumer {
    state: Signal<UploadState>,
    disabled: Signal<bool>,
    requests: Signal<Vec<Value>>,
    commands: Signal<usize>,
}
impl Consumer {
    fn snapshot(self) -> String {
        json!({"state":state_name((self.state)()),"requests":(self.requests)(),"disabled":(self.disabled)(),"commands":(self.commands)()}).to_string()
    }
    fn accept(mut self, files: Vec<FileData>) {
        let before = state_name((self.state)());
        self.commands += 1;
        let generation = (self.commands)();
        self.state.set(UploadState::Loading);
        let names = files.iter().map(FileData::name).collect::<Vec<_>>();
        {
            let mut records = self.requests.write();
            let sequence = records.len() + 1;
            records.push(
                json!({"sequence":sequence,"kind":"filesChanged","before":before,"names":names}),
            );
        }
        spawn(async move {
            let mut reads = Vec::new();
            let mut failed = false;
            for file in files {
                let name = file.name();
                match file.read_string().await {
                    Ok(content) => reads.push(json!({"name":name,"content":content})),
                    Err(error) => {
                        failed = true;
                        reads.push(json!({"name":name,"error":error.to_string()}));
                    }
                }
            }
            if (self.commands)() != generation {
                return;
            }
            self.state.set(if failed {
                UploadState::Failed
            } else {
                UploadState::Success
            });
            let mut records = self.requests.write();
            let sequence = records.len() + 1;
            records.push(json!({"sequence":sequence,"kind":"contentRead","files":reads,"state":state_name((self.state)())}));
        });
    }
    fn command(mut self, state: UploadState, reset: bool) {
        self.state.set(state);
        if reset {
            self.disabled.set(false);
            self.requests.set(Vec::new());
        }
        self.commands += 1;
    }
}
#[component]
pub fn UploadReview() -> Element {
    let mut revision = use_signal(|| 0_usize);
    let mut phase = use_signal(|| "baseline".to_string());
    let mut supplied = use_signal(|| true);
    let mut pointer_case = use_signal(|| "enabled".to_string());
    let mut pointer_epoch = use_signal(|| 0_usize);
    let mut consumer = Consumer {
        state: use_signal(|| UploadState::SelectFile),
        disabled: use_signal(|| false),
        requests: use_signal(Vec::<Value>::new),
        commands: use_signal(|| 0_usize),
    };
    let config = spec();
    let selected = config["pointerCases"]
        .as_array()
        .expect("pointer cases")
        .iter()
        .find(|case| case["id"].as_str() == Some(pointer_case().as_str()))
        .cloned()
        .unwrap_or_else(|| config["pointerCases"][0].clone());
    let mut pointer = config["cases"][0].clone();
    pointer["id"] = json!("upload-pointer-sample");
    pointer["key"] = json!("pointer");
    pointer["state"] = selected["state"].clone();
    pointer["disabled"] = selected["disabled"].clone();
    let mut fallback_light = config["cases"][2].clone();
    fallback_light["id"] = json!("upload-fallback-light");
    fallback_light["key"] = json!("fallback-light-normal-failed");
    let mut fallback_dark = config["cases"][13].clone();
    fallback_dark["id"] = json!("upload-fallback-dark");
    fallback_dark["key"] = json!("fallback-dark-compact-loading");
    rsx! {
        style {{STYLES}} style {{override_style(revision())}}
        IxTheme {scheme:ColorScheme::Light,class:"upload-review-shell",
            section {id:"upload-review","data-revision":revision().to_string(),"data-phase":phase(),"data-consumer-command":(consumer.commands)().to_string(),
                onmounted:move |_|{spawn(async move{let task=dioxus::document::eval(OBSERVE);let _=task.join::<Value>().await;});},
                header {h2 {"Upload selection and status"} p {"Forty-eight observations use four authored token scopes and two live revisions. These diagnostic mappings use system fonts and do not certify full Brand appearance."}}
                section {class:"upload-controls",h3 {"Token and prop revisions"}
                    for next in ["baseline","revised"] {button {id:format!("upload-{next}"),r#type:"button",onclick:move |_|{revision.set(usize::from(next=="revised"));phase.set(next.into());},"{next}"}}
                    label {input {id:"upload-supplied",r#type:"checkbox",checked:supplied(),onchange:move |event|supplied.set(event.checked())} " Supply diagnostic system tokens"}
                    button {id:"upload-outside",r#type:"button","Outside focus target"}
                    output {id:"upload-status",role:"status","data-ready":"true","{phase()} — twenty-four cases"}
                }
                section {class:"upload-consumer-panel",h3 {"Choose a local text file"} p {"This accepting Rust consumer reads the chosen file locally and shows its filename. It performs no network transfer. Use native Tab between the adjacent buttons and Upload; original input observations retain trusted-event facts."}
                    div {id:"upload-consumer-target",class:"upload-consumer-flow",button {id:"upload-consumer-before",r#type:"button","Before Upload"}
                        UploadCase {case:config["consumer"].clone(),revision:0,supplied:false,measured:false,consumer:Some(consumer)}
                        button {id:"upload-consumer-after",r#type:"button","After Upload"}
                    }
                    output {id:"upload-consumer-status",role:"status","{state_name((consumer.state)())}"}
                    ul {for record in (consumer.requests)().into_iter().filter(|record|record["kind"]=="filesChanged") {for name in record["names"].as_array().cloned().unwrap_or_default() {li {"{name.as_str().unwrap_or_default()}"}}}}
                    button {id:"upload-consumer-reset",r#type:"button",onclick:move |_|consumer.command(UploadState::SelectFile,true),"Reset consumer"}
                    for next in ["loading","success","failed","disable","enable"] {button {id:format!("upload-consumer-{next}"),r#type:"button",onclick:move |_|{if next=="disable"||next=="enable"{consumer.disabled.set(next=="disable");consumer.commands+=1;}else{consumer.command(state(next),false);}},"{next}"}}
                    input {id:"upload-consumer-json",r#type:"hidden",value:consumer.snapshot()}
                }
                section {class:"upload-pointer-panel",h3 {"Original button input"} p {"Choose a state, arm a fresh Upload, then hold its actual button for 350 ms and release. Capture is passive; the enabled button opens the platform chooser when activated. The matrix runner does not open a chooser."}
                    label {"State " select {id:"upload-pointer-case",value:pointer_case(),onchange:move |event|pointer_case.set(event.value()),for name in ["enabled","loading","disabled"] {option {value:name,"{name}"}}}}
                    button {id:"upload-pointer-arm",r#type:"button",onclick:move |_|pointer_epoch+=1,"Arm original input capture"}
                    div {id:"upload-pointer-target","data-pointer-case":pointer_case(),"data-arm":pointer_epoch().to_string(),for epoch in [pointer_epoch()] {UploadCase {key:"pointer-{epoch}",case:pointer.clone(),revision:0,supplied:true,measured:false}}}
                    output {id:"upload-pointer-status",role:"status","Awaiting original input"}pre {id:"upload-pointer-result",hidden:true,"aria-hidden":"true"}
                }
                div {class:"upload-matrix",for case in config["cases"].as_array().expect("cases").iter().cloned() {UploadCase {key:"{text(&case,\"key\")}",case,revision:revision(),supplied:supplied(),measured:true}}}
                section {class:"upload-fallback-panel",h3 {"Provisional Brand fallback"}p {"These two previews use public fallback values without diagnostic system tokens: light normal failure and dark compact loading. Full Brand fonts, metrics and platform certification remain unverified."}
                    UploadCase {case:fallback_light,revision:0,supplied:false,measured:false}
                    UploadCase {case:fallback_dark,revision:0,supplied:false,measured:false}
                }
                input {id:"upload-spec",r#type:"hidden",value:SPEC}pre {id:"upload-result",hidden:true,"aria-hidden":"true"}
            }
        }
    }
}
#[component]
fn UploadCase(
    case: Value,
    revision: usize,
    supplied: bool,
    measured: bool,
    #[props(default)] consumer: Option<Consumer>,
) -> Element {
    let mut case = current_props(case, revision);
    if let Some(consumer) = consumer {
        case["state"] = json!(state_name((consumer.state)()));
        case["disabled"] = json!((consumer.disabled)());
    }
    let scope = case["scopeIndex"].as_u64().unwrap_or_default() as usize;
    let mut callbacks = use_signal(Vec::<Value>::new);
    let mut focus_in = use_signal(|| 0_usize);
    let mut focus_out = use_signal(|| 0_usize);
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
        "upload-component-override"
    } else {
        ""
    };
    rsx! {article {id:text(&case,"id"),class:"upload-review-case","data-upload-row":measured.to_string(),"data-key":text(&case,"key"),"data-upload-key":text(&case,"key"),"data-scope":scope.to_string(),"data-revision":revision.to_string(),"data-case":case.to_string(),"data-callback-count":callbacks().len().to_string(),"data-focus-in":focus_in().to_string(),"data-focus-out":focus_out().to_string(),onfocusin:move |_|focus_in+=1,onfocusout:move |_|focus_out+=1,
        if measured {h4 {class:"upload-case-label","{text(&case,\"key\")}"}}
        IxTheme {scheme,density,class:"upload-case-theme",div {class:"upload-sentinel",style:if supplied{sentinel_style(scope,revision)}else{String::new()},
            div {class:"upload-parent",style:format!("width:{}px;max-width:100%",case["width"].as_u64().unwrap_or(520)),
                Upload {class,name:consumer.map(|_|"records".to_string()),state:state(&text(&case,"state")),disabled:case["disabled"]==true,multiple:case["multiple"]==true,directory_upload:case["directory"]==true,multiline:case["multiline"]==true,accept:optional(&case,"accept"),select_file_text:optional(&case,"selectFileText"),loading_text:optional(&case,"loadingText"),button_text:optional(&case,"i18nUploadFile"),
                    onfileschanged:move |files:Vec<FileData>|{let mut records=callbacks.write();let sequence=records.len()+1;records.push(json!({"sequence":sequence,"kind":"filesChanged","names":files.iter().map(FileData::name).collect::<Vec<_>>()}));drop(records);if let Some(consumer)=consumer{consumer.accept(files);}},
                    onentrieschanged:move |entries:Vec<UploadEntry>|{let mut records=callbacks.write();let sequence=records.len()+1;records.push(json!({"sequence":sequence,"kind":"entriesChanged","relativePaths":entries.iter().map(|entry|entry.relative_path.to_string_lossy().to_string()).collect::<Vec<_>>()}));},
                    onerror:move |error:UploadError|{let mut records=callbacks.write();let sequence=records.len()+1;records.push(json!({"sequence":sequence,"kind":"error","message":error.to_string()}));}
                }
            }
        }}
        input {class:"upload-callbacks",r#type:"hidden",value:json!({"requests":callbacks()}).to_string()}
    }}
}
const STYLES: &str = r#"
#upload-review{padding:24px;background:#fff;color:#26333d;font:14px/1.5 system-ui,sans-serif}#upload-review .ix-theme{min-height:0}#upload-review header,#upload-review .upload-controls,#upload-review .upload-consumer-panel,#upload-review .upload-pointer-panel,#upload-review .upload-fallback-panel{margin-bottom:24px;padding:16px;background:#eef2f5;color:#26333d}#upload-review h2,#upload-review h3,#upload-review h4,#upload-review p,#upload-review label{color:#26333d}#upload-review h2{font-size:24px;margin:0 0 8px}#upload-review h3{font-size:18px;margin:0 0 8px}#upload-review p{max-width:75ch;margin:0 0 12px}#upload-review .upload-controls>button,#upload-review .upload-pointer-panel>button,#upload-review .upload-consumer-panel>button,#upload-review .upload-pointer-panel select{font:inherit;margin:0 12px 12px 0}#upload-review .upload-matrix{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px}#upload-review .upload-review-case{min-width:0;border:1px solid #ced6db;padding:14px}#upload-review .upload-case-label{font:600 13px/1.4 system-ui,sans-serif;background:#eef2f5;margin:0 0 14px;padding:8px;overflow-wrap:anywhere}#upload-review .upload-sentinel{min-height:80px;padding:8px 0}#upload-review .upload-parent{display:flex;box-sizing:content-box}#upload-review .upload-parent>.ix-upload{width:100%}#upload-review .upload-consumer-flow{display:flex;gap:12px;flex-wrap:wrap;align-items:center}#upload-review .upload-consumer-flow>.upload-review-case{flex:1 1 260px;max-width:550px}#upload-review .upload-consumer-flow button{font:inherit}#upload-review pre[hidden]{display:none}@media(max-width:760px){#upload-review{padding:12px}#upload-review .upload-matrix{grid-template-columns:1fr}}
"#;
const OBSERVE: &str = r#"return (()=>{
if(window.ixUploadReview)return {installed:false};
const spec=JSON.parse(document.getElementById('upload-spec').value),objects=new WeakMap(),events=new WeakMap(),errors=[],journal=[],consumerNative=[],pointerRows=[];let nextObject=0,nextEvent=0,sequence=0,busy=false;
const byId=id=>document.getElementById('upload-'+id);
const oid=n=>{if(!n)return null;if(!objects.has(n))objects.set(n,++nextObject);return objects.get(n)};
const eid=e=>{if(!events.has(e))events.set(e,++nextEvent);return events.get(e)};
const log=(kind,details={})=>journal.push({sequence:++sequence,kind,time:performance.now(),...details});
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const wait=async(predicate,label)=>{const end=performance.now()+30000;while(performance.now()<end){if(predicate())return;await sleep(5)}throw Error('Public Upload receipt timed out: '+label)};
const rows=()=>Array.from(document.querySelectorAll('#upload-review .upload-matrix [data-upload-row="true"]'));
const host=wrap=>wrap?.querySelector('.ix-upload.ix-upload-visual');
const native=wrap=>host(wrap)?.querySelector('input.ix-upload-browser');
const nativeButton=wrap=>host(wrap)?.querySelector('.ix-upload-button-host button.ix-upload-button');
const props=wrap=>JSON.parse(wrap.dataset.case);
const metric=(n,root,pseudo=null)=>{if(!(n instanceof Element))return null;const cs=getComputedStyle(n,pseudo),r=pseudo?null:n.getBoundingClientRect(),rr=root.getBoundingClientRect();return{object:oid(n),tag:n.tagName,id:n.id,class:n.getAttribute('class'),attrs:Object.fromEntries([...n.attributes].map(a=>[a.name,a.value])),pseudo,nativeLayout:pseudo?null:{width:n.offsetWidth??null,height:n.offsetHeight??null,left:n.offsetLeft??null,top:n.offsetTop??null},animation:{name:cs.animationName,transform:cs.transform},computed:Object.fromEntries(spec.computedSchema.map(k=>[k,cs[k]])),rect:r?{x:r.x,y:r.y,width:r.width,height:r.height,relativeX:r.x-rr.x,relativeY:r.y-rr.y}:null}};
const capture=wrap=>{const h=host(wrap),p=props(wrap),area=h?.querySelector('.ix-upload-area'),state=h?.querySelector('.ix-upload-state'),text=h?.querySelector('.ix-upload-text'),input=native(wrap),buttonHost=h?.querySelector('.ix-upload-button-host'),button=nativeButton(wrap),label=button?.querySelector('.ix-button-content'),icon=state?.querySelector('.ix-upload-status-icon'),spinnerHost=state?.querySelector('.ix-upload-spinner-host'),spinner=spinnerHost?.querySelector('.ix-spinner');if(!h||!input||!button)throw Error('Missing actual Upload parts '+wrap.id);return{id:wrap.id,key:wrap.dataset.key,scopeIndex:Number(wrap.dataset.scope),revision:Number(wrap.dataset.revision),props:p,host:metric(h,h),parent:metric(h.parentElement,h),area:metric(area,h),state:metric(state,h),text:metric(text,h),actions:metric(h.querySelector('.ix-upload-actions'),h),input:metric(input,h),buttonHost:metric(buttonHost,h),button:metric(button,h),buttonLabel:metric(label,h),icon:metric(icon,h),spinnerHost:metric(spinnerHost,h),spinner:metric(spinner,h),spinnerBefore:metric(spinner,h,'::before'),spinnerAfter:metric(spinner,h,'::after'),resolvedAliases:Object.fromEntries(spec.aliases.map(k=>[k,getComputedStyle(h).getPropertyValue(k)])),buttonAliases:Object.fromEntries(spec.buttonAliases.map(k=>[k,getComputedStyle(button).getPropertyValue(k)])),textContent:text?.textContent??null,buttonText:button?.textContent??null,nativeFacts:{focused:button===document.activeElement,focusVisible:button.matches(':focus-visible'),hover:button.matches(':hover'),active:button.matches(':active'),hostHover:h.matches(':hover'),hostActive:h.matches(':active'),activeId:document.activeElement?.id??null,activeTag:document.activeElement?.tagName??null,type:input.type,disabled:input.disabled,buttonDisabled:button.disabled,hostDisabled:h.dataset.disabled==='true',hostState:h.dataset.state,tabIndex:input.tabIndex,accept:input.accept,multiple:input.multiple,directory:input.hasAttribute('directory'),webkitdirectory:input.hasAttribute('webkitdirectory'),ariaLabel:input.getAttribute('aria-label'),ariaDisabled:input.getAttribute('aria-disabled'),inputFiles:[...input.files??[]].map(f=>({name:f.name,size:f.size,type:f.type,relativePath:f.webkitRelativePath})),buttonObject:oid(button),hostObject:oid(h),inputObject:oid(input)},callbacks:JSON.parse(wrap.querySelector('.upload-callbacks').value)}};
const expectedText=p=>p.disabled?(p.i18nUploadDisabled??'File upload currently not possible.'):p.state==='loading'?(p.loadingText??(p.directory?'Checking folder…':'Checking files…')):p.state==='failed'?(p.uploadFailedText??'Upload failed. Please try again.'):p.state==='success'?(p.uploadSuccessText??'Upload successful'):(p.selectFileText??(p.directory?'+ Drag folder here or…':'+ Drag files here or…'));
const expectedButton=p=>p.i18nUploadFile??(p.directory?'Upload folder…':'Upload file…');
const receipt=async(wrap,revision=Number(wrap.dataset.revision))=>{await wait(()=>{const p=props(wrap),h=host(wrap),n=native(wrap),b=nativeButton(wrap),blocked=!!p.disabled||p.state==='loading';return Number(wrap.dataset.revision)===revision&&h?.dataset.state===p.state&&h.dataset.disabled===String(!!p.disabled)&&n?.disabled===blocked&&b?.disabled===blocked&&n.type==='file'&&n.tabIndex===-1&&n.accept===(p.accept??'')&&n.multiple===(!!p.multiple||!!p.directory)&&n.hasAttribute('directory')===!!p.directory&&n.hasAttribute('webkitdirectory')===!!p.directory&&h.querySelector('.ix-upload-text')?.textContent===expectedText(p)&&b.textContent===expectedButton(p)},wrap.id+' actual revision/native attrs/button/state/text');const h=host(wrap),n=native(wrap),b=nativeButton(wrap);log('rust-public-native-commit',{id:wrap.id,revision:Number(wrap.dataset.revision),hostState:h.dataset.state,hostDisabled:h.dataset.disabled,input:{type:n.type,disabled:n.disabled,tabIndex:n.tabIndex,accept:n.accept,multiple:n.multiple,directory:n.hasAttribute('directory'),webkitdirectory:n.hasAttribute('webkitdirectory')},buttonDisabled:b.disabled,text:h.querySelector('.ix-upload-text').textContent,buttonText:b.textContent})};
const settle=async label=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{label,index,timestamp});resolve()})))};
const setRevision=async id=>{const revision=spec.revisions.find(r=>r.id===id)?.revision;if(revision===undefined)throw Error('Unknown revision '+id);byId('outside').focus();byId(id).click();await wait(()=>Number(byId('review').dataset.revision)===revision,'root data-revision '+id);await Promise.all(rows().map(wrap=>receipt(wrap,revision)));await settle('rust-public-render-'+id);return revision};
const consumerState=()=>{const model=JSON.parse(byId('consumer-json').value);return{...model,files:model.requests.filter(r=>r.kind==='contentRead').flatMap(r=>r.files),nativeEvents:consumerNative.slice(),row:capture(byId('consumer'))}};
const consumerCommand=async command=>{const before=Number(byId('review').dataset.consumerCommand);byId('consumer-'+command).click();await wait(()=>Number(byId('review').dataset.consumerCommand)>before,'Rust consumer command '+command);await receipt(byId('consumer'));await settle('rust-consumer-'+command);return consumerState()};
const eventFacts=e=>({event:eid(e),type:e.type,trusted:e.isTrusted,defaultPrevented:e.defaultPrevented,bubbles:e.bubbles,composed:e.composed,cancelable:e.cancelable,key:e.key??null,code:e.code??null,button:e.button??null,buttons:e.buttons??null,pointerType:e.pointerType??null,targetTag:e.target?.tagName,path:e.composedPath().filter(n=>n instanceof Element).map(n=>({tag:n.tagName,id:n.id,object:oid(n)}))});
const pointerWrap=()=>byId('pointer-target')?.querySelector('.upload-review-case');
const pointerOutput=()=>{const value={case:byId('pointer-target')?.dataset.pointerCase,rows:pointerRows.slice(),consumer:consumerState()};byId('pointer-result').textContent=JSON.stringify(value);return value};
const watch=(mount,consumer=false)=>{for(const type of ['pointerdown','pointerup','keydown','keyup','click','input','change','cancel','focusin','focusout'])mount.addEventListener(type,e=>{const original=eventFacts(e);if(consumer){const n=native(byId('consumer'));consumerNative.push({sequence:consumerNative.length+1,original,state:host(byId('consumer'))?.dataset.state,disabled:n?.disabled,files:[...n?.files??[]].map(f=>({name:f.name,size:f.size,type:f.type,relativePath:f.webkitRelativePath})),callbacks:JSON.parse(byId('consumer').querySelector('.upload-callbacks').value)})}const count=(type==='pointerup'||type==='keyup'||type==='keydown')?2:1;let frame=0;function next(){requestAnimationFrame(()=>{frame++;if(frame<count)return next();original.defaultPreventedAfterDispatch=e.defaultPrevented;const wrap=consumer?byId('consumer'):pointerWrap();if(!wrap)return;const record={sequence:pointerRows.length+1,phase:type,frame,original,row:capture(wrap)};if(!consumer){pointerRows.push(record);pointerOutput()}else log('consumer-original-input-frame',record)})}next()},{capture:true,passive:true})};
const configurePointer=async id=>{const select=byId('pointer-case'),target=byId('pointer-target');select.value=id;select.dispatchEvent(new Event('change',{bubbles:true}));await wait(()=>target.dataset.pointerCase===id,'Rust pointer case '+id);const old=host(pointerWrap());byId('pointer-arm').click();await wait(()=>host(pointerWrap())!==old,'fresh keyed pointer host');pointerRows.length=0;await receipt(pointerWrap(),0);await settle('rust-pointer-'+id);byId('pointer-status').dataset.ready='true';byId('pointer-status').dataset.case=id;byId('pointer-status').textContent='Ready: '+id;return pointerOutput()};
const run=async()=>{if(busy)return;busy=true;const phases=[];try{await wait(()=>rows().length===24,'twenty-four matrix rows');for(const rev of spec.revisions){await setRevision(rev.id);phases.push({id:rev.id,revision:rev.revision,rows:rows().map(capture)})}const result={ok:errors.length===0,revision:spec.revision,fixture:spec.fixture,phases,controlled:consumerState(),pointer:pointerOutput(),errors:errors.slice(),journal:journal.slice(),limits:['Authored semantic tokens and system fonts only, no full Brand metrics','Source shadow/custom hosts versus Rust native SVG/input APIs retained raw','No automatic picker activation; genuine pointer/key/chooser observations are separate','Independent continuous Spinner animation phases are raw, no motion parity certification','Source setFilesToUpload is excluded','Rust FileData content reads and accepting state remain Rust; source filesChanged is a DOM CustomEvent']};byId('result').textContent=JSON.stringify(result);byId('status').dataset.run='done';return result}catch(error){errors.push(String(error));const result={ok:false,error:String(error),phases,errors:errors.slice()};byId('result').textContent=JSON.stringify(result);byId('status').dataset.run='done';return result}finally{busy=false}};
watch(byId('pointer-target'));watch(byId('consumer-target'),true);
byId('pointer-arm').addEventListener('click',()=>{byId('pointer-status').dataset.ready='false';const previous=host(pointerWrap());(async()=>{await wait(()=>host(pointerWrap())!==previous,'keyed Rust original pointer target');pointerRows.length=0;await receipt(pointerWrap(),0);await settle('rust-pointer-manual-arm');byId('pointer-status').dataset.ready='true';byId('pointer-status').textContent='Ready: '+byId('pointer-target').dataset.pointerCase})().catch(error=>errors.push(String(error)))},{capture:true});
window.addEventListener('error',event=>errors.push(event.message));window.addEventListener('unhandledrejection',event=>errors.push(String(event.reason)));
window.ixUploadReview={spec,run,setRevision,consumerState,consumerCommand,configurePointer,pointerOutput,capture:()=>rows().map(capture),captureRow:capture,metric,receipt,settle,rows,errors,journal};
return{installed:true};
})()"#;

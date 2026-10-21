//! Development-only native/web Spinner presentation runner.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "spinner_review.rs"]
mod spinner_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_SPINNER_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_spinner_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Spinner presentation</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Spinner"
                } else {
                    "Dioxus iX Spinner"
                })
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}
#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Probe);
}
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _fixture = Probe;
    eprintln!("Run the Spinner review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_SPINNER_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(180)).await;
            eprintln!("Spinner native timeout");
            std::process::exit(1);
        });
    });
    rsx! { div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            { let window=window.clone(); spawn(async move {
                let query=dom_query::eval(VERIFY); let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"] == true => { println!("Spinner native observations: {value}"); window.close(); }
                    Ok(value) => { eprintln!("Spinner native failure JSON: {value}"); std::process::exit(1); }
                    Err(error) => { eprintln!("Spinner native transport failure: {error:?}"); std::process::exit(1); }
                }
            }); }
        },
        {web_controls()}
        if !source { spinner_review::SpinnerReview {} }
    } }
}
#[cfg(not(target_arch = "wasm32"))]
fn web_controls() -> Element {
    VNode::empty()
}
#[cfg(target_arch = "wasm32")]
fn web_controls() -> Element {
    rsx! { WebRunner {} }
}
#[cfg(target_arch = "wasm32")]
#[component]
fn WebRunner() -> Element {
    let mut busy = use_signal(|| false);
    let mut status = use_signal(|| "idle".to_string());
    let mut result = use_signal(String::new);
    rsx! { section { style:"padding:16px 24px;font:14px/1.5 system-ui,sans-serif",
        button { id:"spinner-web-run",r#type:"button",disabled:busy(),aria_busy:busy(),
            onclick:move |_| { if busy() { return; } busy.set(true);status.set("running".into());spawn(async move {
                let query=dioxus::document::eval(VERIFY);
                if let Err(error)=query.send(false) { result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());status.set("error".into());busy.set(false);return; }
                match query.join::<Value>().await {
                    Ok(mut value)=>{let ok=value["ok"]==true;if let Some(object)=value.as_object_mut(){object.insert("renderer".into(),Value::String("web browser Rust".into()));}result.set(value.to_string());status.set(if ok{"done"}else{"error"}.into());}
                    Err(error)=>{result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());status.set("error".into());}
                }
                busy.set(false);
            }); },
            "Run Spinner presentation states"
        }
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 112 Spinner presentation observations."} else {"{status()}"}}
        pre {id:"spinner-web-result",hidden:true,"aria-hidden":"true","data-run":status(),"{result()}"}
    } }
}
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
const VERIFY: &str = r#"
const source=await dioxus.recv(),assertions=[],errors=[];
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const wait=async(predicate,label)=>{const until=performance.now()+45000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
try{
 await wait(()=>window.ixSpinnerReview?.run,'Spinner public observer API');
 if(source)await wait(()=>document.getElementById('spinner-status')?.dataset.ready==='true','pinned source public readiness acknowledgement');
 const result=await window.ixSpinnerReview.run();
 const count=result.phases?.reduce((sum,phase)=>sum+phase.rows.length,0);
 assertions.push({ok:count===112,label:'fifty-six actual cases in each of two revisions'});
 if(count!==112)errors.push('Expected 112 actual rows; observed '+count);
 const consumers=result.phases?.reduce((sum,phase)=>sum+(phase.consumers?.length??0),0),motion=result.motion?.reduce((sum,row)=>sum+(row.samples?.length??0),0),running=result.running?.flatMap(phase=>phase.rows??[]);
 assertions.push({ok:consumers===62,label:'thirty-one actual loading consumers per revision'},{ok:motion===36,label:'four actual animations at nine native seek fractions'},{ok:running?.length===8&&running.every(row=>row.advanced===true),label:'two native frames advance each selected running CSSAnimation before pause'});
 if(consumers!==62)errors.push('Expected 62 actual consumer rows; observed '+consumers);
 if(motion!==36)errors.push('Expected 36 actual motion samples; observed '+motion);
 if(running?.length!==8||!running.every(row=>row.advanced===true))errors.push('Expected eight advancing native running animation observations');
 return{...result,ok:result.ok===true&&errors.length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:source?'Linux desktop WebView pinned source':'Linux desktop WebView Rust'};
}catch(error){return{ok:false,errors:[...errors,String(error)],assertions};}
"#;

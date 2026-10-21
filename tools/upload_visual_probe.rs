//! Development-only native/web Upload presentation runner.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "upload_review.rs"]
mod upload_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_UPLOAD_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_upload_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Upload presentation</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Upload"
                } else {
                    "Dioxus iX Upload"
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
    eprintln!("Run the Upload review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_UPLOAD_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(180)).await;
            eprintln!("Upload native timeout");
            std::process::exit(1);
        });
    });
    rsx! { div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            { let window=window.clone(); spawn(async move {
                let query=dom_query::eval(VERIFY); let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"] == true => { println!("Upload native observations: {value}"); window.close(); }
                    Ok(value) => { eprintln!("Upload native failure JSON: {value}"); std::process::exit(1); }
                    Err(error) => { eprintln!("Upload native transport failure: {error:?}"); std::process::exit(1); }
                }
            }); }
        },
        {web_controls()}
        if !source { upload_review::UploadReview {} }
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
        button { id:"upload-web-run",r#type:"button",disabled:busy(),aria_busy:busy(),
            onclick:move |_| { if busy() { return; } busy.set(true);status.set("running".into());spawn(async move {
                let query=dioxus::document::eval(VERIFY);
                if let Err(error)=query.send(false) { result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());status.set("error".into());busy.set(false);return; }
                match query.join::<Value>().await {
                    Ok(mut value)=>{let ok=value["ok"]==true;if let Some(object)=value.as_object_mut(){object.insert("renderer".into(),Value::String("web browser Rust".into()));}result.set(value.to_string());status.set(if ok{"done"}else{"error"}.into());}
                    Err(error)=>{result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());status.set("error".into());}
                }
                busy.set(false);
            }); },
            "Run Upload presentation states"
        }
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 48 Upload presentation observations."} else {"{status()}"}}
        pre {id:"upload-web-result",hidden:true,"aria-hidden":"true","data-run":status(),"{result()}"}
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
 await wait(()=>window.ixUploadReview?.run,'Upload public observer API');
 if(source)await wait(()=>document.getElementById('upload-status')?.dataset.ready==='true','pinned source public readiness acknowledgement');
 const result=await window.ixUploadReview.run();
 const count=result.phases?.reduce((sum,phase)=>sum+phase.rows.length,0);
 assertions.push({ok:count===48,label:'twenty-four actual cases in each of two revisions'});
 if(count!==48)errors.push('Expected 48 actual rows; observed '+count);
 return{...result,ok:result.ok===true&&errors.length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:source?'Linux desktop WebView pinned source':'Linux desktop WebView Rust'};
}catch(error){return{ok:false,errors:[...errors,String(error)],assertions};}
"#;

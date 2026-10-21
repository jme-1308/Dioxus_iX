//! Excluded native ExpandingSearch computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "expanding_search_review.rs"]
mod expanding_search_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_SEARCH_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_expanding_search_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ExpandingSearch sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX ExpandingSearch"
                } else {
                    "Dioxus iX ExpandingSearch"
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
    eprintln!("Run the ExpandingSearch review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_SEARCH_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(180)).await;
            eprintln!("ExpandingSearch native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("ExpandingSearch native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("ExpandingSearch native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("ExpandingSearch native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {expanding_search_review::ExpandingSearchReview {}}
    }}
}

#[cfg(target_arch = "wasm32")]
fn web_controls() -> Element {
    rsx! {WebRunner {}}
}
#[cfg(not(target_arch = "wasm32"))]
fn web_controls() -> Element {
    VNode::empty()
}

#[cfg(target_arch = "wasm32")]
#[component]
fn WebRunner() -> Element {
    let mut busy = use_signal(|| false);
    let mut result = use_signal(String::new);
    let mut status = use_signal(|| "idle".to_string());
    rsx! {section {style:"padding:16px 24px;font:14px/1.5 system-ui,sans-serif",
        button {id:"search-run",r#type:"button",disabled:busy(),aria_busy:busy(),
            onclick:move |_| {
                if busy() {return;}
                busy.set(true);status.set("running".into());
                spawn(async move {
                    let query=dioxus::document::eval(VERIFY);
                    if let Err(error)=query.send(false) {
                        result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());
                        status.set("error".into());busy.set(false);return;
                    }
                    match query.join::<Value>().await {
                        Ok(mut value)=>{
                            let ok=value["ok"]==true;
                            if let Some(object)=value.as_object_mut() {
                                object.insert("renderer".to_string(),Value::String("web browser Rust".to_string()));
                            }
                            result.set(value.to_string());status.set(if ok{"done"}else{"error"}.to_string());
                        }
                        Err(error)=>{
                            result.set(serde_json::json!({"ok":false,"error":format!("{error:?}")}).to_string());
                            status.set("error".into());
                        }
                    }
                    busy.set(false);
                });
            },
            "Run token states"
        }
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 72 ExpandingSearch token/state observations."} else {"{status()}"}}
        pre {id:"search-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:120px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
    }}
}

#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
const VERIFY: &str = r#"
const source=await dioxus.recv(),assertions=[],errors=[],phases=[];
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const wait=async(predicate,label)=>{const until=performance.now()+25000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const check=(ok,label)=>{assertions.push({ok:!!ok,label});if(!ok)throw Error(label);};
let api=null;
try{
 if(source){await wait(()=>window.upstreamExpandingSearchReview&&document.getElementById('search-status')?.dataset.ready==='true','pinned source readiness');const result=await window.upstreamExpandingSearchReview.run();check(result?.ok===true,'source run completed');check(result.phases?.length===2,'source two revisions');check(result.phases.reduce((sum,phase)=>sum+phase.rows.length,0)===72,'source 72 actual observations');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>window.ixExpandingSearchReview?.rows().length===36,'Rust observer and 36 cases committed');
 api=window.ixExpandingSearchReview;
 const {capture,hostFor,inputFor,frames,log,originalClick,nativeInput,valueReceipt,requestCount,requestReceipt,blurReceipt,expand,timer}=api;
 const outside=()=>document.getElementById('search-outside');
 const keyboard=()=>{outside().focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});outside().dispatchEvent(event);log('original-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
 const props=wrap=>JSON.parse(wrap.dataset.case);
 for(const id of ['baseline','revised']){
  outside().focus();document.getElementById('search-'+id).click();await wait(()=>document.getElementById('search-review').dataset.phase===id&&api.rows().every(wrap=>wrap.dataset.revision===String(id==='revised'?1:0)),'Rust revision commit '+id);
  await Promise.all(api.rows().map(valueReceipt));await frames();keyboard();
  const edited=api.rows().filter(wrap=>wrap.dataset.pose==='edited');
  for(const wrap of edited){expand(wrap);nativeInput(wrap,props(wrap).editedValue??'Query');}
  if(edited.length){await timer(edited[edited.length-1]);await Promise.all(edited.map(valueReceipt));await frames();}
  const rows=[];
  for(const wrap of api.rows()){
   outside().focus();if(wrap.dataset.pose==='pristine'){expand(wrap);await timer(wrap);}
   await valueReceipt(wrap);await frames();rows.push(capture(wrap));
  }
  check(rows.length===36,id+' 36 actual rows');phases.push({id,revision:id==='revised'?1:0,rows,settleFrames:2,settlement:{publicValueReceipt:true,actual300msTimers:true,fontsReady:true,selectedRowFrames:2}});
 }
 const consumerWrap=document.getElementById('search-consumer');
 const consumerState=()=>({...JSON.parse(document.getElementById('search-consumer-json').value),row:capture(consumerWrap)});
 const command=async name=>{const root=document.getElementById('search-review'),before=Number(root.dataset.consumerCommand);document.getElementById('search-consumer-'+name).click();await wait(()=>Number(root.dataset.consumerCommand)>before,'consumer public command '+name);await wait(()=>inputFor(consumerWrap).value===JSON.parse(document.getElementById('search-consumer-json').value).value,'consumer public/native value '+name);log('public-consumer-native-value-commit',{name,command:Number(root.dataset.consumerCommand),rustValue:JSON.parse(document.getElementById('search-consumer-json').value).value,nativeValue:inputFor(consumerWrap).value,object:api.object(inputFor(consumerWrap))});await frames();};
 const existingClear=hostFor(consumerWrap).querySelector('.ix-search-clear');if(existingClear){const count=requestCount(consumerWrap);originalClick(existingClear);await requestReceipt(consumerWrap,count);await valueReceipt(consumerWrap);await frames();}
 await command('reset');inputFor(consumerWrap).focus();await blurReceipt(consumerWrap,outside());await frames();
 const steps=[];
 for(const name of ['initial','expand','empty-blur','type','typed-blur','clear','clear-empty-blur','first-empty','first-empty-blur','second-empty','second-empty-blur','external-value','escape']){
  const before=consumerState(),callbacksBefore=requestCount(consumerWrap);let original=null;
  if(['expand','type','first-empty','escape'].includes(name)){
   original=expand(consumerWrap);
   if(name==='type')original={click:original,input:nativeInput(consumerWrap,'Consumer Query')};
   if(name==='first-empty')original={click:original,input:nativeInput(consumerWrap,'')};
   await timer(consumerWrap);
  }
  if(['empty-blur','typed-blur','clear-empty-blur','first-empty-blur','second-empty-blur'].includes(name))await blurReceipt(consumerWrap,outside());
  if(name==='clear')original=originalClick(hostFor(consumerWrap).querySelector('.ix-search-clear'));
  if(name==='second-empty'){inputFor(consumerWrap).focus();original=nativeInput(consumerWrap,'');}
  if(name==='external-value')await command('external');
  if(name==='escape'){const control=inputFor(consumerWrap),event=new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true,composed:true,cancelable:true});const dispatched=control.dispatchEvent(event);original={click:original,key:{type:'keydown',key:'Escape',dispatched,defaultPrevented:event.defaultPrevented,trusted:event.isTrusted,target:api.object(control)}};}
  if(['type','clear','first-empty','second-empty'].includes(name))await requestReceipt(consumerWrap,callbacksBefore);
  await valueReceipt(consumerWrap);await frames();const after=consumerState();steps.push({name,before,after,requestsDelta:after.requests.slice(before.requests.length),original,row:capture(consumerWrap)});
 }
 const initialWrap=document.getElementById('search-search-initial-value');
 const extensionInput=document.querySelector('#search-extension > .ix-expanding-search .ix-search-input'),extensionKey=new KeyboardEvent('keydown',{key:'Enter',code:'Enter',bubbles:true,composed:true,cancelable:true});const extensionBefore=Number(document.getElementById('search-extension-count').textContent);extensionInput.dispatchEvent(extensionKey);await frames();const extensions={unpaired:true,enter:{defaultPrevented:extensionKey.defaultPrevented,callbackDelta:Number(document.getElementById('search-extension-count').textContent)-extensionBefore},disabled:{input:document.querySelector('#search-extension-disabled input').disabled,search:document.querySelector('#search-extension-disabled .ix-search-trigger').disabled}};
 const identities=phases[0].rows.map((row,index)=>({id:row.id,sameHost:row.host.object===phases[1].rows[index].host.object,sameControl:row.control.object===phases[1].rows[index].control.object,sameSearch:row.search.object===phases[1].rows[index].search.object,sameSearchControl:row.searchControl.object===phases[1].rows[index].searchControl.object}));
 check(identities.every(row=>row.sameHost&&row.sameControl&&row.sameSearch&&row.sameSearchControl),'all main actual search/input objects retained across revisions');
 return{ok:errors.length===0&&api.errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token ExpandingSearch adapter; not full Brand metrics',phases,identities,extensions,controlled:{steps,state:consumerState(),initialValueDiagnostic:capture(initialWrap)},pointer:api.pointer(),journal:api.journal,errors:[...errors,...api.errors],assertions,renderer:'Linux desktop WebView Rust',limits:['Source mutable value versus accepting Rust controlled requests is an explicit API adapter','Native public value receipts precede fonts/two fixed frames; source component/renderer publication chronology is not asserted equal','Original search clicks use real 300ms elapsed observations; scoped timer IPC exact ordering remains open','Source custom IconButton host maps separate native span; inner controls and direct SVG glyphs retained separately','Supplied 0s/.001s timing tokens verify settled declarations, not full motion interpolation','Disabled and Enter onsearch are Rust-only extensions outside the paired72 and consumer histories','No-token fallback is provisional Brand evidence; trusted held pointer capture is separate']};
}catch(error){errors.push(String(error));return{ok:false,phases,errors,journal:api?.journal??[],assertions};}
"#;

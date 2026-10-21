//! Excluded native Slider computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "slider_review.rs"]
mod slider_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_SLIDER_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_slider_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Slider sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Slider"
                } else {
                    "Dioxus iX Slider"
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
    eprintln!("Run the Slider review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_SLIDER_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(180)).await;
            eprintln!("Slider native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("Slider native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Slider native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Slider native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {slider_review::SliderReview {}}
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
        button {id:"slider-run",r#type:"button",disabled:busy(),aria_busy:busy(),
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
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 72 Slider token/state observations."} else {"{status()}"}}
        pre {id:"slider-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:120px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
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
 if(source){await wait(()=>window.upstreamSliderReview&&document.getElementById('slider-status')?.dataset.ready==='true','pinned source readiness');const result=await window.upstreamSliderReview.run();check(result?.ok===true,'source run completed');check(result.phases?.length===2,'source two revisions');check(result.phases.reduce((sum,phase)=>sum+phase.rows.length,0)===72,'source 72 actual observations');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>window.ixSliderReview?.rows().length===36,'Rust observer and 36 cases committed');api=window.ixSliderReview;
 const {capture,inputFor,frames,log,nativeInput,valueReceipt,requestCount,requestReceipt,focusReceipt,blurReceipt}=api;
 const outside=()=>document.getElementById('slider-outside');
 const keyboard=()=>{outside().focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});outside().dispatchEvent(event);log('original-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
 for(const id of ['baseline','revised']){
  await blurReceipt(outside());document.getElementById('slider-'+id).click();await wait(()=>document.getElementById('slider-review').dataset.phase===id&&api.rows().every(wrap=>wrap.dataset.revision===String(id==='revised'?1:0)),'actual Rust revision commit '+id);
  await Promise.all(api.rows().map(valueReceipt));await frames();keyboard();const rows=[];
  for(const wrap of api.rows()){
   await blurReceipt(outside());if(wrap.dataset.condition==='focus')await focusReceipt(wrap);
   await valueReceipt(wrap);await frames();rows.push(capture(wrap));
  }
  check(rows.length===36,id+' 36 actual rows');phases.push({id,revision:id==='revised'?1:0,rows,settleFrames:2,settlement:{publicValueReceipt:true,focusEventReceipt:true,fontsReady:true,selectedRowFrames:2}});
 }
 await blurReceipt(outside());
 const wrap=document.getElementById('slider-consumer'),consumerState=()=>({...JSON.parse(document.getElementById('slider-consumer-json').value),nativeValue:inputFor(wrap).value,formData:Array.from(new FormData(document.getElementById('slider-consumer-form')).entries()),row:capture(wrap)});
 const command=async name=>{const root=document.getElementById('slider-review'),before=Number(root.dataset.consumerCommand);document.getElementById('slider-consumer-'+name).click();await wait(()=>Number(root.dataset.consumerCommand)>before,'consumer public command '+name);await wait(()=>Number(inputFor(wrap).value)===JSON.parse(document.getElementById('slider-consumer-json').value).value,'consumer actual public/native value '+name);log('public-consumer-native-value-commit',{name,command:Number(root.dataset.consumerCommand),rustValue:JSON.parse(document.getElementById('slider-consumer-json').value).value,nativeValue:inputFor(wrap).value,object:api.object(inputFor(wrap))});await frames();};
 await command('reset');const steps=[];
 for(const operation of [{name:'input-20',value:20},{name:'input-65',value:65},{name:'input-min',value:0},{name:'input-max',value:100},{name:'external-45',value:45},{name:'disabled-input',value:60}]){
  if(operation.name==='disabled-input')await command('disable');const before=consumerState();let original=null;
  if(operation.name==='external-45')await command('external');
  else {const beforeCount=requestCount(wrap);original=nativeInput(wrap,operation.value);if(operation.name!=='disabled-input'){await requestReceipt(wrap,beforeCount);await valueReceipt(wrap);}}
  await frames();const after=consumerState();steps.push({name:operation.name,before,after,requestsDelta:after.requests.slice(before.requests.length),original,row:capture(wrap)});
 }
 const identities=phases[0].rows.map((row,index)=>({id:row.id,sameHost:row.host.object===phases[1].rows[index].host.object,sameControl:row.control.object===phases[1].rows[index].control.object,sameTrack:row.track.object===phases[1].rows[index].track.object,sameAnchor:row.anchor.object===phases[1].rows[index].anchor.object,sameTicks:row.ticks.object===phases[1].rows[index].ticks.object}));
 check(identities.every(row=>row.sameHost&&row.sameControl&&row.sameTrack&&row.sameAnchor&&row.sameTicks),'main actual host/input/track/anchor/tickSVG retained across revisions');
 return{ok:errors.length===0&&api.errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token Slider presentation adapter; not full Brand metrics',phases,identities,controlled:{steps,state:consumerState()},pointer:api.pointer(),journal:api.journal,errors:[...errors,...api.errors],assertions,renderer:'Linux desktop WebView Rust',limits:['Source accepted native input emits two actual valueChange events; accepting Rust consumer receives one callback; request count/cancellation are explicit API adapters','Source disabled host suppresses pointer/tab navigation but its input lacks disabled and accepts synthetic input; Rust disabled native/handler blocking is retained separately','Rust public value and actual input receipts precede fonts/two fixed frames, not source/custom-element/IPC chronology parity','Actual native pseudo CSSOM is raw engine output; transparent anchor is not a simulated native thumb and painted thumb cannot be inferred universally from CSSOM','Marker captions/show-value and full tooltip FloatingUI/drag/global pointerup/form/AT/RTL remain outside paired72','Authored0s/.001s motion verifies settled declarations, not full interpolation','Trusted held pointer and real native keyboard evidence use separate public pointer panel; no synthetic hover/focus classes','No-token fallback remains provisional Brand evidence']};
}catch(error){errors.push(String(error));return{ok:false,phases,errors,journal:api?.journal??[],assertions};}
"#;

//! Excluded native Group computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "group_review.rs"]
mod group_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_GROUP_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_group_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Group sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Group"
                } else {
                    "Dioxus iX Group"
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
    eprintln!("Run the Group review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_GROUP_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(120)).await;
            eprintln!("Group token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("Group token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Group token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Group token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {group_review::GroupReview {}}
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
        button {id:"group-run",r#type:"button",disabled:busy(),aria_busy:busy(),
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
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 80 Group token/state observations."} else {"{status()}"}}
        pre {id:"group-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:120px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
    }}
}

#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
const VERIFY: &str = r#"
const source=await dioxus.recv(),errors=[],phases=[],journal=[],assertions=[],objects=new WeakMap();let nextObject=0,sequence=0;
const abort=new AbortController(),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const log=(kind,details={})=>journal.push({seq:++sequence,kind,time:performance.now(),...details});
const object=node=>{if(!objects.has(node))objects.set(node,++nextObject);return objects.get(node);};
window.addEventListener('error',event=>errors.push(event.message),{signal:abort.signal});window.addEventListener('unhandledrejection',event=>errors.push(String(event.reason)),{signal:abort.signal});
const wait=async(predicate,label)=>{const until=performance.now()+20000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const check=(condition,label)=>{assertions.push({ok:!!condition,label});if(!condition)throw Error(label);};
const fields=["display","width","height","minWidth","boxSizing","paddingTop","paddingRight","paddingBottom","paddingLeft","borderTopWidth","borderTopColor","borderTopStyle","borderTopLeftRadius","borderTopRightRadius","borderBottomLeftRadius","borderBottomRightRadius","backgroundColor","color","fontSize","lineHeight","fontWeight","fontFamily","fontStyle","letterSpacing","fontFeatureSettings","outlineColor","outlineWidth","outlineStyle","outlineOffset","opacity","pointerEvents","justifyContent","textAlign","marginLeft","marginRight","maxWidth","minHeight","maxHeight","position","flexShrink","gap","top","right","bottom","left","content","fill","stroke","strokeWidth","cursor","visibility","alignItems","marginTop","marginBottom","borderRightWidth","borderBottomWidth","borderLeftWidth","transform","transitionProperty","transitionDuration","transitionTimingFunction","transitionDelay","flexDirection","wordBreak","whiteSpace","overflow","marginBlockStart","marginBlockEnd","marginInlineStart","rowGap","columnGap","flexWrap","marginInlineEnd","minInlineSize"];
const metrics=(node,origin)=>{if(!(node instanceof Element))return null;const cs=getComputedStyle(node),r=node.getBoundingClientRect(),o=origin.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height,relativeX:r.x-o.x,relativeY:r.y-o.y}};};
const capture=wrap=>{
 const host=wrap.querySelector('.ix-group-native');if(!(host instanceof HTMLFieldSetElement))throw Error('Missing actual native group '+wrap.id);
 const part=selector=>host.querySelector(selector),wrapper=part('.ix-group-wrapper'),top=part('.ix-group-top'),labelHost=part('.ix-group-label'),label=part('.ix-group-label .ix-typography'),slot=part('.ix-group-slot'),choices=part('.ix-group-choices'),bottom=part('.ix-group-bottom'),feedback=part('.ix-group-bottom-text'),icon=part('.ix-group-text-icon'),right=part('.ix-group-bottom-right');
 const feedbackText=feedback?Array.from(feedback.childNodes).filter(node=>node.nodeType===Node.TEXT_NODE).map(node=>node.textContent).join(''):null;
 return{id:wrap.id,key:wrap.dataset.key,kind:wrap.dataset.kind,scopeIndex:Number(wrap.dataset.scopeIndex),revision:Number(wrap.dataset.revision),props:JSON.parse(wrap.dataset.props),host:metrics(host,host),wrapper:metrics(wrapper,host),top:metrics(top,host),labelHost:metrics(labelHost,host),label:metrics(label,host),labelText:label?.textContent??null,slot:metrics(slot,host),choices:metrics(choices,host),bottom:metrics(bottom,host),feedback:metrics(feedback,host),feedbackText,icon:metrics(icon,host),iconSvg:metrics(icon?.matches('svg')?icon:icon?.querySelector('svg'),host),right:metrics(right,host),children:Array.from(choices?.children??[],node=>metrics(node,host)),requiredMarker:!!label&&Array.from(label.querySelectorAll('span')).some(node=>node.textContent==='*'),nativeFacts:{tag:host.tagName,fieldset:true,disabledProperty:host.disabled,role:host.getAttribute('role'),ariaLabel:host.getAttribute('aria-label'),ariaLabelledby:host.getAttribute('aria-labelledby'),wrapperRole:wrapper?.getAttribute('role')??null,wrapperAriaLive:wrapper?.getAttribute('aria-live')??null,consumer:JSON.parse(wrap.querySelector('.group-consumer-state').value),controlInputs:Array.from(host.querySelectorAll('input'),input=>({object:object(input),type:input.type,name:input.name,value:input.value,checked:input.checked,disabled:input.disabled,required:input.required,tabIndex:input.tabIndex,attrs:Object.fromEntries(Array.from(input.attributes,a=>[a.name,a.value]))}))}};
};
const rows=()=>Array.from(document.querySelectorAll('[data-group-row="true"]'));
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('group-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);button.click();log('public-revision-input',{id});await wait(()=>document.getElementById('group-review')?.dataset.phase===id&&rows().length===40&&rows().every(row=>row.dataset.revision===String(id==='revised'?1:0)),'public Rust token/prop commit '+id);await settle();};
try{
 if(source){await wait(()=>window.upstreamGroupTokenReview&&document.getElementById('group-status')?.dataset.ready==='true','pinned group readiness');const result=await window.upstreamGroupTokenReview.run();check(result?.ok===true,'source completed');check(result.phases?.length===2,'source two revisions');check(result.phases.reduce((sum,phase)=>sum+phase.rows.length,0)===80,'source 80 actual group observations');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>rows().length===40,'40 actual Rust group cases committed');
 for(const id of ['baseline','revised']){await setRevision(id);const observed=rows().map(capture);check(observed.length===40,id+' forty actual groups');check(observed.filter(row=>row.kind==='checkbox').length===20&&observed.filter(row=>row.kind==='radio').length===20,id+' twenty groups per kind');phases.push({id,revision:id==='revised'?1:0,settleFrames:2,rows:observed,settlement:{publicCommit:'actual Rust root and case revision DOM attrs',finalObservationFrames:2,fontsReady:true}});}
 const identities=phases[0].rows.map((row,index)=>({id:row.id,sameHost:row.host.object===phases[1].rows[index].host.object,sameWrapper:row.wrapper.object===phases[1].rows[index].wrapper.object,sameChoices:row.choices.object===phases[1].rows[index].choices.object}));
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token group inline-layout adapter; not full Brand metrics',phases,identities,errors,journal,assertions,renderer:'Linux desktop WebView Rust',limits:['Source custom group and nested shadow FieldWrapper/FieldLabel map to native fieldset with flattened children; raw roles/ARIA/fieldset behavior retained separately','Public Rust token/prop revision commit precedes fonts and two fixed RAF; source settles actual custom-child readiness separately, without equal IPC/publication timing claim','Required source marker derives from real first child.required; Rust explicit group.required and option propagation are separate API adapters','Group styles and anatomy are measured; RadioGroup forwarding state to child controls and independent child feedback are raw outside this named group recipe','Children accept user callbacks in Rust, but this compact layout probe does not certify cancellation, descendant auto-validation, touched/reset, source label-click focus, radio roving, tooltip, physical input or AT','Only public group-host choices gap/margin overrides are authored; source internal wrapper/label/icon aliases remain unmodified','Relative rectangles compare within each host; raw native/custom/shadow node identities and actual glyph SVG facts remain separate','Two no-token fallback groups are provisional display diagnostics, not full numeric Brand parity']};
}catch(error){errors.push(String(error));return{ok:false,phases,errors,journal,assertions};}finally{abort.abort();}
"#;

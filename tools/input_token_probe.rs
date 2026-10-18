//! Excluded native Input computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/input_review.rs"]
mod input_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_INPUT_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_input_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Input sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX Input"
                } else {
                    "Dioxus iX Input"
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
    eprintln!("Run the Input review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_INPUT_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(120)).await;
            eprintln!("Input token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("Input token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("Input token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("Input token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {input_review::InputReview {}}
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
        button {id:"input-run",r#type:"button",disabled:busy(),aria_busy:busy(),
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
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 110 Input token/state observations."} else {"{status()}"}}
        pre {id:"input-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:120px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
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
const wait=async(predicate,label)=>{const until=performance.now()+25000;while(performance.now()<until){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const check=(condition,label)=>{assertions.push({ok:!!condition,label});if(!condition)throw Error(label);};
const fields=["display","width","height","minWidth","boxSizing","paddingTop","paddingRight","paddingBottom","paddingLeft","borderTopWidth","borderTopColor","borderTopStyle","borderTopLeftRadius","borderTopRightRadius","borderBottomLeftRadius","borderBottomRightRadius","backgroundColor","color","fontSize","lineHeight","fontWeight","fontFamily","fontStyle","letterSpacing","fontFeatureSettings","outlineColor","outlineWidth","outlineStyle","outlineOffset","opacity","pointerEvents","justifyContent","textAlign","marginLeft","marginRight","maxWidth","minHeight","maxHeight","position","flexShrink","gap","top","right","bottom","left","content","fill","stroke","strokeWidth","cursor","visibility","alignItems","marginTop","marginBottom","borderRightWidth","borderBottomWidth","borderLeftWidth","transform","transitionProperty","transitionDuration","transitionTimingFunction","transitionDelay","flexDirection","wordBreak","whiteSpace","overflow","marginBlockStart","marginBlockEnd","marginInlineStart","rowGap","columnGap","flexWrap","marginInlineEnd","minInlineSize","appearance","textOverflow","resize","caretColor","outline","userSelect"];
const metrics=(node,origin,pseudo=null)=>{if(!(node instanceof Element))return null;const cs=getComputedStyle(node,pseudo),r=node.getBoundingClientRect(),o=origin.getBoundingClientRect();return{object:object(node),tag:node.tagName,id:node.id,class:node.getAttribute('class'),attrs:Object.fromEntries(Array.from(node.attributes,a=>[a.name,a.value])),computed:Object.fromEntries(fields.map(key=>[key,cs[key]])),rect:{x:r.x,y:r.y,width:r.width,height:r.height,relativeX:r.x-o.x,relativeY:r.y-o.y},pseudo};};
const rows=()=>Array.from(document.querySelectorAll('[data-input-row="true"]'));
const hostFor=wrap=>wrap.querySelector('.ix-text-field-native');
const nativeFor=wrap=>hostFor(wrap)?.querySelector('.ix-input-wrap > input,.ix-input-wrap > textarea');
const geometryReady=wrap=>{if(wrap.dataset.kind!=='input')return true;const host=hostFor(wrap),inputWrap=host?.querySelector('.ix-input-wrap'),start=host?.querySelector('.ix-input-start'),end=host?.querySelector('.ix-input-end');if(!inputWrap||!start||!end||inputWrap.dataset.paddingReady!=='true')return false;const reportedStart=Number(inputWrap.dataset.paddingStart),reportedEnd=Number(inputWrap.dataset.paddingEnd);return Number.isFinite(reportedStart)&&Number.isFinite(reportedEnd)&&Math.abs(reportedStart-start.getBoundingClientRect().width)<0.01&&Math.abs(reportedEnd-end.getBoundingClientRect().width)<0.01;};
const capture=wrap=>{
 const host=hostFor(wrap),control=nativeFor(wrap);if(!(host instanceof HTMLElement)||!(control instanceof HTMLInputElement||control instanceof HTMLTextAreaElement))throw Error('Missing actual native text field '+wrap.id);
 const part=selector=>host.querySelector(selector),wrapper=part('.ix-text-field-wrapper'),top=part('.ix-text-field-top'),labelHost=part('.ix-text-field-label'),label=part('.ix-text-field-label .ix-typography'),slot=part('.ix-text-field-slot'),inputWrap=part('.ix-input-wrap'),start=part('.ix-input-start'),end=part('.ix-input-end'),password=part('.ix-input-password-eye'),bottom=part('.ix-text-field-bottom'),feedback=part('.ix-text-field-bottom-text'),icon=part('.ix-text-field-text-icon'),right=part('.ix-text-field-bottom-right'),counter=part('.ix-text-field-counter');
 const feedbackText=feedback?Array.from(feedback.childNodes).filter(node=>node.nodeType===Node.TEXT_NODE).map(node=>node.textContent).join(''):null;
 const startChildren=Array.from(start?.children??[],node=>metrics(node,host)),endChildren=Array.from(end?.children??[],node=>metrics(node,host));
 return{id:wrap.id,key:wrap.dataset.key,kind:wrap.dataset.kind,scopeIndex:Number(wrap.dataset.scopeIndex),revision:Number(wrap.dataset.revision),condition:wrap.dataset.condition,props:JSON.parse(wrap.dataset.props),host:metrics(host,host),wrapper:metrics(wrapper,host),top:metrics(top,host),labelHost:metrics(labelHost,host),label:metrics(label,host),labelText:label?.textContent??null,slot:metrics(slot,host),inputWrap:metrics(inputWrap,host),control:metrics(control,host),placeholder:metrics(control,host,'::placeholder'),start:metrics(start,host),end:metrics(end,host),startChildren,endChildren,password:metrics(password,host),passwordGlyph:metrics(password?.querySelector('.ix-icon-button-glyph'),host),bottom:metrics(bottom,host),feedback:metrics(feedback,host),feedbackText,icon:metrics(icon,host),iconSvg:metrics(icon?.matches('svg')?icon:icon?.querySelector('svg'),host),right:metrics(right,host),counter:metrics(counter,host),counterText:counter?.textContent??null,requiredMarker:!!label&&Array.from(label.querySelectorAll('span')).some(node=>node.textContent==='*'),nativeFacts:{value:control.value,type:control instanceof HTMLInputElement?control.type:'textarea',disabled:control.disabled,readonly:control.readOnly,required:control.required,maxLength:control.maxLength,rows:control instanceof HTMLTextAreaElement?control.rows:null,rowsAttr:control.getAttribute('rows'),cols:control instanceof HTMLTextAreaElement?control.cols:null,colsAttr:control.getAttribute('cols'),resize:getComputedStyle(control).resize,focused:control===document.activeElement,focus:control.matches(':focus'),focusVisible:control.matches(':focus-visible'),hover:control.matches(':hover'),active:control.matches(':active'),hostHover:host.matches(':hover'),wrapHover:inputWrap?.matches(':hover')??false,placeholderShown:control.matches(':placeholder-shown'),ariaLabel:control.getAttribute('aria-label'),ariaInvalid:control.getAttribute('aria-invalid'),ariaDescribedby:control.getAttribute('aria-describedby'),role:control.getAttribute('role'),labelFor:labelHost?.querySelector('label')?.htmlFor??null,labelForMatches:labelHost?labelHost.querySelector('label')?.htmlFor===control.id:null,paddingReady:inputWrap?.dataset.paddingReady??null,paddingStart:inputWrap?.dataset.paddingStart??null,paddingEnd:inputWrap?.dataset.paddingEnd??null,actualStartWidth:start?.getBoundingClientRect().width??null,actualEndWidth:end?.getBoundingClientRect().width??null,publicGeometryCurrent:geometryReady(wrap),consumer:JSON.parse(wrap.querySelector('.input-review-consumer').value)}};
};
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('input-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);button.click();log('public-revision-input',{id});await wait(()=>document.getElementById('input-review')?.dataset.phase===id&&rows().length===55&&rows().every(row=>row.dataset.revision===String(id==='revised'?1:0)),'public Rust token/prop commit '+id);await document.fonts.ready;await wait(()=>rows().every(geometryReady),'current native adornment raw-facts Rust commit '+id);await settle();};
const seedKeyboard=()=>{const node=document.getElementById('input-outside');node.focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});node.dispatchEvent(event);log('native-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
try{
 if(source){await wait(()=>window.upstreamInputTokenReview&&document.getElementById('input-status')?.dataset.ready==='true','pinned Input/Textarea readiness');const result=await window.upstreamInputTokenReview.run();check(result?.ok===true,'source completed');check(result.phases?.length===2,'source two revisions');check(result.phases.reduce((sum,phase)=>sum+phase.rows.length,0)===110,'source 110 actual Input/Textarea observations');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>rows().length===55,'55 actual Rust text field cases committed');
 for(const id of ['baseline','revised']){await setRevision(id);seedKeyboard();const observed=[];for(const wrap of rows()){if(wrap.dataset.condition==='focus'){const control=nativeFor(wrap);log('native-focus-input-before',{id:wrap.id});control.focus();await settle();log('native-focus-input-observed',{id:wrap.id,focused:document.activeElement===control});}else document.getElementById('input-outside').focus();observed.push(capture(wrap));}check(observed.length===55,id+' fifty-five actual text fields');phases.push({id,revision:id==='revised'?1:0,settleFrames:2,rows:observed,settlement:{publicCommit:'actual Rust root/case revision attrs and current raw adornment rect receipt',finalObservationFrames:2,fontsReady:true,focusPerSelectedRowFrames:2}});}
 const identities=phases[0].rows.map((row,index)=>({id:row.id,sameHost:row.host.object===phases[1].rows[index].host.object,sameControl:row.control.object===phases[1].rows[index].control.object,startChildrenRetained:row.startChildren.every((part,childIndex)=>part.object===phases[1].rows[index].startChildren[childIndex]?.object),endChildrenRetained:row.endChildren.every((part,childIndex)=>part.object===phases[1].rows[index].endChildren[childIndex]?.object)}));
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token Input/Textarea anatomy adapter; not full Brand metrics',phases,identities,errors,journal,assertions,renderer:'Linux desktop WebView Rust',limits:['Source custom/shadow hosts map to flattened Rust field/control/parts; raw IDs, ARIA/native labels and callback behavior remain explicit adapters','Source caret/password/autofill/native input type behavior and complete validation/touched/reset/cancellation/form APIs are not certified by this visual probe','Input geometry fence observes actual Rust committed raw start/end widths matching current native rectangles, not expected CSS padding or identical source frame timing','Public Rust token/prop and raw-geometry commits precede fonts and two fixed RAF; source custom-child readiness and source padding publication have separate chronology','Native focus is real sequential .focus after a synthetic Tab seed; browser pointer states are separately observed through genuine event captures without aliases','Textarea rows omission, optional dimensions and authored resize properties are measured; manual resize persistence/normalization and all unit families remain open','Actual source Icon24 is paired using authored size70=24px; password uses genuine delivered tertiary16 IconButton and complete child tokens, no computed-style masks','No-token fallback controls remain separate provisional Brand diagnostics; physical input/AT/other platforms remain open']};
}catch(error){errors.push(String(error));return{ok:false,phases,errors,journal,assertions};}finally{abort.abort();}
"#;

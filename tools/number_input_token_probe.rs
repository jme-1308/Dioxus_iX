//! Excluded native NumberInput computed-token runner; application intent stays Rust.
use dioxus::prelude::*;
#[cfg(any(
    target_arch = "wasm32",
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
use serde_json::Value;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/number_input_token_review.rs"]
mod number_input_token_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let source = std::env::var("IX_NUMBER_INPUT_SOURCE").is_ok_and(|value| value == "1");
    let index = if source {
        include_str!("upstream_number_input_token_review.html").replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        )
    } else {
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>NumberInput sizing and states</title></head><body style=\"margin:0\"><div id=\"main\"></div><!-- MODULE LOADER --></body></html>".to_string()
    };
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title(if source {
                    "Pinned iX NumberInput"
                } else {
                    "Dioxus iX NumberInput"
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
    eprintln!("Run the NumberInput review with desktop or web features");
}

#[component]
fn Probe() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let source = std::env::var("IX_NUMBER_INPUT_SOURCE").is_ok_and(|value| value == "1");
    #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
    let source = false;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(120)).await;
            eprintln!("NumberInput token native timeout");
            std::process::exit(1);
        });
    });
    rsx! {div {
        onmounted:move |_| {
            #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
            {let window=window.clone();spawn(async move{
                let query=dom_query::eval(VERIFY);let _=query.send(source);
                match query.join::<Value>().await {
                    Ok(value) if value["ok"]==true=>{println!("NumberInput token native observations: {value}");window.close();}
                    Ok(value)=>{eprintln!("NumberInput token native failure JSON: {value}");std::process::exit(1);}
                    Err(error)=>{eprintln!("NumberInput token native transport failure: {error:?}");std::process::exit(1);}
                }
            });}
        },
        {web_controls()}
        if !source {number_input_token_review::NumberInputTokenReview {}}
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
        button {id:"number-input-run",r#type:"button",disabled:busy(),aria_busy:busy(),
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
        span {role:"status",style:"margin-left:12px",if status()=="done" {"Completed: 104 NumberInput token/state observations."} else {"{status()}"}}
        pre {id:"number-input-web-result","data-run":status(),"aria-hidden":"true",style:"max-height:120px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px","{result()}"}
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
const rows=()=>Array.from(document.querySelectorAll('[data-number-input-row="true"]'));
const hostFor=wrap=>wrap.querySelector('.ix-number-input-native');
const nativeFor=wrap=>hostFor(wrap)?.querySelector('.ix-input-wrap > input,.ix-input-wrap > textarea');
const geometryReady=wrap=>{const host=hostFor(wrap),inputWrap=host?.querySelector('.ix-input-wrap'),start=host?.querySelector('.ix-input-start'),end=host?.querySelector('.ix-input-end');if(!inputWrap||!start||!end||inputWrap.dataset.paddingReady!=='true')return false;const reportedStart=Number(inputWrap.dataset.paddingStart),reportedEnd=Number(inputWrap.dataset.paddingEnd);return Number.isFinite(reportedStart)&&Number.isFinite(reportedEnd)&&Math.abs(reportedStart-start.getBoundingClientRect().width)<0.01&&Math.abs(reportedEnd-end.getBoundingClientRect().width)<0.01;};
const capture=wrap=>{
 const host=hostFor(wrap),control=nativeFor(wrap);if(!(host instanceof HTMLElement)||!(control instanceof HTMLInputElement||control instanceof HTMLTextAreaElement))throw Error('Missing actual native text field '+wrap.id);
 const part=selector=>host.querySelector(selector),wrapper=part('.ix-text-field-wrapper'),top=part('.ix-text-field-top'),labelHost=part('.ix-text-field-label'),label=part('.ix-text-field-label .ix-typography'),slot=part('.ix-text-field-slot'),inputWrap=part('.ix-input-wrap'),start=part('.ix-input-start'),end=part('.ix-input-end'),password=part('.ix-input-password-eye'),bottom=part('.ix-text-field-bottom'),feedback=part('.ix-text-field-bottom-text'),icon=part('.ix-text-field-text-icon'),right=part('.ix-text-field-bottom-right'),counter=part('.ix-text-field-counter'),generated=part('.ix-number-steppers'),minus=part('.ix-number-minus'),plus=part('.ix-number-plus');
 const feedbackText=feedback?Array.from(feedback.childNodes).filter(node=>node.nodeType===Node.TEXT_NODE).map(node=>node.textContent).join(''):null;
 const startChildren=Array.from(start?.children??[],node=>metrics(node,host)),endChildren=Array.from(end?.children??[],node=>metrics(node,host));
 return{id:wrap.id,key:wrap.dataset.key,kind:wrap.dataset.kind,scopeIndex:Number(wrap.dataset.scopeIndex),revision:Number(wrap.dataset.revision),condition:wrap.dataset.condition,props:JSON.parse(wrap.dataset.props),host:metrics(host,host),outer:metrics(wrap.querySelector('.ix-number-field'),host),wrapper:metrics(wrapper,host),top:metrics(top,host),labelHost:metrics(labelHost,host),label:metrics(label,host),labelText:label?.textContent??null,slot:metrics(slot,host),inputWrap:metrics(inputWrap,host),control:metrics(control,host),placeholder:metrics(control,host,'::placeholder'),start:metrics(start,host),end:metrics(end,host),startChildren,endChildren,startChild:startChildren[0]??null,endChild:metrics(Array.from(end?.children??[]).find(node=>!node.matches('.ix-number-steppers')),host),stepper:metrics(generated,host),minus:metrics(minus,host),minusControl:metrics(minus,host),plus:metrics(plus,host),plusControl:metrics(plus,host),minusGlyph:metrics(minus?.querySelector('.ix-icon-button-glyph'),host),plusGlyph:metrics(plus?.querySelector('.ix-icon-button-glyph'),host),bottom:metrics(bottom,host),feedback:metrics(feedback,host),feedbackText,icon:metrics(icon,host),iconSvg:metrics(icon?.matches('svg')?icon:icon?.querySelector('svg'),host),right:metrics(right,host),counter:metrics(counter,host),counterText:counter?.textContent??null,requiredMarker:!!label&&Array.from(label.querySelectorAll('span')).some(node=>node.textContent==='*'),nativeFacts:{showStepperRequested:host.classList.contains('ix-number-has-steppers'),generatedHidden:generated?.dataset.hidden??null,min:control.min,max:control.max,step:control.step,valueAsNumber:Number.isNaN(control.valueAsNumber)?null:control.valueAsNumber,value:control.value,type:control instanceof HTMLInputElement?control.type:'textarea',disabled:control.disabled,readonly:control.readOnly,required:control.required,maxLength:control.maxLength,rows:control instanceof HTMLTextAreaElement?control.rows:null,rowsAttr:control.getAttribute('rows'),cols:control instanceof HTMLTextAreaElement?control.cols:null,colsAttr:control.getAttribute('cols'),resize:getComputedStyle(control).resize,focused:control===document.activeElement,focus:control.matches(':focus'),focusVisible:control.matches(':focus-visible'),hover:control.matches(':hover'),active:control.matches(':active'),hostHover:host.matches(':hover'),wrapHover:inputWrap?.matches(':hover')??false,placeholderShown:control.matches(':placeholder-shown'),ariaLabel:control.getAttribute('aria-label'),ariaInvalid:control.getAttribute('aria-invalid'),ariaDescribedby:control.getAttribute('aria-describedby'),role:control.getAttribute('role'),labelFor:labelHost?.querySelector('label')?.htmlFor??null,labelForMatches:labelHost?labelHost.querySelector('label')?.htmlFor===control.id:null,paddingReady:inputWrap?.dataset.paddingReady??null,paddingStart:inputWrap?.dataset.paddingStart??null,paddingEnd:inputWrap?.dataset.paddingEnd??null,minusDisabled:minus?.disabled??null,plusDisabled:plus?.disabled??null,actualStartWidth:start?.getBoundingClientRect().width??null,actualEndWidth:end?.getBoundingClientRect().width??null,publicGeometryCurrent:geometryReady(wrap),consumer:JSON.parse(wrap.querySelector('.number-input-review-consumer').value)}};
};
const settle=async()=>{await document.fonts.ready;for(let index=1;index<=2;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>queueMicrotask(()=>{log('native-frame',{index,timestamp});resolve();})));};
const setRevision=async id=>{const button=document.getElementById('number-input-'+id);check(button instanceof HTMLButtonElement,'real Rust revision control '+id);button.click();log('public-revision-input',{id});await wait(()=>document.getElementById('number-input-review')?.dataset.phase===id&&rows().length===52&&rows().every(row=>row.dataset.revision===String(id==='revised'?1:0)),'public Rust token/prop commit '+id);await wait(()=>rows().every(wrap=>nativeFor(wrap)?.value===JSON.parse(wrap.dataset.props).valueText),'actual public Rust matrix value/native commit '+id);log('public-matrix-native-value-commit',{id,rows:rows().map(wrap=>({id:wrap.id,rustValueText:JSON.parse(wrap.dataset.props).valueText,nativeValue:nativeFor(wrap).value}))});await document.fonts.ready;await wait(()=>rows().every(geometryReady),'current native adornment raw-facts Rust commit '+id);await settle();};
const seedKeyboard=()=>{const node=document.getElementById('number-input-outside');node.focus();const event=new KeyboardEvent('keydown',{key:'Tab',bubbles:true,composed:true,cancelable:true});node.dispatchEvent(event);log('native-keyboard-seed',{defaultPrevented:event.defaultPrevented});};
const nativeClick=node=>{const events=[];let original=null;const capture=event=>{if(event.composedPath()[0]!==node)return;original=event;events.push({site:'document-capture',trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,tag:event.target.tagName});},bubble=event=>{if(event.composedPath()[0]!==node)return;events.push({site:'window-bubble',trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,tag:event.target.tagName});};document.addEventListener('click',capture,true);window.addEventListener('click',bubble);node.click();document.removeEventListener('click',capture,true);window.removeEventListener('click',bubble);return{eventObserved:!!original,defaultPrevented:original?.defaultPrevented??null,events};};
const consumerState=()=>({...JSON.parse(document.getElementById('number-input-consumer-json').value),formData:Array.from(new FormData(document.getElementById('number-input-consumer-form')).entries())});
const consumerCommand=async key=>{const root=document.getElementById('number-input-review'),before=Number(root.dataset.consumerCommand);document.getElementById('number-input-consumer-'+key).click();await wait(()=>Number(root.dataset.consumerCommand)>before,'Rust consumer command commit '+key);const wrap=document.getElementById('number-input-number-consumer');await wait(()=>{const model=JSON.parse(document.getElementById('number-input-consumer-json').value);return nativeFor(wrap)?.value===model.valueText;},'Rust consumer public value/native commit '+key);const model=JSON.parse(document.getElementById('number-input-consumer-json').value);log('public-consumer-native-value-commit',{key,command:Number(root.dataset.consumerCommand),rustValueText:model.valueText,nativeValue:nativeFor(wrap).value});await document.fonts.ready;await wait(()=>geometryReady(wrap),'consumer current raw geometry '+key);await settle();};
const consumerHistory=async()=>{await consumerCommand('reset');const wrap=document.getElementById('number-input-number-consumer'),steps=[];for(const name of ['plus','plus-again','minus','external-1.5','disabled-arrow-up','readonly-arrow-up','reset']){if(name==='disabled-arrow-up')await consumerCommand('disable');if(name==='readonly-arrow-up')await consumerCommand('readonly');await settle();const before=consumerState();let original=null;if(name==='external-1.5')await consumerCommand('external');else if(name==='reset')await consumerCommand('reset');else if(name.endsWith('arrow-up')){const control=nativeFor(wrap),event=new KeyboardEvent('keydown',{key:'ArrowUp',code:'ArrowUp',bubbles:true,composed:true,cancelable:true});const dispatched=control.dispatchEvent(event);original={type:'keydown',key:event.key,defaultPrevented:event.defaultPrevented,dispatched,targetTag:control.tagName,trusted:event.isTrusted};}else{original=nativeClick(hostFor(wrap).querySelector('.ix-number-'+(name==='minus'?'minus':'plus')));}await settle();const after=consumerState();steps.push({name,before,after,original,requestsDelta:name==='reset'?[]:after.requests.slice(before.requests.length),row:capture(wrap)});}return{steps,state:consumerState()};};

try{
 if(source){await wait(()=>window.upstreamNumberInputTokenReview&&document.getElementById('number-input-status')?.dataset.ready==='true','pinned NumberInput readiness');const result=await window.upstreamNumberInputTokenReview.run();check(result?.ok===true,'source completed');check(result.phases?.length===2,'source two revisions');check(result.phases.reduce((sum,phase)=>sum+phase.rows.length,0)===104,'source 104 actual NumberInput observations');return{...result,ok:result.ok&&errors.length===0&&(result.errors??[]).length===0,runnerAssertions:assertions,runnerErrors:errors,renderer:'Linux desktop WebView pinned source'};}
 await wait(()=>rows().length===52,'52 actual Rust text field cases committed');
 for(const id of ['baseline','revised']){await setRevision(id);seedKeyboard();const observed=[];for(const wrap of rows()){if(wrap.dataset.condition==='focus'){const control=nativeFor(wrap);log('native-focus-input-before',{id:wrap.id});control.focus();await settle();log('native-focus-input-observed',{id:wrap.id,focused:document.activeElement===control});}else document.getElementById('number-input-outside').focus();observed.push(capture(wrap));}check(observed.length===52,id+' fifty-two actual text fields');phases.push({id,revision:id==='revised'?1:0,settleFrames:2,rows:observed,settlement:{publicCommit:'actual Rust root/case revision attrs and current raw adornment rect receipt',finalObservationFrames:2,fontsReady:true,focusPerSelectedRowFrames:2}});}
 const focusRows=[];seedKeyboard();const focusWrap=document.getElementById('number-input-number-default-rest-stepper');for(const side of ['minus','plus']){const button=hostFor(focusWrap).querySelector('.ix-number-'+side);button.focus();await settle();focusRows.push({target:side,focused:document.activeElement===button,focusVisible:button.matches(':focus-visible'),button:metrics(button,hostFor(focusWrap)),host:metrics(button,hostFor(focusWrap)),row:capture(focusWrap)});}const stepperFocus={key:'number-default-rest-stepper',revision:1,settleFrames:2,rows:focusRows};
 const consumer=await consumerHistory();
 const identities=phases[0].rows.map((row,index)=>({id:row.id,sameHost:row.host.object===phases[1].rows[index].host.object,sameControl:row.control.object===phases[1].rows[index].control.object,sameGenerated:row.stepper.object===phases[1].rows[index].stepper.object,sameMinus:row.minus.object===phases[1].rows[index].minus.object,samePlus:row.plus.object===phases[1].rows[index].plus.object,startChildrenRetained:row.startChildren.every((part,childIndex)=>part.object===phases[1].rows[index].startChildren[childIndex]?.object),endChildrenRetained:row.endChildren.every((part,childIndex)=>part.object===phases[1].rows[index].endChildren[childIndex]?.object)}));
 return{ok:errors.length===0,revision:'8bf599bc68365d7752e12832e71b625883b93d7a',fixture:'Developer-authored system-token NumberInput anatomy adapter; not full Brand metrics',phases,identities,stepperFocus,controlled:consumer,errors,journal,assertions,renderer:'Linux desktop WebView Rust',limits:['Source numeric property/auto mutation and Rust raw-string controlled request/normalization APIs remain explicit adapters','Actual nested Input root maps source NumberInput host; native outer ix-number-field retained as structural adapter','Geometry fence observes Rust committed raw current container widths, not expected padding; source MO/IO versus Rust RO cache invalidation remains measured/open','Current matrix public props and consumer commands acknowledge actual native input values matching current Rust props/consumer valueText before fonts, geometry and two fixed RAF; consumer commands acknowledge actual native input value matching current Rust consumer valueText before fixed frames, not expected test values; exact source/custom-child/IPC timing is not asserted','Visible generated controls are genuine subtle-tertiary16 IconButtons; hidden containers stay mounted and requested minwidth is measured independently from visibility','Actual minus/plus .focus snapshots are separate from main104; trusted pointer qualification stays in separate genuine event captures','Native numeric parser, scientific text, normalization/commit/form/cancellation/AT/autofill and full input types are not certified','No-token fallback remains separate provisional Brand evidence']};

}catch(error){errors.push(String(error));return{ok:false,phases,errors,journal,assertions};}finally{abort.abort();}
"#;

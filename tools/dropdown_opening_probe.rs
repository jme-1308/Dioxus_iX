//! Excluded opening-focus diagnostic. Rust owns the fixture and public commands;
//! native scripts deliver original inputs and record actual DOM/frame facts.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
use std::{cell::RefCell, rc::Rc};

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX AD opening focus")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1100.0, 950.0)),
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
    eprintln!("Run the desktop or web AD opening probe");
}

fn record(mut rows: Signal<Vec<Value>>, kind: &'static str, details: Value) {
    if let Ok(mut values) = rows.try_write() {
        let seq = values
            .last()
            .and_then(|row| row["seq"].as_u64())
            .unwrap_or(0)
            + 1;
        values.push(json!({"seq":seq,"kind":kind,"details":details}));
    }
}

#[component]
fn Probe() -> Element {
    let mut epoch = use_signal(|| 0_usize);
    let mut selected = use_signal(|| 1_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(90)).await;
            eprintln!("AD opening native timeout");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme:ColorScheme::Dark,
        style {{STYLES}}
        main { id:"opening-probe",style:"padding:20px;min-height:900px",
            onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=dom_query::eval(VERIFY);let _=query.send(());
                    match query.join::<Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dropdown opening native observations: {value}");window.close();}
                        Ok(value)=>{eprintln!("Dropdown opening native failure JSON: {value}");std::process::exit(1);}
                        Err(error)=>{eprintln!("Dropdown opening native transport failure: {error:?}");std::process::exit(1);}
                    }
                });}
            },
            h1 {"AD opening focus"}
            p {"Eighteen fresh public-input histories, each followed by twelve fixed frame observations."}
            div {style:"display:flex;gap:6px;flex-wrap:wrap",
                for next in 1..=18 {button {id:format!("opening-fresh-{next}"),onclick:move |_|{selected.set(next);epoch+=1;},"Fresh O{next:02}"}}
            }
            input {id:"opening-mode",r#type:"hidden",oninput:move |event|{selected.set(event.value().parse::<usize>().unwrap_or(1).clamp(1,18));epoch+=1;}}
            button {id:"opening-outside","Outside keyboard target"}
            for current in [epoch()] {Fixture {key:"{current}",epoch:current,case_id:selected()}}
            NativeJournal {}
            pre {id:"opening-native-log","aria-label":"Native opening observations",style:"max-height:180px;overflow:auto;font-size:12px;white-space:pre-wrap"}
        }
    }}
}

#[component]
fn Fixture(epoch: usize, case_id: usize) -> Element {
    let dropdown = use_dropdown();
    let rows = use_signal(Vec::<Value>::new);
    let checked = (3..=6).contains(&case_id);
    rsx! {section {id:"opening-fixture","data-epoch":epoch.to_string(),"data-case":case_id.to_string(),
        div {id:"opening-owner",
            DropdownTrigger {dropdown,"Open AD candidates"}
            Dropdown {controller:dropdown,label:"AD opening candidates",class:"opening-dropdown",navigation_mode:DropdownNavigationMode::ActiveDescendant,
                close_behavior:DropdownCloseBehavior::None,enable_top_layer:true,suppress_automatic_placement:true,suppress_trigger_visibility_check:true,
                focus_checked_item:checked,disable_focus_handling:case_id==15,
                onbeforechange:move |event:DropdownChangeEvent|{
                    let veto=case_id==16&&event.open;
                    if veto {event.prevent_default();}
                    record(rows,"request",json!({"open":event.open,"previous":event.previous,"veto":veto,"current":dropdown.is_open()}));
                },
                onchange:move |open|record(rows,"completion",json!({"open":open,"current":dropdown.is_open()})),
                div {id:"opening-content","data-opening-role":"content",
                    OpeningContents {case_id}
                }
            }
        }
        input {id:"opening-command",r#type:"hidden",oninput:move |event|{
            let action=event.value();
            record(rows,"command-before",json!({"action":action,"current":dropdown.is_open()}));
            match action.as_str() {"open"=>dropdown.open(),"close"=>dropdown.close(false),"toggle"=>dropdown.toggle(),_=>{}}
            record(rows,"command-return",json!({"action":action,"current":dropdown.is_open()}));
        }}
        div {style:"display:flex;gap:6px",
            button {id:"opening-public-open",onmousedown:move |event|event.prevent_default(),onclick:move |_|dropdown.open(),"Public open()"}
            button {id:"opening-public-close",onmousedown:move |event|event.prevent_default(),onclick:move |_|dropdown.close(false),"Public close(false)"}
        }
        input {id:"opening-rust-json",r#type:"hidden",value:serde_json::to_string(&rows()).unwrap_or_default()}
    }}
}

const NODE_STYLE: &str = "display:block;box-sizing:border-box;min-height:40px;margin:0;";

#[component]
fn OpeningContents(case_id: usize) -> Element {
    match case_id {
        1 | 2 => rsx! {
            button {style:NODE_STYLE,id:"opening-native-first","data-opening-role":"native-first",r#type:"button","First native button"}
            DropdownItem {style:NODE_STYLE,id:"opening-dropdown-middle","data-opening-role":"dropdown-middle",label:"Middle Dropdown item"}
            SelectItem {style:NODE_STYLE,id:"opening-select-middle","data-opening-role":"select-middle",value:"select-middle",label:"Middle Select item"}
            div {id:"opening-spacer","data-opening-role":"spacer","aria-hidden":"true",style:"height:240px;min-height:240px"}
            input {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"text",value:"Last native input"}
        },
        3 => rsx! {
            button {style:NODE_STYLE,id:"opening-native-first","data-opening-role":"native-first",r#type:"button","First native button"}
            input {style:NODE_STYLE,id:"opening-checked-native","data-opening-role":"checked-native",r#type:"checkbox",initial_checked:true,checked:true}
            button {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"button","Last native button"}
        },
        4 => rsx! {
            button {style:NODE_STYLE,id:"opening-native-first","data-opening-role":"native-first",r#type:"button","First native button"}
            DropdownItem {style:NODE_STYLE,id:"opening-checked-dropdown","data-opening-role":"checked-dropdown",label:"Checked Dropdown item",checked:true}
            button {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"button","Last native button"}
        },
        5 => rsx! {
            button {style:NODE_STYLE,id:"opening-native-first","data-opening-role":"native-first",r#type:"button","First native button"}
            SelectItem {style:NODE_STYLE,id:"opening-checked-select","data-opening-role":"checked-select","checked":true,value:"checked-select",label:"Own checked Select attribute"}
            button {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"button","Last native button"}
        },
        6 => rsx! {
            input {style:NODE_STYLE,id:"opening-checked-native","data-opening-role":"checked-native",r#type:"checkbox",initial_checked:true,checked:true}
            div {id:"opening-spacer","data-opening-role":"spacer","aria-hidden":"true",style:"height:240px;min-height:240px"}
            button {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"button","Last native button"}
        },
        7 => rsx! {
            button {style:NODE_STYLE,id:"opening-negative-button","data-opening-role":"negative-button",tabindex:"-2","Negative tabindex"}
            button {style:NODE_STYLE,id:"opening-hidden-button","data-opening-role":"hidden-button",hidden:true,"Own hidden button"}
            button {style:NODE_STYLE,id:"opening-disabled-button","data-opening-role":"disabled-button",disabled:true,"Own disabled button"}
            button {style:NODE_STYLE,id:"opening-aria-only-button","data-opening-role":"aria-only-button","aria-disabled":"true","ARIA only button"}
            button {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"button","Last native button"}
        },
        8 => rsx! {
            input {style:NODE_STYLE,id:"opening-failure-candidate","data-opening-role":"failure-candidate",r#type:"hidden"}
            button {style:NODE_STYLE,id:"opening-after-failure","data-opening-role":"after-failure",r#type:"button","After failed focus"}
        },
        9..=11 => rsx! {
            button {id:"opening-failure-candidate","data-opening-role":"failure-candidate",style:match case_id {9=>"display:block;box-sizing:border-box;min-height:40px;margin:0;visibility:hidden",10=>"display:block;box-sizing:border-box;min-height:40px;margin:0;display:none",_=>NODE_STYLE},inert:(case_id==11).then_some(""),"Failure candidate"}
            button {style:NODE_STYLE,id:"opening-after-failure","data-opening-role":"after-failure",r#type:"button","After failed focus"}
        },
        12 => rsx! {
            button {style:NODE_STYLE,id:"opening-negative-button","data-opening-role":"negative-button",tabindex:"-1","Negative tabindex"}
            button {style:NODE_STYLE,id:"opening-disabled-button","data-opening-role":"disabled-button",disabled:true,"Own disabled button"}
            input {style:NODE_STYLE,id:"opening-hidden-input","data-opening-role":"hidden-input",r#type:"text",hidden:true}
        },
        13 => rsx! {
            SelectItem {style:NODE_STYLE,id:"opening-shadow-select","data-opening-role":"shadow-select",value:"shadow-select",label:"First Select own control"}
            button {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"button","Last native button"}
        },
        14 => rsx! {
            button {style:NODE_STYLE,id:"opening-native-first","data-opening-role":"native-first",r#type:"button","First native button"}
            SelectItem {style:NODE_STYLE,id:"opening-shadow-select","data-opening-role":"shadow-select",value:"shadow-select",label:"Last Select own first control"}
        },
        _ => rsx! {
            button {style:NODE_STYLE,id:"opening-native-first","data-opening-role":"native-first",r#type:"button","First native button"}
            button {style:NODE_STYLE,id:"opening-native-last","data-opening-role":"native-last",r#type:"button","Last native button"}
        },
    }
}

#[component]
fn NativeJournal() -> Element {
    let transport = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let dropped = transport.clone();
    use_drop(move || {
        if let Some(task) = dropped.borrow_mut().take() {
            let _ = task.send("finish");
        }
    });
    use_effect(move || {
        *transport.borrow_mut() = Some(dioxus::document::eval(OBSERVE));
    });
    VNode::empty()
}

const STYLES: &str = r#"
#opening-fixture{padding-top:16px;min-height:400px}
#opening-content{box-sizing:content-box;width:320px;max-height:160px;overflow:auto}
#opening-owner>.opening-dropdown>dialog{max-height:220px;padding:0}
"#;

const OBSERVE: &str = r#"
const probe=document.getElementById('opening-probe');if(!probe)return;
const rows=[],objects=new WeakMap(),events=new WeakMap(),queryReports=new WeakMap(),abort=new AbortController();let sequence=0,objectSequence=0,eventSequence=0,context='bootstrap',alive=true;
const object=el=>{if(!el)return null;if(!objects.has(el))objects.set(el,++objectSequence);return objects.get(el);};
const eventId=event=>{if(!events.has(event))events.set(event,++eventSequence);return events.get(event);};
const role=el=>el?.getAttribute?.('data-opening-role')??el?.closest?.('[data-opening-role]')?.getAttribute('data-opening-role')??(el?.matches?.('[data-ix-overlay-trigger]')?'root-trigger':el?.matches?.('[data-ix-overlay-root]')?'root':el?.matches?.('[data-ix-overlay-host]')?'items-host':null);
const node=el=>el instanceof Element?{object:object(el),id:el.id,tag:el.tagName,role:role(el),connected:el.isConnected,part:el.getAttribute('data-ix-item-part'),control:el.hasAttribute('data-ix-item-control')}:null;
const deepActive=()=>{let el=document.activeElement;while(el?.shadowRoot?.activeElement)el=el.shadowRoot.activeElement;return el;};
const root=()=>document.querySelector('#opening-owner>.opening-dropdown[data-ix-overlay-root]');
const trigger=()=>document.querySelector('#opening-owner>[data-ix-overlay-trigger]');
const attributes=el=>{if(!(el instanceof Element))return null;const rect=el.getBoundingClientRect(),style=getComputedStyle(el);return{node:node(el),attrs:Object.fromEntries([...el.attributes].map(attr=>[attr.name,attr.value])),active:el.getAttribute('data-active'),focused:el.classList.contains('ix-focused'),tabindex:el.getAttribute('tabindex'),aria:el.getAttribute('aria-activedescendant'),checked:el.hasAttribute('checked'),checkedProperty:typeof el.checked==='boolean'?el.checked:null,disabled:el.hasAttribute('disabled'),disabledProperty:typeof el.disabled==='boolean'?el.disabled:null,hidden:el.hasAttribute('hidden'),inert:el.hasAttribute('inert'),type:el.getAttribute('type'),display:style.display,visibility:style.visibility,rect:{x:rect.x,y:rect.y,width:rect.width,height:rect.height},scrollTop:el.scrollTop,scrollLeft:el.scrollLeft,scrollHeight:el.scrollHeight,clientHeight:el.clientHeight};};
const snapshot=()=>{const r=root(),host=r?.querySelector(':scope>dialog'),content=document.getElementById('opening-content');let native=false;try{native=!!host?.matches(':popover-open');}catch{}return{epoch:document.getElementById('opening-fixture')?.dataset.epoch,case:document.getElementById('opening-fixture')?.dataset.case,root:attributes(r),itemsHost:attributes(host),content:attributes(content),owner:r?.dataset.ixOverlayOwner,generation:r?.dataset.ixOverlayGeneration,logical:r?.dataset.ixOverlayOpen,native,published:r?.dataset.ixOverlayActiveItem,trigger:attributes(trigger()),focus:node(document.activeElement),deepFocus:node(deepActive()),members:[...content?.children??[]].map(el=>({host:attributes(el),controls:[...el.querySelectorAll('[data-ix-item-control],[data-ix-item-part]')].map(attributes),ownShadow:el.shadowRoot?[...el.shadowRoot.querySelectorAll('*')].map(attributes):[]})),rust:JSON.parse(document.getElementById('opening-rust-json')?.value||'[]'),documentScroll:{x:scrollX,y:scrollY}};};
const publish=row=>{const el=document.getElementById('opening-native-log');if(el)el.textContent=JSON.stringify({context,last:row});};
const record=(kind,details={},withState=false)=>{const row={seq:++sequence,context,kind,time:performance.now(),...details,...(withState?{state:snapshot()}:{})};rows.push(row);if(rows.length>5000)rows.shift();if(details.report?.phase!=='retain')publish(row);return row;};
for(const type of ['keydown','keyup','click','focusin','focusout','scroll']){
 document.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('document-capture',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented,path:event.composedPath().filter(el=>el instanceof Element).map(node)},true);},{capture:true,passive:true,signal:abort.signal});
 window.addEventListener(type,event=>{if(!probe.contains(event.target))return;record('window-bubble',{event:eventId(event),type,key:event.key,target:node(event.composedPath()[0]),trusted:event.isTrusted,defaultPrevented:event.defaultPrevented},true);},{passive:true,signal:abort.signal});
}
const queryReport=event=>{if(queryReports.has(event))return queryReports.get(event);let value=null;if(event.target.matches?.('input[data-ix-overlay-transport]'))try{const report=JSON.parse(event.target.value);if(report.descendant_focus)value={kind:'descendant-focus',owner:report.owner,...report.descendant_focus};else if(report.item_focus_phase)value={kind:'item-focus',owner:report.owner,phase:report.item_focus_phase,sequence:report.item_focus_sequence,root:report.item_focus_root,generation:report.item_focus_generation,target:report.item_focus_target,capture:report.item_focus_capture};}catch{}queryReports.set(event,value);return value;};
document.addEventListener('change',event=>{const report=queryReport(event);if(!report)return;const id=eventId(event);record(report.kind+'-change-capture',{event:id,report,defaultPrevented:event.defaultPrevented},true);queueMicrotask(()=>{if(alive)record(report.kind+'-queued-capture-microtask',{event:id,report,defaultPrevented:event.defaultPrevented},true);});},{capture:true,passive:true,signal:abort.signal});
window.addEventListener('change',event=>{const report=queryReport(event);if(report)record(report.kind+'-change-window',{event:eventId(event),report,defaultPrevented:event.defaultPrevented},true);},{passive:true,signal:abort.signal});
const observer=new MutationObserver(changes=>{const facts=changes.filter(change=>change.target.closest?.('#opening-owner')).map(change=>({node:node(change.target),attribute:change.attributeName,oldValue:change.oldValue,currentValue:change.target.getAttribute(change.attributeName)}));if(facts.length)record('native-mutation-delivery',{changes:facts},true);});
observer.observe(probe,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:['data-active','class','aria-activedescendant','data-ix-overlay-active-item','data-ix-overlay-open','data-ix-overlay-focus-target','data-ix-overlay-focus-version','hidden','inert','tabindex']});
window.__ixOpeningProbe={rows,snapshot,node,eventId,record,setContext:value=>{context=value;record('context',{},true);}};record('journal-ready');
try{await dioxus.recv();}finally{alive=false;abort.abort();observer.disconnect();delete window.__ixOpeningProbe;}
"#;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const VERIFY: &str = r#"
await dioxus.recv();
const observations=[],assertions=[],driver=[];let context='bootstrap';
const get=id=>document.getElementById('opening-'+id),api=()=>window.__ixOpeningProbe,sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const root=()=>document.querySelector('#opening-owner>.opening-dropdown[data-ix-overlay-root]'),trigger=()=>document.querySelector('#opening-owner>[data-ix-overlay-trigger]');
const state=()=>api()?.snapshot(),record=(kind,details={})=>{const row={context,kind,time:performance.now(),...details,state:state()};driver.push(row);return row;};
const assert=(ok,label)=>{assertions.push({context,label,ok:!!ok});if(!ok)throw Error(label+' '+JSON.stringify(state()));};
const wait=async(fn,label,ms=6000)=>{const end=performance.now()+ms;while(performance.now()<end){if(fn())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify(state()));};
const frame=()=>new Promise(resolve=>requestAnimationFrame(resolve));
async function fixedFrames(count,label,event=null){for(let index=1;index<=count;index++){const timestamp=await frame();api().record('fixed-observer-frame',{label,event,index,timestamp},true);await Promise.resolve();api().record('fixed-observer-frame-microtask',{label,event,index,timestamp},true);}}
const key=(target,value,label)=>{assert(target?.isConnected,'connected actual key target');const event=new KeyboardEvent('keydown',{key:value,bubbles:true,composed:true,cancelable:true}),id=api().eventId(event);api().record('original-key-before',{label,event:id,key:value,target:api().node(target)},true);target.dispatchEvent(event);api().record('original-key-return',{label,event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented},true);return{event:id,key:value,target:api().node(target),defaultPrevented:event.defaultPrevented};};
const command=action=>{const input=get('command');api().record('public-command-before',{action},true);input.value=action;input.dispatchEvent(new Event('input',{bubbles:true}));api().record('public-command-return',{action},true);};
async function fresh(id){context='O'+String(id).padStart(2,'0');const old=get('fixture');get('mode').value=String(id);get('mode').dispatchEvent(new Event('input',{bubbles:true}));await wait(()=>get('fixture')!==old&&get('fixture')?.dataset.case===String(id)&&root()?.dataset.ixOverlayGeneration&&root()?.dataset.ixOverlayOwner&&api(),'fresh Rust opening fixture');api().setContext(context);await fixedFrames(5,'fixture-bind');get('outside').focus();key(get('outside'),'Tab','outside-keyboard-setup');trigger().focus();get('content').scrollTop=0;await fixedFrames(2,'fresh-closed');assert(state().logical==='false'&&!state().native,'fresh fixture starts closed');record('fixture-ready',{sourceRootFallbackAdapter:'source Dropdown host without own tabindex versus actual Rust external owner/native dialog; raw held identity retained'});}
const sameObjects=(before,after)=>JSON.stringify({root:before.root.node.object,trigger:before.trigger.node.object,members:before.members.map(item=>({host:item.host.node.object,controls:item.controls.map(el=>el.node.object)}))})===JSON.stringify({root:after.root.node.object,trigger:after.trigger.node.object,members:after.members.map(item=>({host:item.host.node.object,controls:item.controls.map(el=>el.node.object)}))});
try{
 await wait(()=>api(),'passive native opening journal');
 const keys=['ArrowDown','ArrowUp','Home','ArrowDown','ArrowDown','End','ArrowDown','ArrowDown','ArrowDown','ArrowDown','ArrowDown','ArrowDown','ArrowDown','ArrowUp','ArrowDown','ArrowDown','ArrowDown',null];
 for(let index=0;index<keys.length;index++){
  const id=index+1;await fresh(id);const cursor=api().rows.at(-1)?.seq??0,before=state();
  const original=keys[index]?key(trigger(),keys[index],context):null;
  if(id===17)command('close');if(id===18)command('open');
  api().record('operation-return',{original,action:id===17?'key-then-public-close':id===18?'public-open':'key-only'},true);
  await Promise.resolve();api().record('operation-return-microtask',{event:original?.event??null},true);
  await fixedFrames(12,context,original?.event??null);api().record('fixed-frame-drain-end',{event:original?.event??null},true);
  const after=state(),journal=api().rows.filter(row=>row.context===context&&row.seq>cursor);
  assert(sameObjects(before,after),'same actual Rust hosts and generated controls through opening');
  observations.push({case:id,label:context,before,original,action:id===17?'key-then-same-turn-close-false':id===18?'public-handle-open':'key-only',state:after,journal});record('context-complete',{case:id});
 }
 assert(observations.length===18,'eighteen opening diagnostic histories');
 return{ok:true,observations,assertions,driver,limits:['raw opening outcomes; setup/identity assertions are not source parity','exact12 observers+microtasks with no expected focus/marker waits','source public show property versus Rust admission/controller focus are explicit API adapters in17/18','source own shadow versus Rust generated Select/Dropdown controls remain structural renderer adapters','source original Dropdown fallback differs from Rust external owner/native dialog; no physical focus normalization','no fake actual shadow/assigned-slot proof or callbackFocusElement API; source19–21 remain unpaired','native focus may scroll without an explicit nearest job; no method wrappers/job-count inference','native transport phase journal excludes pure cue/work queries','synthetic DOM keys are not physical OS/AT/platform/Brand proof']};
}catch(error){return{ok:false,error:String(error),context,state:state(),observations,assertions,driver,journal:api()?.rows??[]};}
"#;

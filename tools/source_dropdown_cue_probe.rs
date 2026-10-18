//! Excluded pinned-source nested-focus cue diagnostic; no source policy emulation.
use dioxus::prelude::*;
use std::time::Duration;
#[path = "../src/dom_query.rs"]
mod dom_query;

fn main() {
    let index = include_str!("upstream_dropdown_publication_review.html").replace(
        "</body>",
        "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
    );
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Pinned source Dropdown cue redirect")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
await dioxus.recv();
const rows=[],operations=[],assertions=[],objects=new WeakMap(),events=new WeakMap(),abort=new AbortController();let seq=0,obj=0,ev=0,stage='bootstrap',armed=false,detachedPair=null;
const api=()=>window.upstreamDropdownPublicationReview,sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms)),state=()=>api()?.snapshot();
const node=el=>{if(!(el instanceof Element))return null;if(!objects.has(el))objects.set(el,++obj);return{object:objects.get(el),id:el.id,tag:el.tagName,role:el.dataset.itemsRole??null,connected:el.isConnected};};
const eid=event=>{if(!events.has(event))events.set(event,++ev);return events.get(event);};
const marker=el=>el?{node:node(el),class:el.getAttribute('class'),focused:el.classList.contains('ix-focused'),ixFocusVisible:typeof el.ixFocusVisible==='boolean'?el.ixFocusVisible:null,reflected:el.getAttribute('ix-focus-visible'),tabindex:el.getAttribute('tabindex'),physical:document.activeElement===el,scrollTop:el.scrollTop,scrollLeft:el.scrollLeft}:null;
const record=(kind,details={})=>{const row={seq:++seq,stage,kind,time:performance.now(),...details,state:state(),held:detachedPair?{original:marker(detachedPair.original),replacement:marker(detachedPair.replacement)}:null};rows.push(row);return row;};
const assert=(ok,label)=>{assertions.push({label,ok:!!ok});if(!ok)throw Error(label);};
const wait=async(predicate,label)=>{const end=performance.now()+25000;while(performance.now()<end){if(predicate())return;await sleep(5);}throw Error('Timeout '+label);};
const frames=async label=>{for(let index=1;index<=12;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>{record('fixed-native-frame',{label,index,timestamp});queueMicrotask(()=>{record('fixed-native-frame-microtask',{label,index,timestamp});resolve();});}));};
const key=(target,value)=>{assert(target?.isConnected,'connected actual source key target');const event=new KeyboardEvent('keydown',{key:value,bubbles:true,composed:true,cancelable:true});record('original-key-before',{event:eid(event),key:value,target:node(target)});target.dispatchEvent(event);record('original-key-return',{event:eid(event),key:value,target:node(target),defaultPrevented:event.defaultPrevented});return{event:eid(event),key:value,target:node(target),defaultPrevented:event.defaultPrevented};};
const operation=async(target,value,label)=>{stage=label;const from=seq+1,original=key(target,value);await new Promise(resolve=>queueMicrotask(()=>{record('original-return-microtask',{event:original.event});resolve();}));await frames(label);const op={label,from,to:seq,original,state:state()};operations.push(op);return op;};
try{
 await wait(()=>api()?.snapshot()?.ready&&!api().snapshot().busy,'source public fixture ready');const before=state().generation;document.getElementById('items-fresh-ad').click();await wait(()=>state()?.generation>before&&state().ready&&!state().busy,'fresh actual source AD fixture');
 for(const type of ['keydown','focusin','focusout'])for(const capture of [true,false])document.addEventListener(type,event=>record('native-event',{event:eid(event),type,site:capture?'document-capture':'document-bubble',target:node(event.target),origin:node(event.composedPath()[0]),key:event.key??null,defaultPrevented:event.defaultPrevented,trusted:event.isTrusted}),{capture,passive:true,signal:abort.signal});
 document.addEventListener('ixFocusVisibleChange',event=>record('source-visible-focus',{event:eid(event),currentFocus:Array.from(event.detail?.currentFocus??[]).map(node)}),{passive:true,signal:abort.signal});
 document.addEventListener('keyboardmodechange',event=>record('source-mode',{detail:event.detail}),{passive:true,signal:abort.signal});
 const root=document.getElementById(state().root.node.id),trigger=document.getElementById(state().trigger.id),menu=root.querySelector('[data-items-role="menu-c"]'),native=root.querySelector('[data-items-role="native-d"]'),outside=document.getElementById('items-outside');assert(root&&trigger&&menu&&native,'actual source root trigger Menu/native candidates');
 const observer=new MutationObserver(changes=>record('cue-mutation-delivery',{changes:changes.map(change=>({node:node(change.target),attribute:change.attributeName,oldValue:change.oldValue,currentValue:change.target.getAttribute(change.attributeName)}))}));observer.observe(root,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:['class','ix-focus-visible','aria-selected','tabindex']});abort.signal.addEventListener('abort',()=>observer.disconnect(),{once:true});
 stage='actual-keyboard-seed';outside.focus();key(outside,'Tab');trigger.focus();assert(state().mode===true,'actual source detector keyboard true');await frames('actual-keyboard-seed');
 await operation(trigger,'Home','C4-Home');await operation(trigger,'ArrowDown','C4-Select');
 const redirects=[];armed=true;document.addEventListener('focusin',event=>{if(!armed||!event.composedPath().includes(menu))return;armed=false;redirects.push(record('nested-focus-redirect-before',{event:eid(event),attempted:node(menu),destination:node(native)}));native.focus();redirects.push(record('nested-focus-redirect-return',{event:eid(event),attempted:node(menu),destination:node(native)}));},{capture:true,signal:abort.signal});
 await operation(trigger,'ArrowDown','C4-Menu-redirect');record('final-independent-checkpoint');const c4={ok:true,case:4,caseCount:1,label:'Actual source C4 one-shot Menu focusin redirect',redirectObserved:!armed,redirects,operations:operations.slice(),assertions:assertions.slice(),root:node(root),attemptedMenu:node(menu),redirectNative:node(native),sameActualMenu:root.querySelector('[data-items-role="menu-c"]')===menu,sameActualNative:root.querySelector('[data-items-role="native-d"]')===native,state:state(),journal:rows.slice(),limits:['Unchanged source publication HTML and pinned runtime; no private functions/controllers/flags or prototype/RAF wrappers','Actual public Fresh AD setup then detector Tab; three source keys with unconditional12RAF and microtasks','One native .focus redirect in capture focusin is an explicit diagnostic input, not browser default behavior','No final cue golden/marker wait or extra flushing key; actual outcome may differ from static source expectation','Source event object IDs and public snapshot object IDs are separate local identity spaces']};
 stage='C5-fresh-public-AD';const c5generation=state().generation;document.getElementById('items-fresh-ad').click();await wait(()=>state()?.generation>c5generation&&state().ready&&!state().busy,'C5 fresh actual source AD fixture');const c5root=document.getElementById(state().root.node.id),c5trigger=document.getElementById(state().trigger.id),oldA=c5root.querySelector('[data-items-role="dropdown-a"]');assert(oldA&&c5trigger,'C5 actual source host and trigger');
 const c5Observer=new MutationObserver(changes=>record('C5-held-mutation-delivery',{changes:changes.map(change=>({node:node(change.target),attribute:change.attributeName,oldValue:change.oldValue,currentValue:change.target.getAttribute(change.attributeName)}))}));c5Observer.observe(c5root,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:['class','ix-focus-visible','aria-selected','tabindex']});c5Observer.observe(oldA,{attributes:true,subtree:true,attributeOldValue:true,attributeFilter:['class','ix-focus-visible','aria-selected','tabindex']});abort.signal.addEventListener('abort',()=>c5Observer.disconnect(),{once:true});
 stage='C5-actual-keyboard-seed';outside.focus();key(outside,'Tab');c5trigger.focus();assert(state().mode===true,'C5 actual source keyboard mode true');await frames('C5-keyboard-seed');stage='C5-Home-detach';const c5from=seq+1,replacement=oldA.cloneNode(true);detachedPair={original:oldA,replacement};record('C5-clone-before-admission',{original:marker(oldA),replacement:marker(replacement)});const c5original=key(c5trigger,'Home');record('C5-detach-before-first-focus',{original:marker(oldA),replacement:marker(replacement)});oldA.parentNode.insertBefore(replacement,oldA);oldA.remove();record('C5-detach-return',{original:marker(oldA),replacement:marker(replacement)});await new Promise(resolve=>queueMicrotask(()=>{record('C5-input-return-microtask',{event:c5original.event});resolve();}));await frames('C5-Home-detached-fixed12');const c5={case:5,label:'C5 source admitted host detached with same-ID copy',original:c5original,from:c5from,to:seq,sameId:oldA.id===replacement.id,distinctActualObjects:oldA!==replacement,originalHeld:marker(oldA),replacementCurrent:marker(replacement),sameCurrentReplacement:c5root.querySelector('[data-items-role="dropdown-a"]')===replacement,state:state(),journal:rows.filter(row=>row.seq>=c5from),classification:'Explicit native DOM detachment/replacement before first queued source focus RAF; no original marker fabricated, no private policy/controller writes, retained old source object versus current root query kept distinct'};const result={ok:true,caseCount:2,observations:[c4,c5],assertions,journal:rows,limits:['C4 retained as separate fresh context; C5 is explicit native DOM diagnostic rather than a Rust/source component replacement API parity claim','Source C5 has no private focus/scroll tuple instrumentation; original actual marker/visible events/scroll offsets and fixed RAF facts are observed','No expected cue wait, new input flush, prototype/RAF/focus wrapping; old source HTML unchanged']};abort.abort();return result;

}catch(error){const result={ok:false,error:String(error),stage,armed,operations,assertions,state:state(),journal:rows};abort.abort();return result;}

"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Pinned source cue redirect global timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => { println!("Pinned source Dropdown cue redirect checks: {result}"); window.close(); }
                Ok(result) => { eprintln!("Pinned source Dropdown cue redirect failure: {result}"); std::process::exit(1); }
                Err(error) => { eprintln!("Pinned source Dropdown cue redirect eval failure: {error:?}"); std::process::exit(1); }
            }
        });
    } } }
}

//! Excluded G04.5 Linux WebView port probe. Native PointerEvent objects are
//! synthetic; real Rust policy, timers, DOM presentation and child scopes run.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/menu_hover_review.rs"]
mod menu_hover_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX category hover and grace probe")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1400.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const CAPABILITIES: &str = r#"
await dioxus.recv();
return {moveBefore:typeof Element.prototype.moveBefore,popover:typeof HTMLElement.prototype.showPopover,renderer:navigator.userAgent};
"#;

const VERIFY: &str = r#"
try{
 await dioxus.recv();
 const get=id=>document.getElementById('g04-hover-'+id),sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const journal=[],observations=[];let scenario='bootstrap',dispatch=0;
 const stage=()=>get('stage'),menu=()=>stage()?.querySelector('nav.ix-menu'),state=()=>JSON.parse(get('state').textContent),events=()=>JSON.parse(get('events').textContent);
 const findCategory=(root,label)=>[...root?.querySelectorAll('.ix-menu-category')??[]].find(host=>host.querySelector(':scope > [role=menuitem]')?.getAttribute('aria-label')===label);
 const a=()=>findCategory(stage(),'Category A'),b=()=>findCategory(stage(),'Category B'),peer=()=>findCategory(get('peer-stage'),'Peer category');
 const panel=host=>host?.querySelector(':scope > [data-menu-category-popup]'),expandedPanel=host=>host?.querySelector(':scope > [data-menu-category-panel]'),body=host=>host?.querySelector('[data-menu-category-body]'),popup=host=>host?.getAttribute('data-menu-category-popup-open')==='true',panelOpen=host=>host?.getAttribute('data-menu-category-panel-open')==='true';
 const nativeOpen=host=>{try{return!!panel(host)?.matches(':popover-open');}catch{return false;}};
 const visible=host=>{const p=panel(host),r=p?.getBoundingClientRect();return!!host?.isConnected&&!!p&&!p.hidden&&!!r?.width&&!!r?.height&&getComputedStyle(p).display!=='none';};
 const presented=host=>popup(host)&&visible(host)&&nativeOpen(host),closed=host=>!popup(host)&&!nativeOpen(host)&&!visible(host);
 const node=el=>el?{id:el.id,tag:el.tagName,label:el.getAttribute?.('aria-label')}:null;
 const deepActive=()=>{let el=document.activeElement;while(el?.shadowRoot?.activeElement)el=el.shadowRoot.activeElement;return el;};
 const categoryState=host=>({node:node(host),contentHost:node(expandedPanel(host)),currentPopupHost:node(panel(host)),connected:!!host?.isConnected,mode:host?.getAttribute('data-menu-category-mode'),popup:popup(host),panel:panelOpen(host),popupHidden:panel(host)?.hidden,panelHidden:expandedPanel(host)?.hidden,bodySurface:host?.getAttribute('data-menu-category-body-surface'),bodyParent:!body(host)?null:body(host).parentElement===panel(host)?'popup':body(host).parentElement===expandedPanel(host)?'panel':null,nativePopover:nativeOpen(host),visible:visible(host)});
 const snapshot=()=>({scenario,state:state(),focus:node(deepActive()),a:categoryState(a()),b:categoryState(b()),peer:categoryState(peer()),events:events()});
 const assert=(ok,message)=>{if(!ok)throw Error(message+' '+JSON.stringify({snapshot:snapshot(),tail:journal.slice(-14)}));};
 const wait=async(predicate,label,timeout=4000)=>{const end=performance.now()+timeout;while(performance.now()<end){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({snapshot:snapshot(),tail:journal.slice(-14)}));};
 const listener=new AbortController();
 document.addEventListener('input',event=>{if(!event.target.matches?.('input[data-menu-hover-observer]'))return;try{journal.push({scenario,kind:'raw-hover-transport',time:performance.now(),detail:JSON.parse(event.target.value)});}catch{}},{capture:true,passive:true,signal:listener.signal});
 const mutations=new MutationObserver(records=>{for(const record of records){if(!record.target.matches?.('.ix-menu-category'))continue;journal.push({scenario,kind:'dom-state-commit',time:performance.now(),attribute:record.attributeName,previous:record.oldValue,value:record.target.getAttribute(record.attributeName),category:record.target.getAttribute('data-menu-category-root'),label:record.target.querySelector(':scope > [role=menuitem]')?.getAttribute('aria-label')});}});
 mutations.observe(document.body,{subtree:true,attributes:true,attributeOldValue:true,attributeFilter:['data-menu-category-popup-open','data-menu-category-panel-open','data-menu-category-mode','data-menu-category-body-surface']});
 const readyHost=host=>host?.dataset.categoryHoverNative==='true'&&host?.dataset.categoryNative==='true';
 const ready=()=>menu()?.dataset.menuNative==='true'&&readyHost(a())&&readyHost(b())&&readyHost(peer());
 function pointer(target,type,relatedTarget,pointerType='mouse'){
  assert(target?.isConnected,'connected synthetic pointer target');const event=new PointerEvent(type,{pointerType,pointerId:1,isPrimary:true,buttons:0,button:-1,relatedTarget,bubbles:false,cancelable:false,composed:false});
  const time=performance.now();journal.push({scenario,kind:'driver-pointer',time,dispatch:++dispatch,type,pointerType,buttons:event.buttons,trusted:event.isTrusted,target:node(target),relatedTarget:node(relatedTarget)});target.dispatchEvent(event);return time;
 }
 const command=id=>{journal.push({scenario,kind:'fixture-control-click',id,time:performance.now()});get(id).click();};
 function control(id,value){const target=get(id),time=performance.now();if(target instanceof HTMLInputElement&&target.type==='checkbox')target.checked=value;else target.value=value;journal.push({scenario,kind:'fixture-public-control-change',id,value,time});target.dispatchEvent(new Event('change',{bubbles:true}));return time;}
 async function fresh(label){
  scenario=label;if(popup(peer())){pointer(peer(),'pointerleave',get('body'));await wait(()=>closed(peer()),'previous peer closed');}
  const previous=menu();command('fresh');await wait(()=>menu()&&menu()!==previous&&ready()&&!state().expanded&&closed(a())&&closed(b()),'new collapsed owner/readiness');
 }
 async function open(host,pointerType='mouse'){
  const focus=deepActive();journal.push({scenario,kind:'observed-focus',stage:'before-entry',time:performance.now(),node:node(focus)});
  pointer(host,'pointerenter',get('body'),pointerType);assert(deepActive()===focus,'entry has no synchronous deep focus move');
  await wait(()=>presented(host),'actual native flyout presentation');await new Promise(requestAnimationFrame);await new Promise(requestAnimationFrame);
  journal.push({scenario,kind:'observed-focus',stage:'after-open-commit',time:performance.now(),node:node(deepActive())});assert(deepActive()===focus,'entry/committed presentation preserves deep focus');
 }
 async function elapsed(start,delay){const left=start+delay-performance.now();if(left>0)await sleep(left);return performance.now()-start;}
 const commit=(host,start)=>journal.find(row=>row.kind==='dom-state-commit'&&row.scenario===scenario&&row.category===host.getAttribute('data-menu-category-root')&&row.attribute==='data-menu-category-popup-open'&&row.value==='false'&&row.time>=start);
 async function closeAfter(host,start,label){await wait(()=>closed(host),label+' closed DOM/native state',2000);const change=commit(host,start);assert(change,label+' records popup close commit');const delta=change.time-start;assert(delta>=248,label+' close commit not before250ms (2ms clock tolerance)');return{leave:start,closeCommit:change.time,closeCommitDelta:delta,settledDelta:performance.now()-start};}
 const retain=(host,label)=>assert(presented(host),label+' retains actual popup/native presentation');
 const save=(label,extra={})=>observations.push({label,...extra,snapshot:snapshot()});
 await wait(ready,'all real Menu hover listeners registered');
 await fresh('outside leave250ms');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body')),early=await elapsed(leave,100);assert(early<200,'observed early sample before200ms');retain(a(),'early leave');save(scenario,{early,...await closeAfter(a(),leave,'mouse outside')});
 }
 await fresh('host reentry cancels pending leave');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,80);const enter=pointer(a(),'pointerenter',get('body'));assert(enter-leave<200,'reentry before original deadline');await elapsed(leave,450);retain(a(),'host reentry');save(scenario,{leave,enter,elapsed:performance.now()-leave});
 }
 await fresh('leave enter leave restarts deadline');await open(a());{
  const first=pointer(a(),'pointerleave',get('body'));await elapsed(first,70);pointer(a(),'pointerenter',get('body'));await elapsed(first,130);const last=pointer(a(),'pointerleave',get('body'));assert(last-first<200,'restart setup before first deadline');const oldSample=await elapsed(first,290);assert(performance.now()-last<200,'old deadline sample before latest deadline');retain(a(),'past old deadline');save(scenario,{first,last,oldSample,...await closeAfter(a(),last,'latest outside leave')});
 }
 for(const internal of ['host-child','flyout']){
  await fresh('accepted related target '+internal);await open(a());const target=internal==='flyout'?panel(a()):a().querySelector(':scope > [role=menuitem]');const leave=pointer(a(),'pointerleave',target);await elapsed(leave,450);retain(a(),'accepted internal leave without pending timer');save(scenario,{leave,target:node(target)});
 }
 await fresh('internal leave does not clear existing timer');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,80);const internal=pointer(a(),'pointerleave',panel(a()));assert(internal-leave<200,'internal leave before old deadline');save(scenario,{internal,...await closeAfter(a(),leave,'old outside timeout')});
 }
 await fresh('flyout entry clears existing timer');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,80);const enter=pointer(panel(a()),'pointerenter',a());assert(enter-leave<200,'flyout entry before deadline');await elapsed(leave,450);retain(a(),'flyout entry');save(scenario,{leave,enter});
 }
 await fresh('touch leave exemption');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'),'touch');await elapsed(leave,450);retain(a(),'touch leave');save(scenario,{leave,elapsed:performance.now()-leave});
 }
 await fresh('sibling A to B survives stale A deadline');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,70);const enter=pointer(b(),'pointerenter',a());await wait(()=>closed(a())&&presented(b()),'B current/A closed');await elapsed(leave,450);retain(b(),'B past old A deadline');save(scenario,{leave,enter});
 }
 await fresh('same VDOM separate Menu ownership');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,70);const enter=pointer(peer(),'pointerenter',a());await wait(()=>closed(a())&&presented(peer()),'separate Menu peer closes A');await elapsed(leave,450);retain(peer(),'peer survives old A deadline');save(scenario,{leave,enter,scope:'two Menu owners in one VDOM; not process/window-global equivalence'});
 }
 await fresh('category-only removal with pending timeout');await open(a());{
  const old=a(),oldInput=get('input-a'),leave=pointer(old,'pointerleave',get('body'));await elapsed(leave,70);const removed=control('category-toggle',false);await wait(()=>!a()&&!old.isConnected,'only A removed');const focusAfter=deepActive();await elapsed(leave,450);assert(!old.isConnected&&!oldInput.isConnected&&!nativeOpen(old),'removed A has no native revival');assert(deepActive()===focusAfter,'no delayed post-removal focus steal');assert(readyHost(b()),'B stays owned/ready');control('category-toggle',true);await wait(()=>readyHost(a())&&closed(a()),'new A ready/closed');assert(a()!==old&&get('input-a')!==oldInput,'restored A is a new category/scope');save(scenario,{leave,removed});
 }
 await fresh('fresh owner cancels prior pending hover');await open(a());{
  const old=a(),leave=pointer(old,'pointerleave',get('body'));await elapsed(leave,70);command('fresh');await wait(()=>a()&&a()!==old&&ready()&&closed(a())&&closed(b()),'fresh replacement owner');await elapsed(leave,450);assert(closed(a())&&closed(b())&&!old.isConnected,'old timer cannot mutate new owner');save(scenario,{leave,elapsed:performance.now()-leave});
 }
 await fresh('public expansion with pending timer');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,70);const mode=control('mode','expanded');await wait(()=>a()?.dataset.menuCategoryMode==='expanded','expanded public state');const early=await elapsed(leave,150),first=categoryState(a());assert(first.popup&&!first.panel,'measured source early expanded retains popup/panel false');const late=await elapsed(leave,450),last=categoryState(a());assert(!last.popup&&!last.panel&&!last.nativePopover,'measured source late expanded popup/panel closed');save(scenario,{leave,mode,samples:[{elapsed:early,state:first},{elapsed:late,state:last}],causality:'named observed endpoint; no private timeout introspection'});
 }
 await fresh('expanded host entry cancels before expanded guard');await open(a());{
  const leave=pointer(a(),'pointerleave',get('body'));await elapsed(leave,70);const mode=control('mode','expanded');await wait(()=>a()?.dataset.menuCategoryMode==='expanded','expanded before host entry');const enter=pointer(a(),'pointerenter',get('body'));assert(enter-leave<200,'expanded entry before old deadline');const early=await elapsed(leave,150),first=categoryState(a());const late=await elapsed(leave,450),last=categoryState(a());assert(first.popup&&!first.panel&&last.popup&&!last.panel,'measured source expanded reentry retains latent popup/panel false');retain(a(),'expanded latent popup');save(scenario,{leave,mode,enter,samples:[{elapsed:early,state:first},{elapsed:late,state:last}]});
 }
 await fresh('fresh expanded entry guard');control('mode','expanded');await wait(()=>a()?.dataset.menuCategoryMode==='expanded','fresh expanded mode');pointer(a(),'pointerenter',get('body'));await sleep(350);assert(closed(a())&&!panelOpen(a()),'expanded entry does not present closed popup/panel');save(scenario);
 await fresh('retained input and stateful child across hover/mode');await open(a());{
  const memory=get('memory-a'),input=get('input-a'),scope=memory.dataset.scopeToken;input.value='hover retained';input.dispatchEvent(new Event('input',{bubbles:true}));get('count-a').click();await wait(()=>memory.dataset.localCount==='1','local state increment');
  const retained=()=>get('memory-a')===memory&&get('input-a')===input&&memory.dataset.scopeToken===scope&&memory.dataset.localCount==='1'&&input.value==='hover retained';
  const leave=pointer(a(),'pointerleave',get('body'));await closeAfter(a(),leave,'retained-case hover close');assert(retained(),'hover close retains native nodes/scope/local state/value');control('mode','expanded');await wait(()=>a()?.dataset.menuCategoryMode==='expanded','expanded retained case');assert(retained(),'expanded mode retains input/child scope');control('mode','collapsed');await wait(()=>a()?.dataset.menuCategoryMode==='collapsed','collapsed retained case');await open(a());assert(retained(),'collapsed reopen retains input/child scope');save(scenario,{scope,inputValue:input.value,count:memory.dataset.localCount});
 }
 await fresh('dual surfaces with active child; body projection and live edits');control('active',true);await wait(()=>state().one_active,'active child public prop');await open(a());{
  const contentHost=expandedPanel(a()),initialPopupHost=panel(a()),originalBody=body(a()),input=get('input-a'),memory=get('memory-a'),scope=memory.dataset.scopeToken,child=a().querySelector('[data-menu-category-child]');
  input.value='projection retained';input.dispatchEvent(new Event('input',{bubbles:true}));get('count-a').click();await wait(()=>memory.dataset.localCount==='1','projection counter');
  const retained=()=>expandedPanel(a())===contentHost&&body(a())?.parentElement===contentHost&&body(a())===originalBody&&get('input-a')===input&&get('memory-a')===memory&&memory.dataset.scopeToken===scope&&memory.dataset.localCount==='1'&&input.value==='projection retained';
  child.focus();const beforeFocus=deepActive();journal.push({scenario,kind:'projection-focus',stage:'before public expansion',time:performance.now(),node:node(beforeFocus)});
  control('mode','expanded');await wait(()=>popup(a())&&panelOpen(a())&&body(a())?.parentElement===expandedPanel(a()),'active expanded panel plus independent latent popup');await new Promise(requestAnimationFrame);await new Promise(requestAnimationFrame);
  const afterFocus=deepActive(),projectionFocus={before:node(beforeFocus),after:node(afterFocus),retained:beforeFocus===afterFocus};journal.push({scenario,kind:'projection-focus',stage:'after public expansion',time:performance.now(),...projectionFocus});assert(projectionFocus.retained,'measured source oracle preserves deep child focus across public expansion');
  assert(retained(),'panel projection retains actual body/input/memory/scope/state');assert(nativeOpen(a())&&visible(a())&&!expandedPanel(a()).hidden,'both actual panel and native popup surfaces remain present');
  control('extra',true);await wait(()=>get('extra-a')?.closest('[data-menu-category-body]')===originalBody,'append arbitrary descendant after panel projection');const extra=get('extra-a');
  control('reverse',true);await wait(()=>state().reversed&&originalBody.firstElementChild===extra,'keyed reverse actual moved body children');assert(retained()&&get('extra-a')===extra,'reverse retains arbitrary node and memory');
  control('second',false);await wait(()=>!([...originalBody.querySelectorAll('[data-menu-category-child]')].some(item=>item.getAttribute('aria-label')==='Category A item 2')),'remove native child after projection');assert(retained(),'remove preserves body and memory');
  control('extra',false);await wait(()=>!get('extra-a')&&!extra.isConnected,'remove arbitrary appended descendant');assert(retained(),'arbitrary removal preserves body and memory');
  get('after').click();await wait(()=>!popup(a())&&panelOpen(a())&&body(a())?.parentElement===expandedPanel(a()),'expanded outside click closes only latent popup, retains panel');assert(retained(),'expanded outside closure preserves projected body/state');
  const cycles=[];for(let cycle=0;cycle<2;cycle++){
   control('mode','collapsed');await wait(()=>a()?.dataset.menuCategoryMode==='collapsed'&&body(a())?.parentElement===panel(a()),'body projects to popup');assert(retained(),'collapsed repeated projection preserves fixed content host/state');
   if(cycle===0){control('extra',true);await wait(()=>get('extra-a')?.closest('[data-menu-category-body]')===originalBody,'append arbitrary content while fixed host is popup');control('second',true);await wait(()=>[...originalBody.querySelectorAll('[data-menu-category-child]')].some(item=>item.getAttribute('aria-label')==='Category A item 2'),'restore child while fixed host is popup');control('extra',false);await wait(()=>!get('extra-a'),'remove arbitrary content while fixed host is popup');assert(retained(),'popup-side edits preserve fixed content/state');}
   if(!presented(a()))await open(a());control('mode','expanded');await wait(()=>panelOpen(a())&&body(a())?.parentElement===expandedPanel(a()),'body projects to active expanded panel');assert(retained(),'expanded repeated projection preserves state');cycles.push(categoryState(a()));
  }
  save(scenario,{scope,projectionFocus,cycles,surfaceAdapter:{fixedContentHost:expandedPanel(a())===contentHost,currentPopupMarkerSwitched:panel(a())!==initialPopupHost},identity:'same permanent content host/body/input/memory objects, scope token, value and local state; deep focus retained per source oracle without fixture repair'});
  control('mode','collapsed');await wait(()=>body(a())?.parentElement===panel(a()),'popup projection before category-only teardown');if(!presented(a()))await open(a());
  const removedCategory=a(),removedBody=body(a()),removedInput=get('input-a'),removedScope=get('memory-a').dataset.scopeToken,removeLeave=pointer(a(),'pointerleave',get('body'));await elapsed(removeLeave,70);control('category-toggle',false);
  await wait(()=>!a()&&!removedCategory.isConnected&&!removedBody.isConnected&&!removedInput.isConnected,'projected category and body unmount together');await elapsed(removeLeave,450);assert(!nativeOpen(removedCategory),'projected category popup never revives after drop');assert(events().some(row=>row.event==='memory-drop'&&String(row.value.scope)===removedScope),'category-only teardown drops owned stateful scope');
  control('category-toggle',true);await wait(()=>readyHost(a())&&closed(a()),'restored category after projected drop');assert(body(a())!==removedBody&&get('input-a')!==removedInput,'restored body/input have fresh identity');save('projected category-only teardown',{removedScope});
  await open(a());const oldMenu=menu(),oldBody=body(a()),oldInput=get('input-a'),oldScope=get('memory-a').dataset.scopeToken,ownerLeave=pointer(a(),'pointerleave',get('body'));await elapsed(ownerLeave,70);control('mount',false);
  await wait(()=>!menu()&&!oldMenu.isConnected&&!oldBody.isConnected&&!oldInput.isConnected,'full Menu owner removes projected content');await elapsed(ownerLeave,450);assert(!oldBody.isConnected&&!oldInput.isConnected,'old owner content cannot revive');assert(events().some(row=>row.event==='memory-drop'&&String(row.value.scope)===oldScope),'Menu owner teardown drops owned stateful scope');control('mount',true);await wait(()=>ready()&&closed(a())&&closed(b()),'new Menu owner after projected teardown');assert(menu()!==oldMenu&&body(a())!==oldBody&&get('input-a')!==oldInput,'new Menu owner does not reuse old body/input');save('projected full Menu owner teardown',{oldScope});
 }
 mutations.disconnect();listener.abort();
 return{ok:true,observations,journal,input:'synthetic untrusted native PointerEvent objects; real LinuxWebView DOM, native popovers and Rust timer/state; public fixture controls via change events',limits:['not normal/physical hover, actual touch hardware or held-button driver','native close COMMIT times differ from source request/completion events','two Menu ownership is per VDOM, not source Window/process-wide equivalence','no native 4px gap/diagonal hit-path proof from synthetic dispatch','source shadow related-target variants and external own-ID CustomEvent interface are not duplicated','fixed content host preserves body/focus while popup marker switches to empty chrome for dual state; literal source popup-host identity is an explicit adapter','no all-mode/veto/keyboard-to-hover focus/pen/platform/AT/Brand or private timer-resource proof','G04.6 nearest scrolling remains separate'],renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error)}}
"#;

#[component]
fn PeerMenu() -> Element {
    rsx! {
        div { id: "g04-hover-peer-stage", style: "height:12rem;position:relative;min-width:0",
            Application { start_expanded: false, pinned_menu: true, force_breakpoint: ApplicationBreakpoint::Large,
                menu: rsx! { Menu { label: "Hover peer navigation", application_name: "Peer", start_expanded: false, pinned: true,
                    MenuCategory { label: "Peer category", icon: "rocket", MenuItem { label: "Peer child" } }
                } },
                p { "Separate Menu owner in the same VDOM." }
            }
        }
    }
}

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Category hover native probe global timeout");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let window = window.clone();
            spawn(async move {
                let capabilities = dom_query::eval(CAPABILITIES);
                let _ = capabilities.send(());
                match capabilities.join::<serde_json::Value>().await {
                    Ok(value) => println!("Category hover native capabilities: {value}"),
                    Err(error) => { eprintln!("Category hover capability failure: {error:?}"); std::process::exit(1); }
                }
                let query = dom_query::eval(VERIFY);
                let _ = query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"] == true => { println!("Category hover native checks: {result}"); window.close(); }
                    result => { eprintln!("Category hover native failure: {result:?}"); std::process::exit(1); }
                }
            });
        }, menu_hover_review::MenuHoverReview {} PeerMenu {} }
    } }
}

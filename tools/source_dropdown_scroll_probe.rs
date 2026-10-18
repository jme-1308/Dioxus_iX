//! Excluded pinned-source nearest-scroll diagnostic; no source policy emulation.
use dioxus::prelude::*;
use std::time::Duration;
#[path = "../src/dom_query.rs"]
mod dom_query;

fn main() {
    let index = include_str!("upstream_menu_hover_review.html")
        .replace("/www/", "http://127.0.0.1:8089/www/")
        .replace(
            "/keyboard-foundation.css",
            "http://127.0.0.1:8089/keyboard-foundation.css",
        )
        .replace(
            "</body>",
            "<div id=\"main\" hidden></div><!-- MODULE LOADER --></body>",
        );
    let config = dioxus::desktop::Config::new()
        .with_custom_index(index)
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Pinned source Dropdown nearest scroll")
                .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 1000.0))
                .with_visible(true),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

const VERIFY: &str = r#"
const journal=[],observations=[],requests=[],completions=[],focusEvents=[],visibleFocusEvents=[],scrollEvents=[],modeEvents=[];
try {
 await dioxus.recv();
 const revision='8bf599bc68365d7752e12832e71b625883b93d7a',sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const objects=new WeakMap(),events=new WeakMap(),globalAbort=new AbortController();let objectSequence=0,eventSequence=0,sequence=0,generation=0,scenario='bootstrap',current=null,stage=null,mode=null;
 const oid=node=>{if(!node)return null;if(!objects.has(node))objects.set(node,++objectSequence);return objects.get(node);};
 const simple=node=>node instanceof Element?{object:oid(node),id:node.id,tag:node.tagName,connected:node.isConnected}:null;
 const rect=node=>{const r=node?.getBoundingClientRect();return r?{x:r.x,y:r.y,width:r.width,height:r.height,top:r.top,bottom:r.bottom,left:r.left,right:r.right}:null;};
 const deepActive=()=>{let node=document.activeElement;while(node?.shadowRoot?.activeElement)node=node.shadowRoot.activeElement;return node;};
 const focus=()=>{const chain=[];let node=document.activeElement;while(node){chain.push(simple(node));node=node.shadowRoot?.activeElement;}return chain;};
 const within=(ancestor,target=deepActive())=>{for(let node=target;node;){if(node===ancestor)return true;if(node.assignedSlot){node=node.assignedSlot;continue;}const p=node.parentNode;node=p instanceof ShadowRoot?p.host:p;}return false;};
 const eid=event=>{if(!events.has(event))events.set(event,++eventSequence);return events.get(event);};
 const record=(kind,data={})=>{const row={seq:++sequence,generation,scenario,kind,time:performance.now(),...data};journal.push(row);return row;};
 const boxState=node=>node?{node:simple(node),rect:rect(node),scrollTop:node.scrollTop,scrollLeft:node.scrollLeft,scrollHeight:node.scrollHeight,scrollWidth:node.scrollWidth,clientHeight:node.clientHeight,clientWidth:node.clientWidth}:null;
 const itemState=node=>node?{node:simple(node),rect:rect(node),tabIndex:node.tabIndex,disabled:node.disabled,hidden:node.hidden,sourceFocused:node.classList.contains('ix-focused'),actualFocused:within(node),ariaSelected:node.getAttribute('aria-selected')}:null;
 const dropdownState=node=>({node:simple(node),show:node?.show,topLayer:node?.enableTopLayer,nativeOpen:node?.shadowRoot?.querySelector('dialog')?.matches(':popover-open')??null,rect:rect(node)});
 const state=()=>current?{mode,focus:focus(),deepActive:simple(deepActive()),root:dropdownState(current.root),trigger:simple(current.trigger),triggerFocused:within(current.trigger),boxes:current.boxes.map(boxState),items:current.items.map(itemState),held:current.held.map(itemState),currentSameId:current.held.map(node=>({held:simple(node),current:simple(document.getElementById(node.id)),same:document.getElementById(node.id)===node})),category:current.category?{node:simple(current.category),parent:itemState(current.category.shadowRoot?.querySelector('.category-parent')),dropdown:dropdownState(current.category.shadowRoot?.querySelector('ix-dropdown')),menuExpanded:current.menu.expand,items:current.menuItems.map(itemState),body:boxState(current.category.shadowRoot?.querySelector('.category-dropdown-body'))}:null}:null;
 const assert=(ok,label)=>{if(!ok)throw Error(label+' '+JSON.stringify({state:state(),tail:journal.slice(-12)}));};
 const wait=async(predicate,label,timeout=6000)=>{const end=performance.now()+timeout;while(performance.now()<end){if(predicate())return;await sleep(5);}throw Error('Timeout '+label+' '+JSON.stringify({state:state(),tail:journal.slice(-12)}));};
 const frames=async(count=6,label='drain')=>{for(let index=1;index<=count;index++)await new Promise(resolve=>requestAnimationFrame(timestamp=>{record('native-frame-checkpoint',{label,index,timestamp,state:state()});resolve();}));};
 const key=(target,key,extra={})=>{const event=new KeyboardEvent('keydown',{key,bubbles:true,cancelable:true,composed:true,...extra});const before=record('driver-key-before',{event:eid(event),target:simple(target),key,altKey:event.altKey,state:state()});target.dispatchEvent(event);record('driver-key-return',{event:eid(event),before:before.seq,key,target:simple(target),defaultPrevented:event.defaultPrevented,state:state()});return event;};
 const observe=(node,name,abort)=>{for(const type of ['showChange','showChanged'])node.addEventListener(type,event=>{if(event.composedPath()[0]!==node)return;const row=record(type==='showChange'?'source-request':'source-completion',{event:eid(event),overlay:name,node:simple(node),detail:event.detail,currentShow:node.show,defaultPrevented:event.defaultPrevented,focus:focus()});(type==='showChange'?requests:completions).push(row);queueMicrotask(()=>{row.defaultPreventedQueued=event.defaultPrevented;});},{passive:true,signal:abort.signal});};
 document.addEventListener('ixFocusVisibleChange',event=>visibleFocusEvents.push(record('source-visible-focus-change',{event:eid(event),currentFocus:Array.from(event.detail?.currentFocus??[]).map(simple),trusted:event.isTrusted,focus:focus(),boxes:current?.boxes.map(boxState)})),{passive:true,signal:globalAbort.signal});
 document.addEventListener('keyboardmodechange',event=>{if(typeof event.detail?.keyboardMode==='boolean'){mode=event.detail.keyboardMode;modeEvents.push(record('source-keyboardmodechange',{detail:event.detail}));}},{passive:true,signal:globalAbort.signal});
 for(const type of ['focusin','focusout'])document.addEventListener(type,event=>focusEvents.push(record('native-'+type,{event:eid(event),target:simple(event.target),origin:simple(event.composedPath()[0]),trusted:event.isTrusted,focus:focus(),boxes:current?.boxes.map(boxState)})),{capture:true,passive:true,signal:globalAbort.signal});
 const ready=async node=>{await node.componentOnReady?.();};
 const makeItem=(ctx,index,tag='ix-dropdown-item')=>{const item=document.createElement(tag);item.id='scroll-'+ctx.generation+'-'+tag+'-'+index;item.label='Item '+String(index+1).padStart(2,'0');item.style.cssText='display:block;box-sizing:border-box;height:40px;min-height:40px;max-height:40px;flex:none';return item;};
 const makeBox=()=>{const node=document.createElement('div');node.style.cssText='display:block;box-sizing:border-box;width:280px;height:160px;overflow:auto;scroll-behavior:auto;border:0;padding:0;flex:none';return node;};
 const watchScroll=(ctx,box,label)=>box.addEventListener('scroll',event=>scrollEvents.push(record('native-scroll',{event:eid(event),box:label,state:boxState(box),focus:focus()})),{passive:true,signal:ctx.abort.signal});
 async function fresh(label,{roving=false,disabledLast=false,noFocus=false,nested=false,category=false}={}){
  current?.abort.abort();stage?.remove();await frames(4,'previous-scope-drain');generation++;scenario=label;
  stage=document.createElement('section');stage.id='scroll-stage';stage.style.cssText='position:fixed;inset:16px;z-index:1000;background:white;color:#202932;padding:16px;font:16px/1.4 system-ui;overflow:auto';
  const outside=document.createElement('button');outside.textContent='Outside keyboard seed';const trigger=document.createElement('button');trigger.id='scroll-'+generation+'-trigger';trigger.textContent='Root trigger';
  const root=document.createElement('ix-dropdown'),box=makeBox(),ctx={generation,root,box,trigger,outside,items:[],held:[],boxes:[],abort:new AbortController(),roving,menuItems:[]};current=ctx;
  root.id='scroll-'+generation+'-root';root.trigger=trigger;root.enableTopLayer=true;root.navigationMode=roving?'roving-tabindex':'active-descendant';root.disableFocusHandling=noFocus;root.suppressAutomaticPlacement=true;root.suppressTriggerVisibilityCheck=true;root.suppressOverflowBehavior=true;
  const heading=document.createElement('p');heading.textContent='Pinned source nearest-scroll diagnostic: '+label;stage.append(heading,outside,trigger);
  if(category){const app=document.createElement('ix-application'),menu=document.createElement('ix-menu'),cat=document.createElement('ix-menu-category');app.breakpoints=['lg'];app.style.cssText='height:600px;width:100%';menu.startExpanded=false;menu.pinned=false;menu.style.cssText='position:relative;height:560px;--ix-menu--width:52px;--ix-menu--expanded-width:256px';cat.id='scroll-'+generation+'-category';cat.label='Category first';for(let i=0;i<18;i++){const item=makeItem(ctx,i,'ix-menu-item');cat.append(item);ctx.menuItems.push(item);}box.append(cat);ctx.category=cat;ctx.menu=menu;ctx.app=app;menu.append(root);app.append(menu);stage.append(app);}
  else stage.append(root);
  for(let i=0;i<18;i++){const item=makeItem(ctx,i);if(disabledLast&&i===17)item.disabled=true;ctx.items.push(item);box.append(item);}
  const tail=document.createElement('div');tail.style.cssText='height:120px;min-height:120px;pointer-events:none';box.append(tail);
  if(nested){const outer=document.createElement('div');outer.style.cssText='width:320px;height:200px;overflow:auto;scroll-behavior:auto;padding:0;border:0';const lead=document.createElement('div'),end=document.createElement('div');lead.style.height='80px';end.style.height='80px';outer.append(lead,box,end);root.append(outer);ctx.boxes=[box,outer];}else{root.append(box);ctx.boxes=[box];}
  document.body.append(stage);await Promise.all([root,...ctx.items,...ctx.menuItems,...(category?[ctx.app,ctx.menu,ctx.category]:[])].map(ready));
  await wait(()=>trigger.hasAttribute('data-ix-dropdown-trigger'),'source registered actual trigger');observe(root,'root',ctx.abort);ctx.boxes.forEach((node,i)=>watchScroll(ctx,node,'box-'+i));
  outside.focus();key(outside,'Tab');assert(mode===true,'actual upstream detector keyboard mode');trigger.focus();record('driver-public-show',{node:simple(root),value:true,classification:'public property setup, no controller request implied'});root.show=true;
  await wait(()=>root.show===true&&root.shadowRoot?.querySelector('dialog')?.matches(':popover-open'),'Root actual native open');await frames(8,'open-settle');
  if(roving){key(trigger,'ArrowDown');await frames(8,'public-roving-initialize');assert(ctx.items.some(item=>item.tabIndex===0),'source roving item initialized');}
  box.scrollTop=0;if(nested)ctx.boxes[1].scrollTop=0;await frames(2,'geometry-reset');
  assert(box.clientHeight===160&&box.scrollHeight>box.clientHeight,'actual constrained scrollbox');
  if(!category){const first=rect(ctx.items[0]),last=rect(ctx.items[17]);assert(Math.abs(first.height-40)<1&&Math.abs(last.height-40)<1&&Math.abs(last.top-first.top-680)<1&&Math.abs(box.scrollHeight-840)<1,'actual18×40 plus120 geometry');}
  record('fixture-ready',{options:{roving,disabledLast,noFocus,nested,category,suppressOverflowBehavior:true},state:state()});return ctx;
 }
 const inputTarget=ctx=>ctx.roving?(ctx.items.find(item=>item.tabIndex===0)??ctx.items[0]):ctx.trigger;
 const capture=(ctx,extra={})=>{const row={case:observations.length+1,label:scenario,generation:ctx.generation,from:ctx.from,to:sequence,state:state(),...extra};observations.push(row);return row;};
 async function runKeys(ctx,keys,label='input'){record('operation-before',{label,state:state()});for(const value of keys)key(inputTarget(ctx),value);record('operation-sync',{label,state:state()});await frames(8,label);}
 await wait(()=>window.upstreamMenuHoverReview?.snapshot()?.ready,'cached source bootstrap',20000);assert(window.upstreamMenuHoverReview.snapshot().revision===revision,'source revision');document.getElementById('upstream-menu-hover-application')?.remove();await frames(4,'bootstrap-remove');
 await Promise.all(['ix-dropdown','ix-dropdown-item','ix-application','ix-menu','ix-menu-category','ix-menu-item'].map(name=>customElements.whenDefined(name)));
 {
  const ctx=await fresh('AD End offscreen');ctx.from=sequence+1;await runKeys(ctx,['End']);capture(ctx);
 }
 {
  const ctx=await fresh('Roving End offscreen',{roving:true});ctx.from=sequence+1;await runKeys(ctx,['End']);capture(ctx);
 }
 {
  const ctx=await fresh('AD visible Home then partial End');ctx.from=sequence+1;await runKeys(ctx,['Home'],'visible-Home');const visible=record('visible-Home-settled',{state:state()});ctx.box.scrollTop=540;await frames(2,'partial-condition');const b=rect(ctx.box),r=rect(ctx.items[17]);assert(Math.abs(r.bottom-b.bottom-20)<1&&r.top>=b.top,'actual last item clipped20px');record('partial-End-conditioned',{state:state()});await runKeys(ctx,['End'],'partial-End');capture(ctx,{visibleHome:visible.seq});
 }
 {
  const ctx=await fresh('AD nested scroll containers',{nested:true});ctx.from=sequence+1;await runKeys(ctx,['End']);capture(ctx);
 }
 for(const roving of [false,true]){
  const ctx=await fresh((roving?'Roving':'AD')+' same-turn End Home End',{roving});ctx.from=sequence+1;await runKeys(ctx,['End','Home','End'],'three-admitted-inputs');const endHomeEnd=record('End-Home-End-settled',{state:state()});await runKeys(ctx,['Home'],'drained-Home-before-arrow-burst');const home=record('Home-before-arrow-burst-settled',{state:state()}),origin=inputTarget(ctx),arrowFrom=sequence+1;record('arrow-burst-before',{origin:simple(origin),state:state()});key(origin,'ArrowDown');key(origin,'ArrowDown');record('arrow-burst-sync',{origin:simple(origin),state:state()});await frames(8,'same-turn-two-ArrowDown');const arrows=record('arrow-burst-settled',{state:state()});capture(ctx,{endHomeEnd:{seq:endHomeEnd.seq,state:endHomeEnd.state},homeBeforeArrowBurst:{seq:home.seq,state:home.state},arrowDownBurst:{origin:simple(origin),from:arrowFrom,to:arrows.seq,state:arrows.state}});
 }
 {
  const ctx=await fresh('Roving explicit second-frame nearest intervention',{roving:true});ctx.from=sequence+1;key(inputTarget(ctx),'End');let intervention=null;await new Promise(resolve=>requestAnimationFrame(timestamp=>{const actualFocused=within(ctx.items[17]);record('post-focus-observer',{timestamp,actualFocused,state:state()});if(actualFocused){ctx.box.scrollTop=540;intervention=record('driver-public-scroll-intervention',{scrollTop:540,reason:'already-focused target lower20px clipped before nested source scroll',state:state()}).seq;}else record('intervention-condition-missed',{state:state()});resolve();}));await frames(7,'after-intervention');capture(ctx,{intervention,conditionMet:intervention!==null});
 }
 {
  const ctx=await fresh('Disabled last endpoint and Alt ArrowDown negative',{disabledLast:true});ctx.from=sequence+1;await runKeys(ctx,['End'],'eligible-End');const eligible=record('eligible-End-settled',{state:state()});key(ctx.trigger,'ArrowDown',{altKey:true});await frames(8,'Alt-ArrowDown');capture(ctx,{eligible:eligible.seq});
 }
 {
  const ctx=await fresh('Initial disableFocusHandling End negative',{noFocus:true});ctx.from=sequence+1;await runKeys(ctx,['End']);capture(ctx);
 }
 {
  const ctx=await fresh('Actual collapsed MenuCategory special target and internal End',{category:true});ctx.from=sequence+1;await wait(()=>ctx.menu.expand===false,'actual collapsed Menu');const parent=ctx.category.shadowRoot?.querySelector('.category-parent'),inner=ctx.category.shadowRoot?.querySelector('ix-dropdown'),body=ctx.category.shadowRoot?.querySelector('.category-dropdown-body');assert(parent&&inner&&body,'actual Category parent Dropdown and body');await ready(inner);observe(inner,'category',ctx.abort);body.style.cssText='height:160px;max-height:160px;min-height:160px;overflow:auto;scroll-behavior:auto';const spacer=document.createElement('div');spacer.style.height='120px';body.append(spacer);watchScroll(ctx,body,'category-body');record('actual-query-inventory',{rootItems:[...ctx.root.querySelectorAll('ix-dropdown-item:not([tabindex^="-"]):not([disabled]):not([hidden]),ix-menu-item:not([tabindex^="-"]):not([disabled]):not([hidden]),ix-menu-category:not([tabindex^="-"]):not([disabled]):not([hidden])')].map(simple),categoryParent:simple(parent),state:state()});await runKeys(ctx,['Home'],'outer-Home-category');const special=record('category-special-target-settled',{state:state()});key(parent,'Enter');await wait(()=>inner.show===true,'actual collapsed Category Enter opens its Dropdown');await frames(10,'category-entry-focus');record('category-entry-settled',{state:state()});body.scrollTop=0;await frames(2,'category-scroll-reset');key(ctx.menuItems[0],'End');await frames(10,'category-internal-End');capture(ctx,{special:special.seq,categoryParent:simple(parent),classification:'outer special target, collapsed entry focus-only, then internal keyboard navigation are distinct actual paths'});
 }
 {
  const ctx=await fresh('Pending End then actual Root removal');ctx.from=sequence+1;ctx.held=[ctx.items[17]];key(ctx.trigger,'End');ctx.root.remove();record('driver-actual-root-remove',{state:state()});await frames(10,'detached-old-root');capture(ctx,{classification:'retained native source objects; Rust owner cancellation is an adapter'});
 }
 {
  const ctx=await fresh('Pending End then same-ID actual item replacement');ctx.from=sequence+1;const old=ctx.items[17],replacement=makeItem(ctx,17);ctx.held=[old];key(ctx.trigger,'End');old.replaceWith(replacement);ctx.items[17]=replacement;record('driver-actual-item-replace',{old:simple(old),replacement:simple(replacement),samePublicId:old.id===replacement.id,state:state()});await ready(replacement);await frames(10,'replacement');capture(ctx,{classification:'source captured old actual item; public-ID reuse does not establish retargeting'});
 }
 assert(observations.length===12,'twelve source scroll diagnostics');current?.abort.abort();globalAbort.abort();return{ok:true,revision,caseCount:observations.length,observations,sourceRequests:requests,sourceCompletions:completions,nativeFocusEvents:focusEvents,sourceVisibleFocusEvents:visibleFocusEvents,nativeScrollEvents:scrollEvents,sourceModeEvents:modeEvents,journal,limits:['synthetic key input and property setup; no physical default input or Brand geometry parity','focus may autoscroll in first frame; explicit nearest is independently observed only with qualifying frame/offset evidence','own native RAF observers do not expose source RAF identities or wrap its scheduler','case7 public scroll intervention is deliberate diagnostic, not ordinary user flow','Category outer Home, collapsed entry focus-only and subsequent internal End are separate paths','detached and same-ID retained native objects differ from required Rust owner/replacement cancellation adapters','foundation/system font diagnostic geometry and cache provenance require external integrity record']};
} catch(error){return{ok:false,error:String(error),caseCount:observations.length,observations,sourceRequests:requests,sourceCompletions:completions,nativeFocusEvents:focusEvents,sourceVisibleFocusEvents:visibleFocusEvents,nativeScrollEvents:scrollEvents,sourceModeEvents:modeEvents,journal};}
"#;

#[component]
fn Probe() -> Element {
    let window = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(90)).await;
            eprintln!("Pinned source nearest-scroll global timeout");
            std::process::exit(1);
        });
    });
    rsx! { div { onmounted: move |_| {
        let window = window.clone();
        spawn(async move {
            let query = dom_query::eval(VERIFY);
            let _ = query.send(());
            match query.join::<serde_json::Value>().await {
                Ok(result) if result["ok"] == true => { println!("Pinned source Dropdown scroll checks: {result}"); window.close(); }
                Ok(result) => { eprintln!("Pinned source Dropdown scroll failure: {result}"); std::process::exit(1); }
                Err(error) => { eprintln!("Pinned source Dropdown scroll eval failure: {error:?}"); std::process::exit(1); }
            }
        });
    } } }
}

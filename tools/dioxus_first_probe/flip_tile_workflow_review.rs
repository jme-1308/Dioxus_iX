//! Excluded actual-gallery flip tile workflow and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/asset_flip_tile.rs"]
mod gallery_tile;

#[component]
pub fn FlipTileWorkflowPreview() -> Element {
    let mut narrow = use_signal(|| false);
    let mut reduced = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_tile::STYLES} }
        IxTheme { class: if reduced() { "flip-tile-workflow-review flip-tile-force-reduced" } else { "flip-tile-workflow-review" },
            h1 { "Dioxus asset flip tile" }
            Button { id: "flip-tile-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            Button { id: "flip-tile-reduced", variant: ButtonVariant::Secondary,
                onclick: move |_| reduced.toggle(), if reduced() { "Use normal motion" } else { "Use reduced motion" }
            }
            div { id: "flip-tile-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_tile::AssetFlipTileExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_FLIP_TILE_WORKFLOW_CHECK);
                        if let Err(error) = query.send("fliptileworkflow") {
                            eprintln!("Flip tile workflow native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native flip tile workflow: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native flip tile workflow failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Flip tile workflow native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.flip-tile-workflow-review{box-sizing:border-box;min-height:100vh;padding:24px}.flip-tile-workflow-review h1{font-size:28px;margin:0 0 24px}.flip-tile-workflow-review h2{font-size:22px;margin:24px 0 16px}.flip-tile-workflow-review h3{font-size:18px}.flip-tile-workflow-review p{max-width:72ch;line-height:1.5}#flip-tile-frame{margin-top:24px}.flip-tile-force-reduced .ix-flip-tile-container[data-phase=outgoing],.flip-tile-force-reduced .ix-flip-tile-container[data-phase=incoming]{animation-duration:.01ms}@media(max-width:600px){.flip-tile-workflow-review{padding:16px}}";

// Excluded driver only. The gallery owns all state and normal CSS animation
// events finish turns. This never fabricates animationend, measures elapsed
// chronology, or substitutes a timeout for animation completion. DOM clicks
// activate native buttons; untrusted keyboard events establish permission only.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_FLIP_TILE_WORKFLOW_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},animationNames=[],focusReturns=[];
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('asset-flip-tile'),result=()=>get('tile-result');
 const tile=()=>get('tile-preview')?.querySelector('article.ix-flip-tile');
 const container=()=>tile()?.querySelector('.ix-flip-tile-container');
 const eye=()=>tile()?.querySelector('button.ix-flip-tile-toggle');
 const pages=()=>[...tile()?.querySelectorAll('[data-flip-tile-page]')??[]];
 const page=id=>pages().find(node=>node.dataset.flipTilePage===id);
 const rendered=node=>!!node&&node.isConnected&&node.getClientRects().length>0&&!node.closest('[hidden]')&&getComputedStyle(node).visibility!=='hidden';
 const visible=()=>pages().filter(rendered).map(node=>node.dataset.flipTilePage);
 const phase=()=>container()?.dataset.phase;
 const data=name=>result()?.dataset[name],number=name=>Number(data(name));
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<200;attempt++){
   stable=predicate()?stable+1:0;if(stable===3)return;await hold(20);
  }
  throw Error(label+'; visible='+JSON.stringify(visible())+'; phase='+phase()+'; host='+result()?.textContent+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const focus=async(node,label)=>{assert(node&&rendered(node),label+' exists and is rendered');node.focus();await wait(label+' native focus',()=>active()===node)};
 const click=async(node,label)=>{assert(node&&rendered(node)&&!node.disabled,label+' is an available rendered control');node.click();await hold(30)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const input=async value=>{const node=get('tile-note');assert(rendered(node),'service textarea rendered');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('local service note input applied',()=>get('tile-note')?.value===value)};
 const option=async(id,value)=>{assert(get(id)&&rendered(get(id)),id+' option is rendered');if(get(id).checked!==value)await click(get(id),id);await wait(id+' native checked state applied',()=>get(id).checked===value)};
 const choice=async value=>{const node=get('tile-appearance');assert(node&&[...node.options].some(option=>option.value===value),'native appearance option available');node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('appearance selection applied',()=>node.value===value&&tile()?.classList.contains('ix-flip-tile-'+value))};
 const settle=async(id,label)=>wait(label,()=>phase()==='idle'&&visible().join(',')===id&&data('key')===(id||'none')&&Number(tile()?.dataset.count)===pages().length&&number('count')===pages().length&&Number(tile()?.dataset.index)===number('index'));
 const external=async(id,label)=>{await click(get('tile-'+id),label);await settle(id,label+' actual page settled')};
 const turn=async(id,label)=>{await click(eye(),label);await settle(id,label+' completed by actual CSS animation')};
 const snapshot=()=>({index:number('index'),count:number('count'),key:data('key'),requests:number('requests'),accepted:number('accepted'),actualIndex:Number(tile()?.dataset.index),visible:visible(),order:pages().map(node=>node.dataset.flipTilePage),phase:phase(),note:get('tile-note')?.value,inspected:get('tile-inspected')?.checked,active:active()?.getAttribute('aria-label')||active()?.id||active()?.tagName});
 const reset=async()=>{await click(get('tile-reset'),'Restore tile');await settle('overview','reset initial page settled');await wait('reset clears host and retained local service state',()=>number('count')===3&&number('index')===0&&number('requests')===0&&number('accepted')===0&&get('tile-note')?.value===''&&get('tile-inspected')?.checked===false&&!get('tile-veto').checked&&!get('tile-hold').checked&&get('tile-history').checked&&!get('tile-single').checked&&!get('tile-empty').checked&&get('tile-appearance').value==='filled'&&!get('tile-auto').checked)};

 await wait('actual retained asset tile gallery mounted',()=>host()&&tile()&&result()&&pages().length===3&&eye()&&get('tile-note')&&get('tile-inspected'));
 await wait('WebView ready for native focus',()=>document.hasFocus());
 host().addEventListener('animationend',event=>{if(event.target.classList?.contains('ix-flip-tile-container')&&['ix-pattern-flip-out','ix-pattern-flip-in'].includes(event.animationName))animationNames.push(event.animationName)});
 host().addEventListener('focusin',event=>{if(event.target===eye()||event.target===tile())focusReturns.push({owner:event.target===eye()?'eye':'article',eyeDisabled:eye().disabled,phase:phase()})});
 await settle('overview','initial Overview is the single actually visible retained page');
 assert(pages().map(node=>node.dataset.flipTilePage).join(',')==='overview,service,events'&&number('count')===3&&number('index')===0&&number('requests')===0&&number('accepted')===0,'initial stable pages and counters agree');
 assert(pages().every(node=>node.getAttribute('role')==='group'&&!!node.getAttribute('aria-label'))&&!rendered(get('tile-note')),'hidden retained pages remain mounted with accessible labels');
 assert(tile().getAttribute('aria-label')==='Motor 001 views'&&eye().getAttribute('aria-label')==='Show next asset view'&&page('overview').textContent.includes('42.6')&&tile().querySelector('footer')?.textContent.includes('Monthly service'),'actual visible measurements and named tile/header/footer rendered');
 snapshots.initial=snapshot();

 const stableEye=eye();await focus(eye(),'native eye button');const enter=key(active(),'Enter');keyPermissions.eyeEnter=!enter.defaultPrevented;assert(!enter.defaultPrevented,'native eye retains Enter default permission');
 await turn('service','native eye activation');await wait('actual CSS outgoing and incoming completion events observed',()=>animationNames.includes('ix-pattern-flip-out')&&animationNames.includes('ix-pattern-flip-in'));
 assert(eye()===stableEye&&active()===stableEye&&!eye().disabled&&number('requests')===1&&number('accepted')===1,'completed turn retains native eye focus for repeated keyboard use and one request');
 await focus(get('tile-note'),'local service textarea');for(const value of ['Home','End','ArrowLeft',' ','Tab']){const event=key(active(),value);keyPermissions['note'+value]=!event.defaultPrevented;assert(!event.defaultPrevented,'native textarea '+value+' remains available')}
 await input('Check motor bearing');await click(get('tile-inspected'),'Inspection complete');await wait('local inspection checkbox checked',()=>get('tile-inspected').checked);
 await turn('events','turn to recent events');assert(page('events').textContent.includes('Inspection scheduled')&&page('events').textContent.includes('Motor restarted'),'actual events page content rendered');
 await turn('overview','cycle to overview');await turn('service','cycle back to retained service page');
 assert(get('tile-note').value==='Check motor bearing'&&get('tile-inspected').checked&&pages().length===3,'ordinary local Dioxus service state survives full hidden cycle');
 snapshots.retainedCycle=snapshot();

 const options=get('tile-options');assert(options?.tagName==='DETAILS','advanced options use actual native details');if(!options.open)await click(options.querySelector('summary'),'Tile options summary');await wait('native details opens actual tile controls',()=>options.open&&rendered(get('tile-veto')));
 await option('tile-veto',true);let before=number('requests'),accepted=number('accepted');await focus(eye(),'veto eye');await click(eye(),'veto turn');
 await wait('synchronous veto leaves actual visible page and eye focus unchanged',()=>number('requests')===before+1&&number('accepted')===accepted&&phase()==='idle'&&visible().join(',')==='service'&&active()===eye());
 await option('tile-veto',false);await option('tile-hold',true);before=number('requests');accepted=number('accepted');await click(eye(),'held turn');
 await wait('accepted controlled-held turn preserves actual page and focus',()=>number('requests')===before+1&&number('accepted')===accepted+1&&phase()==='idle'&&visible().join(',')==='service'&&active()===eye()&&get('tile-note').value==='Check motor bearing');
 await option('tile-hold',false);snapshots.vetoAndHeld=snapshot();

 await focus(get('tile-note'),'inside service before external switch');const priorFocusReturns=focusReturns.length;await external('overview','external Overview');await wait('inside focus returns to an available mounted tile owner',()=>active()===eye()||active()===tile());
 const externalReturn=focusReturns.slice(priorFocusReturns).at(-1);assert(externalReturn&&(externalReturn.owner==='eye'&&!externalReturn.eyeDisabled||externalReturn.owner==='article'&&externalReturn.eyeDisabled),'actual focus return chooses enabled eye or article while eye is disabled');
 await focus(get('tile-appearance'),'outside appearance selector');await external('service','external Service note');await wait('outside focus survives actual page switch',()=>active()===get('tile-appearance'));
 assert(get('tile-note').value==='Check motor bearing'&&get('tile-inspected').checked,'external selection retains local service state');snapshots.externalFocus=snapshot();

 await external('overview','Overview before reorder');await click(get('tile-reverse'),'Reverse views');await settle('overview','reorder preserves current stable view');
 assert(pages().map(node=>node.dataset.flipTilePage).join(',')==='events,service,overview'&&number('index')===2&&get('tile-note').value==='Check motor bearing','reordered stable wrappers preserve page identity and local state');
 await turn('events','reversed cycle wraps to recent events');await option('tile-history',false);await settle('overview','removing current history selects available Overview');
 assert(!page('events')&&pages().length===2&&get('tile-events').disabled&&get('tile-note').value==='Check motor bearing','removed current page releases wrapper without dropping remaining service state');
 await external('service','service after history removal');assert(get('tile-inspected').checked,'retained service inspection survives sibling removal');snapshots.dynamicPages=snapshot();

 await focus(get('tile-note'),'inside service before single view');await option('tile-single',true);await settle('overview','single view selects Overview');await wait('single disabled eye returns inside focus to known article',()=>pages().length===1&&eye().disabled&&active()===tile());
 before=number('requests');eye().click();await hold(80);assert(number('requests')===before,'single disabled eye does not request');
 await option('tile-empty',true);await settle('','empty tile settles without visible pages');assert(pages().length===0&&number('count')===0&&number('index')===0&&eye().disabled,'empty tile has no retained content and disabled eye');before=number('requests');eye().click();await hold(80);assert(number('requests')===before,'empty disabled eye does not request');
 const oldTile=tile(),toolbar=get('tile-reset'),optionsNode=get('tile-options');await reset();assert(tile()!==oldTile&&get('tile-reset')===toolbar&&get('tile-options')===optionsNode&&optionsNode.open,'Restore tile remounts tile only and preserves options element');snapshots.singleEmptyReset=snapshot();

 assert([...get('tile-appearance').options].map(node=>node.value).join(',')==='filled,outline,primary,info,warning,alarm','native appearance control retains all supported palettes');
 await choice('warning');assert(tile().querySelector('footer')&&getComputedStyle(tile().querySelector('footer')).color,'warning appearance retains actual footer');await choice('outline');
 await option('tile-auto',true);await external('service','content-height service page');assert(tile().style.height==='auto'&&rendered(get('tile-note')),'content height keeps native service controls rendered');await input('Auto height service note');snapshots.appearanceAndAuto=snapshot();

 await click(get('flip-tile-reduced'),'forced reduced motion preview');assert(host().closest('.flip-tile-force-reduced'),'excluded fixture applies existing reduced-duration policy');
 const priorEvents=animationNames.length;await turn('events','reduced motion turn');await wait('real reduced CSS animation events complete',()=>animationNames.slice(priorEvents).includes('ix-pattern-flip-out')&&animationNames.slice(priorEvents).includes('ix-pattern-flip-in'));
 await turn('overview','reduced cycle Overview');await turn('service','reduced return Service');assert(get('tile-note').value==='Auto height service note','reduced motion preserves ordinary retained state');snapshots.reducedMotion=snapshot();

 await click(get('flip-tile-narrow'),'narrow preview');await wait('narrow tile fits actual 340px frame',()=>get('flip-tile-frame').getBoundingClientRect().width<=341&&tile().getBoundingClientRect().width<=341&&get('tile-note').getBoundingClientRect().width<=tile().getBoundingClientRect().width);
 await focus(get('tile-note'),'narrow native textarea');await input('Narrow motor service check');get('tile-note').scrollIntoView({block:'center',inline:'nearest'});await hold(50);
 const box=get('tile-note').getBoundingClientRect(),hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);assert(hit===get('tile-note'),'narrow textarea remains visible and hit-testable');
 layout={frameWidth:get('flip-tile-frame').getBoundingClientRect().width,tileWidth:tile().getBoundingClientRect().width,tileHeight:tile().getBoundingClientRect().height,noteWidth:get('tile-note').getBoundingClientRect().width,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(layout.pageScroll<=layout.pageClient+1,'narrow retained tile stays within page width');
 await turn('events','narrow eye turn');await turn('overview','narrow cycle Overview');await turn('service','narrow retained Service');assert(get('tile-note').value==='Narrow motor service check','narrow cycle preserves actual locally edited note');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,animationNames,focusReturns,
  checks:['three keyed retained pages with one actual visible page and named tile','native eye activation completed by real CSS animation events','local note and inspection survive ordinary hidden cycle','synchronous veto and controlled-held views retain actual page/focus','external switch returns inside focus and preserves outside focus','stable reorder and removed current history preserve remaining state','single/empty disabled controls and Restore tile lifecycle','appearance options and content-height native controls','forced reduced CSS still completes actual turns','narrow native textarea hit testing and retained cycle'],
  animation:'actual CSS animationend events observed; no fabricated events, elapsed timing measurements or fallback completion',
  input:'native DOM click, synthetic input/key events and explicit native focus; trusted Enter/Space/Tab and textarea editing defaults are not simulated',
  trustedInput:'separate Chromium actual keyboard activation/traversal and textarea typing; no physical-device certification',renderer:navigator.userAgent};
}catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,animationNames,focusReturns,documentFocusedNow:document.hasFocus()}}
"#;

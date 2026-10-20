//! Excluded actual-gallery maintenance task cards and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/maintenance_launch.rs"]
mod gallery_tasks;

#[component]
pub fn TaskCardsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_tasks::STYLES} }
        IxTheme { class: "task-cards-review",
            h1 { "Independent maintenance task cards" }
            Button { id: "task-cards-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "task-cards-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_tasks::MaintenanceLaunchExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_TASK_CARDS_CHECK);
                        if let Err(error) = query.send("taskcards") {
                            eprintln!("Task cards native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native task cards: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native task cards failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Task cards native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.task-cards-review{box-sizing:border-box;min-height:100vh;padding:24px}.task-cards-review h1{font-size:28px;margin:0 0 24px}.task-cards-review h2{font-size:22px;margin:24px 0 16px}.task-cards-review h3{font-size:18px}.task-cards-review p{max-width:72ch;line-height:1.5}#task-cards-frame{margin-top:24px}@media(max-width:600px){.task-cards-review{padding:16px}}";

// Excluded driver JS sends synthetic events and observes actual task records,
// selection, retained input and owned controls. Untrusted native Enter/Space/Tab
// cannot execute browser defaults: permission is checked, followed by one real
// DOM click. Native page scroll positions test targets before programmatic focus;
// no artificial scrollport, production DOM bridge or elapsed timing oracle.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TASK_CARDS_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('maintenance-launch'),result=()=>get('launch-result'),backlog=()=>get('launch-backlog');
 const action=kind=>get(kind==='Inspection'?'launch-inspection':'launch-service');
 const primary=()=>backlog()?.querySelector('button.ix-card-select');
 const review=()=>get('launch-review'),toggle=()=>backlog()?.querySelector('button.ix-disclosure-button[aria-controls]');
 const panel=()=>get(toggle()?.getAttribute('aria-controls')),opened=()=>toggle()?.getAttribute('aria-expanded')==='true';
 const note=()=>get('launch-note'),toolkit=()=>get('launch-toolkit');
 const notification=()=>backlog()?.querySelector('.ix-push-notification');
 const title=()=>backlog()?.querySelector('.ix-card-title-text');
 const records=()=>[...get('launch-task-records')?.querySelectorAll('tbody tr[id^="launch-task-"]')??[]];
 const field=(row,key)=>row?.querySelector('[data-field="'+key+'"]')?.textContent.trim();
 const reviewed=row=>row?.getAttribute('data-reviewed')==='true';
 const count=name=>Number(result()?.dataset[name]);
 const checked=node=>node instanceof HTMLInputElement?node.checked:node.getAttribute('aria-checked')==='true';
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; result='+result()?.textContent+'; details='+opened()+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const key=(node,key)=>{const event=new KeyboardEvent('keydown',{key,code:key===' '?'Space':key,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setNote=async value=>{
  const node=note();assert(opened()&&node,'actual retained preparation input is open');
  Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('actual preparation note input applied',()=>note()===node&&note().value===value);
 };
 const create=async kind=>{
  const before=count('created'),length=records().length,pending=count('pending');action(kind).click();
  await wait('one native '+kind+' action appends an actual unreviewed task',()=>count('created')===before+1&&records().length===length+1&&field(records().at(-1),'kind')===kind&&field(records().at(-1),'status')==='Awaiting review'&&!reviewed(records().at(-1))&&count('pending')===pending+1&&notification()?.textContent.trim()===String(pending+1));
 };
 const reviewAll=async desired=>{
  const before=count('reviews'),created=count('created'),selections=count('cardSelections'),requests=count('detailRequests');review().click();
  await wait('native title action updates actual task review states',()=>count('reviews')===before+1&&records().every(row=>reviewed(row)===desired&&field(row,'status')===(desired?'Reviewed':'Awaiting review'))&&count('pending')===(desired?0:records().length)&&notification()?.textContent.trim()===String(desired?0:records().length));
  assert(count('created')===created&&count('cardSelections')===selections&&count('detailRequests')===requests,'title review never creates/selects/expands the notification card');
 };
 const details=async desired=>{if(opened()!==desired)toggle().click();await wait('actual controlled preparation details '+desired,()=>opened()===desired&&panel().hidden===!desired)};
 const visiblePanel=async()=>wait('actual mounted details region visible before native field focus',()=>{
  if(!opened()||panel().hidden)return false;
  const box=panel().getBoundingClientRect();return box.height>0&&box.top>=0&&box.bottom<=innerHeight+1;
 });
 const hitTestable=(node,sample=node)=>{
  const box=node?.getBoundingClientRect(),point=sample?.getBoundingClientRect(),frame=get('task-cards-frame')?.getBoundingClientRect();
  const hit=point?document.elementFromPoint(point.left+point.width/2,point.top+point.height/2):null;
  failureGeometry={node:node?.id,active:active()?.id,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,sampleRect:point?{left:point.left,right:point.right,top:point.top,bottom:point.bottom}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className,label:hit.getAttribute('aria-label')}:null};
  return !!box&&box.width>0&&box.height>0&&box.left>=frame.left-1&&box.right<=frame.right+1&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const snapshot=()=>({created:count('created'),pending:count('pending'),reviews:count('reviews'),selections:count('cardSelections'),requests:count('detailRequests'),notification:notification()?.textContent.trim(),selected:backlog()?.getAttribute('data-selected'),opened:opened(),note:note()?.value,toolkit:toolkit()?.checked,variant:get('launch-variant')?.value,passive:checked(get('launch-passive')),disabled:checked(get('launch-disable-actions')),custom:checked(get('launch-custom-content')),held:checked(get('launch-hold')),records:records().map(row=>({id:field(row,'id'),kind:field(row,'kind'),status:field(row,'status'),reviewed:reviewed(row)}))});

 await wait('actual maintenance task gallery and retained native details mounted',()=>host()&&backlog()&&primary()&&review()&&toggle()&&panel()&&note()&&toolkit()&&notification()&&result()&&records().length===3);
 await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(action('Inspection').tagName==='BUTTON'&&action('Inspection').type==='button'&&action('Inspection').getAttribute('aria-label')==='Start inspection for Motor 001'&&action('Service').getAttribute('aria-label')==='Plan service for Motor 001','actual ActionCards retain native button semantics and explicit names');
 assert(action('Inspection').querySelector('svg[role="img"][aria-label="Inspection checklist"]')&&action('Service').querySelector('svg[role="img"][aria-label="Service plan"]')&&!action('Inspection').querySelector('button,input,select,a,textarea')&&!action('Service').querySelector('button,input,select,a,textarea'),'Action icons have explicit names and child presentation contains no nested controls');
 assert(backlog().tagName==='ARTICLE'&&primary().getAttribute('aria-label')==='Select pending maintenance queue'&&!primary().contains(review())&&!primary().contains(toggle())&&!primary().contains(note())&&backlog().querySelector('.ix-card-title-actions')?.contains(review()),'Push root owns independent sibling primary/title/disclosure controls without nested buttons');
 assert(toggle().getAttribute('aria-label')==='Show task preparation details'&&toggle().textContent.includes('Task details')&&panel().getAttribute('role')==='region'&&panel().getAttribute('aria-labelledby')===toggle().id&&panel().contains(note())&&panel().contains(toolkit())&&!opened()&&panel().hidden&&note().value==='Prepare tools for Motor 001'&&!toolkit().checked,'actual details ARIA links and hidden retained preparation content');
 assert(records().map(row=>field(row,'id')+':'+field(row,'kind')).join(',')==='MT-101:Inspection,MT-102:Service,MT-103:Inspection'&&records().every(row=>!reviewed(row)&&field(row,'status')==='Awaiting review')&&count('pending')===3&&notification().textContent.trim()==='3'&&count('created')===0&&count('reviews')===0&&count('cardSelections')===0&&count('detailRequests')===0,'seed task records and visible pending notification match source');
 get('launch-locked').click();assert(get('launch-locked').disabled&&records().length===3&&count('created')===0,'passive Remote release cannot create a task');snapshots.semantics=snapshot();

 for(const [kind,value] of [['Inspection','Enter'],['Service',' ']]){
  await focus(action(kind),'native '+kind+' ActionCard');await wait('actual ActionCard receives pointer hit',()=>hitTestable(action(kind)));hitFacts[kind]=failureGeometry.hit;
  const before=count('created'),event=key(action(kind),value);keyPermissions[kind+value]=!event.defaultPrevented;
  assert(!event.defaultPrevented,'ActionCard '+value+' native default remains available');await wait('untrusted ActionCard key does not fabricate creation',()=>count('created')===before);
  await create(kind);
 }
 assert(get('launch-task-104')&&field(get('launch-task-104'),'kind')==='Inspection'&&get('launch-task-105')&&field(get('launch-task-105'),'kind')==='Service'&&action('Service').getAttribute('aria-pressed')==='true'&&action('Inspection').getAttribute('aria-pressed')==='false'&&count('reviews')===0&&count('cardSelections')===0&&count('detailRequests')===0,'actions create exact local kinds and update only most recent action selection');snapshots.created=snapshot();

 await focus(review(),'native review title action');await wait('title review hit testing reaches actual button',()=>hitTestable(review()));hitFacts.review=failureGeometry.hit;await reviewAll(true);
 assert(review().getAttribute('aria-label')==='Reopen all maintenance tasks','reviewed queue exposes real Reopen title action');await create('Inspection');
 assert(count('pending')===1&&records().slice(0,-1).every(reviewed)&&!reviewed(records().at(-1))&&field(get('launch-task-106'),'kind')==='Inspection','new task raises pending count while reviewed records stay reviewed');
 await reviewAll(true);await focus(review(),'native Reopen title action');const reviewKey=key(review(),'Enter');keyPermissions.reviewEnter=!reviewKey.defaultPrevented;assert(!reviewKey.defaultPrevented,'title action native Enter default remains available');await reviewAll(false);
 assert(review().getAttribute('aria-label')==='Review all pending tasks','actual Reopen restores pending task states and review label');snapshots.reviewed=snapshot();

 await focus(primary(),'native notification primary');await wait('plain title hit testing reaches notification selection surface',()=>hitTestable(primary(),title()));hitFacts.primaryTitle=failureGeometry.hit;
 const beforeSelection=count('cardSelections'),beforeCreated=count('created'),beforeReviews=count('reviews'),primaryKey=key(primary(),' ');keyPermissions.primarySpace=!primaryKey.defaultPrevented;assert(!primaryKey.defaultPrevented,'notification primary native Space default remains available');
 primary().click();await wait('only notification selection toggles from primary click',()=>count('cardSelections')===beforeSelection+1&&primary().getAttribute('aria-pressed')==='true'&&backlog().getAttribute('data-selected')==='true');
 assert(count('created')===beforeCreated&&count('reviews')===beforeReviews&&count('detailRequests')===0&&!opened(),'notification selection neither creates/reviews nor expands details');snapshots.selected=snapshot();

 await details(true);await visiblePanel();const retainedNote=note(),retainedToolkit=toolkit();await focus(note(),'actual preparation note');await setNote('Prepare seals and tools');toolkit().click();await wait('actual local toolkit checkbox updated',()=>toolkit().checked);
 await details(false);await details(true);await visiblePanel();
 assert(note()===retainedNote&&toolkit()===retainedToolkit&&note().value==='Prepare seals and tools'&&toolkit().checked&&count('cardSelections')===beforeSelection+1&&count('created')===beforeCreated&&count('reviews')===beforeReviews,'ordinary disclosure close/open retains actual local preparation nodes/state and independent card actions');snapshots.retention=snapshot();

 await details(false);const options=get('launch-options');if(!options.open)options.querySelector('summary').click();await wait('native Card options details opened',()=>options.open);
 get('launch-hold').click();await wait('host holds controlled details requests',()=>checked(get('launch-hold')));
 const heldBefore=count('detailRequests');toggle().click();await wait('held opening request records without changing actual hidden details',()=>count('detailRequests')===heldBefore+1&&!opened()&&panel().hidden);
 const externalBefore=count('detailRequests');get('launch-toggle-details').click();await wait('external host update opens held details without new request',()=>opened()&&!panel().hidden&&count('detailRequests')===externalBefore);
 toggle().click();await wait('held close request leaves accepted details open',()=>count('detailRequests')===externalBefore+1&&opened());
 const beforeExternalClose=count('detailRequests');get('launch-toggle-details').click();await wait('external close changes controlled panel without inventing request',()=>!opened()&&panel().hidden&&count('detailRequests')===beforeExternalClose);
 get('launch-hold').click();await wait('normal controlled request mode restored',()=>!checked(get('launch-hold')));
 assert(note()===retainedNote&&note().value==='Prepare seals and tools'&&toolkit().checked,'held/external details changes preserve independent local preparation');snapshots.held=snapshot();

 get('launch-custom-content').click();await wait('optional action description content removed without changing action name',()=>!checked(get('launch-custom-content'))&&!get('launch-inspection-description')&&!get('launch-service-description')&&action('Inspection').getAttribute('aria-label')==='Start inspection for Motor 001');
 for(const variant of ['filled','warning','primary']){
  const node=get('launch-variant');node.value=variant;node.dispatchEvent(new Event('change',{bubbles:true}));
  await wait('actual '+variant+' Action/Push/disclosure surfaces share semantic appearance',()=>action('Inspection').classList.contains('ix-card-'+variant)&&action('Service').classList.contains('ix-card-'+variant)&&backlog().classList.contains('ix-card-'+variant)&&backlog().querySelector('.ix-card-accordion')?.classList.contains('ix-card-'+variant));
 }
 get('launch-passive').click();await wait('passive notification removes only primary selection action',()=>checked(get('launch-passive'))&&!primary()&&backlog().getAttribute('data-passive')==='true'&&review()&&toggle());
 await reviewAll(true);await details(true);await visiblePanel();assert(note()===retainedNote&&note().value==='Prepare seals and tools'&&toolkit().checked,'passive mode retains useful title/disclosure controls and local preparation');await details(false);
 get('launch-disable-actions').click();await wait('task actions disabled independently of notification title/details',()=>checked(get('launch-disable-actions'))&&action('Inspection').disabled&&action('Service').disabled&&get('launch-locked').disabled);
 const disabledCreated=count('created'),disabledRows=records().length;action('Inspection').click();action('Service').click();await wait('disabled ActionCards cannot fabricate tasks',()=>count('created')===disabledCreated&&records().length===disabledRows);
 get('launch-passive').click();await wait('notification primary restored after passive mode',()=>!checked(get('launch-passive'))&&primary()&&backlog().getAttribute('data-passive')==='false');
 get('launch-disable-actions').click();get('launch-custom-content').click();await wait('action options restore noninteractive description content',()=>!action('Inspection').disabled&&!action('Service').disabled&&get('launch-inspection-description')?.textContent==='Motor 001 local checklist'&&get('launch-service-description')?.textContent==='Maintenance work order');
 assert(note()===retainedNote&&toolkit()===retainedToolkit&&note().value==='Prepare seals and tools'&&toolkit().checked,'live variant/passive/disable/content options never replace retained preparation scope');snapshots.liveOptions=snapshot();

 const resetButton=get('launch-reset'),externalButton=get('launch-toggle-details'),oldAction=action('Inspection'),oldBacklog=backlog(),oldNote=note();await focus(resetButton,'native Restore task launcher');resetButton.click();
 await wait('Restore launcher resets actual seed records, counters, options and owned local fields',()=>records().length===3&&records().map(row=>field(row,'id')).join(',')==='MT-101,MT-102,MT-103'&&records().every(row=>!reviewed(row)&&field(row,'status')==='Awaiting review')&&count('created')===0&&count('pending')===3&&count('reviews')===0&&count('cardSelections')===0&&count('detailRequests')===0&&notification()?.textContent.trim()==='3'&&!opened()&&panel().hidden&&note()?.value==='Prepare tools for Motor 001'&&!toolkit().checked&&primary()?.getAttribute('aria-pressed')==='false'&&!checked(get('launch-passive'))&&!checked(get('launch-disable-actions'))&&!checked(get('launch-hold'))&&checked(get('launch-custom-content'))&&get('launch-variant').value==='outline');
 assert(!oldAction.isConnected&&!oldBacklog.isConnected&&!oldNote.isConnected&&action('Inspection')!==oldAction&&note()!==oldNote&&get('launch-reset')===resetButton&&get('launch-toggle-details')===externalButton&&get('launch-options')===options&&active()===resetButton,'Restore replaces only owned cards/preparation while retaining toolbar/options and native focus');snapshots.reset=snapshot();

 get('task-cards-narrow').click();await wait('340px task cards frame without horizontal page overflow',()=>get('task-cards-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);
 await focus(action('Inspection'),'narrow native Inspection action');await wait('narrow ActionCard fits and receives actual pointer hit',()=>hitTestable(action('Inspection')));await create('Inspection');
 await focus(review(),'narrow native Review title button');await wait('narrow title action fits and receives actual pointer hit',()=>hitTestable(review()));await reviewAll(true);
 await focus(primary(),'narrow native notification selection');await wait('narrow title text reaches actual native selection surface',()=>hitTestable(primary(),title()));primary().click();await wait('narrow selection updates only selected notification state',()=>count('cardSelections')===1&&primary().getAttribute('aria-pressed')==='true');
 await focus(toggle(),'narrow native preparation disclosure');await wait('narrow disclosure fits and receives actual pointer hit',()=>hitTestable(toggle()));await details(true);await visiblePanel();await focus(note(),'narrow native preparation note');await wait('narrow preparation input fits and receives actual pointer hit',()=>hitTestable(note()));await setNote('Narrow preparation note');toolkit().click();await wait('narrow retained toolkit checkbox usable',()=>toolkit().checked);
 await focus(review(),'expanded narrow title action');await wait('expanded narrow title action remains independently hit-testable',()=>hitTestable(review()));await reviewAll(false);
 const frame=get('task-cards-frame').getBoundingClientRect(),input=note().getBoundingClientRect(),reviewBox=review().getBoundingClientRect();
 layout={frameWidth:frame.width,pushWidth:backlog().getBoundingClientRect().width,actionWidth:action('Inspection').getBoundingClientRect().width,inputWidth:input.width,inputHeight:input.height,titleActionWidth:reviewBox.width,titleActionHeight:reviewBox.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(count('created')===1&&records().length===4&&count('pending')===4&&count('reviews')===2&&count('cardSelections')===1&&count('detailRequests')===1&&opened()&&note().value==='Narrow preparation note'&&toolkit().checked&&layout.pageScroll<=layout.pageClient+1,'narrow native cards/title/details/fields update actual records independently in contained layout');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,hitFacts,
  checks:['actual native Action/Push sibling semantics, accessible names, retained ARIA details and seed task records','Action native Enter/Space permission then single actual-kind task creation','independent Review/Reopen updates real task states and pending count including new arrivals','plain title pointer hits notification primary and selection stays independent','controlled disclosure retains local preparation note/toolkit across close/open','held requests and external accepted details changes preserve local scope','custom action content/live variants/passive/disable modes preserve useful independent controls','Restore resets owned card/preparation scopes/records/options/counts while retaining toolbar focus','340px actual Action/title/primary/disclosure/input hit testing and contained workflow'],
  input:'synthetic form/click/key events, explicit native focus and native page scroll setup; native Enter/Space/Tab defaults are not simulated',
  trustedInput:'separate Chromium native Enter/Space/Tab/typing/pointer interactions and visual checks; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;

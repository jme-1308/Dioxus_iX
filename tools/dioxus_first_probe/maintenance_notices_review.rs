//! Excluded actual-gallery maintenance notices preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/maintenance_notices.rs"]
mod gallery_notices;

#[component]
pub fn MaintenanceNoticesPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_notices::STYLES} }
        IxTheme { class: "maintenance-notices-review",
            h1 { "Maintenance notices" }
            Button { id: "maintenance-notices-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "maintenance-notices-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_notices::MaintenanceNoticesExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_MAINTENANCE_NOTICES_CHECK);
                        if let Err(error) = query.send("maintenancenotices") {
                            eprintln!("Maintenance notices native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native maintenance notices: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native maintenance notices failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Maintenance notices native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.maintenance-notices-review{box-sizing:border-box;min-height:100vh;padding:24px}.maintenance-notices-review>h1{font-size:28px;margin:0 0 24px}.maintenance-notices-review h2{font-size:22px;margin:24px 0 16px}.maintenance-notices-review p{max-width:72ch;line-height:1.5}#maintenance-notices-frame{margin-top:24px}@media(max-width:600px){.maintenance-notices-review{padding:16px}}";

// Excluded actual-gallery replay: DOM clicks and synthetic native input setup.
// Trusted native button Enter/Space and Tab are separate lead browser evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_MAINTENANCE_NOTICES_CHECK: &str = r#"
let snapshots={},layout={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('notices-result');
 const bar=()=>get('notices-bar')?.querySelector('.ix-message-bar'),close=()=>bar()?.querySelector('button.ix-message-close'),review=()=>get('notices-review'),note=()=>get('notices-note'),archive=()=>get('notices-archived-motor');
 const count=name=>Number(result()?.dataset[name]);
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const setNote=async text=>{const node=note();Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual independent preparation note updated',()=>note()===node&&node.value===text)};
 const setChecked=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' explicit native form setup applied',()=>node.checked===value)};
 const setSeverity=async value=>{const node=get('notices-severity');node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait('actual notice severity changed',()=>result().dataset.variant===value&&bar()?.dataset.type===value)};
 const hitTestable=node=>{
  const box=node?.getBoundingClientRect(),frame=get('maintenance-notices-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;
  failureGeometry={node:node?.id||node?.className,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameRect:frame?{left:frame.left,right:frame.right,width:frame.width}:null,innerWidth,innerHeight,scrollTop:document.documentElement.scrollTop,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className}:null};
  return !!box&&box.width>0&&box.height>0&&box.left>=frame.left-1&&box.right<=frame.right+1&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(hit===node||node.contains(hit));
 };
 const snapshot=()=>({reviewed:result()?.dataset.reviewed,archived:result()?.dataset.archived,persistent:result()?.dataset.persistent,requireReview:result()?.dataset.requireReview,variant:result()?.dataset.variant,requests:count('closeRequests'),accepted:count('accepted'),completions:count('completions'),blocked:count('blocked'),reviews:count('reviews'),status:get('notices-status')?.textContent,note:note()?.value,barPresent:!!bar(),archive:archive()?{text:archive().textContent,reviewed:archive().dataset.reviewed,variant:archive().dataset.variant}:null});
 const reset=async()=>{await focus(get('notices-reset'),'Restore maintenance notice');get('notices-reset').click();await wait('Restore actual initial notice/options/counters and private preparation',()=>bar()&&close()&&review()&&!review().disabled&&review().getAttribute('aria-disabled')==='false'&&result().dataset.reviewed==='false'&&result().dataset.archived==='false'&&result().dataset.requireReview==='true'&&result().dataset.persistent==='false'&&result().dataset.variant==='warning'&&result().dataset.german==='false'&&['closeRequests','accepted','completions','blocked','reviews'].every(name=>count(name)===0)&&!archive()&&note()?.value==='Prepare Motor 001 inspection')};
 const dismiss=async(reviewed,variant)=>{const oldBar=bar(),oldClose=close(),before={requests:count('closeRequests'),accepted:count('accepted'),completions:count('completions')};oldClose.click();await wait('completed dismissal removes actual bar and creates real archive record',()=>!bar()&&!oldBar.isConnected&&!oldClose.isConnected&&count('closeRequests')===before.requests+1&&count('accepted')===before.accepted+1&&count('completions')===before.completions+1&&archive()?.dataset.reviewed===String(reviewed)&&archive()?.dataset.variant===variant&&archive().querySelector('[data-field=asset]')?.textContent==='Motor 001'&&archive().querySelector('[data-field=review]')?.textContent===(reviewed?'Reviewed':'Unreviewed')&&archive().querySelector('[data-field=severity]')?.textContent===variant&&archive().querySelector('p')?.textContent===(reviewed?'Bearing inspection review recorded locally.':'Bearing inspection awaits local review.')&&result().dataset.archived==='true'&&get('notices-status').textContent==='Archived');return oldBar};

 await wait('actual local notice/review/preparation mounted',()=>result()&&bar()&&close()&&review()&&note());await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(bar().getAttribute('role')==='alert'&&bar().dataset.type==='warning'&&bar().querySelector('.ix-message-bar-content')?.textContent.includes('Motor 001')&&close().tagName==='BUTTON'&&close().type==='button'&&close().getAttribute('aria-label')==='Close message'&&!close().disabled&&close().tabIndex===0&&result().dataset.reviewed==='false'&&result().dataset.requireReview==='true'&&['closeRequests','accepted','completions','blocked','reviews'].every(name=>count(name)===0)&&!archive()&&note().value==='Prepare Motor 001 inspection','actual warning content/native close/initial record and independent preparation');
 const retainedNote=note(),vetoBar=bar();await focus(note(),'native preparation');await setNote('Prepare bearing tools');await focus(close(),'native notice close');await wait('actual close button receives pointer hit',()=>hitTestable(close()));hitFacts.close=failureGeometry.hit;close().click();await wait('review veto keeps actual notice without accepted/completed/archive work',()=>bar()===vetoBar&&vetoBar.isConnected&&count('closeRequests')===1&&count('blocked')===1&&count('accepted')===0&&count('completions')===0&&!archive()&&result().dataset.archived==='false'&&get('notices-status').textContent==='Awaiting review');snapshots.veto=snapshot();

 const beforeReview=bar().textContent;await focus(review(),'native content review action');await wait('actual content review action receives pointer hit',()=>hitTestable(review()));hitFacts.review=failureGeometry.hit;review().click();await wait('content action changes actual reviewed record and message content',()=>result().dataset.reviewed==='true'&&count('reviews')===1&&review()?.getAttribute('aria-disabled')==='true'&&!review().disabled&&get('notices-status').textContent==='Reviewed'&&bar().textContent!==beforeReview&&count('accepted')===0&&count('completions')===0);review().click();await wait('aria-disabled reviewed action remains focusable and rejects duplicate review',()=>count('reviews')===1&&get('notices-detail').textContent==='Bearing inspection review recorded locally.'&&count('accepted')===0&&count('completions')===0);await dismiss(true,'warning');
 assert(note()===retainedNote&&note().value==='Prepare bearing tools'&&count('blocked')===1&&count('reviews')===1&&get('notices-archive').contains(archive()),'completed reviewed record belongs to actual archive while preparation persists');snapshots.reviewedArchive=snapshot();

 const options=get('notices-options');if(!options.open)options.querySelector('summary').click();await wait('native notice options expanded',()=>options.open);await reset();const persistentNote=note();await setNote('Retain preparation through persistent options');await setChecked('notices-persistent',true);await wait('persistent notice has no dismissal affordance',()=>result().dataset.persistent==='true'&&bar()&&!close());
 review().click();await wait('persistent content action remains functional without dismissal callbacks',()=>result().dataset.reviewed==='true'&&count('reviews')===1&&['closeRequests','accepted','completions','blocked'].every(name=>count(name)===0)&&!archive());await setSeverity('alarm');await setChecked('notices-german',true);await wait('persistent localized live options do not emit dismissal',()=>result().dataset.german==='true'&&result().dataset.persistent==='true'&&!close()&&bar().dataset.type==='alarm'&&['closeRequests','accepted','completions'].every(name=>count(name)===0));
 await setChecked('notices-persistent',false);await wait('live closability restores localized native close button',()=>result().dataset.persistent==='false'&&close()?.getAttribute('aria-label')==='Meldung schließen'&&close().tagName==='BUTTON'&&close().type==='button'&&close().tabIndex===0);await dismiss(true,'alarm');assert(note()===persistentNote&&note().value==='Retain preparation through persistent options'&&count('completions')===1&&count('reviews')===1,'live persistent/variant/localized changes retain independent preparation and real review archive');snapshots.persistentOptions=snapshot();

 const resetButton=get('notices-reset'),oldNote=note();await reset();assert(!oldNote.isConnected&&note()!==oldNote&&get('notices-reset')===resetButton&&get('notices-options')===options&&active()===resetButton&&get('notices-status').textContent==='Awaiting review'&&close().getAttribute('aria-label')==='Close message','Restore remounts only owned notice/preparation and preserves toolbar/options/native focus');snapshots.reset=snapshot();

 get('maintenance-notices-narrow').click();await wait('actual340px frame and page fit',()=>get('maintenance-notices-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);await setChecked('notices-german',true);await setSeverity('critical');await setChecked('notices-require-review',false);await wait('narrow unreviewed close policy and localized affordance applied',()=>result().dataset.requireReview==='false'&&close()?.getAttribute('aria-label')==='Meldung schließen');
 await focus(note(),'narrow actual preparation');await wait('narrow preparation fits and receives pointer hit',()=>hitTestable(note()));hitFacts.narrowNote=failureGeometry.hit;await setNote('Narrow bearing inspection');for(const [label,node] of [['review',review()],['close',close()]]){await focus(node,'narrow native '+label);await wait('narrow '+label+' action fits and receives pointer hit',()=>hitTestable(node));hitFacts['narrow '+label]=failureGeometry.hit}
 const frame=get('maintenance-notices-frame').getBoundingClientRect(),barBox=bar().getBoundingClientRect(),input=note().getBoundingClientRect();layout={frameWidth:frame.width,barWidth:barBox.width,barHeight:barBox.height,inputWidth:input.width,inputHeight:input.height,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(layout.barWidth<=341&&layout.barHeight>0&&layout.inputHeight>=31&&layout.pageScroll<=layout.pageClient+1,'narrow actual bar/content/native controls contained');const narrowNote=note();await dismiss(false,'critical');assert(note()===narrowNote&&note().value==='Narrow bearing inspection'&&count('reviews')===0&&count('blocked')===0&&archive().textContent.includes('Motor 001')&&archive().textContent.includes('Unreviewed'),'narrow accepted close archives actual unreviewed critical notice and retains preparation');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,hitFacts,checks:['review veto retains actual alert with no accepted/completed/archive work','content action updates reviewed notice; completed dismissal removes actual bar and creates reviewed archive record','persistent removes close affordance while content action/live severity/localized native close remain useful','scoped Restore resets notice/archive/options/counters and owned preparation while preserving toolbar focus','340px real review/close/note hits, contained bar and actual unreviewed archive workflow'],input:'actual DOM click activation, synthetic input/change events and explicit native focus/normal page scroll setup; no synthetic keyboard activation',trustedInput:'separate Chromium trusted Enter/Space/Tab/pointer/typing and paired visual evidence; native button default admission uses ordinary type=button/tabindex semantics',completion:'Dioxus rendered dismissal notification; actual archive record and disconnected message content are checked without timers or animation chronology',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;

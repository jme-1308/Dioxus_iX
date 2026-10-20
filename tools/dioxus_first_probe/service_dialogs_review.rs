//! Excluded actual-gallery service dialogs preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/service_dialogs.rs"]
mod gallery_dialogs;

#[component]
pub fn ServiceDialogsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_dialogs::STYLES} }
        IxTheme { class: "service-dialogs-review",
            h1 { "Service dialogs" }
            Button { id: "service-dialogs-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "service-dialogs-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_dialogs::ServiceDialogsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_SERVICE_DIALOGS_CHECK);
                        if let Err(error) = query.send("servicedialogs") {
                            eprintln!("Service dialogs native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native service dialogs: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native service dialogs failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Service dialogs native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.service-dialogs-review{box-sizing:border-box;min-height:100vh;padding:24px}.service-dialogs-review>h1{font-size:28px;margin:0 0 24px}.service-dialogs-review .service-dialogs-example>h2{font-size:22px;margin:24px 0 16px}#service-dialogs-frame{margin-top:24px}@media(max-width:600px){.service-dialogs-review{padding:16px}}";

// Excluded actual-gallery replay: synthetic form events, clicks and explicit focus.
// Trusted native input/default keyboard activation are separate browser evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SERVICE_DIALOGS_CHECK: &str = r#"
let snapshots={},layout={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('service-result');
 const button=id=>get(id)?.querySelector('button'),createTrigger=()=>button('service-create-trigger'),confirmTrigger=()=>button('service-cancel-trigger');
 const create=()=>get('service-create-dialog')?.querySelector('dialog'),confirm=()=>get('service-confirm-dialog')?.querySelector('dialog');
 const createOpen=()=>create()?.open&&result()?.dataset.createOpen==='true',confirmOpen=()=>confirm()?.open&&result()?.dataset.confirmOpen==='true';
 const name=()=>get('service-name'),details=()=>get('service-details'),priority=()=>get('service-priority'),note=()=>get('service-preparation');
 const field=name=>get('service-request')?.querySelector('[data-field="'+name+'"]')?.textContent.trim(),count=name=>Number(result()?.dataset[name]);
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.textContent||active()?.tagName))};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' actual native focus',()=>active()===node)};
 const input=async(node,text,label)=>{assert(node,label+' exists');const prototype=node instanceof HTMLTextAreaElement?HTMLTextAreaElement.prototype:HTMLInputElement.prototype;Object.getOwnPropertyDescriptor(prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' actual native draft applied',()=>node.isConnected&&node.value===text)};
 const select=async(id,value)=>{const node=get(id);node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' native selection applied',()=>node.value===value)};
 const checked=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' native option applied',()=>node.checked===value)};
 const closeHeader=dialog=>dialog.querySelector('.ix-dialog-close');
 const icon=()=>confirm()?.querySelector('.ix-modal-header-icon svg');
 const hitTestable=(node,insideFrame=false)=>{const box=node?.getBoundingClientRect(),frame=get('service-dialogs-frame')?.getBoundingClientRect(),hit=box?document.elementFromPoint(box.left+box.width/2,box.top+box.height/2):null;failureGeometry={node:node?.id||node?.textContent,nodeRect:box?{left:box.left,right:box.right,top:box.top,bottom:box.bottom,width:box.width,height:box.height}:null,frameWidth:frame?.width,innerWidth,innerHeight,hit:hit?{id:hit.id,tag:hit.tagName,className:hit.className}:null};return !!box&&box.width>0&&box.height>0&&box.left>=0&&box.right<=innerWidth+1&&box.top>=0&&box.bottom<=innerHeight+1&&(!insideFrame||(box.left>=frame.left-1&&box.right<=frame.right+1))&&(hit===node||node.contains(hit))};
 const openCreate=async()=>{await focus(createTrigger(),'Create service request opener');createTrigger().click();await wait('actual native create dialog opened with name autofocus',()=>createOpen()&&create().matches(':modal')&&active()===name())};
 const openConfirm=async()=>{await focus(confirmTrigger(),'Review service request opener');confirmTrigger().click();await wait('actual MessageDialog opened in native top layer',()=>confirmOpen()&&confirm().matches(':modal'))};
 const returnedCreate=async()=>wait('create dialog closed with actual opener focus restored',()=>!createOpen()&&active()===createTrigger());
 const returnedConfirm=async()=>wait('message dialog closed with actual opener focus restored',()=>!confirmOpen()&&active()===confirmTrigger());
 const save=async(savedName,savedDetails,savedPriority)=>{const before=count('saves');get('service-save').click();await returnedCreate();await wait('Save commits actual service request fields once',()=>count('saves')===before+1&&result().dataset.saved==='true'&&field('name')===savedName&&field('details')===savedDetails&&field('priority')===savedPriority&&field('status')==='Open'&&result().dataset.lastResult==='save-service')};
 const snapshot=()=>({saved:result()?.dataset.saved,status:result()?.dataset.status,createOpen:createOpen(),confirmOpen:confirmOpen(),pending:result()?.dataset.pendingDiscard,saves:count('saves'),cancellations:count('cancellations'),lastResult:result()?.dataset.lastResult,record:{name:field('name'),details:field('details'),priority:field('priority'),status:field('status')},note:note()?.value});

 await wait('actual service workspace/native dialogs/form/preparation mounted',()=>result()&&createTrigger()&&confirmTrigger()&&create()&&confirm()&&name()&&details()&&priority()&&note());await wait('native WebView ready for focus',()=>document.hasFocus());
 assert(result().dataset.saved==='false'&&!createOpen()&&!confirmOpen()&&count('saves')===0&&count('cancellations')===0&&confirmTrigger().disabled&&note().value==='Prepare Motor 001 maintenance'&&get('service-request').textContent.includes('No service request'),'initial actual empty service record and independent preparation');
 assert(create().tagName==='DIALOG'&&create().getAttribute('aria-label')==='Create service request'&&confirm().getAttribute('aria-label')==='Cancel service request?'&&get('service-request-form').hasAttribute('novalidate')&&get('service-save').type==='submit'&&get('service-save').getAttribute('form')==='service-request-form'&&!get('service-request-form').contains(get('service-save')),'actual native dialogs and external native form submit semantics');
 const retainedNote=note();await focus(note(),'outside service preparation');await input(note(),'Prepare bearing service tools','outside preparation');snapshots.initial=snapshot();

 await openCreate();await input(name(),'   ','required request name');get('service-save').click();await wait('Rust trimmed validation keeps actual form open without saving',()=>createOpen()&&get('service-validation')?.getAttribute('role')==='alert'&&count('saves')===0&&result().dataset.saved==='false');
 await input(name(),'Motor 001 bearing service','request name');await input(details(),'Inspect seal and bearing housing','request details');await select('service-priority','urgent');await save('Motor 001 bearing service','Inspect seal and bearing housing','urgent');assert(note()===retainedNote&&note().value==='Prepare bearing service tools'&&!confirmTrigger().disabled,'saved request enables actual review and retains outside preparation');snapshots.validatedSave=snapshot();

 await openCreate();await input(name(),'Uncommitted service draft','dirty request name');await input(details(),'Keep these uncommitted details','dirty details');closeHeader(create()).click();await wait('dirty dismissal is held for authored Keep/Discard decision',()=>createOpen()&&result().dataset.pendingDiscard==='true'&&get('service-discard-decision')&&get('service-keep-editing')&&get('service-discard'));
 await focus(get('service-keep-editing'),'actual Keep editing decision');get('service-keep-editing').click();await wait('Keep editing denies dismissal, restores field focus and retains actual drafts',()=>createOpen()&&result().dataset.pendingDiscard==='false'&&active()===name()&&name().value==='Uncommitted service draft'&&details().value==='Keep these uncommitted details'&&count('saves')===1&&field('name')==='Motor 001 bearing service');button('service-cancel-draft').click();await wait('second dirty dismissal requests an explicit decision',()=>createOpen()&&result().dataset.pendingDiscard==='true');get('service-discard').click();await returnedCreate();await wait('Discard clears owned drafts without changing saved request',()=>result().dataset.pendingDiscard==='false'&&name().value==='Motor 001 inspection'&&details().value==='Inspect bearing housing'&&field('name')==='Motor 001 bearing service'&&count('saves')===1);assert(note()===retainedNote&&note().value==='Prepare bearing service tools','dirty decision retains independent outside note');snapshots.dirtyKeepDiscard=snapshot();

 await openConfirm();const description=get(confirm().getAttribute('aria-describedby'));assert(description?.classList.contains('ix-message-dialog-message')&&description.textContent.includes('Motor 001')&&icon()&&Number(icon().getAttribute('width'))===24,'actual MessageDialog description relationship and preserved header icon');const questionIcon=icon().innerHTML;closeHeader(confirm()).click();await returnedConfirm();await wait('header dismissal reports result without cancelling actual request',()=>field('status')==='Open'&&count('cancellations')===0&&result().dataset.lastResult==='request-kept');
 await openConfirm();button('service-keep-request').click();await returnedConfirm();await wait('explicit Keep request dismisses without actual model mutation',()=>field('status')==='Open'&&count('cancellations')===0&&result().dataset.lastResult==='request-kept');snapshots.messageDismissal=snapshot();

 const options=get('service-options');options.querySelector('summary').click();await wait('native dialog options opened',()=>options.open);const variants=[...get('service-message-variant').options].map(node=>node.value);assert(variants.length===5&&['info','warning','error','success','question'].every(value=>variants.includes(value)),'five useful message intents remain available without duplicating dialog machinery');await select('service-message-variant','info');await checked('service-german',true);await checked('service-hide-close',true);await checked('service-single-action',true);await openConfirm();assert(!closeHeader(confirm())&&confirm().querySelector('.ix-modal-footer').querySelectorAll('button').length===1&&button('service-keep-request'),'hidden header close and single explicit Keep action forward to actual native message composition');button('service-keep-request').click();await returnedConfirm();assert(field('status')==='Open'&&count('cancellations')===0,'single-action message cannot silently cancel saved request');
 await checked('service-hide-close',false);await checked('service-single-action',false);await select('service-message-variant','error');await openConfirm();assert(closeHeader(confirm())?.getAttribute('aria-label')==='Modal schließen'&&icon()?.innerHTML!==questionIcon&&button('service-confirm-cancel'),'localized close and changed severity glyph render with explicit cancellation action');button('service-confirm-cancel').click();await returnedConfirm();await wait('only accepted cancel-service result changes actual saved status',()=>field('status')==='Cancelled'&&count('cancellations')===1&&count('saves')===1&&result().dataset.lastResult==='cancel-service');assert(note()===retainedNote&&note().value==='Prepare bearing service tools','message options and real cancellation preserve outside note');snapshots.optionsCancellation=snapshot();

 const reset=get('service-reset'),oldNote=note(),oldCreate=create();await focus(reset,'Restore service dialogs');reset.click();await wait('Restore remounts owned dialogs/drafts/note and clears actual record/results',()=>create()&&create()!==oldCreate&&!createOpen()&&!confirmOpen()&&result().dataset.saved==='false'&&count('saves')===0&&count('cancellations')===0&&confirmTrigger().disabled&&note()?.value==='Prepare Motor 001 maintenance'&&get('service-message-variant').value==='question'&&!get('service-german').checked&&!get('service-hide-close').checked&&!get('service-single-action').checked);assert(!oldCreate.isConnected&&!oldNote.isConnected&&note()!==oldNote&&get('service-reset')===reset&&get('service-options')===options&&active()===reset,'scoped Restore replaces owned workspace/preparation and preserves outside toolbar/options focus');snapshots.reset=snapshot();

 get('service-dialogs-narrow').click();await wait('340px outside service frame contains local controls without page overflow',()=>get('service-dialogs-frame').getBoundingClientRect().width<=341&&document.documentElement.scrollWidth<=document.documentElement.clientWidth+1);await focus(note(),'narrow outside preparation');await wait('340px outside preparation fits and receives actual pointer hit',()=>hitTestable(note(),true));hitFacts.narrowNote=failureGeometry.hit;await input(note(),'Narrow service preparation','narrow preparation');await focus(createTrigger(),'narrow outside create opener');await wait('340px actual opener fits and receives pointer hit',()=>hitTestable(createTrigger(),true));await openCreate();
 for(const [label,node] of [['name',name()],['details',details()],['priority',priority()],['Save',get('service-save')]]){await focus(node,'actual native modal '+label);await wait('native modal '+label+' fits its real viewport and receives pointer hit',()=>hitTestable(node));hitFacts['modal '+label]=failureGeometry.hit}
 await input(name(),'Motor 001 narrow service','narrow name');await input(details(),'Inspect bearing under planned maintenance','narrow details');await select('service-priority','normal');const modalBox=create().getBoundingClientRect(),modalContentHeight=create().querySelector('.ix-modal-content').getBoundingClientRect().height,modalPanelHeight=create().querySelector('.ix-modal-panel').getBoundingClientRect().height;assert(modalContentHeight>0&&modalPanelHeight>0&&modalBox.left>=0&&modalBox.right<=innerWidth+1&&modalBox.top>=0&&modalBox.bottom<=innerHeight+1,'native top-layer form fits its actual desktop viewport');await save('Motor 001 narrow service','Inspect bearing under planned maintenance','normal');await openConfirm();await focus(button('service-keep-request'),'native message Keep action');await wait('native message action fits actual viewport and receives pointer hit',()=>hitTestable(button('service-keep-request')));button('service-keep-request').click();await returnedConfirm();const frame=get('service-dialogs-frame').getBoundingClientRect();layout={outsideFrameWidth:frame.width,actualNativeViewport:{width:innerWidth,height:innerHeight},formModal:{width:modalBox.width,height:modalBox.height,contentHeight:modalContentHeight,panelHeight:modalPanelHeight,left:modalBox.left,right:modalBox.right},pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};assert(field('name')==='Motor 001 narrow service'&&field('status')==='Open'&&count('saves')===1&&count('cancellations')===0&&note().value==='Narrow service preparation'&&layout.pageScroll<=layout.pageClient+1,'narrow outside frame/native-modal workflow changes real request and retains preparation');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,hitFacts,checks:['actual empty local request/native dialog/external form submission semantics and stable preparation','trim-required form validation then Save commits actual name/details/priority and returns opener focus','dirty dismiss Keep/Discard controls preserve/discard draft without overwriting actual saved request','MessageDialog description/native header plus header/Keep dismissal leave actual request unchanged','five intent options/localized close/hideclose/singleaction and explicit accepted cancellation update real status','scoped Restore resets owned workspace/record/drafts while preserving outside toolbar/options focus','340px outside control hits and real desktop-viewport modal fields/actions/native focus/save workflow'],input:'actual DOM click activation, synthetic input/change events and explicit native focus/normal page scroll setup; native keyboard defaults are not simulated',viewportBoundary:'340px outside gallery frame does not resize native top-layer viewport; actual modal bounds are checked against reported desktop viewport. Lead Chromium360px viewport checks narrow modal layout.',trustedInput:'separate Chromium trusted Enter/Space/Tab/Escape/typing/pointer and paired visual evidence; no physical-device certification',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,hitFacts,failureGeometry,documentFocusedNow:document.hasFocus()}}
"#;

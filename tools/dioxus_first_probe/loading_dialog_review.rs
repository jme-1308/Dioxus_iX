//! Excluded deterministic LoadingDialog adapter review; no simulated gallery job.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn LoadingDraft() -> Element {
    let mut draft = use_signal(|| String::from("Prepare native loading review"));
    rsx! { Input { id:"loading-probe-draft", label:"Retained child draft", value:draft(),
        onvaluechange:move |value|draft.set(value),
    } }
}

#[component]
fn LoadingCase(onreset: EventHandler<()>, generation: usize) -> Element {
    let modal = use_modal();
    let mut message = use_signal(|| String::from("Preparing native inspection…"));
    let mut custom = use_signal(|| false);
    let mut pending = use_signal(|| None::<ModalDismissRequest>);
    let mut requests = use_signal(|| 0_usize);
    let mut results = use_signal(Vec::<String>::new);
    let mut changes = use_signal(Vec::<bool>::new);
    let dismiss = use_callback(move |request: ModalDismissRequest| {
        requests += 1;
        pending.set(Some(request));
    });
    rsx! {
        div { id:"loading-probe-owner", "data-generation":generation,
            div { id:"loading-probe-open", ModalTrigger { modal, "Open guarded loading" } }
            LoadingDialog { modal, message:message(), label:"Native loading inspection", size:ModalSize::Sm,
                ondismissrequest:custom().then_some(dismiss),
                onresult:move |result|results.write().push(format!("{result:?}")),
                onopenchange:move |value| { changes.write().push(value); if !value { pending.set(None); } },
                LoadingDraft {}
                div { class:"loading-probe-actions",
                    Button { id:"loading-probe-update", button_type:ButtonType::Button,
                        onclick:move |_|message.set("Native inspection in progress…".into()), "Update message" }
                    Button { id:"loading-probe-finish", button_type:ButtonType::Button,
                        onclick:move |_|modal.close("native finished"), "Finish" }
                    Button { id:"loading-probe-custom", button_type:ButtonType::Button,
                        onclick:move |_|custom.toggle(), if custom(){"Use default guard"}else{"Use custom guard"} }
                    Button { id:"loading-probe-reset", button_type:ButtonType::Button,
                        onclick:move |_|onreset.call(()), "Replace loading session" }
                    if pending().is_some() {
                        Button { id:"loading-probe-deny", button_type:ButtonType::Button,
                            onclick:move |_| { let request=pending.write().take();if let Some(request)=request { request.deny(); } }, "Keep working" }
                        Button { id:"loading-probe-allow", button_type:ButtonType::Button,
                            onclick:move |_| { let request=pending.write().take();if let Some(request)=request { request.allow(); } }, "Allow dismissal" }
                    }
                }
            }
            output { id:"loading-probe-state", "data-open":modal.is_open(), "data-custom":custom(),
                "data-pending":pending().is_some(), "data-requests":requests(), "data-results":results().len(),
                "data-message":message(), "data-result":format!("{:?}",modal.result()), "data-changes":format!("{:?}",changes()),
                "Results: {results():?}"
            }
        }
    }
}

#[component]
pub fn LoadingDialogPreview() -> Element {
    let mut generation = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"loading-dialog-review",
            h1 { "Guarded loading dialog" }
            p { "Deterministic adapter controls exercise reactive content and explicit completion without a fabricated asynchronous job." }
            Button { id:"loading-probe-outside", button_type:ButtonType::Button, "Independent background control" }
            for owner in [generation()] { LoadingCase { key:"{owner}", generation:owner, onreset:move |_|generation+=1 } }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                { let window=window.clone();spawn(async move {
                    let query=crate::dom_query::eval(NATIVE_LOADING_CHECK);let _=query.send("loadingdialog");
                    match query.join::<serde_json::Value>().await {
                        Ok(value)if value["ok"]==true=>{println!("Dioxus native LoadingDialog: {value}");window.close();}
                        value=>{eprintln!("Dioxus native LoadingDialog failed: {value:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("LoadingDialog native replay timed out");std::process::exit(1);}); }
            } }
        }
    }
}
const STYLES: &str = "body{margin:0}.loading-dialog-review{box-sizing:border-box;min-height:100vh;padding:24px}.loading-dialog-review h1{font-size:28px;line-height:40px;margin:0 0 16px}.loading-dialog-review p{max-width:72ch;line-height:1.5}.loading-probe-actions{display:flex;flex-wrap:wrap;gap:8px;margin-top:20px}@media(max-width:600px){.loading-dialog-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_LOADING_CHECK: &str = r#"
let snapshots={},geometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById('loading-probe-'+id),opener=()=>get('open')?.querySelector('button'),panel=()=>document.getElementById(opener()?.getAttribute('aria-controls')),state=()=>get('state'),active=()=>document.activeElement;
 const assert=(condition,label)=>{if(!condition)throw Error(label)},wait=async(label,predicate)=>{for(let i=0;i<180;i++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; state='+state()?.outerHTML+'; focus='+active()?.id)};
 const open=async()=>{opener().focus();opener().click();await wait('actual LoadingDialog native modal open',()=>panel()?.open&&state()?.dataset.open==='true'&&get('draft'))};
 const message=()=>document.getElementById(panel().id+'-loading-message');
 const capture=()=>({...state()?.dataset,message:message()?.textContent,draft:get('draft')?.value,focus:active()?.id});
 const rect=node=>{const r=node.getBoundingClientRect();return{left:r.left,top:r.top,right:r.right,bottom:r.bottom,width:r.width,height:r.height}};
 await wait('actual LoadingDialog invoker mounted',()=>opener()&&state());await open();const first=panel(),draft=get('draft');assert(first.getAttribute('aria-describedby')===message().id&&message().textContent==='Preparing native inspection…','reactive loading message supplies actual modal accessible description');assert(!!first.querySelector('.ix-modal-loading'),'adapter composes existing ModalLoading presentation');get('outside').focus();assert(first.contains(active()),'native modal makes independent background focus inert');
 draft.value='Retain native child draft';draft.dispatchEvent(new Event('input',{bubbles:true,composed:true}));get('update').click();await wait('reactive message updates actual described loading row',()=>message().textContent==='Native inspection in progress…');assert(panel()===first&&get('draft')===draft&&draft.value==='Retain native child draft','message update preserves actual mounted child and private draft');snapshots.reactive=capture();geometry.open={dialog:rect(panel()),loading:rect(first.querySelector('.ix-modal-loading')),viewport:{width:innerWidth,height:innerHeight}};
 first.dispatchEvent(new Event('cancel',{bubbles:false,cancelable:true}));await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));assert(panel().open&&state().dataset.results==='0'&&state().dataset.pending==='false','default guard denies native Escape request');first.dispatchEvent(new MouseEvent('mousedown',{bubbles:true,cancelable:true,clientX:0,clientY:0}));first.dispatchEvent(new MouseEvent('click',{bubbles:true,cancelable:true,clientX:0,clientY:0}));assert(panel().open&&state().dataset.results==='0','default native backdrop click cannot dismiss loading dialog');get('finish').click();await wait('explicit Finish bypasses guard and restores current invoker',()=>!panel().open&&state().dataset.result.includes('Closed("native finished")')&&active()===opener());snapshots.finished=capture();
 await open();get('custom').click();await wait('optional custom typed guard active',()=>state().dataset.custom==='true');panel().dispatchEvent(new Event('cancel',{bubbles:false,cancelable:true}));await wait('custom typed guard retains native dismissal request',()=>state().dataset.pending==='true'&&state().dataset.requests==='1'&&get('deny'));get('deny').click();await wait('custom denial keeps current loading surface',()=>state().dataset.pending==='false'&&panel().open);panel().dispatchEvent(new Event('cancel',{bubbles:false,cancelable:true}));await wait('second typed request available',()=>state().dataset.pending==='true'&&state().dataset.requests==='2');get('allow').click();await wait('custom typed allow dismisses once and returns invoker',()=>!panel().open&&state().dataset.results==='2'&&state().dataset.result.includes('Dismissed(Escape)')&&active()===opener());snapshots.customGuard=capture();
 await open();const oldPanel=panel(),oldDraft=get('draft'),generation=Number(get('owner').dataset.generation);get('reset').click();await wait('keyed cleanup retires native loading owner',()=>Number(get('owner')?.dataset.generation)>generation&&!oldPanel.isConnected&&!oldDraft.isConnected&&!document.querySelector('dialog:modal'));get('outside').focus();await wait('cleanup leaves independent destination usable',()=>active()===get('outside'));await open();assert(panel()!==oldPanel&&get('draft')!==oldDraft&&get('draft').value==='Prepare native loading review'&&message().textContent==='Preparing native inspection…','fresh owner mounts current default message and child');get('finish').click();await wait('fresh LoadingDialog completion returns fresh invoker',()=>!panel().open&&active()===opener()&&state().dataset.results==='1');snapshots.fresh=capture();
 return{ok:true,snapshots,geometry,checks:['actual Modal/ModalLoading composition has reactive accessible message and retains mounted private child draft; native background is inert','default Escape/backdrop dismissal remain guarded while explicit Finish delivers payload and restores current invoker','optional typed dismissal guard can retain, deny and allow native Escape request with one result','keyed cleanup retires old open surface and child; fresh owner opens/completes through current invoker'],input:'Excluded deterministic adapter controls use actual Rust state, native modal visibility and native focus. Synthetic cancel/click/input dispatch checks handlers and native form-independent state; trusted physical Escape/backdrop and asynchronous gallery file read are separate Chromium evidence.',renderer:navigator.userAgent};
} catch(error){return{ok:false,error:String(error),snapshots,geometry,state:document.getElementById('loading-probe-state')?.outerHTML,focus:document.activeElement?.id}}
"#;

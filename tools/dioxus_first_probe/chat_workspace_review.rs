//! Excluded actual-gallery chat workspace preview and bounded native DOM replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/chat_workspace.rs"]
mod gallery_workspace;

#[component]
pub fn ChatWorkspacePreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_workspace::STYLES} }
        IxTheme { class: "chat-workspace-review",
            h1 { "Chat workspace" }
            Button { id: "chat-workspace-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "chat-workspace-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_workspace::ChatWorkspaceExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CHAT_WORKSPACE_CHECK);
                        if let Err(error) = query.send("chatworkspace") {
                            eprintln!("Chat workspace native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native chat workspace: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native chat workspace failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Chat workspace native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.chat-workspace-review{box-sizing:border-box;min-height:100vh;padding:24px}.chat-workspace-review>h1{font-size:28px;margin:0 0 24px}.chat-workspace-review h2{font-size:22px;margin:24px 0 16px}.chat-workspace-review p{max-width:72ch;line-height:1.5}#chat-workspace-frame{margin-top:24px}@media(max-width:600px){.chat-workspace-review{padding:16px}}";

// Excluded native replay; trusted browser typing and key defaults remain separate lead evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CHAT_WORKSPACE_CHECK: &str = r#"
let snapshots={},layout={},hitFacts={},failureGeometry={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement,result=()=>get('chat-result'),prompt=()=>get('chat-prompt'),form=()=>get('chat-message-form'),note=()=>get('chat-preparation');
 const host=()=>prompt()?.closest('.ix-chat-input'),send=()=>host()?.querySelector('.ix-chat-input-end button'),count=name=>Number(result()?.dataset[name]);
 const record=thread=>get('chat-thread-record-'+thread),draft=thread=>record(thread)?.dataset.draft,last=thread=>record(thread)?.querySelector('[data-field="last"]')?.textContent;
 const messages=thread=>[...get('chat-transcript')?.querySelectorAll('[data-thread="'+thread+'"][data-message-id]')??[]],messageTexts=thread=>messages(thread).map(node=>node.querySelector('.ix-chat-user-bubble')?.textContent);
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{let stable=0;for(let attempt=0;attempt<150;attempt++){stable=predicate()?stable+1:0;if(stable===3)return;await hold(20)}throw Error(label+'; state='+JSON.stringify(result()?.dataset)+'; focus='+(active()?.id||active()?.tagName)+'; prompt='+prompt()?.value)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(label+' native focus',()=>active()===node)};
 const input=async(node,text,label)=>{const proto=node instanceof HTMLTextAreaElement?HTMLTextAreaElement:HTMLInputElement;Object.getOwnPropertyDescriptor(proto.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait(label+' controlled input',()=>node.isConnected&&node.value===text)};
 const key=(options={})=>{const event=new KeyboardEvent('keydown',{key:'Enter',code:'Enter',bubbles:true,cancelable:true,...options});prompt().dispatchEvent(event);return event};
 const thread=async(value,expected)=>{const select=get('chat-thread');await focus(select,'native equipment selector');select.value=value;select.dispatchEvent(new Event('change',{bubbles:true}));await wait(value+' retained draft selected',()=>result()?.dataset.thread===value&&prompt()?.value===expected);assert(active()===select,'thread switch does not steal native selector focus')};
 const option=async(id,value)=>{const node=get(id);node.checked=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' applied',()=>result()?.dataset[id.slice(5)]===String(value))};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height,right:r.right,bottom:r.bottom}};
 const geometry=node=>{const r=node?.getBoundingClientRect(),hit=r&&document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return {node:rect(node),hit:hit?.id||hit?.className||hit?.tagName,frame:rect(get('chat-workspace-frame')),viewport:{width:innerWidth,height:innerHeight},document:document.documentElement.scrollWidth}};
 const hit=async(node,label)=>{await focus(node,label);await wait(label+' contained native hit',()=>{failureGeometry=geometry(node);const r=node.getBoundingClientRect(),point=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return r.width>0&&r.height>0&&r.x>=0&&r.right<=innerWidth+1&&r.y>=0&&r.bottom<=innerHeight+1&&(point===node||node.contains(point))});hitFacts[label]=geometry(node)};
 const snapshot=()=>({state:{...result().dataset},prompt:prompt().value,motorDraft:draft('motor'),pumpDraft:draft('pump'),motorCount:record('motor')?.dataset.count,pumpCount:record('pump')?.dataset.count,focus:active()?.id});
 const sent=async(text,via)=>{await input(prompt(),text,via+' draft');const before=count('sends'),total=count('total');if(via==='Enter'){await focus(prompt(),'Enter composer');assert(key().defaultPrevented,'Enter send cancels insertion')}else if(via==='send'){send().click()}else{form().requestSubmit(get('chat-form-submit'))}await wait(via+' stores exact message once and clears active draft',()=>count('sends')===before+1&&count('total')===total+1&&prompt().value===''&&last(result().dataset.thread)===text&&messageTexts(result().dataset.thread).at(-1)===text)};
 await wait('actual chat workspace mounted',()=>prompt()&&form()&&result()&&record('motor')&&record('pump')&&send());await wait('native document focused',()=>document.hasFocus());
 const composer=prompt(),ownedForm=form(),options=get('chat-options'),preparation=note();
 assert(result().dataset.thread==='motor'&&count('total')===0&&count('sends')===0&&prompt().value===''&&note().value==='Prepare local maintenance discussion','initial local conversation and preparation');
 assert(prompt().form===form()&&!form().contains(prompt())&&prompt().name==='prompt'&&prompt().required&&prompt().maxLength===400&&new FormData(form()).get('prompt')==='','one external native textarea is associated and named');assert(!form().checkValidity()&&prompt().validity.valueMissing&&send().disabled,'required blank draft is natively invalid');form().requestSubmit(get('chat-form-submit'));await hold(60);assert(count('formSubmits')===0&&count('sends')===0,'native required validation prevents blank form callback/message');snapshots.nativeForm=snapshot();

 await focus(note(),'independent preparation');await input(note(),'Prepare thread-specific inspection notes','preparation');await input(prompt(),'Retained motor draft','motor draft');await thread('pump','');await input(prompt(),'Retained pump draft','pump draft');await thread('motor','Retained motor draft');assert(prompt()===composer&&form()===ownedForm&&draft('pump')==='Retained pump draft','thread switch retains one textarea identity and other draft');get('chat-focus').click();await wait('Dioxus mounted focus targets actual composer',()=>active()===composer);snapshots.retainedThreads=snapshot();

 await sent('  Inspect Motor 001 bearings.  ','Enter');await sent('Record Motor 001 vibration.','send');await sent('Schedule Motor 001 review.','form');assert(count('sends')===3&&count('total')===3&&count('formSubmits')===1&&result().dataset.lastFormPrompt==='Schedule Motor 001 review.'&&record('motor').dataset.count==='3'&&draft('pump')==='Retained pump draft','Enter, send and actual FormEvent each create one exact local record');snapshots.submissions=snapshot();

 await input(prompt(),'Motor draft to clear','reset draft');assert(new FormData(form()).get('prompt')==='Motor draft to clear','native FormData contains current controlled prompt');get('chat-form-reset').click();await wait('native reset updates only active controlled draft',()=>prompt().value===''&&draft('motor')===''&&count('resets')===1);assert(count('total')===3&&record('motor').dataset.count==='3'&&draft('pump')==='Retained pump draft'&&note()===preparation&&note().value==='Prepare thread-specific inspection notes','draft reset retains sent messages, other thread and preparation');await thread('pump','Retained pump draft');await sent('Pump 002 inlet checked.','form');assert(count('total')===4&&record('pump').dataset.count==='1'&&record('motor').dataset.count==='3','form submission mutates only selected conversation');await thread('motor','');snapshots.scopedReset=snapshot();

 options.open=true;await input(prompt(),' \n\t ','whitespace draft');const beforeWhitespace=count('sends');key();send().click();form().requestSubmit(get('chat-form-submit'));await wait('owner rejects whitespace through associated form',()=>get('chat-feedback')&&count('formSubmits')===3);assert(count('sends')===beforeWhitespace&&prompt().value===' \n\t '&&count('total')===4,'whitespace has no outgoing record');await input(prompt(),'Retained guarded draft','guarded draft');await option('chat-newlines',true);const newline=key();assert(!newline.defaultPrevented&&count('sends')===beforeWhitespace,'newline mode admits native default without sending');await option('chat-newlines',false);snapshots.ownerGuards=snapshot();

 for(const mode of ['readonly','disabled','processing']){await option('chat-'+mode,true);const before=count('sends');assert(send().disabled,mode+' send is disabled');if(mode==='readonly')assert(prompt().readOnly&&!prompt().disabled,'read-only native textarea remains selectable');if(mode==='disabled')assert(prompt().disabled&&!new FormData(form()).has('prompt'),'disabled native textarea excluded from form');key();send().click();form().requestSubmit(get('chat-form-submit'));await hold(60);assert(count('sends')===before&&count('total')===4&&prompt().value==='Retained guarded draft',mode+' rejects all submission paths and retains draft');await option('chat-'+mode,false)}assert(prompt()===composer&&note()===preparation,'guard changes retain native composer/preparation identity');snapshots.disabledReadonlyProcessing=snapshot();

 await focus(get('chat-german'),'native language option');await option('chat-german',true);assert(active()===get('chat-german'),'language change does not steal focus');await wait('localized composer send and accessible textarea labels',()=>send().getAttribute('aria-label')==='Nachricht senden'&&prompt().getAttribute('aria-label')==='Nachricht für Motor 001');await option('chat-processing',true);assert(send().getAttribute('aria-label')==='Bitte warten','processing action is localized');await option('chat-processing',false);assert(count('total')===4&&draft('motor')==='Retained guarded draft'&&note().value==='Prepare thread-specific inspection notes','localization preserves actual data');const restore=get('chat-restore');await focus(restore,'Restore conversations');restore.click();await wait('Restore remounts owned conversation and preparation',()=>prompt()&&prompt()!==composer&&note()!==preparation&&prompt().value===''&&count('total')===0&&count('sends')===0&&count('resets')===0&&result().dataset.thread==='motor'&&result().dataset.german==='false');assert(!composer.isConnected&&!preparation.isConnected&&get('chat-options')===options&&get('chat-restore')===restore&&active()===restore&&note().value==='Prepare local maintenance discussion','Restore preserves toolbar/options focus and resets owned scopes');snapshots.localizationRestore=snapshot();

 get('chat-workspace-narrow').click();await wait('340px gallery frame',()=>get('chat-workspace-frame').getBoundingClientRect().width<=341);await hit(note(),'narrow preparation');await hit(prompt(),'narrow native composer');await input(prompt(),'Narrow Motor 001 maintenance message.','narrow prompt');await hit(send(),'narrow send');send().click();await wait('narrow send creates actual message',()=>count('sends')===1&&count('total')===1&&last('motor')==='Narrow Motor 001 maintenance message.'&&prompt().value==='');await input(prompt(),'Narrow form-associated message.','narrow form prompt');await hit(get('chat-form-submit'),'narrow form submit');get('chat-form-submit').click();await wait('narrow native submit stores exact associated prompt',()=>count('total')===2&&result().dataset.lastFormPrompt==='Narrow form-associated message.'&&last('motor')==='Narrow form-associated message.');await hit(get('chat-form-reset'),'narrow draft reset');layout={frame:rect(get('chat-workspace-frame')),composer:rect(host()),form:rect(form()),textarea:rect(prompt()),pageWidth:document.documentElement.scrollWidth,viewport:innerWidth};assert(layout.composer.width<=341&&layout.form.width<=341&&layout.textarea.width<=341&&layout.pageWidth<=innerWidth+1,'actual narrow form/composer and page fit');snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,hitFacts,checks:['native external form/name/FormData and required validation','single mounted composer, per-thread retained drafts and explicit Dioxus focus','Enter/send/native form create exact messages once and clear active draft','native reset preserves actual records, other draft and preparation','whitespace guard and newline default permission','disabled/read-only/processing submission guards','localized action labels and scoped Restore focus','340px native composer/form/action hits and exact saved messages'],input:'excluded synthetic input/change/keydown, native requestSubmit/reset/click and explicit native focus; untrusted keys do not simulate typing or browser Tab/default activation',trustedInput:'separate lead Chromium trusted Enter/Tab/typing and hard maxlength evidence; existing composer regression remains separate',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,layout,hitFacts,failureGeometry}}
"#;

//! Excluded actual-gallery retained release news preview and bounded native replay.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/release_news.rs"]
mod gallery_news;

#[component]
pub fn ReleaseNewsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class: "release-news-review",
            h1 { "Retained release news" }
            Button { id: "release-news-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "release-news-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_news::ReleaseNewsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_RELEASE_NEWS_CHECK);
                        if let Err(error) = query.send("releasenews") {
                            eprintln!("Release news native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native release news: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native release news failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Release news native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.release-news-review{box-sizing:border-box;min-height:100vh;padding:24px}.release-news-review>h1{font-size:28px;line-height:40px;margin:0 0 24px}#release-news-frame{margin-top:24px}@media(max-width:600px){.release-news-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_RELEASE_NEWS_CHECK: &str = r#"
let snapshots={},geometry={},hiddenFocus={};
try {
 await dioxus.recv();
 const byId=id=>document.getElementById(id),get=id=>byId('release-news-'+id),active=()=>document.activeElement;
 const root=()=>byId('release-news'),status=()=>get('status'),selected=()=>status()?.dataset.release,reviewed=()=>Number(status()?.dataset.reviewed);
 const news=key=>get(key),note=key=>byId('release-note-'+key),ack=key=>byId('release-ack-'+key),save=key=>byId('release-save-'+key),form=key=>byId('release-review-'+key);
 const close=key=>news(key)?.querySelector('.ix-about-news-close'),more=key=>news(key)?.querySelector('.ix-about-news-footer button');
 const tab=key=>root()?.querySelector('.ix-tab-wrapper[data-tab-key="'+key+'"] [role="tab"]'),panel=key=>byId(tab(key)?.getAttribute('aria-controls'));
 const record=key=>byId('release-record-'+key),records=()=>[...get('records')?.querySelectorAll('[id^="release-record-"]')??[]].map(node=>({id:node.id,note:node.dataset.note,acknowledged:node.dataset.acknowledged,text:node.textContent.trim()}));
 const shown=key=>status()?.dataset.show==='true'&&selected()===key&&news(key)?.hidden===false;
 const visible=node=>{if(!node||node.closest('[hidden]'))return false;const r=node.getBoundingClientRect(),s=getComputedStyle(node);return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'};
 const assert=(condition,label)=>{if(!condition)throw Error(label)},hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await hold(20)}throw Error(label+'; focus='+(active()?.id||active()?.tagName)+'; status='+status()?.outerHTML)};
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus({preventScroll:true});await wait(label+' actual native focus',()=>active()===node)};
 const click=id=>{assert(get(id),id+' control exists');get(id).click()};
 const input=async(node,value)=>{assert(node,'actual field exists');node.value=value;node.dispatchEvent(new Event('input',{bubbles:true}));await wait('controlled field value',()=>node.value===value)};
 const rect=node=>{if(!node)return null;const r=node.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}};
 const hit=node=>{if(!visible(node))return false;const r=node.getBoundingClientRect();if(r.left< -1||r.right>innerWidth+1||r.top< -1||r.bottom>innerHeight+1)return false;const target=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return target===node||node.contains(target)};
 const capture=()=>({release:selected(),show:status()?.dataset.show,reviewed:reviewed(),records:records(),planning:{note:note('planning')?.value,acknowledged:ack('planning')?.checked,hidden:news('planning')?.hidden},diagnostics:{note:note('diagnostics')?.value,acknowledged:ack('diagnostics')?.checked,hidden:news('diagnostics')?.hidden},preparation:get('preparation')?.value,focus:active()?.id||active()?.tagName});
 const assertHidden=key=>{assert(news(key)?.isConnected&&note(key)?.isConnected,'dismissed '+key+' native content stays mounted');assert(news(key).hidden&&getComputedStyle(news(key)).display==='none'&&!visible(note(key)),'hidden '+key+' news is actually excluded from layout');const before=active();note(key).focus({preventScroll:true});assert(active()===before,'hidden '+key+' note cannot take native focus');hiddenFocus[key]={hidden:news(key).hidden,display:getComputedStyle(news(key)).display,focusBefore:before?.id,focusAfter:active()?.id}};
 const closeNews=async key=>{await focus(close(key),key+' native close');close(key).click();await wait(key+' Close hides news and focuses surviving Reopen action',()=>status().dataset.show==='false'&&news(key).hidden&&active()===get('reopen'));assertHidden(key)};
 const reopen=async key=>{await focus(get('reopen'),'Reopen action');click('reopen');await wait(key+' real retained news reopens',()=>shown(key)&&visible(note(key))&&visible(close(key))&&visible(more(key)))};
 const select=async key=>{await focus(tab(key),key+' native release tab');tab(key).click();await wait(key+' actual selected TabSet panel',()=>selected()===key&&tab(key).getAttribute('aria-selected')==='true'&&panel(key)?.hidden===false&&visible(panel(key)))};
 const showMore=async key=>{await focus(more(key),key+' native Show more');more(key).click();await wait(key+' Show more selects real linked TabSet and focuses visible details heading',()=>status().dataset.show==='false'&&news(key).hidden&&tab(key).getAttribute('aria-selected')==='true'&&panel(key)?.hidden===false&&active()===byId('release-details-'+key)&&visible(active()));assertHidden(key)};
 const submit=async(key,value,expectedCount)=>{await input(note(key),value);if(!ack(key).checked)ack(key).click();await wait(key+' actual acknowledgement enables Save',()=>ack(key).checked&&!save(key).disabled);const payload=Object.fromEntries(new FormData(form(key)).entries());assert(payload['review-note']===value&&payload.acknowledged==='yes',key+' native successful controls contain exact note and acknowledgement');await focus(save(key),key+' native Save review');save(key).click();await wait(key+' actual per-release review saved',()=>record(key)?.dataset.note===value&&record(key)?.dataset.acknowledged==='true'&&reviewed()===expectedCount);assert(record(key).textContent.includes(value),key+' visible saved record contains actual note');return payload};
 await wait('actual release gallery and retained native fields mounted',()=>root()&&status()&&news('planning')&&news('diagnostics')&&note('planning')&&note('diagnostics')&&get('preparation')&&get('reopen')&&tab('planning')&&tab('diagnostics'));
 await wait('native document focus',()=>document.hasFocus());
 assert(shown('planning')&&reviewed()===0&&records().length===0,'initial planning bulletin is shown with no saved review');
 assert(news('diagnostics').hidden&&!visible(note('diagnostics')),'other release bulletin is initially hidden');
 assert(close('planning').getAttribute('aria-label')==='Dismiss planning news'&&more('planning').textContent.trim()==='Read planning details','caller-provided planning labels reach actual native actions');
 geometry.wide={news:rect(news('planning')),banner:rect(news('planning').querySelector('.ix-about-news-banner')),background:getComputedStyle(news('planning')).backgroundColor};
 assert(Math.abs(geometry.wide.news.width-320)<=1,'ordinary news width preserves existing320px source style');snapshots.initial=capture();
 await input(get('preparation'),'Prepare local release-review checklist.');await input(note('planning'),'Planning draft survives dismissal.');ack('planning').click();await wait('private planning acknowledgement draft updates',()=>ack('planning').checked);
 await closeNews('planning');snapshots.closed=capture();await reopen('planning');
 assert(note('planning').value==='Planning draft survives dismissal.'&&ack('planning').checked,'private composed planning note and acknowledgement survive Close/reopen');
 const planningPayload=await submit('planning','Planning release reviewed locally.',1);snapshots.planningSave={...capture(),payload:planningPayload};
 await showMore('planning');snapshots.planningDetails=capture();

 await select('diagnostics');await reopen('diagnostics');
 assert(note('diagnostics').value===''&&!ack('diagnostics').checked,'second release owns an independent initial private review draft');
 assert(close('diagnostics').getAttribute('aria-label')==='Dismiss condition news'&&more('diagnostics').textContent.trim()==='Read condition details','caller-provided second-release labels reach native actions');
 const diagnosticsPayload=await submit('diagnostics','Condition insights reviewed locally.',2);snapshots.diagnosticsSave={...capture(),payload:diagnosticsPayload};
 await closeNews('diagnostics');await reopen('diagnostics');assert(note('diagnostics').value==='Condition insights reviewed locally.'&&ack('diagnostics').checked,'second release review remains after its Close/reopen');
 await showMore('diagnostics');snapshots.diagnosticsDetails=capture();
 await select('planning');await reopen('planning');assert(note('planning').value==='Planning release reviewed locally.'&&ack('planning').checked&&record('diagnostics')?.dataset.note==='Condition insights reviewed locally.','returning to planning preserves both private drafts and independent saved release records');
 snapshots.retainedAcrossTabs=capture();

 await focus(get('reset'),'scoped Reset action');click('reset');
 await wait('Reset clears owned saved reviews and private drafts',()=>reviewed()===0&&records().length===0&&note('planning').value===''&&note('diagnostics').value===''&&!ack('planning').checked&&!ack('diagnostics').checked);
 assert(get('preparation').value==='Prepare local release-review checklist.','owned Reset preserves independent preparation');snapshots.reset=capture();
 document.getElementById('release-news-narrow').click();await wait('actual gallery frame340px',()=>document.getElementById('release-news-frame').getBoundingClientRect().width<=341);
 if(selected()!=='planning')await select('planning');if(!shown('planning'))await reopen('planning');
 await input(note('planning'),'Planning review from narrow frame.');if(!ack('planning').checked)ack('planning').click();await wait('narrow draft enables actual Save',()=>ack('planning').checked&&!save('planning').disabled);
 const frame=document.getElementById('release-news-frame');geometry.narrow={viewport:{width:innerWidth,height:innerHeight},frame:rect(frame),news:rect(news('planning')),controls:{}};
 for(const [name,node] of [['note',note('planning')],['acknowledgement',ack('planning')],['save',save('planning')],['close',close('planning')],['showMore',more('planning')],['preparation',get('preparation')]]){node.scrollIntoView({block:'nearest',inline:'nearest'});geometry.narrow.controls[name]={rect:rect(node),hit:hit(node)};await wait('narrow '+name+' actual center hit',()=>hit(node));const r=node.getBoundingClientRect(),f=frame.getBoundingClientRect();assert(r.left>=f.left-1&&r.right<=f.right+1,'narrow '+name+' stays within actual frame');geometry.narrow.controls[name]={rect:rect(node),hit:hit(node)}}
 const narrowPayload=await submit('planning','Planning review from narrow frame.',1);snapshots.narrowSave={...capture(),payload:narrowPayload};
 await showMore('planning');snapshots.final=capture();
 return {ok:true,snapshots,geometry,hiddenFocus,checks:['actual private composed note/ack survive Close/reopen and remain natively hidden/focus-excluded when dismissed','native form values save real independent release review records','Show more hides retained news, selects actual linked TabSet and focuses surviving details heading','scope Reset clears owned private/saved reviews while retaining independent preparation','actual340px gallery frame fields/actions receive native center hits and ordinary news geometry remains recognizable'],input:'Excluded native DOM clicks, explicit focus and input events execute the actual gallery handlers. No trusted Tab/Enter/Space defaults, fabricated hover, callbacks or millisecond timing are claimed.',viewportQualification:'340px narrows the actual gallery frame in the unchanged desktop WebKitGTK viewport; lead Chromium verifies true narrow layout and trusted keyboard/pointer behavior.',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,geometry,hiddenFocus,focus:document.activeElement?.id||document.activeElement?.tagName,status:document.getElementById('release-news-status')?.outerHTML,viewport:{width:innerWidth,height:innerHeight}}}

"#;

//! Excluded actual document workspace named icon-only tab workflows.
use super::document_workspace_review::gallery_documents;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn IconTabsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.icon-tabs-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        style { {gallery_documents::STYLES} }
        IxTheme { class: "icon-tabs-review",
            gallery_documents::DocumentWorkspaceExample {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_ICON_TABS_CHECK);
                        if let Err(error) = query.send("icontabs") {
                            eprintln!("Icon tabs native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native Icon tabs: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native Icon tabs failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(45)).await;
                        eprintln!("Icon tabs native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_ICON_TABS_CHECK: &str = r#"
let snapshots={},checks=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),host=()=>get('documents-tabs'),state=()=>get('documents-result').dataset,wrapper=id=>host().querySelector('.ix-tab-wrapper[data-tab-key="'+id+'"]'),tab=id=>wrapper(id)?.querySelector('[role=tab]'),close=id=>wrapper(id)?.querySelector('.ix-tab-close'),panel=id=>get(tab(id)?.getAttribute('aria-controls')),draft=id=>get('document-draft-'+id),saved=id=>get('document-saved-'+id)?.querySelector('[data-field=saved]')?.textContent,more=()=>host().querySelector('.ix-tabs-overflow .ix-dropdown-external-trigger'),menu=()=>get(more()?.getAttribute('aria-controls')),logical=()=>{const active=get(more()?.getAttribute('aria-activedescendant'))||(menu()?.contains(document.activeElement)?document.activeElement:null);return active?.getAttribute('data-ix-dropdown-label')};
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; state='+JSON.stringify(state()))},focus=async node=>{assert(node,'actual current control exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait('actual target focused',()=>document.activeElement===node)},click=async node=>{await focus(node);node.click()},key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const checked=async(id,value)=>{const node=get(id);if(node.checked!==value)await click(node);await wait('actual checkbox '+id,()=>node.checked===value)},input=async(node,value)=>{Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual document draft',()=>node.value===value)},choose=async id=>{await click(tab(id));await wait('actual selected '+id,()=>state().selected===id&&!panel(id).hidden)},snapshot=()=>({...state(),iconOnly:get('documents-icon-only').checked,tabs:[...host().querySelectorAll('.ix-tab-wrapper')].map(n=>({key:n.dataset.tabKey,label:n.querySelector('[role=tab]').getAttribute('aria-label'),title:n.querySelector('[role=tab]').title,disabled:n.querySelector('[role=tab]').disabled}))});
 await wait('actual document workspace mounted',()=>get('documents-icon-only')&&tab('motor')&&tab('reference'));
 assert(!get('documents-icon-only').checked&&tab('motor').textContent.includes('Motor 001'),'default labelled painting remains');get('documents-options').querySelector('summary').click();await wait('actual options expanded',()=>get('documents-options').open);const motorTab=tab('motor'),motorDraft=draft('motor'),pumpDraft=draft('pump'),motorPanel=panel('motor');await checked('documents-icon-only',true);
 for(const [id,label]of [['motor','Motor 001'],['pump','Pump 002'],['reference','Inspection reference']]){const node=tab(id);assert(node.getAttribute('aria-label')===label&&node.title===label&&node.querySelector(':scope>svg.ix-icon')&&!node.querySelector(':scope>span:not(.ix-tab-counter)')&&close(id).getAttribute('aria-label')==='Close '+label&&panel(id).getAttribute('aria-labelledby')===node.id,'icon-only native tab keeps full name/title/close/panel and no visible label span');}
 assert(tab('motor').querySelector(':scope>svg.ix-icon').innerHTML!==tab('reference').querySelector(':scope>svg.ix-icon').innerHTML,'document/info use distinct authored SVG');assert(!panel('motor').hidden&&panel('pump').hidden&&tab('motor')===motorTab&&panel('motor')===motorPanel,'mode changes retain current linked native tab/panel');snapshots.presentation=snapshot();checks.push('default labelled tabs become named/title-bearing icon-only document/info tabs with unchanged close and native panel links');
 await input(draft('motor'),'Icon-only retained motor draft');await focus(tab('motor'));key(tab('motor'),'ArrowRight');await wait('normal automatic arrow selects Pump',()=>state().selected==='pump'&&document.activeElement===tab('pump'));await input(draft('pump'),'Icon-only saved pump');await click(get('document-save-pump'));await wait('actual Pump Save stored',()=>saved('pump')==='Icon-only saved pump'&&state().saves==='1');await checked('documents-icon-only',false);await wait('ordinary labels restored',()=>tab('motor').textContent.includes('Motor 001'));await checked('documents-icon-only',true);await click(get('documents-reverse'));await wait('actual reverse retained current Pump',()=>state().order==='reference,pump,motor'&&state().selected==='pump');assert(draft('motor')===motorDraft&&draft('pump')===pumpDraft&&draft('motor').value==='Icon-only retained motor draft'&&saved('pump')==='Icon-only saved pump','live presentation/switch/reorder retain actual drafts and Save');await click(get('documents-reverse'));await wait('ordinary order restored',()=>state().order==='motor,pump,reference');snapshots.retention=snapshot();checks.push('ordinary automatic keyboard selection and Save survive live on/off presentation/reorder with retained private fields');
 await checked('documents-disable-reference',true);await wait('actual reference/close disabled',()=>tab('reference').disabled&&close('reference').disabled);const requests=state().closeRequests;close('reference').click();assert(state().closeRequests===requests&&state().count==='3','disabled reference cannot close');await checked('documents-manual',true);await choose('motor');key(tab('motor'),'ArrowRight');await wait('manual focus alone reaches Pump',()=>document.activeElement===tab('pump')&&state().selected==='motor');assert(!key(tab('pump'),'Enter').defaultPrevented,'native manual Enter default remains admitted');tab('pump').click();await wait('explicit native activation selects manual Pump',()=>state().selected==='pump');await choose('motor');key(tab('motor'),'Delete');await wait('dirty owner close request held',()=>state().pendingClose==='motor'&&get('documents-keep'));await click(get('documents-keep'));await wait('Keep retains current dirty draft and editor focus',()=>state().pendingClose==='none'&&document.activeElement===draft('motor')&&draft('motor').value==='Icon-only retained motor draft');await click(close('pump'));await wait('clean ordinary close removes Pump only',()=>!wrapper('pump')&&state().count==='2'&&state().closes==='1');snapshots.guards=snapshot();checks.push('disabled reference/manual activation and dirty Keep/clean close remain usable in icon-only mode');
 await checked('documents-disable-reference',false);await checked('documents-manual',false);await checked('documents-all-tabs',true);await click(get('documents-samples'));await wait('actual sample documents and forced menu available',()=>state().count==='10'&&tab('sample-8')&&more());await focus(more());key(more(),'End');await wait('actual all-documents End uses full far sample name',()=>more().getAttribute('aria-expanded')==='true'&&logical()==='Sample inspection 8');const labels=[...menu().querySelectorAll('[data-ix-dropdown-label]')].map(n=>n.getAttribute('data-ix-dropdown-label'));assert(labels.includes('Motor 001')&&labels.includes('Inspection reference')&&labels.includes('Sample inspection 8'),'forced all-tabs menu keeps full meaningful labels');key(document.activeElement,'Enter');await wait('semantic menu activation selects actual retained panel',()=>state().selected==='sample-8'&&!panel('sample-8').hidden&&more().getAttribute('aria-expanded')==='false');assert(draft('motor')===motorDraft&&draft('motor').value==='Icon-only retained motor draft','overflow retains dirty Motor field');snapshots.overflow={...snapshot(),labels};await click(get('documents-reset'));await wait('Restore returns labelled ordinary defaults',()=>state().count==='3'&&state().selected==='motor'&&!get('documents-icon-only').checked&&tab('motor').textContent.includes('Motor 001'));checks.push('forced semantic all-tabs menu retains full names/selects real panel; Restore resets icon-only mode and owned workspace');
 return{ok:true,checks,snapshots,input:'Excluded synthetic native DOM input/click/change and handled tab/Delete/overflow keys invoke actual DocumentWorkspaceExample. Manual Enter is checked not prevented then separately activated with explicit click; trusted native Enter/Space defaults are Chromium evidence. No application/model/controller/callback/CSS injection or copied state; SVG comparison uses actual document/info artwork, not one unique icon per regular document.'};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:{id:document.activeElement?.id,label:document.activeElement?.getAttribute('aria-label')}}}
"#;

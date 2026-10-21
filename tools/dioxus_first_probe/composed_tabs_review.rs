//! Excluded actual document workspace Dioxus-composed tab-label workflows.
use super::document_workspace_review::gallery_documents;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn ComposedTabsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.composed-tabs-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        style { {gallery_documents::STYLES} }
        IxTheme { class: "composed-tabs-review",
            gallery_documents::DocumentWorkspaceExample {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_COMPOSED_TABS_CHECK);
                        if let Err(error) = query.send("composedtabs") {
                            eprintln!("Composed tabs native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native Composed tabs: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native Composed tabs failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(45)).await;
                        eprintln!("Composed tabs native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_COMPOSED_TABS_CHECK: &str = r#"
let snapshots={},checks=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),host=()=>get('documents-tabs'),state=()=>get('documents-result').dataset,wrapper=id=>host().querySelector('.ix-tab-wrapper[data-tab-key="'+id+'"]'),tab=id=>wrapper(id)?.querySelector('[role=tab]'),label=id=>tab(id)?.querySelector('[data-document-tab-status="'+id+'"]'),pill=id=>label(id)?.querySelector('.ix-pill')?.textContent.trim(),close=id=>wrapper(id)?.querySelector('.ix-tab-close'),panel=id=>get(tab(id)?.getAttribute('aria-controls')),draft=id=>get('document-draft-'+id),saved=id=>get('document-saved-'+id)?.querySelector('[data-field=saved]')?.textContent,more=()=>host().querySelector('.ix-tabs-overflow .ix-dropdown-external-trigger'),menu=()=>get(more()?.getAttribute('aria-controls')),logical=()=>{const active=get(more()?.getAttribute('aria-activedescendant'))||(menu()?.contains(document.activeElement)?document.activeElement:null);return active?.getAttribute('data-ix-dropdown-label')};
 const assert=(value,name)=>{if(!value)throw Error(name)},wait=async(name,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(name+'; state='+JSON.stringify(state()))},focus=async node=>{assert(node,'actual current control exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait('actual target focused',()=>document.activeElement===node)},click=async node=>{await focus(node);node.click()},key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
 const checked=async(id,value)=>{const node=get(id);if(node.checked!==value)await click(node);await wait('actual checkbox '+id,()=>node.checked===value)},input=async(node,value)=>{Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual document draft',()=>node.value===value)},choose=async id=>{await click(tab(id));await wait('actual selected '+id,()=>state().selected===id&&!panel(id).hidden)},snapshot=()=>({...state(),rich:get('documents-rich-labels').checked,icon:get('documents-icon-only').checked,labels:[...host().querySelectorAll('[data-document-tab-status]')].map(n=>({key:n.dataset.documentTabStatus,text:n.textContent,painted:n.getClientRects().length>0}))});
 await wait('actual rich-label option mounted',()=>get('documents-rich-labels')&&tab('motor'));assert(!get('documents-rich-labels').checked&&!label('motor')&&tab('motor').textContent.includes('Motor 001'),'default ordinary label retained');get('documents-options').querySelector('summary').click();await wait('actual options open',()=>get('documents-options').open);await checked('documents-rich-labels',true);await wait('actual reactive label components mounted',()=>['motor','pump','reference'].every(id=>label(id)&&pill(id)==='Saved'));
 for(const[id,name]of[['motor','Motor 001'],['pump','Pump 002'],['reference','Inspection reference']])assert(tab(id).getAttribute('aria-label')===name&&tab(id).title===name&&close(id).getAttribute('aria-label')==='Close '+name&&panel(id).getAttribute('aria-labelledby')===tab(id).id&&!label(id).querySelector('button,input,select,textarea,[tabindex]'),'rich child remains noninteractive with full native names/panel links');const motorLabel=label('motor'),pumpLabel=label('pump'),motorDraft=draft('motor'),pumpDraft=draft('pump');snapshots.rich=snapshot();checks.push('default ordinary labels opt into actual noninteractive title/Pill child components while full native names/panel/close links remain');
 await input(draft('motor'),'Composed saved motor');await wait('only current Motor status becomes Unsaved',()=>pill('motor')==='Unsaved'&&pill('pump')==='Saved'&&pill('reference')==='Saved');assert(saved('motor')==='Inspect bearing housing','typing does not alter saved record');await click(get('document-save-motor'));await wait('actual Save updates child status and stored text',()=>pill('motor')==='Saved'&&saved('motor')==='Composed saved motor');await input(draft('motor'),'Composed retained dirty motor');await wait('actual Motor child reacts again',()=>pill('motor')==='Unsaved');assert(label('motor')===motorLabel&&label('pump')===pumpLabel,'reactive child identity retained');snapshots.reactive=snapshot();checks.push('actual native typing changes only current draft Pill; Save returns Saved and stores exact document without replacing label children');
 await focus(tab('motor'));key(tab('motor'),'ArrowRight');await wait('normal Arrow selection reaches Pump',()=>state().selected==='pump'&&document.activeElement===tab('pump'));await input(draft('pump'),'Composed Pump hidden label draft');await wait('Pump current label Unsaved',()=>pill('pump')==='Unsaved');await click(get('documents-reverse'));await wait('current selected Pump survives keyed reverse',()=>state().order==='reference,pump,motor'&&state().selected==='pump');await checked('documents-icon-only',true);await wait('custom child remains mounted but not painted',()=>label('pump')===pumpLabel&&label('pump').getClientRects().length===0&&tab('pump').querySelector(':scope>svg.ix-icon'));await click(get('document-save-pump'));await wait('hidden reactive label still updates Saved',()=>pill('pump')==='Saved'&&saved('pump')==='Composed Pump hidden label draft');await checked('documents-icon-only',false);await wait('same custom child paints again',()=>label('pump')===pumpLabel&&label('pump').getClientRects().length>0);assert(label('motor')===motorLabel&&draft('motor')===motorDraft&&draft('pump')===pumpDraft&&draft('motor').value==='Composed retained dirty motor'&&pill('motor')==='Unsaved','presentation/reorder retains actual label/editor state');await click(get('documents-reverse'));await wait('ordinary order restored',()=>state().order==='motor,pump,reference');snapshots.retained=snapshot();checks.push('ordinary switch/reorder and icon-only hidden/show retain exact custom labels/private editors; hidden current label still reacts to actual Save');
 await choose('motor');key(tab('motor'),'Delete');await wait('dirty close decision remains owner controlled',()=>state().pendingClose==='motor'&&get('documents-keep'));await click(get('documents-keep'));await wait('Keep retains current editor and rich Unsaved',()=>state().pendingClose==='none'&&document.activeElement===draft('motor')&&pill('motor')==='Unsaved');await click(close('pump'));await wait('clean Pump close removes only current document',()=>!wrapper('pump')&&state().count==='2');await checked('documents-all-tabs',true);await click(get('documents-samples'));await wait('sample documents/forced menu available',()=>state().count==='10'&&more());await focus(more());key(more(),'End');await wait('full semantic sample name targeted',()=>more().getAttribute('aria-expanded')==='true'&&logical()==='Sample inspection 8');const names=[...menu().querySelectorAll('[data-ix-dropdown-label]')].map(n=>n.getAttribute('data-ix-dropdown-label'));assert(names.includes('Motor 001')&&names.includes('Inspection reference')&&names.includes('Sample inspection 8')&&!names.some(n=>n.includes('Unsaved')||n.includes('Saved')),'overflow names use meaningful item labels only');key(document.activeElement,'Enter');await wait('actual far sample panel selected',()=>state().selected==='sample-8'&&!panel('sample-8').hidden&&more().getAttribute('aria-expanded')==='false');assert(label('motor')===motorLabel&&draft('motor')===motorDraft&&pill('motor')==='Unsaved','overflow keeps prior label/editor state');snapshots.menu={...snapshot(),names};await click(get('documents-reset'));await wait('Restore resets rich/icon options and owned workspace',()=>state().count==='3'&&state().selected==='motor'&&!get('documents-rich-labels').checked&&!get('documents-icon-only').checked&&!label('motor'));checks.push('dirty Keep/clean close and full-name forced overflow retain owner semantics; Restore resets composed-label presentation');
 return{ok:true,checks,snapshots,input:'Excluded synthetic DOM input/click/handled tab/Delete/overflow keys with explicit native focus invoke actual DocumentWorkspaceExample. Label/panel/editor identity and hidden rendering are observed, no application/model/controller/callback/CSS injection or copied child state. Trusted Chromium supplies normal native key defaults/typing and paired visual evidence; rounded/no-renderer/counters are component-test coverage.'};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:{id:document.activeElement?.id,label:document.activeElement?.getAttribute('aria-label')}}}
"#;

//! Excluded actual-gallery Select value-derived labels and bounded native driver.
use crate::gallery_forms;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn SelectValueLabelsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {include_str!("../../examples/gallery/styles/main.css")} }
        style { {gallery_forms::STYLES} }
        IxTheme { class: "review-app",
            main { class: "review-content", gallery_forms::FormExamples {} }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_SELECT_VALUE_LABELS_CHECK);
                        if let Err(error) = query.send("selectvaluelabels") {
                            eprintln!("Select value labels native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native Select value labels: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native Select value labels failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;
                        eprintln!("Select value labels native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

// Excluded synthetic input/click and explicit native focus use actual FormExamples.
// Native key defaults, physical input and assistive technology are separate evidence.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SELECT_VALUE_LABELS_CHECK: &str = r#"
let checks=[],snapshots={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),single=()=>get('select-value-label'),multiple=()=>get('select-values-label'),editable=()=>get('select-editable-label'),status=()=>get('select-value-label-status'),inputStatus=()=>get('select-value-label-input'),note=()=>get('choice-retained-notes'),root=node=>node.closest('.ix-select'),native=node=>root(node).querySelector('select.ix-select-native'),list=node=>get(node.getAttribute('aria-controls')),option=(node,value)=>list(node)?.querySelector('button[role=option][value="'+value+'"]'),selected=node=>[...native(node).selectedOptions].map(option=>option.value);
 const assert=(value,message)=>{if(!value)throw Error(message)},wait=async(message,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(message+'; status='+status()?.textContent+'; input='+inputStatus()?.textContent+'; focus='+document.activeElement?.id)};
 const focus=async(node,message)=>{assert(node,message+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(message+' actual native focus',()=>document.activeElement===node)},click=async(node,message)=>{await focus(node,message);node.click()};
 const query=async(node,text)=>{await focus(node,'actual combobox');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,text);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('actual Rust filtering opens current list for '+text,()=>node.value===text&&node.getAttribute('aria-expanded')==='true'&&list(node));};
 const snapshot=()=>({single:{text:single().value,values:selected(single()),name:native(single()).name},multiple:{values:selected(multiple()),name:native(multiple()).name,chips:[...root(multiple()).querySelectorAll('.ix-select-chip-label')].map(node=>node.textContent)},editable:{text:editable().value,values:selected(editable()),name:native(editable()).name},status:status().textContent,input:inputStatus().textContent,note:note().value});
 await wait('actual shared Select value-label examples mounted',()=>single()&&multiple()&&editable()&&status()&&inputStatus()&&note()&&[single(),multiple(),editable()].every(node=>native(node)));await wait('native document ready for focus',()=>document.hasFocus());
 assert([single(),multiple(),editable()].every(node=>node.tagName==='INPUT'&&node.getAttribute('role')==='combobox')&&native(single()).name==='single-choice'&&native(multiple()).name==='multiple-choice'&&native(editable()).name==='editable-choice','actual visible comboboxes retain distinct named native SELECT mirrors');
 const originalNote=note(),noteText='Keep maintenance notes while choosing value-only options';assert(note().tagName==='TEXTAREA'&&!note().form&&[single(),multiple(),editable()].every(node=>!native(node).form),'actual notes and selectors are unassociated; no form submission claim');await focus(note(),'actual retained notes');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(note(),noteText);note().dispatchEvent(new Event('input',{bubbles:true,composed:true}));
 await query(single(),'4');await wait('single filtered value-only option is visibly named',()=>option(single(),'4')?.querySelector('.ix-select-option-label')?.textContent==='4'&&list(single()).querySelectorAll('button[role=option]').length===1);option(single(),'4').click();await wait('single selection exposes fallback text and unchanged value',()=>single().value==='4'&&single().getAttribute('aria-expanded')==='false'&&native(single()).value==='4'&&native(single()).selectedOptions[0]?.text==='4'&&status().textContent.startsWith('Selection: 4.'));
 await click(root(single()).querySelector('button.ix-select-clear'),'actual Clear selection');await wait('normal clear removes actual selection',()=>single().value===''&&native(single()).value===''&&status().textContent.startsWith('Selection: .'));await query(single(),'Item 2');await wait('nonempty option label remains unchanged',()=>option(single(),'2')?.querySelector('.ix-select-option-label')?.textContent==='Item 2');option(single(),'2').click();await wait('nonempty label selects normally',()=>single().value==='Item 2'&&native(single()).value==='2'&&native(single()).selectedOptions[0]?.text==='Item 2'&&status().textContent.startsWith('Selection: 2.'));assert(note()===originalNote&&note().value===noteText,'single choice/clear retains actual notes');snapshots.single=snapshot();checks.push('actual single combobox filters/names/selects value-only 4, clears normally and preserves nonempty Item 2 presentation/value');
 await query(multiple(),'4');await wait('multiple value-only row is named',()=>option(multiple(),'4')?.querySelector('.ix-select-option-label')?.textContent==='4');option(multiple(),'4').click();await wait('multiple selection preserves actual native values',()=>selected(multiple()).join(',')==='1,3,4'&&native(multiple()).querySelector('option[value="4"]').text==='4'&&status().textContent.includes('Multiple: 1, 3, 4.'));
 const removeFour=()=>[...root(multiple()).querySelectorAll('button[aria-label="Remove 4"]')].find(node=>!node.closest('.ix-select-measure'))||null;if(!removeFour()){const more=root(multiple()).querySelector('button.ix-select-overflow');assert(more,'current overflow control exposes value-only chip when necessary');await click(more,'actual hidden selected items');await wait('current overflow exposes named removal',()=>removeFour());}assert(removeFour().closest('.ix-select-chip').querySelector('.ix-select-chip-label')?.textContent==='4','actual current or overflow chip displays value-only text');await click(removeFour(),'actual Remove 4');await wait('named chip removal publishes current native values',()=>selected(multiple()).join(',')==='1,3'&&!removeFour()&&status().textContent.includes('Multiple: 1, 3.'));assert(note()===originalNote&&note().value===noteText,'multiple chip updates retain actual note');snapshots.multiple=snapshot();checks.push('actual multiple choice shows value-only chip and Remove 4 name, preserves native SELECT values and removes the current value normally');
 await query(editable(),'4');await wait('existing value-only option is searchable without duplicate Add',()=>option(editable(),'4')?.querySelector('.ix-select-option-label')?.textContent==='4'&&!list(editable()).querySelector('.ix-select-add')&&inputStatus().textContent==='Input: 4. Added: .');option(editable(),'4').click();await wait('editable existing value-only selection does not invoke Add',()=>editable().value==='4'&&native(editable()).value==='4'&&status().textContent.includes('Editable: 4.')&&inputStatus().textContent==='Input: 4. Added: .');
 const newCode='Motor 005';await query(editable(),newCode);await wait('ordinary new code exposes actual Add action',()=>list(editable()).querySelector('.ix-select-add')?.getAttribute('aria-label')==='Add item: '+newCode);list(editable()).querySelector('.ix-select-add').click();await wait('normal Add invokes existing callbacks and selects actual new value',()=>editable().value===newCode&&native(editable()).value===newCode&&native(editable()).selectedOptions[0]?.text===newCode&&status().textContent.includes('Editable: '+newCode+'.')&&inputStatus().textContent==='Input: '+newCode+'. Added: '+newCode+'.');
 const peers=[...get('selects').querySelector(':scope>.form-demo-grid').querySelectorAll('input[role=combobox]')],disabled=peers.find(node=>node.disabled),readonly=peers.find(node=>node.readOnly);assert(disabled&&readonly&&disabled.value==='Item 1'&&readonly.value==='Item 2'&&disabled.getAttribute('aria-expanded')==='false'&&readonly.getAttribute('aria-expanded')==='false'&&readonly.getAttribute('aria-readonly')==='true'&&native(disabled).value==='1'&&native(readonly).value==='2','existing disabled/read-only choices remain guarded with unchanged named labels');assert(note()===originalNote&&note().value===noteText,'actual new/duplicate decision callbacks preserve textarea node and draft');snapshots.editable=snapshot();checks.push('editable value-only search avoids duplicate Add, ordinary new code invokes current callbacks, and notes/disabled/read-only peers retain behavior');
 return{ok:true,checks,snapshots,input:'Excluded native preview imports actual shared FormExamples once through gallery_forms. Synthetic native input events drive existing Rust filtering and explicit option/chip/Add DOM clicks drive normal callbacks; focus targets actual current native controls. SELECT mirrors and current text/names are observed read-only. Notes/selectors have no form owner, so no FormData/FormEvent/submission claim. No untrusted key is claimed to execute native keyboard defaults, physical OS/AT or exact chronology. Trusted browser workflows are separate. No copied model/state, replacement form, app/controller/callback/CSS injection or production bridge.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:document.activeElement?.id,status:document.getElementById('select-value-label-status')?.textContent,input:document.getElementById('select-value-label-input')?.textContent}}
"#;

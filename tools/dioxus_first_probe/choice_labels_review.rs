//! Excluded actual-gallery composed Checkbox/Radio labels and bounded native driver.
use crate::gallery_forms;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn ChoiceLabelsPreview() -> Element {
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
                        let query = crate::dom_query::eval(NATIVE_CHOICE_LABELS_CHECK);
                        if let Err(error) = query.send("choicelabels") {
                            eprintln!("Choice labels native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native choice labels: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native choice labels failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;
                        eprintln!("Choice labels native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

// Excluded DOM click/input events and native focus requests exercise the actual
// Rust composition. Trusted keyboard defaults and physical/AT input are separate.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CHOICE_LABELS_CHECK: &str = r#"
let checks=[],snapshots={};
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),agreed=()=>get('choice-composed-agreed'),all=()=>get('choice-composed-all'),count=()=>get('choice-selected-count'),checkboxStatus=()=>get('choice-checkbox-status'),radioStatus=()=>get('choice-radio-status'),note=()=>get('choice-retained-notes'),radio=value=>get('radios-toggles')?.querySelector('input[type=radio][name=storage][value="'+value+'"]'),label=value=>get('choice-label-'+value),sub=()=>[...get('checkboxes').querySelectorAll('.form-demo-indent button[role=checkbox]')],mirror=()=>agreed()?.closest('.ix-checkbox-field').querySelector('input.ix-check-native'),toggle=()=>get('radios-toggles')?.querySelector('button[role=switch][aria-label="Enable notifications"]');
 const assert=(value,message)=>{if(!value)throw Error(message)},wait=async(message,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(message+'; checkbox='+checkboxStatus()?.textContent+'; radio='+radioStatus()?.textContent+'; focus='+document.activeElement?.id)};
 const focus=async(node,message)=>{assert(node,message+' exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait(message+' actual native focus',()=>document.activeElement===node)},click=async(node,message)=>{await focus(node,message);node.click()};
 const snapshot=()=>({checkbox:{agreed:agreed().getAttribute('aria-checked'),all:all().getAttribute('aria-checked'),count:count().textContent,status:checkboxStatus().textContent,mirror:{name:mirror().name,value:mirror().value,checked:mirror().checked}},radios:['256','512','1024'].map(value=>({value:radio(value).value,name:radio(value).name,checked:radio(value).checked,disabled:radio(value).disabled,label:radio(value).getAttribute('aria-label'),composed:label(value).textContent})),status:radioStatus().textContent,note:note().value,noteHasForm:!!note().form,toggle:toggle().getAttribute('aria-checked')});
 await wait('actual composed choices and retained textarea mounted',()=>agreed()&&all()&&count()&&checkboxStatus()&&radioStatus()&&note()&&mirror()&&sub().length===3&&['256','512','1024'].every(value=>radio(value)&&label(value))&&toggle());await wait('native document ready for focus',()=>document.hasFocus());
 assert(agreed().tagName==='BUTTON'&&agreed().getAttribute('role')==='checkbox'&&agreed().getAttribute('aria-label')==='I agree everything'&&agreed().querySelector('.ix-checkbox-label strong')?.textContent==='I agree','actual composed Checkbox retains semantic name and noninteractive strong label');assert(all().getAttribute('aria-label')==='Option group'&&agreed().getAttribute('aria-checked')==='false'&&all().getAttribute('aria-checked')==='mixed'&&count().textContent==='2 selected','actual initial Checkbox mixed state and dynamic child count');
 const originalCount=count();await click(agreed(),'actual composed agreement');await wait('agreement callback and native mirror become checked',()=>agreed().getAttribute('aria-checked')==='true'&&mirror().checked&&checkboxStatus().textContent.includes('Agreed: true.'));
 await click(all(),'actual composed group action');await wait('mixed parent selects all actual children and composed count',()=>all().getAttribute('aria-checked')==='true'&&sub().every(node=>node.getAttribute('aria-checked')==='true')&&count().textContent==='3 selected');await click(sub()[2],'actual third child');await wait('current child selection restores mixed parent and count',()=>all().getAttribute('aria-checked')==='mixed'&&count().textContent==='2 selected'&&checkboxStatus().textContent.includes('Selected sub options: 2.'));assert(count()===originalCount,'dynamic composed count remains in its current label');
 const disabledChecks=[...get('checkboxes').querySelectorAll('button[role=checkbox]:disabled')],priorCheckbox=checkboxStatus().textContent;assert(disabledChecks.length===2,'actual disabled Checkbox examples remain');for(const node of disabledChecks)node.click();assert(checkboxStatus().textContent===priorCheckbox,'native disabled Checkbox clicks preserve current selection');snapshots.checkbox=snapshot();checks.push('actual composed Checkbox clicks retain semantic label, native mirror, mixed/count updates and disabled guards');
 const labelNodes=['256','512','1024'].map(label);for(const [value,name] of [['256','256GB SSD storage'],['512','512GB SSD storage'],['1024','1TB SSD storage']])assert(radio(value).tagName==='INPUT'&&radio(value).type==='radio'&&radio(value).name==='storage'&&radio(value).getAttribute('aria-label')===name&&label(value).querySelector('strong')?.textContent===name&&radio(value).closest('label').contains(label(value)),'actual native Radio retains name/value and current composed Typography '+value);
 assert(radio('256').checked&&!radio('512').checked&&radio('1024').disabled,'actual initial native storage selection/disabled guard');await click(radio('512'),'actual 512 storage radio');await wait('native radio change selects current named option',()=>radio('512').checked&&!radio('256').checked&&radioStatus().textContent.includes('Storage: 512GB.'));radio('1024').click();assert(radio('512').checked&&!radio('1024').checked,'native disabled radio cannot change selection');await click(radio('256'),'restore actual 256 storage radio');await wait('native radio callback restores selected value',()=>radio('256').checked&&radioStatus().textContent.includes('Storage: 256GB.'));assert(['256','512','1024'].every((value,index)=>label(value)===labelNodes[index]),'keyed current composed radio labels remain mounted through selection');snapshots.radio=snapshot();checks.push('actual native radio clicks retain semantic names/composed children and normal mutually exclusive/disabled selection');
 const originalNote=note(),noteText='Retain inspection notes across composed choice updates';assert(note().tagName==='TEXTAREA'&&note().value===''&&!note().form,'existing editable textarea is outside a form; no FormEvent claim');await focus(note(),'actual retained notes');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(note(),noteText);note().dispatchEvent(new Event('input',{bubbles:true,composed:true}));await click(agreed(),'current agreement choice');await wait('current agreement update clears actual mirror',()=>agreed().getAttribute('aria-checked')==='false'&&!mirror().checked&&checkboxStatus().textContent.includes('Agreed: false.'));await click(radio('512'),'current native storage choice');await wait('current radio choice updates existing status',()=>radio('512').checked&&radioStatus().textContent.includes('Storage: 512GB.'));await click(toggle(),'existing unchanged notifications Toggle');await wait('existing Toggle still updates actual state',()=>toggle().getAttribute('aria-checked')==='true'&&radioStatus().textContent.includes('Toggle: true.'));await click(toggle(),'restore existing notifications Toggle');await wait('existing Toggle restores actual state',()=>toggle().getAttribute('aria-checked')==='false'&&radioStatus().textContent.includes('Toggle: false.'));
 assert(note()===originalNote&&note().value===noteText&&count()===originalCount&&['256','512','1024'].every((value,index)=>label(value)===labelNodes[index]),'actual textarea draft/node and current composed choice labels survive ordinary parent updates');assert(mirror().name==='agreed'&&mirror().value==='on'&&!mirror().checked&&radio('512').name==='storage'&&radio('512').value==='512'&&radio('512').checked,'current native checkbox mirror and radio name/value/checked remain correct');snapshots.retained=snapshot();checks.push('actual textarea draft/node survives choice and unchanged Toggle callbacks while current native mirror/name/value/checked stay correct');
 return{ok:true,checks,snapshots,input:'Excluded native preview imports actual shared FormExamples. Checkbox interaction is its existing native BUTTON role=checkbox with a separate named native mirror; Radio uses actual native input radio clicks/change. Synthetic DOM input/click events and explicit native focus exercise Rust callbacks and rendered semantic/composed labels. The existing textarea has no form owner; no FormData/FormEvent or native submission claim. No untrusted keyboard/Tab defaults, physical OS/AT, exact chronology, copied model/state, app/controller/callback/CSS injection or production bridge. Trusted browser workflows are separate.',renderer:navigator.userAgent};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:document.activeElement?.id,checkbox:document.getElementById('choice-checkbox-status')?.textContent,radio:document.getElementById('choice-radio-status')?.textContent}}
"#;

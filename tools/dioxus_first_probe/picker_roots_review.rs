//! Excluded actual-gallery picker roots and date-aware boundary workflows.
use super::clock_composition_review::gallery_datetime;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn PickerRootsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.picker-roots-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        style { {gallery_datetime::STYLES} }
        IxTheme { class: "picker-roots-review",
            gallery_datetime::DateTimeExamples {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_PICKER_ROOTS_CHECK);
                        if let Err(error) = query.send("pickerroots") {
                            eprintln!("Picker roots native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native Picker roots: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native Picker roots failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;
                        eprintln!("Picker roots native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_PICKER_ROOTS_CHECK: &str = r#"
let snapshots={},checks=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),root=()=>get('boundary-datetime-picker'),section=()=>get('datetime-boundaries'),status=()=>get('boundary-datetime-status'),clockState=()=>get('clock-composition-status').dataset,day=(host,label)=>host.querySelector('.ix-calendar-day[aria-label="'+label+'"]'),list=(host,unit)=>host.querySelector('.ix-clock-options[aria-label="'+unit+'"]'),option=(host,unit,value)=>[...list(host,unit).querySelectorAll('button')].find(n=>n.textContent.trim()===value),selected=(host,unit)=>list(host,unit).querySelector('[aria-selected=true]')?.textContent.trim(),done=()=>root().querySelector(':scope>.ix-date-card-footer button'),action=label=>[...section().querySelectorAll(':scope>.demo-row button')].find(n=>n.textContent.trim()===label);
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<180;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; status='+status()?.textContent)};
 const focus=async node=>{assert(node,'actual native control exists');node.scrollIntoView({block:'nearest',inline:'nearest'});node.focus();await wait('actual current target receives native focus',()=>document.activeElement===node)},click=async node=>{assert(node&&!node.disabled,'actual native control enabled');await focus(node);node.click()},key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
 const pick=async(unit,value)=>{await click(option(root(),unit,value));await wait('actual selected '+unit+' '+value,()=>selected(root(),unit)===value)},chooseDay=async(label,date)=>{await click(action(label));await wait('actual boundary date '+date,()=>status().textContent.includes('Selected '+date))},snapshot=()=>({status:status().textContent,hour:selected(root(),'Hours'),minute:selected(root(),'Minutes'),doneDisabled:done().disabled});
 const specs=[['inspection-date-picker','Inspection date','ix-date-time-card'],['inspection-time-picker','Inspection time','ix-date-time-card'],['composed-time-picker','Inspection clock','ix-date-time-card'],['boundary-datetime-picker','Date-aware time selection','ix-datetime-picker']];
 await wait('actual four named picker roots mounted',()=>specs.every(([id])=>get(id)));
 const roots=specs.map(([id])=>get(id));assert(specs.every(([id,name,css])=>get(id).getAttribute('role')==='group'&&get(id).getAttribute('aria-label')===name&&get(id).classList.contains(css)),'identity and group name belong to existing picker roots');assert(root().querySelector(':scope>.ix-datetime-pickers')&&!root().querySelector(':scope>.ix-datetime-picker'),'boundary id names actual combined root without redundant wrapper');
 const calendar=get('inspection-date-picker');await focus(day(calendar,'16 October 2026'));key(document.activeElement,'ArrowRight');await wait('actual current calendar Arrow reaches17',()=>document.activeElement===day(calendar,'17 October 2026'));document.activeElement.click();await wait('calendar native callback accepts17',()=>day(calendar,'17 October 2026').getAttribute('aria-selected')==='true'&&document.querySelector('#date-dropdown p[role=status]').textContent.includes('Selected date: 2026-10-17'));
 const clock=get('inspection-time-picker'),minute=option(clock,'Minutes','07');await focus(minute);key(minute,'ArrowDown');await wait('clock Arrow is provisional08',()=>document.activeElement===option(clock,'Minutes','08')&&clockState().draft==='13:07:04');key(document.activeElement,'Enter');await wait('clock handler accepts08',()=>clockState().draft==='13:08:04'&&clockState().confirmed==='');await click(get('composed-time-picker').querySelector('.ix-date-card-footer button'));await wait('actual composed confirmation captures current shared time',()=>clockState().confirmed==='13:08:04');snapshots.roots=specs.map(([id])=>({id,name:get(id).getAttribute('aria-label'),class:get(id).className}));checks.push('actual four named native picker roots retain direct classes; ordinary mounted date/clock keys and current shared confirmation work');
 await chooseDay('First day','2026-10-11');await wait('first-day invalid12:00 disables Done',()=>done().disabled&&option(root(),'Hours','12').disabled);await pick('Hours','13');await wait('hour selection clamps current first-day minimum13:10',()=>selected(root(),'Minutes')==='10'&&!done().disabled);assert(option(root(),'Minutes','09').disabled&&!option(root(),'Minutes','10').disabled,'first-day minute boundary is usable');await click(done());await wait('first-day confirmation exact13:10',()=>status().textContent.includes('Confirmed 2026-10-11 at 13:10'));
 await chooseDay('Middle day','2026-10-16');assert(!option(root(),'Hours','00').disabled&&!option(root(),'Hours','23').disabled,'middle day permits whole clock');await pick('Hours','18');await click(done());await wait('middle-day current18:10 confirmation',()=>status().textContent.includes('Confirmed 2026-10-16 at 18:10'));await chooseDay('Last day','2026-10-21');await wait('last-day current18:10 invalid and guard disables Done',()=>done().disabled&&option(root(),'Hours','18').disabled);await pick('Hours','17');assert(option(root(),'Minutes','31').disabled&&!option(root(),'Minutes','30').disabled,'last-day minute maximum17:30');await pick('Minutes','30');await click(done());await wait('last-day current17:30 confirmation',()=>status().textContent.includes('Confirmed 2026-10-21 at 17:30'));snapshots.boundaries=snapshot();checks.push('first/middle/last actual source controls enforce date-scoped min13:10/max17:30, reject invalid current time and confirm valid values');
 await click(action('Range selection'));await wait('actual Range control pressed',()=>action('Range selection').getAttribute('aria-pressed')==='true');await pick('Hours','00');await pick('Minutes','00');assert(!done().disabled,'range ignores time bounds with both dates retained');await click(done());await wait('actual range confirmation current endpoints and midnight',()=>status().textContent.includes('Confirmed 2026-10-11 – 2026-10-21 at 00:00'));assert(specs.every(([id],index)=>get(id)===roots[index]),'actual native root identity retained through ordinary controlled updates');snapshots.range=snapshot();checks.push('actual range mode ignores clock limits, confirms both bounded dates at midnight and retains current named roots');
 return{ok:true,checks,snapshots,viewport:{width:innerWidth,height:innerHeight},input:'Excluded synthetic DOM click/keydown with explicit native focus exercises actual unchanged DateTimeExamples state. Calendar Arrow followed by explicit .click supplies activation because untrusted Enter does not execute native button defaults; clock Enter is a handled component key. Trusted Chromium defaults are separate. No application/model/controller/callback/CSS injection; all gallery sections stay mounted and unhidden. Mounted-prop forwarding is independently proven by component tests.'};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:{id:document.activeElement?.id,label:document.activeElement?.getAttribute('aria-label')}}}
"#;

//! Excluded actual-gallery DateDropdown entry modality and ordinary range workflow.
#[path = "../../examples/gallery/date_dropdown_review.rs"]
mod gallery_dropdown;
use super::clock_composition_review::gallery_datetime;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn DateDropdownEntryPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.date-dropdown-entry-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        style { {gallery_datetime::STYLES} }
        IxTheme { class: "date-dropdown-entry-review",
            gallery_dropdown::DateDropdownReview {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_DATE_DROPDOWN_ENTRY_CHECK);
                        if let Err(error) = query.send("datedropdownentry") {
                            eprintln!("DateDropdown entry native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native DateDropdown entry: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native DateDropdown entry failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;
                        eprintln!("DateDropdown entry native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_DATE_DROPDOWN_ENTRY_CHECK: &str = r#"
let snapshots={},checks=[];
try{
 await dioxus.recv();
 const get=id=>document.getElementById(id),section=()=>get('date-dropdown-events'),trigger=()=>document.querySelector('#date-dropdown-event-review>.ix-date-dropdown>button'),popup=()=>get(trigger()?.getAttribute('aria-controls')),status=()=>get('date-dropdown-event-status'),open=()=>trigger()?.getAttribute('aria-expanded')==='true'&&popup()&&!popup().hidden,closed=()=>trigger()?.getAttribute('aria-expanded')==='false'&&popup()?.hidden,day=label=>popup()?.querySelector('.ix-calendar-day[aria-label="'+label+'"]'),preset=label=>[...popup()?.querySelectorAll('.ix-date-preset')??[]].find(n=>n.textContent.trim()===label),done=()=>popup()?.querySelector('.ix-date-dropdown-calendar>.ix-date-card-footer button');
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<190;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; status='+status()?.textContent+'; focus='+(document.activeElement?.id||document.activeElement?.getAttribute('aria-label')))};
 const focus=async node=>{assert(node,'actual current target exists');node.scrollIntoView({block:'center',inline:'nearest'});node.focus();await wait('actual native target focus',()=>document.activeElement===node)},key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
 const readyTrigger=async()=>{await focus(trigger());await wait('actual trigger fully inside native viewport',()=>{const r=trigger().getBoundingClientRect();return r.width>0&&r.top>=0&&r.bottom<=innerHeight&&r.left>=0&&r.right<=innerWidth})};
 const escape=async()=>{key(document.activeElement,'Escape');await wait('ordinary Escape closes and returns current trigger',()=>closed()&&document.activeElement===trigger())};
 const keyboardOpen=async(value,label)=>{await readyTrigger();key(trigger(),value);await wait('keyboard opening focuses current eligible selected day',()=>open()&&document.activeElement===day(label)&&!day(label).disabled)};
 const snapshot=()=>({expanded:trigger().getAttribute('aria-expanded'),triggerText:trigger().textContent.trim(),status:status().textContent,focus:{id:document.activeElement?.id,label:document.activeElement?.getAttribute('aria-label')},selected:[...popup()?.querySelectorAll('.ix-calendar-day[aria-selected=true]')??[]].map(n=>n.getAttribute('aria-label'))});
 await wait('actual DateDropdown review mounted',()=>trigger()&&status());await readyTrigger();trigger().click();await wait('initial pointer-like DOM click opens and keeps actual trigger focus',()=>open()&&document.activeElement===trigger());snapshots.initialPointer=snapshot();await escape();
 for(const value of ['Enter',' ','ArrowDown']){await keyboardOpen(value,'16 October 2026');await escape();}snapshots.keyboard=snapshot();checks.push('fully visible actual trigger pointer-like initial click keeps focus; handled Enter/Space/ArrowDown opens current calendar day and Escape returns');
 await keyboardOpen('Enter','16 October 2026');key(document.activeElement,'ArrowRight');await wait('actual calendar Arrow reaches17',()=>document.activeElement===day('17 October 2026'));document.activeElement.click();await wait('actual range first endpoint17 accepted',()=>day('17 October 2026')?.getAttribute('aria-selected')==='true');await focus(day('19 October 2026'));day('19 October 2026').click();await wait('actual second endpoint19 accepted',()=>day('19 October 2026')?.getAttribute('aria-selected')==='true');await focus(done());done().click();await wait('Done closes and publishes actual custom range',()=>closed()&&document.activeElement===trigger()&&status().textContent.includes('custom: 2026-10-17 to 2026-10-19'));await keyboardOpen(' ','17 October 2026');await escape();snapshots.custom=snapshot();checks.push('actual calendar Arrow plus explicit activation and range Done publish17–19; keyboard reopen focuses current selected17 and preserves notifications');
 await keyboardOpen('ArrowDown','17 October 2026');const week=preset('Review last 7 days');await focus(week);key(week,'Enter');week.click();await wait('keyboard modality plus explicit preset click keeps current preset focus',()=>open()&&document.activeElement===week&&week.getAttribute('aria-pressed')==='true'&&status().textContent.includes('week: 2026-10-10 to 2026-10-16')&&day('10 October 2026')?.getAttribute('aria-selected')==='true');snapshots.preset=snapshot();await escape();await keyboardOpen('Enter','10 October 2026');const outside=[...section().querySelectorAll(':scope>button')].find(n=>n.textContent.trim()==='Clear range event log');await focus(outside);outside.click();await wait('actual outside control dismisses without stealing destination focus',()=>closed()&&document.activeElement===outside);snapshots.outside=snapshot();checks.push('keyboard preset updates current range without reclaiming calendar focus; repeated keyboard reopen uses10 and outside action keeps destination');
 return{ok:true,checks,snapshots,viewport:{width:innerWidth,height:innerHeight},input:'Excluded synthetic DOM clicks/key requests and explicit native focus invoke actual DateDropdownReview. Initial pointer-like click precedes keyboard events; switching back to pointer modality is certified only by trusted Chromium. Trigger keys and Arrow/Escape exercise component handlers; calendar/preset activation uses explicit DOM click after key because untrusted Enter cannot execute native button defaults. No application/model/controller/callback/CSS injection or first offscreen focus-race/timing claim.'};
}catch(error){return{ok:false,error:String(error),checks,snapshots,focus:{id:document.activeElement?.id,label:document.activeElement?.getAttribute('aria-label')}}}
"#;

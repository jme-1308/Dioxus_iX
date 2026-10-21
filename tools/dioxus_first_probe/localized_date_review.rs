//! Excluded actual-gallery named-date inputs and bounded Linux native driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/localized_dates.rs"]
mod gallery_dates;

#[component]
pub fn LocalizedDatesPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_dates::STYLES} }
        IxTheme { class: "localized-dates-review",
            h1 { "Named dates in typed inputs" }
            Button { id: "localized-review-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "localized-review-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_dates::LocalizedDatesExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_LOCALIZED_DATES_CHECK);
                        if let Err(error) = query.send("localizeddates") {
                            eprintln!("Localized dates native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native localized dates: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native localized dates failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Localized dates native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.localized-dates-review{box-sizing:border-box;min-height:100vh;padding:24px}.localized-dates-review h1{font-size:28px;margin:0 0 24px}.localized-dates-review h2{font-size:22px;margin:24px 0 16px}.localized-dates-review h3{font-size:18px}.localized-dates-review p{max-width:72ch;line-height:1.5}#localized-review-frame{margin-top:24px}@media(max-width:600px){.localized-dates-review{padding:16px}}";

// Driver-only JS dispatches synthetic input/keyboard/click/change events, requests
// native focus and reads actual validation/picker/layout state. Parsing, bounds,
// localized labels, draft confirmation and reset remain Rust/Dioxus-owned.
// Chromium separately checks actual browser keyboard input and native defaults;
// neither route claims physical-device certification or exact event chronology.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_LOCALIZED_DATES_CHECK: &str = r#"
let snapshots={},locales={},layout={},inputFocusFacts={},surfaceFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('localized-dates'),result=()=>get('localized-result'),language=()=>get('localized-locale');
 const ids=['localized-long','localized-short','localized-weekday','localized-datetime'];
 const input=id=>get(id),field=id=>input(id)?.closest('.ix-temporal-field');
 const toggle=id=>field(id)?.querySelector('button.ix-temporal-toggle');
 const popup=id=>get(input(id)?.getAttribute('aria-controls')),opened=id=>!!popup(id)&&!popup(id).hidden&&input(id)?.getAttribute('aria-expanded')==='true',closed=id=>!!popup(id)&&popup(id).hidden&&!popup(id).querySelector('.ix-date-time-card')&&input(id)?.getAttribute('aria-expanded')==='false';
 const data=name=>result()?.dataset[name];
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; locale='+language()?.value+'; result='+result()?.textContent+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true}));
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus established',()=>active()===node)};
 const setValue=async(id,value)=>{
  const node=input(id);Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);
  node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait(id+' controlled raw text applied',()=>input(id)?.value===value);
 };
 const validity=id=>{const node=input(id);return {value:node.value,required:node.required,ariaInvalid:node.getAttribute('aria-invalid'),
  nativeValid:node.validity.valid,patternMismatch:node.validity.patternMismatch,valueMissing:node.validity.valueMissing,pattern:node.getAttribute('pattern')}};
 const snapshot=()=>({language:language().value,long:data('long'),short:data('short'),weekday:data('weekday'),datetime:data('datetime'),
  fields:Object.fromEntries(ids.map(id=>[id,validity(id)]))});
 const allSample=()=>data('long')==='2026-10-20'&&data('short')==='2026-10-20'&&data('weekday')==='2026-10-20'&&data('datetime')==='2026-10-20T13:30:00';
 const openPicker=async id=>{
  await focus(toggle(id),id+' picker trigger');toggle(id).click();
  await wait(id+' actual dialog and calendar mounted',()=>opened(id)&&popup(id)?.querySelector('.ix-calendar-grid'));
  assert(popup(id).getAttribute('role')==='dialog'&&!popup(id).hasAttribute('popover')&&toggle(id).getAttribute('aria-haspopup')==='dialog'&&toggle(id).getAttribute('aria-controls')===popup(id).id,'default inline shared Dialog matches bound toggle ARIA');
  await wait(id+' inline dialog fits actual viewport',()=>{const r=popup(id).getBoundingClientRect();return r.width>0&&r.height>0&&r.left>=-1&&r.right<=innerWidth+1&&r.top>=-1&&r.bottom<=innerHeight+1});
  const r=popup(id).getBoundingClientRect();surfaceFacts[language().value+':'+id+':'+Math.round(get('localized-review-frame').getBoundingClientRect().width)]={host:popup(id).id,role:popup(id).getAttribute('role'),topLayer:popup(id).hasAttribute('popover'),rect:{left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height},viewport:{width:innerWidth,height:innerHeight}};
 };
 const escapePicker=async id=>{
  const target=popup(id)?.querySelector('.ix-calendar-day[tabindex="0"]')||popup(id);
  await focus(target,id+' picker Escape target');key(target,'Escape');
  await wait(id+' Escape closes picker and restores trigger focus',()=>closed(id)&&active()===toggle(id));
 };
 const nextMonth=id=>popup(id)?.querySelector('.ix-calendar-header button[aria-label="Next month"]');
 const calendar=id=>popup(id)?.querySelector('.ix-calendar-grid');
 const pickDate=(id,label)=>popup(id)?.querySelector('button.ix-calendar-day[aria-label="'+label+'"]');
 const checkAllNativeValid=label=>{
  for(const id of ids){const node=input(id);assert(node.required&&node.validity.valid&&node.checkValidity()&&node.getAttribute('aria-invalid')==='false',label+' '+id+' has valid native/ARIA state');}
 };

 await wait('actual localized date inputs and ISO readouts mounted',()=>host()&&result()&&language()&&ids.every(id=>input(id)&&toggle(id))&&allSample());
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 assert(language().value==='en'&&input('localized-long').value==='20 October 2026'&&input('localized-short').value==='20 Oct 2026'&&input('localized-weekday').value==='Tuesday, 20 October 2026'&&input('localized-datetime').value==='20 October 2026 01:30 PM','English illustrative sample formats named dates and spaced12-hour appointment');
 assert(ids.every(id=>closed(id)&&popup(id).getAttribute('role')==='dialog'&&!popup(id).hasAttribute('popover')),'default inline shared Dialog hosts are persistent and hidden without picker children initially');checkAllNativeValid('initial sample');snapshots.initial=snapshot();
 await focus(input('localized-long'),'long named date');await setValue('localized-long','21 October 2026');
 await wait('typed named date normalizes to stable ISO',()=>data('long')==='2026-10-21'&&input('localized-long').validity.valid);
 inputFocusFacts={documentFocused:document.hasFocus(),active:active()?.id};
 await setValue('localized-datetime','31 October 2026 02:45 PM');
 await wait('spaced named date with AM/PM normalizes appointment',()=>data('datetime')==='2026-10-31T14:45:00'&&input('localized-datetime').validity.valid);
 snapshots.typed=snapshot();
 await setValue('localized-long','31 February 2026');
 await wait('impossible named date invalidates native constraint and ARIA',()=>data('long')==='invalid'&&input('localized-long').getAttribute('aria-invalid')==='true'&&input('localized-long').validity.patternMismatch&&!input('localized-long').validity.valid);
 await setValue('localized-weekday','Monday, 20 October 2026');
 await wait('wrong weekday rejects otherwise valid calendar date',()=>data('weekday')==='invalid'&&input('localized-weekday').getAttribute('aria-invalid')==='true'&&input('localized-weekday').validity.patternMismatch);
 snapshots.invalid=snapshot();
 await setValue('localized-weekday','Tuesday, 20 October 2026');
 await wait('matching weekday recovers normalized ISO and native pattern',()=>data('weekday')==='2026-10-20'&&input('localized-weekday').validity.valid&&!input('localized-weekday').validity.patternMismatch);
 await openPicker('localized-long');
 await wait('invalid-text calendar anchors on illustrative October date with real focus',()=>calendar('localized-long')?.getAttribute('aria-label')==='October 2026'&&popup('localized-long').contains(active())&&active()?.classList.contains('ix-calendar-day'));
 assert(popup('localized-long').querySelector('.ix-calendar-header button[aria-label="Previous month"]').disabled,'October bound disables previous-month navigation');
 nextMonth('localized-long').click();
 await wait('calendar advances to bounded November',()=>calendar('localized-long')?.getAttribute('aria-label')==='November 2026'&&pickDate('localized-long','7 November 2026'));
 assert(nextMonth('localized-long').disabled&&!pickDate('localized-long','7 November 2026').disabled,'November bound preserves enabled in-range dates and blocks later months');
 pickDate('localized-long','7 November 2026').click();
 await wait('calendar selection formats named date and returns trigger focus',()=>data('long')==='2026-11-07'&&input('localized-long').value==='07 November 2026'&&closed('localized-long')&&active()===toggle('localized-long')&&input('localized-long').validity.valid);
 await openPicker('localized-short');await escapePicker('localized-short');
 assert(data('short')==='2026-10-20','Escape preserves short-date value');

 await openPicker('localized-datetime');
 await wait('appointment calendar opens at typed October date',()=>calendar('localized-datetime')?.getAttribute('aria-label')==='October 2026');
 nextMonth('localized-datetime').click();
 await wait('appointment calendar November draft available',()=>pickDate('localized-datetime','7 November 2026'));
 pickDate('localized-datetime','7 November 2026').click();
 const clockList=unit=>popup('localized-datetime')?.querySelector('.ix-clock-options[aria-label="'+unit+'"]');
 const selectedClock=unit=>clockList(unit)?.querySelector('button[aria-selected="true"]');
 const hour=selectedClock('hr');assert(hour?.textContent.trim()==='02','typed PM hour initializes actual clock selection');
 await focus(hour,'appointment hour');key(hour,'ArrowDown');
 await wait('hour keyboard moves to03 option',()=>clockList('hr')?.contains(active())&&active().textContent.trim()==='03');
 key(active(),'Enter');await wait('hour Enter updates draft selection',()=>selectedClock('hr')?.textContent.trim()==='03');
 const minute=selectedClock('min');assert(minute?.textContent.trim()==='45','typed minute initializes actual clock selection');
 await focus(minute,'appointment minute');key(minute,'ArrowDown');
 await wait('minute keyboard moves to46 option',()=>clockList('min')?.contains(active())&&active().textContent.trim()==='46');
 key(active(),'Enter');await wait('minute Enter updates draft selection',()=>selectedClock('min')?.textContent.trim()==='46');
 assert(data('datetime')==='2026-10-31T14:45:00','calendar/clock drafts do not publish before Done');
 const done=popup('localized-datetime').querySelector('.ix-datetime-picker>.ix-date-card-footer button');assert(done&&!done.disabled,'combined picker exposes enabled Done');done.click();
 await wait('Done publishes named-date appointment with selected clock and focus return',()=>data('datetime')==='2026-11-07T15:46:00'&&input('localized-datetime').value==='07 November 2026 03:46 PM'&&closed('localized-datetime')&&active()===toggle('localized-datetime'));
 checkAllNativeValid('calendar recovery');snapshots.confirmed=snapshot();

 const expected={
  de:{long:'20 Oktober 2026',short:'20 Okt. 2026',weekday:'Dienstag, 20 Oktober 2026',month:'Oktober',day:'Dienstag',hour:'Std',minute:'Min',meridiem:'PM',typed:'21 Oktober 2026'},
  fr:{long:'20 octobre 2026',short:'20 oct. 2026',weekday:'mardi, 20 octobre 2026',month:'octobre',day:'mardi',hour:'h',minute:'min',meridiem:'PM',typed:'21 octobre 2026'},
  ja:{long:'20 10月 2026',short:'20 10月 2026',weekday:'火曜日, 20 10月 2026',month:'10月',day:'火曜日',hour:'時',minute:'分',meridiem:'午後',typed:'21 10月 2026'}
 };
 for(const [locale,text] of Object.entries(expected)){
  language().value=locale;language().dispatchEvent(new Event('change',{bubbles:true}));
  await wait(locale+' language reloads localized illustrative sample',()=>language().value===locale&&allSample()&&input('localized-long').value===text.long&&input('localized-short').value===text.short&&input('localized-weekday').value===text.weekday);
  assert(input('localized-datetime').value===text.long+' 01:30 PM',locale+' typed AM/PM representation stays canonical while names are localized');
  checkAllNativeValid(locale+' sample');await openPicker('localized-datetime');
  assert(calendar('localized-datetime').getAttribute('aria-label')===text.month+' 2026',locale+' calendar has actual localized month label');
  const weekdays=[...popup('localized-datetime').querySelectorAll('.ix-calendar-weekday[title]')].map(node=>node.title);
  assert(weekdays.includes(text.day),locale+' calendar exposes translated weekday names');
  const headings=[...popup('localized-datetime').querySelectorAll('.ix-clock-column-heading')].map(node=>node.textContent.trim());
  const meridiems=[...popup('localized-datetime').querySelectorAll('.ix-clock-meridiem button')].map(node=>node.textContent.trim());
  assert(headings[0]===text.hour&&headings[1]===text.minute&&meridiems.includes(text.meridiem),locale+' actual clock columns and PM label are translated');
  const clockNames=[...popup('localized-datetime').querySelectorAll('.ix-clock-column')].map(column=>{const heading=column.querySelector('.ix-clock-column-heading').textContent.trim(),list=column.querySelector('.ix-clock-options'),selected=list.querySelector('button[aria-selected=true]'),first=list.querySelector('button[role=option]');return{heading,list:list.getAttribute('aria-label'),selected:{text:selected.textContent.trim(),name:selected.getAttribute('aria-label')},first:{text:first.textContent.trim(),name:first.getAttribute('aria-label')}}});
  assert(clockNames.length===2&&clockNames.every((unit,index)=>unit.heading===[text.hour,text.minute][index]&&unit.list===unit.heading&&[unit.selected,unit.first].every(option=>option.name===unit.heading+': '+option.text)),locale+' live localized headings also name actual clock lists and formatted options');
  await escapePicker('localized-datetime');await setValue('localized-long',text.typed);
  await wait(locale+' typed named date parses after language change',()=>data('long')==='2026-10-21'&&input('localized-long').validity.valid);
  locales[locale]={snapshot:snapshot(),calendarMonth:text.month,weekdays,clockHeadings:headings,clockNames,meridiems};
 }
 await openPicker('localized-long');assert(popup('localized-long'),'calendar is open before Reset');
 get('localized-reset').focus();get('localized-reset').click();
 await wait('Restore sample clears transient picker and preserves selected language',()=>allSample()&&language().value==='ja'&&ids.every(closed)&&!host().querySelector('.ix-temporal-backdrop')&&active()===get('localized-reset'));
 checkAllNativeValid('restored sample');snapshots.reset=snapshot();
 get('localized-review-narrow').click();
 await wait('Rust-owned narrow fixture contains all actual fields',()=>get('localized-review-frame').getBoundingClientRect().width<=341&&ids.every(id=>field(id).getBoundingClientRect().width<=341));
 layout={frame:get('localized-review-frame').getBoundingClientRect().width,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth,
  fields:Object.fromEntries(ids.map(id=>{const box=field(id).getBoundingClientRect();return [id,{width:box.width,height:box.height}]}))};
 assert(layout.pageScroll<=layout.pageClient+1,'named-date fields and readouts do not overflow page');await openPicker('localized-datetime');await escapePicker('localized-datetime');assert(allSample(),'narrow default inline combined picker closes without changing restored sample');
 return {ok:true,snapshots,locales,layout,inputFocusFacts,surfaceFacts,
  checks:['default inline shared Dialog ARIA links, persistent hidden hosts and wide/narrow viewport containment','four actual named-date sample fields and stable ISO readouts','typed long date and spaced AM/PM datetime','impossible date and weekday mismatch native pattern/ARIA validation','calendar bounded navigation/selection and Escape focus return','combined calendar/hour/minute draft and Done confirmation','German/French/Japanese named input parsing and translated calendar/clock labels','Restore sample language/state/picker cleanup','native validity and narrow field containment'],
  input:'synthetic native input/click/keydown/change events with explicit focus and actual validation/picker layout',
  trustedInput:'not established by this excluded synthetic replay; any current trusted browser evidence is recorded separately by the lead',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),snapshots,locales,layout,inputFocusFacts,surfaceFacts,documentFocusedNow:document.hasFocus()}}
"#;

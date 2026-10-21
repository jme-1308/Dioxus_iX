//! Excluded real native pointer hover over the actual shared calendar/clock gallery.
use super::clock_composition_review::gallery_datetime;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn SelectedHoverPreview() -> Element {
    let light = use_hook(|| std::env::var("DIOXUS_IX_HOVER_LIGHT").is_ok_and(|value| value == "1"));
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.selected-hover-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        style { {gallery_datetime::STYLES} }
        IxTheme { class: "selected-hover-review", scheme: if light { ColorScheme::Light } else { ColorScheme::Dark },
            gallery_datetime::DateTimeExamples {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    window.set_inner_size(dioxus::desktop::LogicalSize::new(1000.0, 850.0));
                    window.set_focus();
                    spawn(async move {
                        let mut query = dioxus::document::eval(NATIVE_SELECTED_HOVER_CHECK);
                        if let Err(error) = query.send(serde_json::json!({"kind":"start","light":light})) {
                            eprintln!("Selected hover native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        loop {
                            let message = match query.recv::<serde_json::Value>().await {
                                Ok(message) => message,
                                Err(error) => {
                                    eprintln!("Selected hover native channel failed: {error:?}");
                                    std::process::exit(1);
                                }
                            };
                            match message["kind"].as_str() {
                                Some("move") => {
                                    let point = message["x"].as_f64().zip(message["y"].as_f64());
                                    let reply = match point {
                                        Some((x, y)) if x.is_finite() && y.is_finite() && x >= 0.0 && y >= 0.0 => {
                                            window.set_focus();
                                            match window.set_cursor_position(dioxus::desktop::LogicalPosition::new(x, y)) {
                                                Ok(()) => serde_json::json!({"kind":"move","ok":true}),
                                                Err(error) => serde_json::json!({"kind":"move","ok":false,"error":error.to_string()}),
                                            }
                                        }
                                        _ => serde_json::json!({"kind":"move","ok":false,"error":"invalid current-window point"}),
                                    };
                                    if let Err(error) = query.send(reply) {
                                        eprintln!("Selected hover native move acknowledgement failed: {error:?}");
                                        std::process::exit(1);
                                    }
                                }
                                Some("done") => {
                                    let result = message["value"].clone();
                                    if let Err(error) = query.send(serde_json::json!({"kind":"done","ok":true})) {
                                        eprintln!("Selected hover final acknowledgement failed: {error:?}");
                                        std::process::exit(1);
                                    }
                                    match query.join::<serde_json::Value>().await {
                                        Ok(joined) if result["ok"] == true && joined["ok"] == true => {
                                            println!("Dioxus native Selected hover: {result}");
                                            window.close();
                                        }
                                        joined => {
                                            eprintln!("Dioxus native Selected hover failed: {result}; join={joined:?}");
                                            std::process::exit(1);
                                        }
                                    }
                                    break;
                                }
                                _ => {
                                    eprintln!("Selected hover native unknown request: {message}");
                                    std::process::exit(1);
                                }
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(55)).await;
                        eprintln!("Selected hover native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SELECTED_HOVER_CHECK: &str = r#"
let facts={},checks=[],result;
try{
 const start=await dioxus.recv();
 const get=id=>document.getElementById(id),single=()=>get('inspection-date-picker'),range=()=>document.querySelector('#date-picker>.demo-date-row>div:nth-child(2)'),clock=()=>get('inspection-time-picker'),third=()=>document.querySelector('#time-picker>.demo-date-row>div:nth-child(3)'),day=(host,label)=>host.querySelector('.ix-calendar-day[aria-label="'+label+'"]'),option=(host,unit,value)=>[...host.querySelectorAll('.ix-clock-options[aria-label="'+unit+'"] button')].find(n=>n.textContent.trim()===value);
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<200;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label)};
 const rgb=value=>{if(value.startsWith('color(srgb'))return value.match(/[\d.]+/g).slice(0,3).map(Number);const values=value.match(/[\d.]+/g);assert(values?.length>=3,'computed opaque RGB color '+value);return values.slice(0,3).map(n=>Number(n)/255)},luminance=value=>rgb(value).map(n=>n<=0.04045?n/12.92:((n+0.055)/1.055)**2.4).reduce((sum,n,index)=>sum+n*[0.2126,0.7152,0.0722][index],0),style=node=>{const s=getComputedStyle(node),a=luminance(s.color),b=luminance(s.backgroundColor);return{background:s.backgroundColor,color:s.color,opacity:s.opacity,contrast:(Math.max(a,b)+0.05)/(Math.min(a,b)+0.05),hover:node.matches(':hover'),selected:node.getAttribute('aria-selected')}};
 const move=async node=>{assert(node,'actual hover target exists');node.scrollIntoView({block:'center',inline:'nearest'});await wait('actual hover target exposed inside current window',()=>{const r=node.getBoundingClientRect(),hit=document.elementFromPoint(r.left+r.width/2,r.top+r.height/2);return r.width>0&&r.height>0&&r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight&&(hit===node||node.contains(hit))});const r=node.getBoundingClientRect();dioxus.send({kind:'move',x:r.left+r.width/2,y:r.top+r.height/2});const reply=await dioxus.recv();assert(reply.kind==='move'&&reply.ok===true,'native cursor movement unsupported or rejected: '+JSON.stringify(reply));await wait('real native pointer establishes :hover',()=>node.matches(':hover'));};
 const away=async()=>move(document.querySelector('#date-picker>h2'));
 const selectedHover=async(node,label)=>{await away();node.scrollIntoView({block:'center',inline:'nearest'});await wait('selected style before hover',()=>!node.matches(':hover'));const before=style(node);await move(node);const after=style(node);assert(after.background===before.background&&after.color===before.color&&after.contrast>=4.5,label+' selected real hover retains own colors and readable contrast');return{before,after}};
 await wait('actual named calendar/clock roots mounted',()=>single()&&range()&&clock()&&third());await wait('actual1000px native window',()=>innerWidth>=950&&innerWidth<=1050&&innerHeight>=700);assert(document.querySelector('.selected-hover-review').dataset.ixColorSchema===(start.light?'light':'dark'),'actual native theme matches process request');
 facts.single=await selectedHover(day(single(),'16 October 2026'),'single16');checks.push('actual native pointer hover over selected single calendar16 retains accent/onaccent colors and >=4.5 contrast');
 facts.range=await selectedHover(day(range(),'16 October 2026'),'range endpoint16');await move(day(single(),'15 October 2026'));const ordinary=style(day(single(),'15 October 2026'));await move(day(range(),'13 October 2026'));const interior=style(day(range(),'13 October 2026'));assert(interior.background===ordinary.background,'ordinary/interior calendar hover retains common neutral surface');facts.neutral={ordinary,interior};checks.push('actual selected range endpoint remains readable; ordinary and range-interior real hover keep neutral surface');
 facts.clock=await selectedHover(option(clock(),'hr','13'),'clock13');await move(option(clock(),'hr','12'));facts.ordinaryClock=style(option(clock(),'hr','12'));assert(facts.ordinaryClock.background===ordinary.background,'ordinary clock hover retains neutral surface');checks.push('actual selected clock13 real hover retains accent/onaccent >=4.5 while ordinary12 stays neutral');
 const disabled=option(third(),'hr','07');assert(disabled.disabled,'existing bounded clock07 is disabled');await move(third().querySelector('h3'));await wait('bounded current13 reveal settles before disabled scroll',()=>{const n=option(third(),'hr','13'),r=n.getBoundingClientRect(),list=n.closest('.ix-clock-options').getBoundingClientRect();return r.top>=list.top&&r.bottom<=list.bottom});disabled.scrollIntoView({block:'center'});const disabledBefore=style(disabled);await move(disabled);const disabledAfter=style(disabled);assert(disabledAfter.background===disabledBefore.background&&disabled.disabled,'disabled native hover leaves guard/background intact');const current=option(clock(),'hr','13');current.scrollIntoView({block:'center'});current.focus();current.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true,cancelable:true}));await wait('actual native provisional focus14',()=>document.activeElement===option(clock(),'hr','14')&&get('clock-composition-status').dataset.draft==='13:07:04');document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));await wait('handled native clock Enter commits14',()=>get('clock-composition-status').dataset.draft==='14:07:04');facts.afterKeyboard=await selectedHover(option(clock(),'hr','14'),'clock14 after key');facts.disabled={before:disabledBefore,after:disabledAfter};checks.push('disabled real native hover retains guard; synthetic handled provisional Arrow/Enter remains usable and new selected14 hover remains readable');
 result={ok:true,checks,facts,theme:start.light?'light':'dark',viewport:{width:innerWidth,height:innerHeight},input:'Real automated native X11 pointer moved within this own desktop window through pinned Tao set_cursor_position; actual :hover/computed colors observed. Focus/Arrow/Enter requests are excluded synthetic component-key input, not physical human/default-key/AT proof. No CSS/pseudostate/application/model/controller/callback injection or fallback; unsupported native pointer movement fails explicitly.'};
}catch(error){result={ok:false,error:String(error),checks,facts}}
dioxus.send({kind:'done',value:result});const finished=await dioxus.recv();if(finished.kind!=='done'||!finished.ok)throw Error('native final acknowledgement missing');return result;
"#;

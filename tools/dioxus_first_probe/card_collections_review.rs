//! Excluded actual-gallery card collection and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/card_collections.rs"]
mod gallery_cards;

#[component]
pub fn CardCollectionsPreview() -> Element {
    let mut narrow = use_signal(|| false);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_cards::STYLES} }
        IxTheme { class: "card-collections-review",
            h1 { "Dioxus maintenance card collections" }
            Button { id: "card-collections-narrow", variant: ButtonVariant::Secondary,
                onclick: move |_| narrow.toggle(), if narrow() { "Use wide preview" } else { "Use narrow preview" }
            }
            div { id: "card-collections-frame", style: if narrow() { "width:340px;max-width:100%;min-width:0" } else { "width:960px;max-width:100%;min-width:0" },
                gallery_cards::CardCollectionsExample {}
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CARD_COLLECTIONS_CHECK);
                        if let Err(error) = query.send("cardcollections") {
                            eprintln!("Card collection native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native card collections: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native card collections failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Card collection native driver timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.card-collections-review{box-sizing:border-box;min-height:100vh;padding:24px}.card-collections-review h1{font-size:28px;margin:0 0 24px}.card-collections-review h2{font-size:22px;margin:24px 0 16px}.card-collections-review h3{font-size:18px}.card-collections-review p{max-width:72ch;line-height:1.5}#card-collections-frame{margin-top:24px}@media(max-width:600px){.card-collections-review{padding:16px}}";

// Excluded driver only: DOM clicks execute native button activation; synthetic
// keyboard events establish default permission, not trusted Enter/Space/Tab.
// Chromium separately exercises actual keyboard traversal and native activation.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CARD_COLLECTIONS_CHECK: &str = r#"
let snapshots={},layout={},keyPermissions={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const host=()=>get('card-collections'),result=()=>get('cards-result');
 const list=()=>host()?.querySelector('.maintenance-card-list');
 const content=()=>list()?.querySelector('.ix-card-list-items');
 const entries=()=>[...list()?.querySelectorAll('[data-card-list-entry]')??[]];
 const entry=id=>entries().find(node=>node.dataset.cardListEntry===id);
 const rendered=node=>!!node&&node.isConnected&&node.getClientRects().length>0&&getComputedStyle(node).visibility!=='hidden'&&!node.closest('[hidden]');
 const visible=()=>entries().filter(rendered).map(node=>node.dataset.cardListEntry);
 const header=()=>list()?.querySelector('.ix-card-list-show-all');
 const collapse=()=>list()?.querySelector('.ix-card-list-header > .ix-disclosure-button');
 const more=()=>list()?.querySelector('button.ix-card-list-more');
 const note=id=>get('cards-note-'+id);
 const button=(id,label)=>[...entry(id)?.querySelectorAll('button')??[]].find(node=>node.getAttribute('aria-label')===label||node.textContent.trim()===label);
 const accordion=id=>entry(id)?.querySelector('.ix-card-accordion button');
 const data=name=>result()?.dataset[name],number=name=>Number(data(name));
 const checked=node=>node.checked;
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;if(stable===3)return;await hold(20);
  }
  throw Error(label+'; visible='+JSON.stringify(visible())+'; host='+result()?.textContent+'; focus='+(active()?.getAttribute('aria-label')||active()?.id||active()?.tagName));
 };
 const focus=async(node,label)=>{assert(node,label+' exists');node.focus();await wait(label+' native focus',()=>active()===node)};
 const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,code:value===' '?'Space':value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event};
 const click=async(node,label)=>{assert(node&&rendered(node)&&!node.disabled,label+' is an available rendered control');node.click();await hold(30)};
 const input=async(node,value)=>{assert(node&&rendered(node),'native note input rendered');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(node,value);node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));await wait('native note input value applied',()=>node.value===value)};
 const select=async(id,value)=>{const node=get(id);assert(node&&rendered(node),'native '+id+' rendered');node.value=value;node.dispatchEvent(new Event('change',{bubbles:true}));await wait(id+' native selection applied',()=>get(id)?.value===value)};
 const option=async(id,value)=>{if(checked(get(id))!==value)await click(get(id),id);await wait(id+' checked state',()=>checked(get(id))===value)};
 const snapshot=()=>({requests:number('requests'),accepted:number('accepted'),showAll:data('showAll'),collapsed:data('collapsed'),selected:data('selected'),inspections:number('inspections'),last:data('last'),count:number('count'),limit:number('limit'),visible:visible(),order:entries().map(node=>node.dataset.cardListEntry),active:active()?.getAttribute('aria-label')||active()?.dataset.cardListEntry||active()?.id});
 const reset=async()=>{await click(get('cards-reset'),'Restore collection');await wait('restored collection clears host and local card state',()=>number('requests')===0&&number('accepted')===0&&number('inspections')===0&&data('selected')==='none'&&number('count')===7&&number('limit')===3&&visible().join(',')==='device-1,device-2,device-3'&&note('device-1')?.value===''&&accordion('device-1')?.getAttribute('aria-expanded')==='false'&&!!more())};

 await wait('actual card collection gallery mounted',()=>host()&&list()&&result()&&entries().length===7&&note('device-1')&&more());
 await wait('WebView document ready for native focus',()=>document.hasFocus());
 assert(visible().join(',')==='device-1,device-2,device-3'&&Number(list().dataset.visibleCount)===3&&Number(list().dataset.count)===7,'actual rendered limit differs from total mounted cards');
 assert(entries().every(node=>node.getAttribute('role')==='listitem'&&node.getAttribute('tabindex')==='-1'&&!!node.getAttribute('aria-label')),'stable item wrappers expose named listitem focus targets');
 assert(!rendered(note('device-4'))&&entry('device-4').hidden,'limited card content remains mounted but hidden');
 assert(header()?.textContent.includes('Show all')&&header()?.textContent.includes('7')&&rendered(more()),'header total and real More card available');
 snapshots.initial=snapshot();

 await focus(more(),'More card');const enter=key(active(),'Enter');keyPermissions.moreEnter=!enter.defaultPrevented;
 assert(!enter.defaultPrevented,'native More button retains Enter default permission');
 await click(more(),'native More activation');
 await wait('More reveals actual seven cards and focuses first newly visible wrapper',()=>visible().length===7&&active()===entry('device-4')&&number('requests')===1&&number('accepted')===1&&data('last')==='more'&&!rendered(more()));
 const tab=key(active(),'Tab');keyPermissions.wrapperTab=!tab.defaultPrevented;assert(!tab.defaultPrevented,'newly visible wrapper permits native Tab to composed controls');
 await focus(button('device-4','Inspect Sensor 004'),'newly revealed Inspect');await click(active(),'Inspect Sensor 004');
 await wait('composed Inspect is usable without primary selection',()=>number('inspections')===1&&data('selected')==='none');
 snapshots.moreReveal=snapshot();

 await click(header(),'Show less');await wait('Show less restores limit without unmounting hidden entries',()=>visible().length===3&&entries().length===7&&!!more()&&number('accepted')===2);
 await click(header(),'header Show all');await wait('header reveal emits its own accepted source and wrapper focus',()=>visible().length===7&&active()===entry('device-4')&&data('last')==='header'&&number('requests')===3&&number('accepted')===3);
 await focus(note('device-4'),'hidden-state note');await input(note('device-4'),'Inspect sensor cable');
 await click(accordion('device-4'),'sensor details');await wait('composed accordion expands without selecting card',()=>accordion('device-4').getAttribute('aria-expanded')==='true'&&data('selected')==='none');
 await click(button('device-4','Select Sensor 004'),'primary sensor selection');await wait('explicit primary card action selects stable identity',()=>data('selected')==='device-4');
 await click(header(),'Show less');await wait('sensor hidden after show less',()=>!rendered(note('device-4'))&&visible().length===3);
 await click(more(),'reveal retained composed state');await wait('hidden composed note, accordion and selection preserved',()=>visible().length===7&&note('device-4').value==='Inspect sensor cable'&&accordion('device-4').getAttribute('aria-expanded')==='true'&&data('selected')==='device-4');
 snapshots.retainedComposition=snapshot();

 await reset();const options=get('cards-options');assert(options?.tagName==='DETAILS','advanced options use actual native details');
 if(!options.open)await click(options.querySelector('summary'),'Collection options summary');await wait('native summary opens actual options',()=>options.open&&rendered(get('cards-veto')));
 await option('cards-veto',true);await focus(header(),'veto header focus');await click(header(),'veto header request');
 await wait('header veto records request without accepting, revealing or moving focus',()=>number('requests')===1&&number('accepted')===0&&data('last')==='header'&&visible().length===3&&active()===header());
 await focus(more(),'veto More focus');await click(more(),'veto More request');
 await wait('More veto retains its real button and focus',()=>number('requests')===2&&number('accepted')===0&&data('last')==='more'&&visible().length===3&&active()===more());
 await option('cards-veto',false);await option('cards-hold',true);await focus(header(),'held header');await click(header(),'held header request');
 await wait('accepted controlled-held header does not reveal or focus hidden items',()=>number('requests')===3&&number('accepted')===1&&visible().length===3&&active()===header());
 await focus(more(),'held More');await click(more(),'held More request');
 await wait('accepted controlled-held More keeps actual limited view',()=>number('requests')===4&&number('accepted')===2&&visible().length===3&&active()===more());
 snapshots.vetoAndHeld=snapshot();await option('cards-hold',false);

 await select('cards-limit','2');await wait('runtime limit immediately hides actual cards',()=>number('limit')===2&&visible().join(',')==='device-1,device-2'&&!rendered(note('device-3')));
 await click(get('cards-append'),'Add asset');await wait('append updates actual stable children and total',()=>number('count')===8&&entries().length===8&&!!entry('device-8')&&visible().length===2&&header().textContent.includes('8'));
 await click(get('cards-remove'),'Remove last asset');await wait('remove releases appended item',()=>number('count')===7&&entries().length===7&&!entry('device-8'));
 await input(note('device-1'),'Keep motor note');await click(get('cards-reverse'),'Reverse assets');
 await wait('reorder changes visible stable IDs while preserving mounted motor note',()=>visible().join(',')==='device-7,device-6'&&note('device-1').value==='Keep motor note');
 await click(header(),'reveal reordered cards');await wait('reordered reveal focuses new stable boundary item',()=>visible().length===7&&active()===entry('device-5')&&note('device-1').value==='Keep motor note');
 await click(get('cards-reverse'),'restore asset ordering');await wait('normal stable order restored',()=>entries()[0]?.dataset.cardListEntry==='device-1'&&note('device-1').value==='Keep motor note');
 await click(header(),'Show less before options');await wait('two-card limit restored',()=>visible().length===2);
 snapshots.dynamicItems=snapshot();

 await option('cards-hide-show-all',true);await wait('hidden header action retains independent More reveal',()=>!rendered(header())&&rendered(more()));
 await click(more(),'More without header action');await wait('More remains usable with header action hidden',()=>visible().length===7&&active()===entry('device-3'));
 await option('cards-hide-show-all',false);await click(header(),'Show less with restored header');await wait('limited view before suppression',()=>visible().length===2);
 await option('cards-suppress',true);await wait('suppression renders all cards without overflow More',()=>visible().length===7&&!rendered(more()));
 await option('cards-suppress',false);await wait('releasing suppression restores actual limit',()=>visible().length===2&&rendered(more()));
 await option('cards-german',true);await wait('localized native labels reach header and More',()=>header()?.textContent.includes('Alle anzeigen')&&!!more()&&((more().getAttribute('aria-label')||'')+' '+more().textContent).includes('Weitere Karten verfügbar'));
 await click(header(),'localized show all');await wait('localized Show less reflects actual reveal',()=>visible().length===7&&header()?.textContent.includes('Weniger anzeigen'));
 snapshots.optionsAndLabels=snapshot();

 await focus(note('device-1'),'inside note before external collapse');await click(get('cards-collapse'),'programmatic Collapse collection');
 await wait('external collapse hides actual content and returns inside focus to its mounted header',()=>visible().length===0&&Number(list().dataset.visibleCount)===0&&content().hidden&&active()===collapse()&&collapse().getAttribute('aria-expanded')==='false');
 await click(collapse(),'native collapsed header expand');await wait('header expands collection normally',()=>visible().length===7&&collapse().getAttribute('aria-expanded')==='true');
 await focus(get('cards-limit'),'outside collection native select');await click(get('cards-collapse'),'outside-focus programmatic collapse');
 await wait('external collapse preserves independently focused outside control',()=>visible().length===0&&active()===get('cards-limit'));
 await click(get('cards-expand'),'programmatic Expand collection');await wait('outside expand restores collection without stealing focus',()=>visible().length===7&&active()===get('cards-limit'));
 snapshots.collapseFocus=snapshot();

 await reset();await focus(more(),'More before external collapse');await click(get('cards-collapse'),'collapse while More focused');
 await wait('More focus also returns to mounted collapse header',()=>visible().length===0&&active()===collapse());
 await reset();snapshots.reset=snapshot();
 await click(get('card-collections-narrow'),'narrow preview');
 await wait('narrow stack contains actual cards',()=>get('card-collections-frame').getBoundingClientRect().width<=341&&visible().length===3&&entry('device-1').getBoundingClientRect().width<=341);
 await focus(note('device-1'),'narrow native note');await input(note('device-1'),'Narrow motor check');
 await click(more(),'narrow stack More');await wait('narrow stack reveals usable real content',()=>visible().length===7&&active()===entry('device-4')&&note('device-1').value==='Narrow motor check');
 await click(header(),'narrow Show less');await wait('narrow stack limit restored',()=>visible().length===3);
 await select('cards-layout','scroll');await wait('limited scroll layout uses actual horizontal overflow',()=>list().dataset.scroll==='true'&&content().scrollWidth>content().clientWidth);
 more().scrollIntoView({block:'center',inline:'center'});await hold(50);
 const box=more().getBoundingClientRect(),hit=document.elementFromPoint(box.left+box.width/2,box.top+box.height/2);
 assert(rendered(more())&&(hit===more()||more().contains(hit)),'narrow scroll More remains reachable and hit-testable');
 layout={frameWidth:get('card-collections-frame').getBoundingClientRect().width,cardWidth:entry('device-1').getBoundingClientRect().width,clientWidth:content().clientWidth,scrollWidth:content().scrollWidth,scrollLeft:content().scrollLeft,pageClient:document.documentElement.clientWidth,pageScroll:document.documentElement.scrollWidth};
 assert(layout.pageScroll<=layout.pageClient+1,'narrow collection remains inside page width');
 await click(more(),'narrow horizontal More');await wait('scroll reveal exposes all real cards and preserves note',()=>visible().length===7&&active()===entry('device-4')&&note('device-1').value==='Narrow motor check');
 snapshots.narrow=snapshot();
 return {ok:true,snapshots,layout,keyPermissions,
  checks:['real limited visibility with stable mounted entries and header/More counts','native More activation and newly shown wrapper focus/Tab permission','composed Inspect without primary selection','header reveal and Show less','note/accordion/selection retained while hidden','cancelable header and More requests','controlled held display without hidden focus','runtime limit/add/remove/reorder with retained state','More remains available with hidden header action','suppressed overflow and localized controls','external collapse returns inside focus and preserves outside focus','Restore collection resets host and local component state','More focus return on programmatic collapse','usable narrow stack input/reveal and horizontal scroll More'],
  input:'native DOM click, synthetic input/change/key events and explicit native focus; untrusted Enter/Space/Tab defaults are not fabricated',
  trustedInput:'separate Chromium actual keyboard activation/traversal and typing; no physical-device certification',renderer:navigator.userAgent};
}catch(error){return {ok:false,error:String(error),snapshots,layout,keyPermissions,documentFocusedNow:document.hasFocus()}}
"#;

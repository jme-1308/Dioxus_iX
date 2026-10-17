//! Bounded Linux/WebView regression for B10.4. Input/focus events are synthetic;
//! this verifies renderer wiring, not physical keyboard or assistive technology.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/filter_order_review.rs"]
mod filter_order_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX filter order probe")
                .with_visible(true)
                .with_inner_size(dioxus::desktop::LogicalSize::new(900.0, 1000.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(40)).await;
            eprintln!("B10.4 native probe timed out");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { onmounted: move |_| {
            let desktop = desktop.clone();
            spawn(async move {
                let query = dom_query::eval(VERIFY);
                let _ = query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"] == true => {
                        println!("B10.4 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("B10.4 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, filter_order_review::FilterOrderReview {} }
    } }
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const get=id=>document.getElementById(id),root=kind=>get(`b10-order-${kind}`),input=kind=>root(kind)?.querySelector('.ix-category-filter-text');
 const open=kind=>root(kind).querySelector('.ix-category-filter').getAttribute('data-open')==='true';
 const events=()=>JSON.parse(get('b10-order-events').textContent),values=()=>JSON.parse(get('b10-order-values').textContent);
 const nodes=kind=>[...root(kind).querySelectorAll('.ix-category-filter-chip')],labels=kind=>nodes(kind).map(n=>n.querySelector('span').textContent.trim()),closes=kind=>nodes(kind).map(n=>n.querySelector('button')).filter(Boolean);
 const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; local='+JSON.stringify(labels('local'))+'; controlled='+JSON.stringify(labels('controlled'))+'; values='+JSON.stringify(values())+'; events='+JSON.stringify(events()))};
 const wait=async(test,label)=>{const end=performance.now()+3000;while(!test()){if(performance.now()>end)throw Error(label+'; local='+JSON.stringify(labels('local'))+'; controlled='+JSON.stringify(labels('controlled'))+'; events='+JSON.stringify(events()));await sleep(20)}};
 const focus=kind=>{input(kind).focus();input(kind).dispatchEvent(new FocusEvent('focusin',{bubbles:true}));};
 const edit=(kind,value)=>{input(kind).value=value;input(kind).dispatchEvent(new InputEvent('input',{bubbles:true,data:value,inputType:'insertText'}));};
 const key=(node,key)=>node.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,cancelable:true,key,code:key===' '?'Space':key}));
 const commit=async(kind,value,count)=>{edit(kind,value);key(input(kind),'Enter');await wait(()=>nodes(kind).length===count,'commit '+kind+' '+value);};
 const choose=async kind=>{focus(kind);if(!open(kind))input(kind).click();await wait(()=>[...root(kind).querySelectorAll('button.ix-category-filter-option')].some(n=>n.textContent.trim()==='Vendor'),'Vendor choice');[...root(kind).querySelectorAll('button.ix-category-filter-option')].find(n=>n.textContent.trim()==='Vendor').click();await wait(()=>root(kind).querySelector('.ix-category-filter-preview')?.textContent==='Vendor','Vendor preview');};
 const fresh=async()=>{const old=input('local');get('b10-order-reset').click();await wait(()=>input('local')!==old&&events().length===0&&nodes('local').length===0&&nodes('controlled').length===0,'fresh review');};
 const mixed=async kind=>{focus(kind);await commit(kind,'first',1);await choose(kind);await commit(kind,'Alpha',2);await commit(kind,'last',3);assert(same(labels(kind),['first','Vendor = Alpha','last']),'interactive chronological order '+kind);assert(same(events().filter(e=>e.filter===kind&&e.event==='oncategory').map(e=>e.value),['vendor']),'only selection category event '+kind);};
 await wait(()=>input('local')&&input('controlled'),'mount');await fresh();await mixed('local');
 let start=events().length;key(input('local'),'Backspace');await wait(()=>nodes('local').length===2,'last plain backspace');assert(same(labels('local'),['first','Vendor = Alpha']),'reverse chronological last plain');key(input('local'),'Backspace');await wait(()=>nodes('local').length===1,'last category backspace');key(input('local'),'Backspace');await wait(()=>nodes('local').length===0,'first plain backspace');key(input('local'),'Backspace');await sleep(60);assert(same(events().slice(start).map(e=>e.event),['onchange','onchange','onchange']),'backspace one change each, empty silent');
 await choose('local');const operator=()=>root('local').querySelector('[aria-label="Toggle comparison operator"]');operator().click();await wait(()=>operator()?.getAttribute('aria-pressed')==='true','preview !=');start=events().length;key(input('local'),'Backspace');await wait(()=>!root('local').querySelector('.ix-category-filter-preview'),'preview backspace');assert(events().length===start,'preview backspace silent');await choose('local');assert(operator()?.getAttribute('aria-pressed')==='true','preview backspace retains operator');
 // Typed Enter from a closed dropdown must not open it or emit category(None).
 edit('local','closed commit');await sleep(40);key(input('local'),'Escape');await wait(()=>!open('local'),'Escape closes');start=events().length;key(input('local'),'Enter');await wait(()=>nodes('local').length===1,'closed Enter commit');await sleep(60);assert(!open('local')&&same(events().slice(start).map(e=>e.event),['onchange']),'closed Enter stays closed and emits only change');
 await fresh();get('b10-order-programmatic').click();await wait(()=>get('b10-order-command-result').textContent==='Ok'&&nodes('local').length===4,'mixed command');assert(same(labels('local'),['first','Vendor = Alpha','middle','Vendor = Alpha']),'programmatic reserved-token chronology');
 const original=nodes('local'),firstDuplicate=original[1],lastDuplicate=original[3];const removed=closes('local')[1];removed.focus();start=events().length;removed.click();await wait(()=>nodes('local').length===3,'first duplicate removal');await sleep(80);assert(!firstDuplicate.isConnected&&lastDuplicate.isConnected&&same(labels('local'),['first','middle','Vendor = Alpha']),'only clicked duplicate occurrence removed');assert(same(events().slice(start).map(e=>e.event),['onchange'])&&document.activeElement!==input('local'),'removal one change no input focus');
 const middleButton=closes('local')[1];middleButton.focus();start=events().length;key(middleButton,'Enter');await wait(()=>nodes('local').length===2,'keyboard Enter close');await sleep(60);assert(same(labels('local'),['first','Vendor = Alpha'])&&same(events().slice(start).map(e=>e.event),['onchange']),'Enter closes exact occurrence once');const spaceButton=closes('local')[1];spaceButton.focus();start=events().length;key(spaceButton,' ');await wait(()=>nodes('local').length===1,'Space close');assert(same(events().slice(start).map(e=>e.event),['onchange']),'Space one change');
 const retained=closes('local')[0];retained.focus();start=events().length;key(retained,'Backspace');key(retained,'ArrowDown');await sleep(60);assert(nodes('local').length===1&&events().length===start,'nonactivation chip keys do not remove');
 get('b10-order-disabled').click();await wait(()=>input('local').disabled,'disabled');assert(closes('local').length===0,'disabled no close controls');start=events().length;key(input('local'),'Backspace');await sleep(40);assert(nodes('local').length===1&&events().length===start,'disabled guard');get('b10-order-disabled').click();get('b10-order-read-only').click();await wait(()=>input('local').readOnly,'readonly');assert(closes('local').length===0,'readonly no close controls');start=events().length;key(input('local'),'Backspace');await sleep(40);assert(nodes('local').length===1&&events().length===start,'readonly guard');
 await fresh();await mixed('controlled');get('b10-order-reject').click();await sleep(40);focus('controlled');edit('controlled','pending last');key(input('controlled'),'Enter');await wait(()=>values().controlled_proposal.tokens.includes('pending last'),'controlled rejected append');assert(same(labels('controlled'),['first','Vendor = Alpha','last']),'rejection retains authoritative chronology');const eventCount=events().length;get('b10-order-accept-last').click();await wait(()=>nodes('controlled').length===4,'delayed append acceptance');assert(same(labels('controlled'),['first','Vendor = Alpha','last','pending last'])&&events().length===eventCount,'delayed acceptance preserves proposed chronology without extra event');
 const heldClose=closes('controlled')[1];heldClose.focus();heldClose.click();await wait(()=>values().controlled_proposal.categories.length===0,'rejected category removal');assert(same(labels('controlled'),['first','Vendor = Alpha','last','pending last']),'rejected removal preserves authoritative chronology');const removalEvents=events().length;get('b10-order-accept-last').click();await wait(()=>nodes('controlled').length===3,'delayed removal acceptance');assert(same(labels('controlled'),['first','last','pending last'])&&events().length===removalEvents,'accepted removal chronology without extra event');
 await fresh();focus('local');await sleep(80);const scrollY=window.scrollY;for(let index=1;index<=30;index++){await commit('local',`token-${String(index).padStart(2,'0')}`,index);}const list=()=>root('local').querySelector('.ix-category-filter-tokens');await wait(()=>list().scrollTop>0&&Math.abs(list().scrollTop-(list().scrollHeight-list().clientHeight))<=2,'interactive token scroll to end');const a=list().getBoundingClientRect(),b=input('local').getBoundingClientRect();assert(b.top>=a.top-2&&b.bottom<=a.bottom+2&&window.scrollY===scrollY&&document.activeElement===input('local'),'interactive scrolling scoped, input visible/focus retained');assert(events().filter(e=>e.event==='onchange').length===30&&events().every(e=>e.event!=='oncategory'),'interactive input/Enter lossless with exactly one change per append');
 return {ok:true,checks:['interactive and programmatic chronology','reserved custom ID retains late position','reverse Backspace and silent preview exit','closed-popup Enter stays closed','exact duplicate occurrence removal','Enter/Space close once and other keys guarded','no removal focus','disabled/readonly no close controls','controlled reject/deferred append/removal acceptance','interactive overflow scroll/input/page/focus','same-turn input/Enter lossless'],input:'synthetic DOM',renderer:navigator.userAgent};
} catch(error) {return {ok:false,error:String(error)}}
"#;

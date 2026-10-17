//! Bounded Linux/WebView regression for B10.3. Input/focus events are synthetic;
//! this verifies renderer wiring, not physical keyboard or assistive technology.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "../examples/gallery/filter_state_review.rs"]
mod filter_state_review;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("iX filter state probe")
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
            eprintln!("B10.3 native probe timed out");
            std::process::exit(1);
        });
    });
    rsx! { IxTheme { scheme: ColorScheme::Dark,
        div { style: "padding-block:150vh", onmounted: move |_| {
            let desktop = desktop.clone();
            spawn(async move {
                let query = dom_query::eval(VERIFY);
                let _ = query.send(());
                match query.join::<serde_json::Value>().await {
                    Ok(result) if result["ok"] == true => {
                        println!("B10.3 native synthetic renderer checks: {result}");
                        desktop.close();
                    }
                    result => { eprintln!("B10.3 native probe failed: {result:?}"); std::process::exit(1); }
                }
            });
        }, filter_state_review::FilterStateReview {} }
    } }
}

const VERIFY: &str = r#"
try {
 await dioxus.recv();
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 let rafDelivered=false;requestAnimationFrame(()=>{rafDelivered=true});
 const get=id=>document.getElementById(id),root=kind=>get(`b10-state-${kind}`),input=kind=>root(kind)?.querySelector('.ix-category-filter-text');
 const open=kind=>root(kind).querySelector('.ix-category-filter').getAttribute('data-open')==='true';
 const events=()=>JSON.parse(get('b10-state-events').textContent),values=()=>JSON.parse(get('b10-state-values').textContent),results=()=>JSON.parse(get('b10-state-results').textContent);
 const preview=kind=>root(kind).querySelector('.ix-category-filter-preview'),operator=kind=>root(kind).querySelector('[aria-label="Toggle comparison operator"]');
 const chips=kind=>root(kind).querySelectorAll('.ix-category-filter-chip').length;
 const assert=(ok,label)=>{if(!ok)throw Error(label+'; values='+JSON.stringify(values())+'; events='+JSON.stringify(events())+'; results='+JSON.stringify(results()))};
 const wait=async(test,label)=>{const end=performance.now()+3000;while(!test()){if(performance.now()>end)throw Error(label+'; native='+JSON.stringify({visibility:document.visibilityState,rafDelivered,scrollTop:root('local')?.querySelector('.ix-category-filter-tokens')?.scrollTop,scrollHeight:root('local')?.querySelector('.ix-category-filter-tokens')?.scrollHeight,clientHeight:root('local')?.querySelector('.ix-category-filter-tokens')?.clientHeight,revision:root('local')?.querySelector('.ix-category-filter-tokens')?.dataset.scrollRevision,directChips:root('local')?.querySelector('.ix-category-filter-tokens')?.querySelectorAll(':scope > .ix-category-filter-chip').length})+'; values='+JSON.stringify(values())+'; events='+JSON.stringify(events())+'; results='+JSON.stringify(results()));await sleep(20)}};
 const focus=kind=>{input(kind).focus();input(kind).dispatchEvent(new FocusEvent('focusin',{bubbles:true}));};
 const openInput=kind=>{focus(kind);if(!open(kind))input(kind).click();};
 const edit=(kind,value)=>{input(kind).value=value;input(kind).dispatchEvent(new InputEvent('input',{bubbles:true,data:value,inputType:'insertText'}));};
 const choose=async()=>{openInput('local');await wait(()=>[...root('local').querySelectorAll('button.ix-category-filter-option')].some(n=>n.textContent.trim()==='Vendor'),'Vendor choice');root('local').querySelector('button.ix-category-filter-option').click();await wait(()=>preview('local')?.textContent==='Vendor'&&operator('local'),'Vendor preview');};
 const fresh=async()=>{const old=input('local');get('b10-state-reset').click();await wait(()=>input('local')!==old&&events().length===1&&values().prop.tokens[0]==='Initial prop','fresh cases');};
 const since=(start,kind)=>events().slice(start).filter(e=>e.filter===kind);
 await wait(()=>input('local')&&events().length===1&&values().prop.tokens[0]==='Initial prop','post-mount initial property');
 assert(events()[0].filter==='prop'&&events()[0].event==='onchange'&&chips('local')===1,'initial property once, default local silent');assert(document.activeElement!==input('prop'),'initial property no forced focus');assert(input('prop').getBoundingClientRect().top>innerHeight,'initial property fixture is below fold');
 let start=events().length;get('b10-state-prop-update').focus();await sleep(80);const propScroll=window.scrollY;get('b10-state-prop-update').click();await wait(()=>values().prop.tokens[0]==='Updated prop','changed property');await sleep(60);assert(since(start,'prop').map(e=>e.event).join(',')==='onchange','property change emits exactly once');assert(document.activeElement!==input('prop')&&window.scrollY===propScroll,'property update no focus/page scroll '+JSON.stringify({before:propScroll,after:window.scrollY,active:document.activeElement?.id,inputFocused:document.activeElement===input('prop')}));
 await choose();operator('local').click();await wait(()=>operator('local')?.getAttribute('aria-pressed')==='true','draft !=');edit('local','kept uncommitted');await sleep(40);input('local').dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,cancelable:true,key:'Escape',code:'Escape'}));await wait(()=>!open('local'),'explicit Escape closes before empty command');get('b10-state-after').focus();await sleep(40);const openBefore=open('local'),emptyScroll=window.scrollY;assert(!openBefore,'empty command starts from explicitly closed popup');start=events().length;get('b10-state-empty').click();await wait(()=>chips('local')===0,'empty state');await sleep(60);assert(input('local').value==='kept uncommitted'&&preview('local')?.textContent==='Vendor','empty command preserves draft/category');assert(open('local')===openBefore&&document.activeElement!==input('local')&&window.scrollY===emptyScroll,'empty command preserves popup/focus/page position');assert(since(start,'local').map(e=>e.event).join(',')==='onchange','empty command exactly one change');
 openInput('local');await wait(()=>operator('local'),'retained operator visible');assert(operator('local').getAttribute('aria-pressed')==='true','empty command retains operator');get('b10-state-after').focus();await sleep(40);start=events().length;get('b10-state-apply').click();await wait(()=>chips('local')===3&&values().local.tokens[0]==='Applied','source normalized command');await sleep(60);assert(input('local').value===''&&!preview('local'),'accepted tokens clear draft/category');assert(values().local.categories.length===2&&values().local.categories.every(c=>c.id==='vendor'&&c.value==='Alpha'&&c.operator==='!='),'source setter retains duplicate category values');assert(since(start,'local').map(e=>e.event).join(',')==='onchange'&&document.activeElement!==input('local'),'accepted command once without focus');
 start=events().length;get('b10-state-equal').click();await wait(()=>since(start,'local').length===1,'equal command event');await sleep(60);assert(since(start,'local').map(e=>e.event).join(',')==='onchange'&&chips('local')===3,'equal command not deduplicated');
 openInput('local');await wait(()=>[...root('local').querySelectorAll('button.ix-category-filter-option')].some(n=>n.textContent.trim()==='Vendor'),'Vendor after accepted command');root('local').querySelector('button.ix-category-filter-option').click();await wait(()=>operator('local'),'operator after accepted command');assert(operator('local').getAttribute('aria-pressed')==='false','accepted token resets Equal');
 get('b10-state-many').focus();await sleep(40);const manyScroll=window.scrollY;start=events().length;get('b10-state-many').click();await wait(()=>chips('local')===40,'many tokens');const list=()=>root('local').querySelector('.ix-category-filter-tokens');await wait(()=>list().scrollTop>0&&Math.abs(list().scrollTop-(list().scrollHeight-list().clientHeight))<=2,'native token list scroll to end');const inputRect=input('local').getBoundingClientRect(),listRect=list().getBoundingClientRect();assert(inputRect.top>=listRect.top-2&&inputRect.bottom<=listRect.bottom+2,'last input visible within scoped list');assert(window.scrollY===manyScroll&&document.activeElement!==input('local'),'token scrolling no page movement/focus');assert(since(start,'local').map(e=>e.event).join(',')==='onchange','many command one notification');
 start=events().length;get('b10-state-mount').click();await wait(()=>!input('local'),'unmount local');get('b10-state-apply').click();await wait(()=>results().at(-1)?.result==='NotMounted','detached controller rejects');assert(events().length===start,'detached command emits nothing');get('b10-state-mount').click();await wait(()=>input('local')&&chips('local')===1,'remount local');await sleep(60);get('b10-state-apply').click();await wait(()=>results().at(-1)?.result==='Ok'&&chips('local')===3,'controller rebinds after remount');
 get('b10-state-reject').click();await sleep(40);start=events().length;get('b10-state-controlled-apply').click();await wait(()=>values().controlled_proposal.tokens[0]==='controlled command','controlled proposal');await sleep(60);assert(values().controlled_parent.tokens[0]==='parent'&&root('controlled').textContent.includes('parent'),'controlled rejection authoritative');assert(since(start,'controlled').map(e=>e.event).join(',')==='onchange','controlled rejection once');const delayedCount=events().length;get('b10-state-accept-last').focus();await sleep(80);const delayedY=window.scrollY;get('b10-state-accept-last').click();await wait(()=>chips('controlled')===40,'delayed parent acceptance without command');const controlledList=()=>root('controlled').querySelector('.ix-category-filter-tokens');await wait(()=>controlledList().scrollTop>0&&Math.abs(controlledList().scrollTop-(controlledList().scrollHeight-controlledList().clientHeight))<=2,'delayed acceptance scroll acknowledgment');assert(events().length===delayedCount&&window.scrollY===delayedY&&document.activeElement!==input('controlled'),'deferred acceptance no extra notification/page/focus');get('b10-state-reject').click();await sleep(40);start=events().length;const acceptedResults=results().length;get('b10-state-controlled-apply').click();await wait(()=>results().length===acceptedResults+1&&since(start,'controlled').length===1&&values().controlled_parent.tokens[0]==='controlled command','controlled accept acknowledgment');await sleep(60);assert(since(start,'controlled').map(e=>e.event).join(',')==='onchange','controlled acceptance once');
 await fresh();openInput('local');await wait(()=>[...root('local').querySelectorAll('button.ix-category-filter-option')].some(node=>node.textContent.trim()==='Vendor'),'queued Vendor category');const queuedCategory=[...root('local').querySelectorAll('button.ix-category-filter-option')].find(node=>node.textContent.trim()==='Vendor');queuedCategory.click();edit('local','queued draft');get('b10-state-after').focus();get('b10-state-empty').click();await wait(()=>results().at(-1)?.command==='empty','queued focus command');await sleep(120);assert(document.activeElement!==input('local')&&!open('local')&&input('local').value==='queued draft'&&preview('local')?.textContent==='Vendor','command cancels queued category focus, outside click closes, draft retained');
 await fresh();window.scrollTo(0,document.documentElement.scrollHeight);await sleep(40);const offscreenY=window.scrollY;const offscreenRect=input('prop').getBoundingClientRect();assert(offscreenRect.bottom<0||offscreenRect.top>innerHeight,'property fixture actually offscreen');start=events().length;get('b10-state-prop-update').click();await wait(()=>values().prop.tokens[0]==='Updated prop','offscreen property');await sleep(80);assert(window.scrollY===offscreenY&&document.activeElement!==input('prop'),'offscreen property no forced focus/page scroll');assert(since(start,'prop').length===1,'offscreen property once');
 return {ok:true,checks:['post-mount/changed property exactly once','default state silent','source normalization/category duplicates','equal commands notify','empty preserves draft/operator, accepted token resets','explicitly closed popup/focus preserved','scoped token scrolling without page movement','detached rejection/remount rebinding','queued focus invalidation','controlled acceptance/rejection and delayed acceptance scrolling','offscreen property no focus/page scroll'],input:'synthetic DOM',renderer:navigator.userAgent};
} catch(error) {return {ok:false,error:String(error)}}
"#;

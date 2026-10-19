//! Excluded workflow order preview; native driver uses synthetic DOM input.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn WorkflowOrderReview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {scheme:ColorScheme::Dark,class:"workflow-order-review",
            h1 {"Workflow steps in Rust order"}
            p {"Reorder, add or remove steps, then activate them. Dynamic lists supply their visual order explicitly; independent and nested workflows keep their own selection."}
            super::workflow_review::WorkflowReview {}
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_WORKFLOW_CHECK);let _=task.send(());
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native workflow order: {value}");window.close();}
                        result=>{eprintln!("Dioxus native workflow order failed: {result:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native workflow order timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.workflow-order-review{box-sizing:border-box;min-height:100vh;padding:24px}.workflow-order-review h1{font-size:30px;line-height:1.2;margin:0 0 16px}.workflow-order-review h2{font-size:22px;margin:32px 0 12px}.workflow-order-review h3{font-size:18px;margin:28px 0 12px}.workflow-order-review p{max-width:72ch;line-height:1.5}.workflow-order-review .demo-section{max-width:1040px}.workflow-order-review .demo-row{display:flex;flex-wrap:wrap;gap:10px;margin:20px 0}.workflow-order-review pre{max-width:100%;padding:12px;background:var(--ix-color-surface-1);font-size:13px;line-height:1.5}.workflow-order-review ::selection{background:var(--ix-color-primary);color:var(--ix-color-on-primary)}@media(max-width:480px){.workflow-order-review{padding:16px}.workflow-order-review h1{font-size:26px}.workflow-order-review .demo-row .ix-button{max-width:100%;white-space:normal}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_WORKFLOW_CHECK: &str = r#"
await dioxus.recv();
const get=id=>document.getElementById(id),events=()=>JSON.parse(get('g04-events').textContent),state=()=>JSON.parse(get('g04-state').textContent);
const action=id=>get(id).querySelector('.ix-workflow-step-content')||get(id),click=id=>get(id).click();
const selected=root=>[...get(root).querySelectorAll('.ix-workflow-step[data-selected=true]')].filter(node=>node.closest('.ix-workflow-steps')===get(root)).map(node=>node.id);
const main=()=>selected('g04-workflow')[0],key=(id,key)=>action(id).dispatchEvent(new KeyboardEvent('keydown',{key,bubbles:true,cancelable:true}));
const assert=(ok,label)=>{if(!ok)throw Error(label+'; events='+JSON.stringify(events())+'; selected='+main());};
const wait=async(label,test)=>{for(let i=0;i<150;i++){if(test())return;await new Promise(resolve=>setTimeout(resolve,20));}throw Error(label+'; events='+JSON.stringify(events())+'; selected='+main());};
await wait('initial registered order',()=>main()==='g04-step-prepare'&&get('g04-step-finish').dataset.index==='3');
assert(get('g04-step-prepare').dataset.position==='first'&&get('g04-step-finish').dataset.position==='last','initial endpoints');
action('g04-step-inspect').click();await wait('wrapped pointer request',()=>events().length===1&&main()==='g04-step-inspect');
assert(events()[0].requested===1&&events()[0].previous===0&&state().selected_index===0,'wrapped ordinal and request do not mutate controlled prop');
action('g04-step-inspect').click();await wait('repeat request',()=>events().length===2);assert(events()[1].previous===1,'repeat previous index');
click('g04-rerender');await wait('parent restores controlled index',()=>main()==='g04-step-prepare');
click('g04-veto');await wait('veto enabled',()=>state().veto);
action('g04-step-inspect').click();await wait('veto request',()=>events().length===3);assert(events()[2].vetoed&&main()==='g04-step-prepare','veto keeps previous selection');
action('g04-step-disabled').click();key('g04-step-disabled','Enter');
assert(action('g04-step-disabled').getAttribute('tabindex')==='-1'&&action('g04-step-disabled').getAttribute('aria-disabled')==='true','disabled accessibility');
click('g04-veto');await wait('veto cleared',()=>!state().veto);
key('g04-step-inspect','Enter');await wait('Enter request and disabled silence',()=>events().length===4&&main()==='g04-step-inspect');
key('g04-step-finish',' ');await wait('Space request',()=>events().length===5&&main()==='g04-step-finish');
click('g04-select-disabled');await wait('programmatic disabled selection',()=>main()==='g04-step-disabled');
click('g04-select-out-of-range');await wait('out of range selects none',()=>state().selected_index===99&&!main());
click('g04-select-negative');await wait('negative selects none',()=>state().selected_index===-1&&!main());
click('g04-select-zero');await wait('programmatic zero',()=>main()==='g04-step-prepare');
click('g04-orientation');await wait('vertical layout',()=>get('g04-workflow').dataset.vertical==='true'&&get('g04-step-inspect').dataset.vertical==='true');
click('g04-append');await wait('append supplies last order',()=>get('g04-step-added-1')?.dataset.position==='last'&&get('g04-step-added-1').dataset.index==='4');
click('g04-reorder');await wait('keyed reverse applies explicit order',()=>main()==='g04-step-added-1'&&get('g04-step-prepare').dataset.index==='4');
assert(get('g04-step-added-1').dataset.position==='first'&&get('g04-step-prepare').dataset.position==='last','reversed endpoints');
action('g04-step-prepare').click();await wait('reversed request',()=>events().length===6&&main()==='g04-step-prepare');
assert(events()[5].requested===4&&events()[5].previous===0,'request follows Rust order after reverse');
click('g04-remove');await wait('remove reindexes and reapplies controlled index',()=>!get('g04-step-prepare')&&main()==='g04-step-added-1'&&get('g04-step-inspect').dataset.position==='last');
action('g04-step-inspect').click();await wait('wrapped reordered request',()=>events().length===7&&main()==='g04-step-inspect');assert(events()[6].requested===3,'wrapper forwards latest order');
action('g04-other-b').click();await wait('independent selection',()=>events().length===8&&selected('g04-other')[0]==='g04-other-b');assert(main()==='g04-step-inspect','independent workflow isolation');
key('g04-inner-b','Enter');await wait('nested activation',()=>events().length===9&&selected('g04-nested-inner')[0]==='g04-inner-b');
assert(events()[8].source==='inner'&&selected('g04-nested-outer')[0]==='g04-outer-a','nested request stays in inner owner');
action('g04-outer-b').click();await wait('outer activation',()=>events().length===10&&selected('g04-nested-outer')[0]==='g04-outer-b');assert(selected('g04-nested-inner')[0]==='g04-inner-b','outer preserves inner selection');
click('g04-clickable');await wait('navigation disabled',()=>!state().clickable&&action('g04-step-inspect').getAttribute('role')!=='button');
action('g04-step-inspect').click();key('g04-step-inspect',' ');
click('g04-status');await wait('disabled-navigation event barrier',()=>state().prepare_status==='error');assert(events().length===10,'nonclickable steps cannot request selection');
assert(get('g04-custom-content').textContent==='Release ready','custom content preserved');
click('g04-reset');await wait('reset barrier',()=>events().length===0&&main()==='g04-step-prepare'&&state().keys.length===4);
for(const [removed,count] of [['finish',3],['disabled',2],['inspect',1]]){click('g04-remove');await wait('remove '+removed,()=>!get('g04-step-'+removed)&&state().keys.length===count);}
await wait('single connector and selection',()=>get('g04-step-prepare').dataset.position==='single'&&main()==='g04-step-prepare');
return {ok:true,syntheticInput:true,pointerEnterSpace:true,vetoRepeatDisabled:true,explicitKeyedOrderAndEndpoints:true,appendRemove:true,signedAndOutOfRange:true,orientation:true,independentAndNested:true};
"#;

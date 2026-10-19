//! Excluded mixed Dropdown item order, explicit custom bindings and local Category lists.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn CustomOrderedItem(
    order: usize,
    editing: bool,
    value: Signal<String>,
    onaction: EventHandler<String>,
) -> Element {
    let id = if editing {
        "r0243-field"
    } else {
        "r0243-custom"
    };
    let binding = use_dropdown_item_with_options(
        id,
        DropdownItemOptions {
            order: Some(order),
            ..Default::default()
        },
    );
    let mut value = value;
    if editing {
        rsx! {input {id,class:"item-order-field",aria_label:"Editable equipment note",value:value(),onmounted:binding.onmounted,tabindex:binding.tabindex,oninput:move |event|value.set(event.value()),..binding.attributes()}}
    } else {
        rsx! {button {id,r#type:"button",class:"ix-dropdown-item",onmounted:binding.onmounted,onkeydown:binding.onkeydown,tabindex:binding.tabindex,onclick:move |_|onaction.call("Custom action".into()),..binding.attributes(),"Custom action"}}
    }
}

#[component]
fn MixedOrderedItem(
    name: String,
    order: usize,
    hidden: bool,
    disabled: bool,
    linked: DropdownHandle,
    note: Signal<String>,
    onaction: EventHandler<String>,
) -> Element {
    match name.as_str() {
        "menu" => {
            rsx! {div {class:"item-order-wrapper",MenuItem {id:"r0243-menu-item",label:"Menu action",order:Some(order),hidden,disabled,close_on_select:false,onclick:move |_|onaction.call("Menu action".into())}}}
        }
        "select" => {
            rsx! {div {class:"item-order-wrapper",SelectItem {id:"r0243-select",value:"pump",label:"Select option",order:Some(order),hidden,disabled,onitemclick:move |_|onaction.call("Select option".into())}}}
        }
        "category" => {
            rsx! {div {class:"item-order-wrapper",MenuCategory {id:"r0243-category",label:"Local category",order:Some(order),hidden,
                MenuItem {label:"Local task one",order:Some(0),close_on_select:false}
                div {MenuItem {label:"Local task two",order:Some(1),close_on_select:false}}
            }}}
        }
        "custom" => rsx! {CustomOrderedItem {order,editing:false,value:note,onaction}},
        "field" => rsx! {CustomOrderedItem {order,editing:true,value:note,onaction}},
        "linked" => {
            rsx! {DropdownItem {id:"r0243-linked",label:"Linked maintenance actions",order:Some(order),dropdown:linked}}
        }
        _ => {
            let checked = name == "checked";
            let (id, label) = if checked {
                ("r0243-checked", "Checked action")
            } else if name == "gamma" {
                ("r0243-gamma", "Inserted action")
            } else {
                ("r0243-alpha", "Alpha action")
            };
            rsx! {div {class:"item-order-wrapper",DropdownItem {id,label,order:Some(order),hidden,disabled,checked,checkable:checked,keep_open:true,onitemclick:move |_|onaction.call(label.into())}}}
        }
    }
}

#[component]
pub fn ItemOrderReview() -> Element {
    let child = use_dropdown();
    let mut mode = use_signal(|| "ad".to_string());
    let mut reversed = use_signal(|| false);
    let mut inserted = use_signal(|| false);
    let mut hidden = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut checked_focus = use_signal(|| false);
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut selections = use_signal(|| 0_usize);
    let mut last = use_signal(String::new);
    let note = use_signal(|| "Pump note".to_string());
    let selected = use_callback(move |label: String| {
        last.set(label);
        selections += 1;
    });
    let mut entries = vec![
        "alpha", "checked", "menu", "select", "category", "custom", "field", "linked",
    ];
    if inserted() {
        entries.push("gamma");
    }
    if reversed() {
        entries.reverse();
    }
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"item-order-review",
            h1 {"Mixed Dropdown order owned by Rust"}
            p {"Each wrapped item forwards its current position. Keyed moves, insertion, removal and hidden or disabled props update navigation without discovering DOM order. A category keeps its own local child list. ArrowUp on the closed trigger opens at the last item."}
            div {class:"item-order-controls",
                label {r#for:"r0243-mode","Navigation mode"}
                select {id:"r0243-mode",value:mode(),onchange:move |event|mode.set(event.value()),option {value:"ad","Active descendant"}option {value:"roving","Roving focus"}}
                button {id:"r0243-reverse",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|reversed.toggle(),"Reverse ordered peers"}
                button {id:"r0243-insert",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|inserted.toggle(),"Insert / remove action"}
                button {id:"r0243-hidden",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|hidden.toggle(),"Hide / show Alpha"}
                button {id:"r0243-disabled",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|disabled.toggle(),"Disable / enable checked action"}
                button {id:"r0243-checked-focus",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|checked_focus.toggle(),"Toggle checked opening focus"}
                button {id:"r0243-reset",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|{reversed.set(false);inserted.set(false);hidden.set(false);disabled.set(false);checked_focus.set(false);},"Reset list options"}
                button {id:"r0243-mount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|mounted.toggle(),"Hide / show mixed menu"}
                button {id:"r0243-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|epoch+=1,"Remount mixed menu"}
            }
            section {h2 {"Equipment actions"}
                if mounted() {
                    Dropdown {key:"mixed-{epoch()}",label:"Equipment actions",class:"r0243-mixed",enable_top_layer:true,focus_checked_item:checked_focus(),navigation_mode:if mode()=="roving"{DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant},
                        DropdownItem {id:"r0243-start",label:"Start action",order:Some(0),keep_open:true,onitemclick:move |_|selected.call("Start action".into())}
                        for (index,name) in entries.into_iter().enumerate() {
                            MixedOrderedItem {key:"{name}",name:name.to_string(),order:index+1,hidden:name=="alpha"&&hidden(),disabled:name=="checked"&&disabled(),linked:child,note,onaction:move |label|selected.call(label)}
                        }
                        DropdownItem {id:"r0243-finish",label:"Finish action",order:Some(100),keep_open:true,onitemclick:move |_|selected.call("Finish action".into())}
                    }
                }
                Dropdown {controller:child,label:"Maintenance submenu",class:"r0243-child",navigation_mode:DropdownNavigationMode::RovingTabindex,
                    DropdownItem {id:"r0243-child-first",label:"Maintenance record",order:Some(0)}
                    DropdownItem {label:"Inspection history",order:Some(1)}
                }
                p {"Custom input editing keys retain their native defaults. Custom buttons bind navigation explicitly. Linked maintenance actions use their own Dropdown owner."}
            }
            label {r#for:"r0243-outside","Outside focus destination"}input {id:"r0243-outside",class:"item-order-field",value:"Independent focus"}
            output {id:"r0243-state",role:"status","aria-live":"polite","data-mode":mode(),"data-reversed":reversed().to_string(),"data-inserted":inserted().to_string(),"data-hidden":hidden().to_string(),"data-disabled":disabled().to_string(),"data-checked-focus":checked_focus().to_string(),"data-mounted":mounted().to_string(),"data-epoch":epoch().to_string(),"data-selections":selections().to_string(),"data-last":last(),"data-note":note(),
                "Selections: {selections()} ({last()}); note: {note()}; owner generation: {epoch()}"
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {let task=super::dom_query::eval(NATIVE_ITEM_ORDER_CHECK);let _=task.send("item-order");match task.join::<serde_json::Value>().await {Ok(value) if value["ok"]==true=>{println!("Dioxus native Rust item order: {value}");window.close();}other=>{eprintln!("Dioxus native Rust item order failed: {other:?}");std::process::exit(1);}}});spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native Rust item order timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.item-order-review{box-sizing:border-box;min-height:100vh;padding:24px}.item-order-review h1{font-size:28px;margin:0 0 16px}.item-order-review h2{font-size:20px;margin:0 0 16px}.item-order-review p{max-width:70ch;line-height:1.5}.item-order-controls{display:flex;flex-wrap:wrap;align-items:center;gap:10px;max-width:900px;margin:20px 0}.item-order-review section{margin:28px 0;max-width:900px;min-height:120px}.item-order-review .ix-dropdown{max-width:100%}.item-order-review .ix-dropdown-menu{max-width:calc(100vw - 32px)}.item-order-field{display:block;box-sizing:border-box;min-height:34px;width:min(calc(100% - 16px),300px);margin:8px;padding:6px 8px;background:var(--ix-color-surface);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.item-order-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}@media(max-width:600px){.item-order-review{padding:16px}.item-order-controls{gap:8px}}";

// Synthetic Linux keys verify Rust focus/cues and state; real editing and native
// keyboard defaults are checked separately in Chromium.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_ITEM_ORDER_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),state=()=>byId('r0243-state'),root=()=>document.querySelector('.r0243-mixed'),menu=()=>root()?.querySelector(':scope > [role="menu"]'),child=()=>document.querySelector('.r0243-child');
const open=()=>root()?.getAttribute('data-ix-overlay-open')==='true',control=id=>id==='r0243-category'?byId(id)?.querySelector('[data-ix-item-part="category-parent"]'):byId(id);
const at=id=>root()?.getAttribute('data-ix-overlay-active-item')===id||document.activeElement===control(id);
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' mode='+state()?.dataset.mode+' focus='+document.activeElement?.id+' active='+root()?.getAttribute('data-ix-overlay-active-item'));};
const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const press=value=>key(document.activeElement,value),click=id=>byId(id).click();
const show=async()=>{root().focus();key(root(),'ArrowDown');await wait('mixed menu opens by keyboard',open);};
const close=async()=>{press('Escape');await wait('mixed menu closes',()=>!open());};
const home=async()=>{menu().focus();press('Home');await wait('Home selects fixed first item',()=>at('r0243-start'));};
const results=[];
await wait('mixed menu mounted',()=>root()&&byId('r0243-alpha'));
root().click();await wait('pointer opens mixed menu',open);await close();
for(const mode of ['ad','roving']){
    click('r0243-reset');byId('r0243-mode').value=mode;byId('r0243-mode').dispatchEvent(new Event('change',{bubbles:true}));await wait('mode and list reset',()=>state().dataset.mode===mode&&state().dataset.reversed==='false'&&state().dataset.disabled==='false');
    await show();await wait('ordinary first opening focus',()=>at('r0243-start'));await home();press('ArrowDown');await wait('ordered Alpha peer',()=>at('r0243-alpha'));press('ArrowDown');await wait('ordered checked peer',()=>at('r0243-checked'));press('End');await wait('End selects final peer',()=>at('r0243-finish'));
    await home();press('ArrowDown');await wait('Alpha activation setup',()=>at('r0243-alpha'));const count=Number(state().dataset.selections);press('Enter');await wait('typed semantic selection once',()=>Number(state().dataset.selections)===count+1&&state().dataset.last==='Alpha action'&&open());await close();
    root().focus();key(root(),'ArrowUp');await wait('ArrowUp opens at the last item',()=>open()&&at('r0243-finish'));await close();
    click('r0243-checked-focus');await wait('checked opening enabled',()=>state().dataset.checkedFocus==='true');await show();await wait('checked item receives opening focus',()=>at('r0243-checked'));await close();click('r0243-checked-focus');
    const alpha=byId('r0243-alpha');click('r0243-reverse');await wait('keyed peers reversed',()=>state().dataset.reversed==='true');if(byId('r0243-alpha')!==alpha)throw new Error('keyed Alpha replaced');await show();await home();press('ArrowDown');await wait('current reversed first peer',()=>at('r0243-linked'));await close();
    click('r0243-insert');await wait('new peer inserted',()=>byId('r0243-gamma'));await show();await home();press('ArrowDown');await wait('inserted peer has explicit first position',()=>at('r0243-gamma'));await close();click('r0243-insert');await wait('inserted peer removed',()=>!byId('r0243-gamma'));
    click('r0243-hidden');click('r0243-disabled');await wait('current hidden and disabled props',()=>state().dataset.hidden==='true'&&state().dataset.disabled==='true');await show();menu().focus();press('End');await wait('last peer before eligibility navigation',()=>at('r0243-finish'));press('ArrowUp');await wait('hidden Alpha and disabled checked item skipped',()=>at('r0243-menu-item'));
    if(mode==='ad'){
        press('ArrowUp');await wait('AD reaches Select through outer peer keys',()=>at('r0243-select'));
        press('ArrowUp');await wait('AD reaches Category parent through outer peer keys',()=>at('r0243-category'));
    }else{
        byId('r0243-custom').focus();key(byId('r0243-custom'),'ArrowDown');await wait('custom binding follows explicit peer order',()=>at('r0243-category'));
    }
    press('ArrowDown');await wait('Category stays a parent in outer peer order',()=>at('r0243-select'));
    const field=byId('r0243-field');field.focus();field.value='Equipment note revised';field.dispatchEvent(new Event('input',{bubbles:true}));await wait('registered input edits',()=>state().dataset.note==='Equipment note revised');for(const value of ['Home','End','ArrowLeft','ArrowRight']){if(key(field,value).defaultPrevented)throw new Error('native editing key canceled '+value);}await close();
    if(mode==='roving'){await show();await home();press('ArrowDown');await wait('linked peer focused',()=>at('r0243-linked'));press('ArrowRight');await wait('linked submenu opens current child',()=>child().getAttribute('data-ix-overlay-open')==='true'&&document.activeElement===byId('r0243-child-first'));press('ArrowLeft');await wait('linked submenu returns current row',()=>child().getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r0243-linked'));await close();}
    results.push({mode,firstOpening:true,lastOpening:true,checkedOpening:true,arrowsHomeEnd:true,keyedOrderInsertRemove:true,hiddenDisabled:true,mixedCategoryLocality:true,editingDefaults:true});
}
byId('r0243-outside').focus();const retired=root();click('r0243-mount');await wait('owner unmounted',()=>!root()&&!retired.isConnected);click('r0243-mount');await wait('fresh owner restored',()=>root()&&byId('r0243-start'));const before=root();click('r0243-remount');await wait('keyed owner remounted',()=>state().dataset.epoch==='1'&&root()!==before&&!before.isConnected);await show();await home();await close();
return {ok:true,syntheticEvents:true,modes:results,linkedSubmenuReturn:true,ownerCleanupAndRemount:true};
"#;

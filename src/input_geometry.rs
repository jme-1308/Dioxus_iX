//! Native adornment facts; padding policy and CSS remain in Rust.
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Deserialize)]
pub(crate) struct Adornments {
    pub start: f64,
    pub end: f64,
}

impl Adornments {
    fn valid(self) -> bool {
        self.start.is_finite() && self.end.is_finite() && self.start >= 0.0 && self.end >= 0.0
    }

    pub(crate) fn padding(self) -> String {
        // Pinned input.util.ts converts measured pixels to rem by dividing by16.
        format!(
            "padding-left:calc({}rem + var(--si-sys-sizing-spacing-x-40, .5rem));padding-right:calc({}rem + var(--si-sys-sizing-spacing-x-40, .5rem));",
            self.start / 16.0,
            self.end / 16.0
        )
    }
}

pub(crate) fn use_adornments(id: &str) -> (Option<Adornments>, EventHandler<()>) {
    let mut measured = use_signal(|| None::<Adornments>);
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let generation = use_hook(|| Rc::new(Cell::new(0_u64)));
    let channel = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let cleanup_alive = alive.clone();
    let cleanup_channel = channel.clone();
    use_drop(move || {
        cleanup_alive.set(false);
        if let Some(channel) = cleanup_channel.borrow_mut().take() {
            let _ = channel.send("stop");
        }
    });
    let id = id.to_string();
    let mounted = use_callback(move |_| {
        generation.set(generation.get().wrapping_add(1));
        let current = generation.get();
        let generation = generation.clone();
        measured.set(None);
        if let Some(old) = channel.borrow_mut().take() {
            let _ = old.send("stop");
        }
        let id = id.clone();
        let alive = alive.clone();
        let channel = channel.clone();
        spawn(async move {
            if !alive.get() || generation.get() != current {
                return;
            }
            let mut task = dioxus::document::eval(OBSERVE);
            if task.send(id).is_err() {
                return;
            }
            *channel.borrow_mut() = Some(task);
            while let Ok(facts) = task.recv::<Adornments>().await {
                if !alive.get() || generation.get() != current {
                    break;
                }
                if facts.valid() && *measured.peek() != Some(facts) {
                    measured.set(Some(facts));
                }
            }
        });
    });
    (measured(), mounted)
}

// Observe captured native objects only. No CSS writes or UI policy in this bridge.
const OBSERVE: &str = r#"
const id=await dioxus.recv(),root=document.getElementById(id);
if(!root)return;
let frame=0,previous='',disconnected;
const gone=new Promise(resolve=>{disconnected=resolve;});
const measure=()=>{
 frame=0;if(!root.isConnected){disconnected();return;}
 const start=root.querySelector('.ix-input-start'),end=root.querySelector('.ix-input-end');
 if(!start||!end)return;
 const facts={start:start.getBoundingClientRect().width,end:end.getBoundingClientRect().width};
 const json=JSON.stringify(facts);if(json!==previous){previous=json;dioxus.send(facts);}
};
const schedule=()=>{if(!frame)frame=requestAnimationFrame(measure);};
const resize=new ResizeObserver(schedule);
const observe=()=>{resize.disconnect();root.querySelectorAll('.ix-input-start,.ix-input-end').forEach(node=>resize.observe(node));schedule();};
const changes=new MutationObserver(observe);changes.observe(root,{childList:true,subtree:true});
const removal=new MutationObserver(()=>{if(!root.isConnected)disconnected();});
removal.observe(document.documentElement,{childList:true,subtree:true});
try{observe();await Promise.race([dioxus.recv(),gone]);}
finally{resize.disconnect();changes.disconnect();removal.disconnect();cancelAnimationFrame(frame);}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_facts_reject_nonfinite_or_negative_dimensions() {
        for bad in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                !Adornments {
                    start: bad,
                    end: 0.0
                }
                .valid()
            );
            assert!(
                !Adornments {
                    start: 0.0,
                    end: bad
                }
                .valid()
            );
        }
        assert!(
            Adornments {
                start: 0.0,
                end: 28.5
            }
            .valid()
        );
    }
    #[test]
    fn fractional_measurements_keep_source_rem_conversion_and_live_base_token() {
        assert_eq!(
            Adornments {
                start: 20.5,
                end: 40.25
            }
            .padding(),
            "padding-left:calc(1.28125rem + var(--si-sys-sizing-spacing-x-40, .5rem));padding-right:calc(2.515625rem + var(--si-sys-sizing-spacing-x-40, .5rem));"
        );
    }
}

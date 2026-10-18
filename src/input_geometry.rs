//! Dioxus-owned adornment measurements; padding policy and CSS remain in Rust.
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

/// Dioxus emits the initial border-box size as well as later resize changes.
/// The wrappers are framework-owned: no queries, mounted-handle registry or
/// independent observer lifecycle is needed for their widths.
#[derive(Clone, Copy)]
pub(crate) struct AdornmentMeasurement {
    pub facts: Option<Adornments>,
    pub start_resized: EventHandler<ResizeEvent>,
    pub end_resized: EventHandler<ResizeEvent>,
}

pub(crate) fn use_adornments() -> AdornmentMeasurement {
    let mut widths = use_signal(|| [None::<f64>; 2]);
    let record = use_callback(move |(side, event): (usize, ResizeEvent)| {
        if let Ok(size) = event.get_border_box_size() {
            let width = size.width;
            if width.is_finite() && width >= 0.0 && widths.peek()[side] != Some(width) {
                widths.write()[side] = Some(width);
            }
        }
    });
    let [start, end] = widths();
    AdornmentMeasurement {
        facts: start
            .zip(end)
            .map(|(start, end)| Adornments { start, end })
            .filter(|facts| facts.valid()),
        start_resized: use_callback(move |event| record.call((0, event))),
        end_resized: use_callback(move |event| record.call((1, event))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    struct Resized(f64);
    impl dioxus::html::HasResizeData for Resized {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn get_border_box_size(
            &self,
        ) -> dioxus::html::ResizeResult<dioxus::html::geometry::PixelsSize> {
            Ok(dioxus::html::geometry::PixelsSize::new(self.0, 24.0))
        }
    }
    fn resized(width: f64) -> ResizeEvent {
        Event::new(
            Rc::new(dioxus::html::ResizeData::new(Resized(width))),
            false,
        )
    }
    #[derive(Clone)]
    struct Fixture(Rc<RefCell<Option<AdornmentMeasurement>>>);
    impl PartialEq for Fixture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    fn fixture(props: Fixture) -> Element {
        *props.0.borrow_mut() = Some(use_adornments());
        rsx! { div {} }
    }
    #[test]
    fn framework_resize_reports_initial_and_changed_adornment_widths() {
        let capture = Fixture(Rc::new(RefCell::new(None)));
        let mut dom = VirtualDom::new_with_props(fixture, capture.clone());
        dom.rebuild_in_place();
        let handlers = capture.0.borrow().unwrap();
        handlers.start_resized.call(resized(20.5));
        dom.render_immediate_to_vec();
        assert_eq!(
            capture.0.borrow().unwrap().facts,
            None,
            "both initial sizes are required"
        );
        handlers.end_resized.call(resized(32.0));
        dom.render_immediate_to_vec();
        assert_eq!(
            capture.0.borrow().unwrap().facts,
            Some(Adornments {
                start: 20.5,
                end: 32.0
            })
        );
        handlers.start_resized.call(resized(88.0));
        handlers.end_resized.call(resized(0.0));
        dom.render_immediate_to_vec();
        assert_eq!(
            capture.0.borrow().unwrap().facts,
            Some(Adornments {
                start: 88.0,
                end: 0.0
            })
        );
        handlers.end_resized.call(resized(f64::NAN));
        dom.render_immediate_to_vec();
        assert_eq!(
            capture.0.borrow().unwrap().facts,
            Some(Adornments {
                start: 88.0,
                end: 0.0
            })
        );
    }

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

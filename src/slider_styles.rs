//! Slider recipes translated from the MIT-licensed pinned iX 5.2.1 source.
//! System tokens override provisional fallbacks; the visible thumb stays native.

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-slider-field {
  --ix-slider-thumb--background--active: var(--si-sys-color-background-accent-active, color-mix(in srgb, var(--ix-color-primary) 72%, var(--ix-color-text)));
  --ix-slider-thumb--background--disabled: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-slider-thumb--background--hover: var(--si-sys-color-background-accent-hover, color-mix(in srgb, var(--ix-color-primary) 85%, var(--ix-color-text)));
  --ix-slider-thumb--background--info: var(--si-sys-color-background-information, var(--ix-color-info));
  --ix-slider-thumb--background--invalid--active: var(--si-sys-color-background-danger-active, color-mix(in srgb, var(--ix-color-danger) 72%, var(--ix-color-text)));
  --ix-slider-thumb--background--invalid--hover: var(--si-sys-color-background-danger-hover, color-mix(in srgb, var(--ix-color-danger) 85%, var(--ix-color-text)));
  --ix-slider-thumb--background--invalid: var(--si-sys-color-background-danger, var(--ix-color-danger));
  --ix-slider-thumb--background--success: var(--si-sys-color-background-success, var(--ix-color-success));
  --ix-slider-thumb--background--warning: var(--si-sys-color-background-warning, var(--ix-color-critical));
  --ix-slider-thumb--background: var(--si-sys-color-background-accent, var(--ix-color-primary));
  --ix-slider-thumb--outline-color--focus: var(--si-sys-color-effects-focus, var(--ix-color-focus));
  --ix-slider-trace--background--disabled: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-slider-trace--background--info: var(--si-sys-color-border-information, var(--ix-color-info-text));
  --ix-slider-trace--background--invalid: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-slider-trace--background--success: var(--si-sys-color-border-success, var(--ix-color-success));
  --ix-slider-trace--background--warning: var(--si-sys-color-border-warning, var(--ix-color-warning-text));
  --ix-slider-trace--background: var(--si-sys-color-border-accent, var(--ix-color-primary));
  --ix-slider-trace-marker--background--disabled: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-slider-trace-marker--background--info: var(--si-sys-color-background-information, var(--ix-color-info));
  --ix-slider-trace-marker--background--invalid: var(--si-sys-color-background-danger, var(--ix-color-danger));
  --ix-slider-trace-marker--background--success: var(--si-sys-color-background-success, var(--ix-color-success));
  --ix-slider-trace-marker--background--warning: var(--si-sys-color-background-warning, var(--ix-color-critical));
  --ix-slider-trace-marker--background: var(--si-sys-color-background-accent, var(--ix-color-primary));
  --ix-slider-track--background--disabled: var(--si-sys-color-background-2, var(--ix-color-surface-2));
  --ix-slider-track--background: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-slider-track-marker--background--disabled: var(--si-sys-color-background-2, var(--ix-color-surface-2));
  --ix-slider-track-marker--background: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-slider-thumb--transition-duration: var(--theme-default-time, 150ms);
  --ix-slider--min-height: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-slider--thumb-size: var(--si-sys-sizing-size-50, 16px);
  --ix-slider--height: var(--si-sys-sizing-size-70, 24px);
  --ix-slider-label--margin-top: var(--si-sys-sizing-spacing-y-40, .5rem);
  --ix-slider-input--margin-top: calc( (var(--ix-slider-input--track-height) - var(--ix-slider--thumb-size)) / 2 );
  --ix-slider-trace--height: var(--ix-slider-track--height);
  --ix-slider-trace--border-radius: var(--si-sys-sizing-border-radius-xs, 2px);
  --ix-slider-reference--width: var(--si-sys-sizing-size-10, 2px);
  --ix-slider-reference--height: var(--si-sys-sizing-size-50, 16px);
  --ix-slider-input--track-height: var(--ix-slider-track--height);
  --ix-slider--z-index: 10;
  --ix-slider--outline-width--focus: var(--si-sys-sizing-border-width-default, 1px);
  --ix-slider--outline-offset: var(--si-sys-sizing-focus-ring-offset, 3px);
  --ix-slider-ticks--tick-size: calc(var(--ix-slider--thumb-size) * 0.375);
  --ix-slider-track--height: var(--si-sys-sizing-size-20, 4px);
  --ix-slider-track--border-radius: var(--si-sys-sizing-border-radius-xs, 2px);
  --ix-slider-thumb--border-radius: var(--si-sys-sizing-border-radius-full, 50%);
  --thumb-size: var(--ix-slider--thumb-size);
  --value: 0;
  --trace-start: 0;
  --trace-end: 0;
  --trace-reference: 0;
  --trace-reference-color: var(--ix-slider-track-marker--background);
  --trace-color: var(--ix-slider-trace--background);
  --tick-color: var(--ix-slider-track-marker--background);
  --tick-color--active: var(--ix-slider-trace-marker--background);
  --track-color: var(--ix-slider-track--background);
  display: flex;
  flex-direction: column;
  position: static;
  box-sizing: content-box;
  min-height: var(--ix-slider--min-height);
  justify-content: center;
}
.ix-theme .ix-slider-field input.ix-slider[type=range] {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  left: 0;
  appearance: none;
  background: transparent;
  cursor: pointer;
  outline-offset: revert;
  width: 100%;
  height: var(--ix-slider--height);
  margin: 0;
  box-sizing: border-box;
  font: revert;
  caret-color: auto;
}
.ix-theme .ix-slider-field input.ix-slider[type=range].trace::before {
  content: '';
  position: absolute;
  display: block;
  z-index: -1;
  width: calc((var(--trace-end) - var(--trace-start)) * 100%);
  left: calc(var(--trace-start) * 100%);
  height: var(--ix-slider-trace--height);
  background-color: var(--trace-color);
  top: 50%;
  transform: translateY(-50%);
  border-radius: clamp(0rem, calc((0.01 - var(--trace-start)) * 999rem), var(--ix-slider-trace--border-radius))
    clamp(0rem, calc((var(--trace-end) - 0.99) * 999rem), var(--ix-slider-trace--border-radius))
    clamp(0rem, calc((var(--trace-end) - 0.99) * 999rem), var(--ix-slider-trace--border-radius))
    clamp(0rem, calc((0.01 - var(--trace-start)) * 999rem), var(--ix-slider-trace--border-radius));
}
.ix-theme .ix-slider-field .ix-slider-container,
.ix-theme .ix-slider-field .ix-slider-control,
.ix-theme .ix-slider-field .ix-slider-control input.ix-slider[type=range] { width: 100%; }
.ix-theme .ix-slider-field input.ix-slider[type=range].trace:not(.hide-trace-reference)::after {
  content: '';
  position: absolute;
  display: block;
  width: var(--ix-slider-reference--width);
  height: var(--ix-slider-reference--height);
  background-color: var(--trace-reference-color);
  top: 50%;
  transform: translateY(-50%);
  left: calc(100% * var(--trace-reference));
}
.ix-theme .ix-slider-field input.ix-slider[type=range]::-webkit-slider-runnable-track {
  background: transparent;
  height: var(--ix-slider-input--track-height);
}
.ix-theme .ix-slider-field input.ix-slider[type=range]::-moz-range-track {
  background: transparent;
  height: var(--ix-slider-input--track-height);
}
.ix-theme .ix-slider-field input.ix-slider[type=range]::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  margin-top: var(--ix-slider-input--margin-top);
}
.ix-theme .ix-slider-field input.ix-slider[type=range]::-moz-range-thumb {
  border: none;
  border-radius: 0;
}
.ix-theme .ix-slider-field input.ix-slider[type=range]::-webkit-slider-thumb {
  border-radius: 50%;
  background-color: var(--ix-slider-thumb--background);
  height: var(--thumb-size);
  width: var(--thumb-size);
  transition: all var(--ix-slider-thumb--transition-duration) ease-in-out;
  z-index: var(--ix-slider--z-index);
}
.ix-theme .ix-slider-field input.ix-slider[type=range]:hover::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--hover); }
.ix-theme .ix-slider-field input.ix-slider[type=range]:active::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--active); }
.ix-theme .ix-slider-field input.ix-slider[type=range]::-moz-range-thumb {
  border-radius: 50%;
  background-color: var(--ix-slider-thumb--background);
  height: var(--thumb-size);
  width: var(--thumb-size);
  transition: all var(--ix-slider-thumb--transition-duration) ease-in-out;
  z-index: var(--ix-slider--z-index);
}
.ix-theme .ix-slider-field input.ix-slider[type=range]:hover::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--hover); }
.ix-theme .ix-slider-field input.ix-slider[type=range]:active::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--active); }
.ix-theme .ix-slider-field input.ix-slider[type=range]:focus { outline: none; }
.ix-theme .ix-slider-field input.ix-slider[type=range]:focus-visible::-webkit-slider-thumb {
  outline: var(--ix-slider--outline-width--focus) solid var(--ix-slider-thumb--outline-color--focus);
  outline-offset: var(--ix-slider--outline-offset);
}
.ix-theme .ix-slider-field input.ix-slider[type=range]:focus-visible::-moz-range-thumb {
  outline: var(--ix-slider--outline-width--focus) solid var(--ix-slider-thumb--outline-color--focus);
  outline-offset: var(--ix-slider--outline-offset);
}
.ix-theme .ix-slider-field .ix-slider-ticks {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
  pointer-events: none;
}
.ix-theme .ix-slider-field .ix-slider-ticks circle.ix-slider-tick {
  --tick-size: var(--ix-slider-ticks--tick-size);
  --tick-radius: calc(var(--tick-size) / 2);
  r: var(--tick-radius);
  fill: var(--tick-color);
  transform: translate(calc(var(--tick-value) * (100% - var(--thumb-size)) + var(--thumb-size) / 2), 50%);
}
.ix-theme .ix-slider-field .ix-slider-ticks .ix-slider-tick.ix-slider-tick-at-min { transform: translate(var(--tick-radius), 50%); }
.ix-theme .ix-slider-field .ix-slider-ticks .ix-slider-tick.ix-slider-tick-at-max { transform: translate(calc(100% - var(--tick-radius)), 50%); }
.ix-theme .ix-slider-field .ix-slider-ticks .ix-slider-tick.ix-slider-tick-active { fill: var(--tick-color--active); }
.ix-theme .ix-slider-field .ix-slider-control {
  position: relative;
  display: block;
  width: 100%;
  height: var(--ix-slider--height);
}
.ix-theme .ix-slider-field .ix-slider-track {
  position: absolute;
  background-color: var(--track-color);
  height: var(--ix-slider-track--height);
  width: 100%;
  top: 50%;
  transform: translateY(-50%);
  left: 0;
  border-radius: var(--ix-slider-track--border-radius);
}
.ix-theme .ix-slider-field .ix-slider-thumb-anchor {
  display: block;
  position: absolute;
  background-color: transparent;
  height: var(--thumb-size);
  width: var(--thumb-size);
  border-radius: var(--ix-slider-thumb--border-radius);
  top: 50%;
  transform: translateY(-50%);
}
.ix-theme .ix-slider-field .ix-slider-endpoints {
  display: flex;
  position: relative;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  margin-top: var(--ix-slider-label--margin-top);
  min-height: 0;
  color: inherit;
  font: var(--si-sys-typography-body, 14px/20px var(--ix-font));
  font-feature-settings: 'clig' off, 'liga' off;
  font-style: normal;
  letter-spacing: var(--si-ref-typography-letter-spacing-normal, normal);
  text-decoration: none;
  -webkit-font-smoothing: antialiased;
}
.ix-theme .ix-slider-field:is(.ix-field-invalid,.invalid) {
  --trace-color: var(--ix-slider-trace--background--invalid);
  --tick-color--active: var(--ix-slider-trace-marker--background--invalid);
}
.ix-theme .ix-slider-field:is(.ix-field-invalid,.invalid) input.ix-slider[type=range]::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--invalid); }
.ix-theme .ix-slider-field:is(.ix-field-invalid,.invalid) input.ix-slider[type=range]:hover::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--invalid--hover); }
.ix-theme .ix-slider-field:is(.ix-field-invalid,.invalid) input.ix-slider[type=range]:active::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--invalid--active); }
.ix-theme .ix-slider-field:is(.ix-field-invalid,.invalid) input.ix-slider[type=range]::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--invalid); }
.ix-theme .ix-slider-field:is(.ix-field-invalid,.invalid) input.ix-slider[type=range]:hover::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--invalid--hover); }
.ix-theme .ix-slider-field:is(.ix-field-invalid,.invalid) input.ix-slider[type=range]:active::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--invalid--active); }
.ix-theme .ix-slider-field:is([data-disabled=true],.disabled) {
  pointer-events: none;
  --track-color: var(--ix-slider-track--background--disabled);
  --tick-color: var(--ix-slider-track-marker--background--disabled);
  --trace-color: var(--ix-slider-trace--background--disabled);
  --tick-color--active: var(--ix-slider-trace-marker--background--disabled);
}
.ix-theme .ix-slider-field:is([data-disabled=true],.disabled) input.ix-slider[type=range]::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--disabled); }
.ix-theme .ix-slider-field:is([data-disabled=true],.disabled) input.ix-slider[type=range]::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--disabled); }
.ix-theme .ix-slider-field:is(.ix-field-warning,.warning) {
  --trace-color: var(--ix-slider-trace--background--warning);
  --tick-color--active: var(--ix-slider-trace-marker--background--warning);
}
.ix-theme .ix-slider-field:is(.ix-field-warning,.warning) input.ix-slider[type=range]::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--warning); }
.ix-theme .ix-slider-field:is(.ix-field-warning,.warning) input.ix-slider[type=range]::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--warning); }
.ix-theme .ix-slider-field:is(.ix-field-info,.info) {
  --trace-color: var(--ix-slider-trace--background--info);
  --tick-color--active: var(--ix-slider-trace-marker--background--info);
}
.ix-theme .ix-slider-field:is(.ix-field-info,.info) input.ix-slider[type=range]::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--info); }
.ix-theme .ix-slider-field:is(.ix-field-info,.info) input.ix-slider[type=range]::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--info); }
.ix-theme .ix-slider-field:is(.ix-field-valid,.valid) {
  --trace-color: var(--ix-slider-trace--background--success);
  --tick-color--active: var(--ix-slider-trace-marker--background--success);
}
.ix-theme .ix-slider-field:is(.ix-field-valid,.valid) input.ix-slider[type=range]::-webkit-slider-thumb { background-color: var(--ix-slider-thumb--background--success); }
.ix-theme .ix-slider-field:is(.ix-field-valid,.valid) input.ix-slider[type=range]::-moz-range-thumb { background-color: var(--ix-slider-thumb--background--success); }
/* Field-wrapper text keeps its source system-token roles over legacy helper paint. */
.ix-theme .ix-slider-field .ix-text-field-bottom-text { color: var(--si-sys-color-text-primary, var(--ix-color-text)); }
.ix-theme .ix-slider-field .ix-text-field-bottom-text.ix-text-soft { color: var(--si-sys-color-text-secondary, var(--ix-color-text-secondary)); }
.ix-theme .ix-slider-field .ix-text-field-bottom-text.ix-text-alarm { color: var(--si-sys-color-text-danger, var(--ix-color-danger-text)); }
/* Existing Rust-only composition, caption and value-tooltip adapters. */
.ix-theme .ix-range-field > .ix-slider-field { flex: 1; min-width: 0; }
.ix-theme .ix-slider-field .ix-slider-marker-captions {
  position: relative;
  height: 1.5rem;
  font-size: .75rem;
  color: var(--ix-color-text-secondary);
}
.ix-theme .ix-slider-field .ix-slider-marker-captions > span { position: absolute; transform: translateX(-50%); }
.ix-theme .ix-slider-field .ix-slider-tooltip {
  position: absolute;
  bottom: 100%;
  transform: translateX(-50%);
  font-size: .75rem;
  line-height: 1.5rem;
  padding: 0 .375rem;
  border-radius: var(--ix-radius);
  background: var(--ix-color-surface-2);
  color: var(--ix-color-text);
  pointer-events: none;
  white-space: nowrap;
}
"#;

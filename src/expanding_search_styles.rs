//! ExpandingSearch recipes translated from the MIT-licensed pinned iX 5.2.1 source.
//! Numeric defaults remain provisional; public system/component tokens take precedence.

pub(crate) const STYLES: &str = r#"
.ix-theme {
  --_ix-search-size-150: 160px;
  --_ix-search-space-80: 24px;
  --_ix-search-space-90: 32px;
  --_ix-search-space-100: 40px;
}
.ix-theme[data-ix-density=compact] {
  --_ix-search-size-150: 120px;
  --_ix-search-space-80: 20px;
  --_ix-search-space-90: 24px;
  --_ix-search-space-100: 32px;
}
.ix-theme .ix-expanding-search {
  --ix-input--background: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-input--background--autofill: rgba(0, 0, 0, 0);
  --ix-input--background--disabled: rgba(0, 0, 0, 0);
  --ix-input--background--focus: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input--background--hover: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input--background--invalid: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-input--background--invalid--focus: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input--background--invalid--hover: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input--background--readonly: rgba(0, 0, 0, 0);
  --ix-input--background--warning: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-input--background--warning--focus: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input--background--warning--hover: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input--border-color: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-input--border-color--autofill: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-input--border-color--disabled: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-input--border-color--focus: var(--si-sys-color-border-1, var(--ix-color-text-secondary));
  --ix-input--border-color--hover: var(--si-sys-color-border-1, var(--ix-color-text-secondary));
  --ix-input--border-color--info: var(--si-sys-color-border-information, var(--ix-color-info-text));
  --ix-input--border-color--info--active: var(--si-sys-color-border-information, var(--ix-color-info-text));
  --ix-input--border-color--info--hover: var(--si-sys-color-border-information, var(--ix-color-info-text));
  --ix-input--border-color--invalid: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-input--border-color--invalid--active: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-input--border-color--invalid--hover: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-input--border-color--readonly: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-input--border-color--warning: var(--si-sys-color-border-warning, var(--ix-color-warning-text));
  --ix-input--border-color--warning--active: var(--si-sys-color-border-warning, var(--ix-color-warning-text));
  --ix-input--border-color--warning--hover: var(--si-sys-color-border-warning, var(--ix-color-warning-text));
  --ix-input--border-color-bottom--disabled: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-input--border-color-bottom--readonly: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-input--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-input--color--autofill: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-input--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-input--outline-color--focus: var(--si-sys-color-effects-focus, var(--ix-color-focus));
  --ix-input-error--background: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-input-error--border-color: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-input-error-icon--color: var(--si-sys-color-text-danger, var(--ix-color-danger-text));
  --ix-input-extra--background--active: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input-extra--background--hover: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-input-gripper--color: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-input-gripper--color--focus: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-input-gripper--color--hover: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-input-hint--color: var(--si-sys-color-text-secondary, var(--ix-color-text-secondary));
  --ix-input-search-icon--color: var(--si-sys-color-text-accent, var(--ix-color-secondary));
  --ix-input-search-icon--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-input-search-icon--color--focus: var(--si-sys-color-text-accent, var(--ix-color-secondary));
  --ix-input-search-icon--color--hover: var(--si-sys-color-text-accent-hover, var(--ix-color-primary));
  --ix-input-select-icon--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-input-select-icon--color--active: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-input-select-icon--color--hover: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-input-unit--color: var(--si-sys-color-text-secondary, var(--ix-color-text-secondary));
  --ix-input--border-radius: var(--si-sys-sizing-border-radius-xs, 2px);
  --ix-input--border-width: var(--si-sys-sizing-border-width-default, 1px);
  --ix-input--outline-width--focus: var(--si-sys-sizing-border-width-default, 1px);
  --ix-input--focus--outline-offset: var(--si-sys-sizing-focus-ring-offset, 3px);
  --ix-input--min-height: var(--si-sys-sizing-size-80, var(--_ix-input-size-80));
  --ix-input--padding: var(--si-sys-sizing-spacing-y-20, .25rem) var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-input-spin-button--margin-right: calc( var(--si-sys-sizing-spacing-x-10, .125rem) * -1 );
  --ix-input-spin-button--margin-left: var(--si-sys-sizing-spacing-x-10, .125rem);
  --ix-input-textarea--padding: calc( var(--si-sys-sizing-spacing-y-30, .375rem) - var(--ix-input--border-width) ) calc(var(--si-sys-sizing-spacing-x-40, .5rem) - var(--ix-input--border-width));
  --ix-input-textarea--height: calc( var(--si-sys-sizing-size-100, var(--_ix-input-size-100)) + 2 * (var(--si-sys-sizing-spacing-y-20, .25rem) - var(--si-sys-sizing-spacing-y-10, .125rem)) );
  --ix-input-start-container--margin-left: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-input-icon-compact--margin-left: var(--si-sys-sizing-spacing-x-20, .25rem);
  --ix-input-end-container--inset: var(--si-sys-sizing-spacing-x-20, .25rem);
  --ix-input-end-container--margin-right: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-input-icon-compact--margin-right: var(--si-sys-sizing-spacing-x-20, .25rem);
  --ix-input-bottom-text--margin-top: var(--si-sys-sizing-spacing-y-20, .25rem);
  --ix-input-bottom-text--margin-bottom: var(--si-sys-sizing-spacing-y-20, .25rem);
  --ix-input-show-stepper-buttons--min-width: var(--si-sys-sizing-size-130, 80px);
  --ix-input-password-eye--margin-left: var(--si-sys-sizing-spacing-x-10, .125rem);
  --ix-expanding-search-button-search--outline-color--focus: var(--focus--border-color, var(--si-sys-color-effects-focus, var(--ix-color-focus)));
  --ix-expanding-search-button--border--active: var(--si-sys-sizing-border-width-default, 1px) solid var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-expanding-search-input-container--transition-duration: var(--theme-medium-time, 300ms);
  --ix-expanding-search-button--transition-duration: var(--theme-default-time, 150ms);
  --ix-expanding-search--border-radius: var(--si-sys-sizing-border-radius-xs, 2px);
  --ix-expanding-search--container-width: calc(var(--si-sys-sizing-size-150, var(--_ix-search-size-150)) + var(--si-sys-sizing-spacing-x-80, var(--_ix-search-space-80)) + var(--si-sys-sizing-border-width-default, 1px));
  --ix-expanding-search--height: var(--ix-input--min-height);
  --ix-expanding-search-collapsed--width: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-expanding-search-button-search--max-width: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-expanding-search-button-search--max-height: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-expanding-search-button-search-expanded--margin-left: var(--si-sys-sizing-spacing-x-20, .25rem);
  --ix-expanding-search-button-search-active--width: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-expanding-search-button-search-active--height: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-expanding-search-button-clear--margin-right: var(--si-sys-sizing-spacing-x-20, .25rem);
  --ix-expanding-search-input--padding-left: var(--si-sys-sizing-spacing-x-90, var(--_ix-search-space-90));
  --ix-expanding-search-input--padding-right: var(--si-sys-sizing-spacing-x-100, var(--_ix-search-space-100));
  --ix-expanding-search--outline-width--focus: var(--si-sys-sizing-border-width-default, 1px);
  display: inline-flex;
  box-sizing: content-box;
  width: var(--ix-expanding-search-collapsed--width);
  height: var(--ix-expanding-search--height);
  align-items: center;
  justify-content: space-between;
  position: relative;
  transition: width var(--ix-expanding-search-input-container--transition-duration) ease-in-out;
}
.ix-theme .ix-expanding-search.expanded { width: var(--ix-expanding-search--container-width); }
.ix-theme .ix-expanding-search.expanded.fullWidth { width: 100%; }
.ix-theme .ix-expanding-search > .ix-search-input-container {
  display: flex;
  position: absolute;
  inset-inline-start: 0;
  width: 100%;
  align-items: center;
  flex-wrap: nowrap;
}
.ix-theme .ix-expanding-search .ix-search-input {
  color: var(--ix-input--color);
  border-radius: var(--ix-input--border-radius);
  height: var(--ix-input--min-height);
  min-height: var(--ix-input--min-height);
  min-width: var(--ix-input--min-height);
  background-color: var(--ix-input--background);
  border: solid var(--ix-input--border-width) var(--ix-input--border-color);
  padding: var(--ix-input--padding);
  font: var(--si-sys-typography-body, 14px/20px var(--ix-font));
  font-feature-settings: 'clig' off, 'liga' off;
  font-style: normal;
  letter-spacing: var(--si-ref-typography-letter-spacing-normal, normal);
  text-decoration: none;
  -webkit-font-smoothing: antialiased;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: auto !important;
  width: 100%;
  transition-property: padding, opacity;
  transition-duration: var(--ix-expanding-search-input-container--transition-duration), var(--theme-default-time, 150ms);
  transition-timing-function: ease-in-out;
}
.ix-theme .ix-expanding-search .ix-search-input::placeholder {
  color: var(--ix-input-hint--color);
  transition: opacity var(--theme-default-time, 150ms) ease-in-out;
}
.ix-theme .ix-expanding-search .ix-search-input:not(:read-only):not([readonly]):not(.readonly):not(.disabled):not(:disabled):is(:hover,.hover) {
  border-color: var(--ix-input--border-color--hover) !important;
  background-color: var(--ix-input--background--hover);
}
.ix-theme .ix-expanding-search .ix-search-input:not(:read-only):not([readonly]):not(.readonly):not(.disabled):not(:disabled):focus-visible {
  outline: var(--ix-input--outline-width--focus) solid var(--ix-input--outline-color--focus);
  outline-offset: var(--ix-input--focus--outline-offset);
  border-color: var(--ix-input--border-color--focus) !important;
}
.ix-theme .ix-expanding-search .ix-search-input:focus-visible { color: var(--ix-input--color); }
.ix-theme .ix-expanding-search .ix-search-input:is(.readonly,[readonly]) {
  background: transparent !important;
  border-block-start: none !important;
  border-inline-start: none !important;
  border-inline-end: none !important;
  border-radius: 0;
}
.ix-theme .ix-expanding-search .ix-search-input:is(:read-only,[readonly],.readonly) {
  outline: none !important;
  border-color: var(--ix-input--border-color--readonly);
  cursor: default !important;
}
.ix-theme .ix-expanding-search .ix-search-input:is(:disabled,.disabled) {
  background: transparent !important;
  border-block-start: none !important;
  border-inline-start: none !important;
  border-inline-end: none !important;
  border-radius: 0;
  color: var(--ix-input--color--disabled);
  border-color: var(--ix-input--border-color--disabled);
}
.ix-theme .ix-expanding-search .ix-search-input:is(:read-only,[readonly],.readonly,:disabled,.disabled)::placeholder { color: transparent; }
.ix-theme .ix-expanding-search .ix-search-input-expanded {
  padding-left: var(--ix-expanding-search-input--padding-left) !important;
  padding-right: var(--ix-expanding-search-input--padding-right) !important;
}
.ix-theme .ix-expanding-search .ix-search-input-container[data-expanded=false] .ix-search-input { opacity: 0; }
.ix-theme .ix-expanding-search .ix-search-input-container[data-expanded=false] .ix-search-input::placeholder { opacity: 0; }
.ix-theme .ix-expanding-search .ix-search-input-container[data-expanded=true] .ix-search-input { opacity: 1; }
.ix-theme .ix-expanding-search .ix-search-input-container[data-expanded=true] .ix-search-input::placeholder {
  opacity: 1;
  transition-delay: var(--ix-expanding-search-button--transition-duration);
}
.ix-theme .ix-expanding-search .ix-search-trigger-host,
.ix-theme .ix-expanding-search .ix-search-clear-host {
  --ix-icon-button-extra-small--size: var(--si-sys-sizing-size-50, 16px);
  --ix-icon-button-small--size: var(--si-sys-sizing-size-70, 24px);
  --ix-icon-button-large--size: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-icon-button-btn-oval--border-radius: var(--si-sys-sizing-border-radius-full, 50%);
  display: inline-flex;
  box-sizing: border-box;
  width: var(--ix-icon-button-large--size);
  height: var(--ix-icon-button-large--size);
  min-width: var(--ix-icon-button-large--size);
  min-height: var(--ix-icon-button-large--size);
  justify-content: center;
  align-items: center;
  vertical-align: middle;
  cursor: pointer;
  font: var(--si-sys-typography-body-sbold, 600 14px/22px var(--ix-font));
  font-feature-settings: 'clig' off, 'liga' off;
  font-style: normal;
  letter-spacing: var(--si-ref-typography-letter-spacing-normal, normal);
  text-decoration: none;
  -webkit-font-smoothing: antialiased;
}
.ix-theme .ix-expanding-search .ix-search-trigger-host[data-expanded=true],
.ix-theme .ix-expanding-search .ix-search-clear-host {
  width: var(--ix-icon-button-small--size);
  height: var(--ix-icon-button-small--size);
  min-width: var(--ix-icon-button-small--size);
  min-height: var(--ix-icon-button-small--size);
}
.ix-theme .ix-expanding-search .ix-search-trigger-host > .ix-button.ix-icon-button-native.ix-search-trigger,
.ix-theme .ix-expanding-search .ix-search-clear-host > .ix-button.ix-icon-button-native.ix-search-clear {
  width: 100%;
  height: 100%;
  min-width: auto;
  min-height: auto;
  display: inline-flex;
  flex-shrink: 1;
  transition-property: all;
}
.ix-theme .ix-expanding-search > .ix-search-trigger-host {
  display: flex;
  max-width: var(--ix-expanding-search-button-search--max-width);
  max-height: var(--ix-expanding-search-button-search--max-height);
  border-radius: var(--ix-expanding-search--border-radius);
  transition: all var(--ix-expanding-search-button--transition-duration) ease-in-out;
  z-index: 1;
  align-items: center;
  position: relative;
  border: none;
}
.ix-theme .ix-expanding-search > .ix-search-trigger-host:is(:hover,.hover,:active,.active) { border-color: transparent; }
.ix-theme .ix-expanding-search > .ix-search-trigger-host:has(> .ix-search-trigger:focus-visible) {
  outline: var(--ix-expanding-search-button-search--outline-color--focus);
  outline-width: var(--ix-expanding-search--outline-width--focus);
  outline-style: solid;
}
.ix-theme .ix-expanding-search > .ix-search-trigger-host.ix-search-trigger-expanded {
  margin-left: var(--ix-expanding-search-button-search-expanded--margin-left);
  pointer-events: none;
}
.ix-theme .ix-expanding-search > .ix-search-trigger-host:active {
  width: var(--ix-expanding-search-button-search-active--width);
  height: var(--ix-expanding-search-button-search-active--height);
  border-radius: var(--ix-expanding-search--border-radius);
  border: var(--ix-expanding-search-button--border--active) !important;
}
.ix-theme .ix-expanding-search .ix-search-clear-host {
  position: absolute;
  border-radius: var(--ix-expanding-search--border-radius);
  right: 0;
  margin-right: var(--ix-expanding-search-button-clear--margin-right);
}
.ix-theme .ix-expanding-search .ix-search-trigger-host:has(> button:disabled),
.ix-theme .ix-expanding-search .ix-search-clear-host:has(> button:disabled) { cursor: default; }
"#;

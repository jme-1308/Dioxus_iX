//! Select recipes translated from the pinned MIT-licensed iX 5.2.1 source.
//! SPDX-FileCopyrightText: 2023 Siemens AG
//! SPDX-License-Identifier: MIT
//! Public system tokens override provisional fallbacks. Private wrappers project
//! source shadow layers without changing generic Dropdown or FilterChip styles.

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-select.ix-select-visual {
  --ix-select--outline-color--focus: var(--si-sys-color-effects-focus, var(--ix-color-focus));
  --ix-select--background: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-select--background--hover: var(--si-sys-color-background-4, var(--ix-color-surface-2));
  --ix-select--background--invalid: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-select--background--warning: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-select--border-color: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-select--border-color--disabled: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-select--border-color--hover: var(--si-sys-color-border-1, var(--ix-color-text-secondary));
  --ix-select--border-color--info: var(--si-sys-color-border-information, var(--ix-color-info-text));
  --ix-select--border-color--info--active: var(--si-sys-color-border-information, var(--ix-color-info-text));
  --ix-select--border-color--info--hover: var(--si-sys-color-border-information, var(--ix-color-info-text));
  --ix-select--border-color--invalid: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-select--border-color--invalid--active: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-select--border-color--invalid--hover: var(--si-sys-color-border-danger, var(--ix-color-danger-text));
  --ix-select--border-color--readonly: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-select--border-color--warning: var(--si-sys-color-border-warning, var(--ix-color-warning-text));
  --ix-select--border-color--warning--active: var(--si-sys-color-border-warning, var(--ix-color-warning-text));
  --ix-select--border-color--warning--hover: var(--si-sys-color-border-warning, var(--ix-color-warning-text));
  --ix-select--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-select--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-select--color--readonly: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-select-item-hint--color: var(--si-sys-color-text-secondary, var(--ix-color-text-secondary));
  --ix-select-placeholder--color: var(--si-sys-color-text-secondary, var(--ix-color-text-secondary));
  --ix-select-overflow-button--color--active: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-select--focus--outline-offset: var(--si-sys-sizing-focus-ring-offset, 3px);
  --ix-select--border-radius: var(--si-sys-sizing-border-radius-xs, 2px);
  --ix-select--border-width: var(--si-sys-sizing-border-width-default, 1px);
  --ix-select-chip--horizontal-margin: calc(var(--si-sys-sizing-spacing-x-10, .125rem) * 0.8);
  --ix-select-trigger--min-width: calc(var(--si-sys-sizing-size-120, 80px) - var(--si-sys-sizing-spacing-x-10, .125rem));
  --ix-select-overflow-chip--fallback-width: calc(var(--si-sys-sizing-size-100, 48px) + var(--si-sys-sizing-spacing-x-20, .25rem));
  --ix-select--min-height: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-select--padding: 0 0 0 var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-select-chips--max-width: min(40%, calc(var(--si-sys-sizing-size-150, 160px) + var(--si-sys-sizing-spacing-x-120, 48px)));
  --ix-select-icon--icon-size: calc(var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px)) - var(--si-sys-sizing-spacing-y-10, .125rem));
  --ix-select-icon--icon-size--icon: calc(var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px)) - var(--si-sys-sizing-spacing-x-10, .125rem));
  --ix-select-input-container--height: calc(var(--si-sys-sizing-size-70, 24px) + var(--si-sys-sizing-spacing-y-20, .25rem));
  --ix-select-overflow-dropdown-content--gap: var(--si-sys-sizing-spacing-y-20, .25rem);
  --ix-select-overflow-dropdown-content--max-height: calc(var(--si-sys-sizing-size-160, 192px) + var(--si-sys-sizing-spacing-y-60, 1rem));
  --ix-select-overflow-dropdown-content--padding: var(--si-sys-sizing-spacing-y-40, .5rem) var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-select-list-header--height: var(--si-sys-sizing-size-80, var(--_ix-input-size-80, 32px));
  --ix-select-list-header--margin: 0 var(--si-sys-sizing-spacing-x-40, .5rem) 0 var(--si-sys-sizing-spacing-x-60, 1rem);
  --ix-select--outline-width--focus: var(--si-sys-sizing-border-width-default, 1px);
  --ix-select-chip-horizontal-margin: var(--ix-select-chip--horizontal-margin);
  --ix-select-trigger-min-width: var(--ix-select-trigger--min-width);
  --ix-select-overflow-chip-fallback-width: var(--ix-select-overflow-chip--fallback-width);
  display: inline-block;
  position: relative;
  min-height: var(--ix-select--min-height);
  height: auto;
  border-radius: var(--ix-select--border-radius);
  gap: normal;
  box-sizing: content-box;
  pointer-events: auto;
}
.ix-theme .ix-select.ix-select-visual :is(.ix-text-field-top,.ix-text-field-label,.ix-text-field-label .ix-typography,.ix-text-field-slot,.ix-text-field-bottom,.ix-text-field-bottom-text) {
  box-sizing: content-box;
}
.ix-theme .ix-select.ix-select-visual .ix-text-field-bottom-text {
  min-height: auto;
  color: var(--si-sys-color-text-secondary, var(--ix-color-text-secondary));
}
.ix-theme .ix-select.ix-select-visual .ix-text-field-bottom-text:is(.ix-field-valid,.ix-field-info,.ix-field-warning) { color: var(--si-sys-color-text-primary, var(--ix-color-text)); }
.ix-theme .ix-select.ix-select-visual .ix-text-field-bottom-text.ix-field-invalid { color: var(--si-sys-color-text-danger, var(--ix-color-danger-text)); }
.ix-theme .ix-select.ix-select-visual .ix-select-control {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  background-color: var(--ix-select--background);
  border: var(--ix-select--border-width) solid var(--ix-select--border-color);
  border-radius: var(--ix-select--border-radius);
  padding: var(--ix-select--padding);
  outline: none;
}
.ix-theme .ix-select.ix-select-visual .ix-select-direction-probe {
  position: absolute;
  visibility: hidden;
  pointer-events: none;
  box-sizing: border-box;
  width: 8px;
  height: 1px;
  padding: 0;
  padding-left: 4px;
  padding-inline-start: 0;
  writing-mode: horizontal-tb;
}
.ix-theme .ix-select.ix-select-visual .ix-select-control:hover:not([data-readonly=true]):not([data-disabled=true]),
.ix-theme .ix-select.ix-select-visual .ix-select-control[data-open=true]:not([data-readonly=true]):not([data-disabled=true]) {
  background-color: var(--ix-select--background--hover);
  border-color: var(--ix-select--border-color--hover);
}
.ix-theme .ix-select.ix-select-visual:is(.show-focus-outline,.ix-focused) .ix-select-control:not([data-readonly=true]):not([data-disabled=true]) {
  outline: var(--ix-select--outline-width--focus) solid var(--ix-select--outline-color--focus);
  outline-offset: var(--ix-select--focus--outline-offset);
}
.ix-theme .ix-select.ix-select-visual .ix-select-control[data-readonly=true] {
  border: var(--ix-select--border-width) solid var(--ix-select--border-color--readonly);
}
.ix-theme .ix-select.ix-select-visual .ix-select-control[data-disabled=true] {
  border: var(--ix-select--border-width) solid var(--ix-select--border-color--disabled);
}
.ix-theme .ix-select.ix-select-visual .ix-select-control:is([data-readonly=true],[data-disabled=true]) {
  background-color: transparent !important;
  outline: none;
}
.ix-theme .ix-select.ix-select-visual .ix-select-input-container {
  display: flex;
  position: relative;
  align-items: flex-start;
  width: 100%;
  height: 100%;
}
.ix-theme .ix-select.ix-select-visual .ix-select-chips {
  position: relative;
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  height: 100%;
  flex: 1 1 auto;
  overflow: hidden;
  padding: 0;
  gap: normal;
  opacity: 1;
  overflow-wrap: normal;
}
.ix-theme .ix-select.ix-select-visual .ix-select-trigger-rail {
  display: flex;
  align-items: center;
  flex-grow: 1;
  height: 100%;
}
.ix-theme .ix-select.ix-select-visual input.ix-select-trigger {
  display: inline-block;
  background: transparent;
  color: var(--ix-select--color);
  height: var(--ix-select-input-container--height);
  width: 100%;
  min-height: auto;
  min-width: auto;
  flex: 0 1 auto;
  padding: revert;
  margin: 0;
  border: none;
  border-radius: 0;
  outline: none;
  outline-offset: revert;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font: var(--si-sys-typography-body, normal 14px/20px var(--ix-font));
  font-feature-settings: 'clig' off, 'liga' off;
  font-style: normal;
  letter-spacing: var(--si-ref-typography-letter-spacing-normal, normal);
  text-decoration: none;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  opacity: 1;
  cursor: revert;
  caret-color: auto;
  text-align: start;
}
.ix-theme .ix-select.ix-select-visual input.ix-select-trigger::placeholder {
  color: var(--ix-select-placeholder--color);
  opacity: 1;
}
.ix-theme .ix-select.ix-select-visual input.ix-select-trigger-multiple::placeholder { opacity: 0; }
.ix-theme .ix-select.ix-select-visual .ix-select-control[data-readonly=true] input.ix-select-trigger { color: var(--ix-select--color--readonly); }
.ix-theme .ix-select.ix-select-visual .ix-select-control[data-disabled=true] input.ix-select-trigger { color: var(--ix-select--color--disabled); }
.ix-theme .ix-select.ix-select-visual .ix-select-action-host {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  height: var(--ix-select-icon--icon-size);
  width: var(--ix-select-icon--icon-size--icon);
  min-height: var(--ix-select-icon--icon-size);
  min-width: var(--ix-select-icon--icon-size--icon);
  font: var(--si-sys-typography-body-sbold, 600 14px/20px var(--ix-font));
  font-feature-settings: 'clig' off, 'liga' off;
  font-style: normal;
  letter-spacing: var(--si-ref-typography-letter-spacing-normal, normal);
  cursor: pointer;
}
.ix-theme .ix-select.ix-select-visual .ix-select-action-host > .ix-button.ix-icon-button-native {
  width: 100%;
  height: 100%;
  min-width: auto;
  min-height: auto;
  flex: 0 1 auto;
  padding: 0;
  gap: normal;
  white-space: normal;
  transition-property: all;
}
.ix-theme .ix-select.ix-select-visual .ix-select-control[data-open=true] .ix-select-chevron-host { --ix-icon-button-color: var(--ix-select-overflow-button--color--active); }
.ix-theme .ix-select.ix-select-visual .ix-select-chip-overflow { all: unset; }
.ix-theme .ix-select.ix-select-visual .ix-select-chip {
  --ix-filter-chip--background: transparent;
  --ix-filter-chip--border-color--disabled: var(--si-sys-color-border-3, var(--ix-color-border));
  --ix-filter-chip--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-filter-chip--background--hover: var(--si-sys-color-background-accent-secondary-hover, var(--ix-color-surface-2));
  --ix-filter-chip--background--active: var(--si-sys-color-background-accent-secondary-active, var(--ix-color-surface-2));
  --ix-filter-chip-primary-outline--border-color: var(--si-sys-color-border-accent, var(--ix-color-primary));
  --ix-filter-chip-primary-outline--border-color--active: var(--si-sys-color-border-accent-active, var(--ix-color-primary));
  --ix-filter-chip-primary-outline--border-color--hover: var(--si-sys-color-border-accent-hover, var(--ix-color-primary));
  --ix-filter-chip-primary-outline--color: var(--si-sys-color-border-accent, var(--ix-color-primary));
  --ix-filter-chip-primary-outline--color--active: var(--si-sys-color-border-accent-active, var(--ix-color-primary));
  --ix-filter-chip-primary-outline--color--hover: var(--si-sys-color-border-accent-hover, var(--ix-color-primary));
  --ix-filter-chip--padding-left: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-filter-chip-icon--margin-left: var(--si-sys-sizing-spacing-x-20, .25rem);
  --ix-filter-chip-disabled--padding-right: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-filter-chip-hide-close-button-readonly--padding-right: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-filter-chip--border-width: var(--si-sys-sizing-border-width-default, 1px);
  --ix-filter-chip--height: var(--si-sys-sizing-size-70, 24px);
  --ix-filter-chip--border-radius: var(--si-sys-sizing-spacing-x-90, 32px);
  --ix-filter-chip-slot-container--line-height: var(--si-sys-sizing-size-70, 24px);
  --ix-filter-chip-icon--icon-size: var(--si-sys-sizing-size-70, 24px);
  display: inline-flex;
  box-sizing: border-box;
  position: static;
  align-items: center;
  justify-content: space-between;
  height: var(--ix-filter-chip--height);
  padding: 0;
  padding-left: var(--ix-filter-chip--padding-left);
  border: var(--ix-filter-chip--border-width) solid var(--ix-filter-chip-primary-outline--border-color);
  border-radius: var(--ix-filter-chip--border-radius);
  background-color: var(--ix-filter-chip--background);
  color: var(--ix-filter-chip-primary-outline--color);
  font: inherit;
  line-height: inherit;
  white-space: normal;
  overflow: visible;
  cursor: pointer;
}
.ix-theme .ix-select.ix-select-visual :is(.ix-select-chips,.ix-select-measure) > .ix-select-chip {
  margin: var(--ix-select-chip-horizontal-margin);
  flex: 0 0 auto;
  min-width: auto;
  max-width: var(--ix-select-chips--max-width);
}
.ix-theme .ix-select.ix-select-visual .ix-select-chip-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: var(--ix-filter-chip-slot-container--line-height);
  min-width: 0;
}
.ix-theme .ix-select.ix-select-visual .ix-select-chip-close-host {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: var(--ix-filter-chip-icon--icon-size);
  width: var(--si-sys-sizing-size-70, 24px);
  min-width: var(--si-sys-sizing-size-70, 24px);
  min-height: var(--si-sys-sizing-size-70, 24px);
  margin-left: var(--ix-filter-chip-icon--margin-left);
  font: var(--si-sys-typography-body-sbold, 600 14px/20px var(--ix-font));
  font-feature-settings: 'clig' off, 'liga' off;
  font-style: normal;
  letter-spacing: var(--si-ref-typography-letter-spacing-normal, normal);
  cursor: pointer;
}
.ix-theme .ix-select.ix-select-visual .ix-select-chip-close-host > .ix-button.ix-icon-button-native {
  height: 100%;
  width: 100%;
  min-height: auto;
  min-width: auto;
  margin: 0;
  padding: 0;
  flex-shrink: 1;
  gap: normal;
  white-space: normal;
  transition-property: all;
}
.ix-theme .ix-select.ix-select-visual .ix-select-chip:not([data-chip-disabled=true]):hover {
  background-color: var(--ix-filter-chip--background--hover);
  border-color: var(--ix-filter-chip-primary-outline--border-color--hover);
  color: var(--ix-filter-chip-primary-outline--color--hover);
}
.ix-theme .ix-select.ix-select-visual .ix-select-chip:not([data-chip-disabled=true]):active {
  background-color: var(--ix-filter-chip--background--active);
  border-color: var(--ix-filter-chip-primary-outline--border-color--active);
  color: var(--ix-filter-chip-primary-outline--color--active);
}
.ix-theme .ix-select.ix-select-visual .ix-select-chip[data-chip-disabled=true] {
  border-color: var(--ix-filter-chip--border-color--disabled);
  color: var(--ix-filter-chip--color--disabled);
  padding-right: var(--ix-filter-chip-disabled--padding-right);
  cursor: default;
}
.ix-theme .ix-select.ix-select-visual .ix-select-chip[data-hide-close=true] { padding-right: var(--ix-filter-chip-hide-close-button-readonly--padding-right); }
.ix-theme .ix-select.ix-select-visual .ix-select-chip.ix-select-chip-overflow { cursor: pointer; }
.ix-theme .ix-select.ix-select-visual .ix-select-chip-overflow:focus-visible {
  outline: var(--ix-select--outline-width--focus) solid var(--ix-select--outline-color--focus);
  outline-offset: var(--ix-select--focus--outline-offset);
}
.ix-theme .ix-select.ix-select-visual .ix-select-measure {
  position: fixed;
  inset: 0 auto auto 0;
  display: flex;
  gap: 0;
  height: auto;
  visibility: hidden;
  pointer-events: none;
}
.ix-theme .ix-select.ix-select-visual .ix-select-trigger-size {
  display: block;
  position: fixed;
  inset: 0 auto auto 0;
  width: var(--ix-select-trigger-min-width);
  height: 0;
  flex: none;
  visibility: hidden;
  pointer-events: none;
}
.ix-theme .ix-select.ix-select-visual > .ix-select-backdrop {
  z-index: calc(var(--ix-dropdown--z-index, var(--theme-z-index-dropdown, 91)) - 1);
}
.ix-theme .ix-select.ix-select-visual .ix-select-popup {
  --ix-dropdown--background: var(--si-sys-color-background-3, var(--ix-color-surface-1));
  --ix-dropdown--box-shadow: var(--si-sys-color-effects-shadow-4, 0 8px 24px #0004);
  --ix-dropdown--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-dropdown-header--color: var(--si-sys-color-text-secondary, var(--ix-color-text-secondary));
  --ix-dropdown--border-radius: var(--si-sys-sizing-border-radius-sm, 4px);
  --ix-dropdown--z-index: var(--theme-z-index-dropdown, 91);
  --ix-dropdown--padding: var(--si-sys-sizing-spacing-y-20, .25rem) 0;
  --ix-dropdown-header--height: var(--si-sys-sizing-size-90, var(--ix-control-height, 40px));
  --ix-dropdown-header--padding: 0 var(--si-sys-sizing-spacing-x-60, 1rem);
  --ix-dropdown--offset: var(--si-sys-sizing-size-100, 48px);
  --ix-dropdown-dialog--padding: var(--si-sys-sizing-spacing-y-20, .25rem) 0;
  --ix-dropdown-overflow--max-height: 50vh;
  display: flex;
  position: fixed;
  inset: auto;
  margin: 0;
  flex-direction: column;
  max-width: 100%;
  max-height: calc(var(--ix-dropdown-overflow--max-height) - var(--ix-dropdown--offset));
  min-width: 0;
  background-color: var(--ix-dropdown--background);
  color: var(--ix-dropdown--color);
  border: 0;
  border-radius: var(--ix-dropdown--border-radius);
  box-shadow: var(--ix-dropdown--box-shadow);
  padding: var(--ix-dropdown--padding);
  z-index: var(--ix-dropdown--z-index);
  overflow-y: auto;
}
.ix-theme .ix-select.ix-select-visual .ix-select-popup[data-top-layer=true] { overflow: visible; }
.ix-theme .ix-select.ix-select-visual .ix-select-list-header,
.ix-theme .ix-select.ix-select-visual .ix-select-empty {
  display: flex;
  align-items: center;
  height: var(--ix-select-list-header--height);
  color: var(--ix-select-item-hint--color);
  margin: var(--ix-select-list-header--margin);
  padding: 0;
  font: inherit;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.ix-theme .ix-select.ix-select-visual .ix-select-list-header[hidden] { display: none !important; }
.ix-theme .ix-select.ix-select-visual .ix-select-list {
  max-height: none;
  overflow: visible;
}
.ix-theme .ix-select.ix-select-visual .ix-select-popup[data-top-layer=false] > .ix-select-list { display: contents; }
.ix-theme .ix-select.ix-select-visual .ix-select-overflow-popup {
  position: absolute;
  top: 100%;
  inset-inline: 0;
  align-items: flex-start;
  gap: var(--ix-select-overflow-dropdown-content--gap);
  max-height: var(--ix-select-overflow-dropdown-content--max-height);
  overflow-y: auto;
  padding: var(--ix-select-overflow-dropdown-content--padding);
}
.ix-theme .ix-select.ix-select-visual .ix-select-item-host {
  --ix-select-item--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-select-item--background--selected: var(--si-sys-color-background-selected, var(--ix-color-surface-2));
  display: block;
  position: relative;
  box-sizing: content-box;
  min-width: auto;
  min-height: auto;
}
.ix-theme .ix-select.ix-select-visual .ix-select-item-host[data-disabled=true] { pointer-events: none; cursor: default; }
.ix-theme .ix-select.ix-select-visual .ix-select-option-host {
  --ix-dropdown-item-checkmark--color: var(--si-sys-color-text-accent, var(--ix-color-secondary));
  --ix-dropdown-item-icon--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-dropdown-item--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-secondary));
  --ix-dropdown-item--border-color--focus: var(--si-sys-color-effects-focus, var(--ix-color-focus));
  --ix-dropdown-item--outline-color--focus: var(--si-sys-color-effects-focus, var(--ix-color-focus));
  --ix-dropdown-item--item-padding: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-dropdown-item--item-padding-right: var(--si-sys-sizing-spacing-x-80, 24px);
  --ix-dropdown-item--height: var(--si-sys-sizing-size-90, var(--ix-control-height, 40px));
  --ix-dropdown-item-no-checkmark--padding-inline-start: var(--si-sys-sizing-spacing-x-60, 1rem);
  --ix-dropdown-item-checked--width: var(--si-sys-sizing-size-50, 16px);
  --ix-dropdown-item-checked--min-width: var(--si-sys-sizing-size-50, 16px);
  --ix-dropdown-item-checked--margin-right: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-dropdown-item-icon--icon-size: var(--si-sys-sizing-icon-md, 24px);
  --ix-dropdown-item-icon--margin-right: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-dropdown-item-icon-only--padding-inline: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-dropdown-item-submenu--padding-inline-end: var(--si-sys-sizing-spacing-x-40, .5rem);
  --ix-dropdown-item--border-width--focus: var(--si-sys-sizing-border-width-default, 1px);
  --ix-dropdown-item--inner-height: calc(100% - 2 * var(--ix-dropdown-item--border-width--focus));
  --ix-dropdown-item--outline-width--focus: var(--si-sys-sizing-border-width-default, 1px);
  --ix-dropdown-item-outline-visible--outline-offset: calc(0px - var(--ix-dropdown-item--outline-width--focus));
  display: flex;
  flex-direction: row;
  position: relative;
  height: var(--ix-dropdown-item--height);
  width: 100%;
  overflow: hidden;
  cursor: pointer;
  color: var(--ix-select-item--color);
  box-sizing: content-box;
}
.ix-theme .ix-select.ix-select-visual .ix-select-option-host[data-selected=true] {
  background-color: var(--ix-select-item--background--selected);
  --ix-dropdown-item-checkmark--color: var(--ix-select-item--color);
}
.ix-theme .ix-select.ix-select-visual .ix-select-option-host:not([data-disabled=true]):is(:hover,.hover) { background-color: var(--si-sys-color-background-hover, var(--ix-color-surface-2)); }
.ix-theme .ix-select.ix-select-visual .ix-select-option-host:not([data-disabled=true]):active { background-color: var(--si-sys-color-background-active, var(--ix-color-surface-2)); }
.ix-theme .ix-select.ix-select-visual .ix-select-option-host[data-disabled=true] {
  pointer-events: none;
  color: var(--ix-dropdown-item--color--disabled) !important;
}
.ix-theme .ix-select.ix-select-visual button.ix-select-option {
  all: unset;
  display: flex;
  flex-direction: row;
  align-items: center;
  position: relative;
  height: var(--ix-dropdown-item--inner-height);
  border: var(--ix-dropdown-item--border-width--focus) solid transparent;
  white-space: nowrap;
  width: calc(100% - var(--ix-dropdown-item--item-padding) - var(--ix-dropdown-item--item-padding-right));
  padding: 0 var(--ix-dropdown-item--item-padding);
  padding-right: var(--ix-dropdown-item--item-padding-right);
}
.ix-theme .ix-select.ix-select-visual button.ix-select-option.ix-select-add {
  width: 100%;
  height: var(--si-sys-sizing-size-90, var(--ix-control-height, 40px));
  box-sizing: border-box;
  padding: 0 var(--si-sys-sizing-spacing-x-60, 1rem);
  border: var(--si-sys-sizing-border-width-default, 1px) solid transparent;
  gap: var(--si-sys-sizing-spacing-x-40, .5rem);
  cursor: pointer;
}
.ix-theme .ix-select.ix-select-visual button.ix-select-option.ix-select-add:hover { background-color: var(--si-sys-color-background-hover, var(--ix-color-surface-2)); }
.ix-theme .ix-select.ix-select-visual button.ix-select-option.ix-select-add.ix-select-active {
  outline: var(--ix-select--outline-width--focus) solid var(--ix-select--outline-color--focus);
  outline-offset: calc(0px - var(--ix-select--outline-width--focus));
}
.ix-theme .ix-select.ix-select-visual .ix-select-option-host:is(:has(> .ix-select-active),:focus-within) {
  outline: var(--ix-dropdown-item--outline-width--focus) solid var(--ix-dropdown-item--outline-color--focus);
  outline-offset: var(--ix-dropdown-item-outline-visible--outline-offset);
}
.ix-theme .ix-select.ix-select-visual .ix-select-choice {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  height: 100%;
  width: var(--ix-dropdown-item-checked--width);
  min-width: var(--ix-dropdown-item-checked--min-width);
  margin-right: var(--ix-dropdown-item-checked--margin-right);
  border: 0;
  border-radius: 0;
  background: transparent;
  color: var(--ix-dropdown-item-checkmark--color);
  flex-shrink: 1;
}
.ix-theme .ix-select.ix-select-visual .ix-select-choice > .ix-icon {
  width: var(--si-sys-sizing-size-50, 16px);
  height: var(--si-sys-sizing-size-50, 16px);
  min-width: var(--si-sys-sizing-size-50, 16px);
  min-height: var(--si-sys-sizing-size-50, 16px);
}
.ix-theme .ix-select.ix-select-visual .ix-select-option-label {
  display: block;
  position: relative;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: pre;
}
.ix-theme .ix-select.ix-select-visual .ix-select-option-end { margin-left: auto; }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-info .ix-select-control { border-color: var(--ix-select--border-color--info); }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-info .ix-select-control:hover { border-color: var(--ix-select--border-color--info--hover) !important; }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-info .ix-select-control:active { border-color: var(--ix-select--border-color--info--active) !important; }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-warning .ix-select-control { border-color: var(--ix-select--border-color--warning); background-color: var(--ix-select--background--warning); }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-warning .ix-select-control:hover { border-color: var(--ix-select--border-color--warning--hover) !important; }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-warning .ix-select-control:active { border-color: var(--ix-select--border-color--warning--active) !important; }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-invalid .ix-select-control { border-color: var(--ix-select--border-color--invalid); background-color: var(--ix-select--background--invalid); }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-invalid .ix-select-control:hover { border-color: var(--ix-select--border-color--invalid--hover) !important; }
.ix-theme .ix-select.ix-select-visual[data-disabled=false][data-readonly=false].ix-field-invalid .ix-select-control:active { border-color: var(--ix-select--border-color--invalid--active) !important; }
"#;

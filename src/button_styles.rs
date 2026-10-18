//! Source-aligned sizing, focus, layout, and 9-variant state token styles for regular [`crate::Button`].
//!
//! Scoped strictly to `.ix-theme .ix-button.ix-button-standard` so legacy `.ix-button` rules
//! in `src/actions.rs` continue to govern sibling controls (`IconButton`, `ToggleButton`,
//! `IconToggleButton`, `SplitButton`, and dropdown triggers) until their own slices are migrated.

pub(crate) const STYLES: &str = r#"
.ix-theme {
  --_ix-button-height: 32px;
}
.ix-theme[data-ix-density=compact] {
  --_ix-button-height: 24px;
}
.ix-theme .ix-button.ix-button-standard {
  --ix-button--outline-color--focus: var(--si-sys-color-effects-focus, var(--ix-color-focus));
  --ix-button-danger-primary--background: var(--si-sys-color-background-danger, var(--ix-color-danger));
  --ix-button-danger-primary--background--active: var(--si-sys-color-background-danger-active, color-mix(in srgb, var(--ix-color-danger) 72%, var(--ix-color-text)));
  --ix-button-danger-primary--background--disabled: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-button-danger-primary--background--hover: var(--si-sys-color-background-danger-hover, color-mix(in srgb, var(--ix-color-danger) 85%, var(--ix-color-text)));
  --ix-button-danger-primary--border-color: rgba(0, 0, 0, 0);
  --ix-button-danger-primary--border-color--active: rgba(0, 0, 0, 0);
  --ix-button-danger-primary--border-color--disabled: rgba(0, 0, 0, 0);
  --ix-button-danger-primary--border-color--hover: rgba(0, 0, 0, 0);
  --ix-button-danger-primary--color: var(--si-sys-color-text-on-danger, var(--ix-color-on-danger, #fff));
  --ix-button-danger-primary--color--active: var(--si-sys-color-text-on-danger, var(--ix-color-on-danger, #fff));
  --ix-button-danger-primary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-danger-primary--color--hover: var(--si-sys-color-text-on-danger, var(--ix-color-on-danger, #fff));
  --ix-button-danger-secondary--background: rgba(0, 0, 0, 0);
  --ix-button-danger-secondary--background--active: var(--si-sys-color-background-danger-active, color-mix(in srgb, var(--ix-color-danger) 72%, var(--ix-color-text)));
  --ix-button-danger-secondary--background--disabled: rgba(0, 0, 0, 0);
  --ix-button-danger-secondary--background--hover: var(--si-sys-color-background-danger-hover, color-mix(in srgb, var(--ix-color-danger) 85%, var(--ix-color-text)));
  --ix-button-danger-secondary--border-color: var(--si-sys-color-text-danger, var(--ix-color-danger-text));
  --ix-button-danger-secondary--border-color--active: var(--si-sys-color-border-danger, var(--ix-color-danger));
  --ix-button-danger-secondary--border-color--disabled: var(--si-sys-color-border-3, color-mix(in srgb, var(--ix-color-border) 60%, var(--ix-color-surface-1)));
  --ix-button-danger-secondary--border-color--hover: var(--si-sys-color-border-danger, var(--ix-color-danger));
  --ix-button-danger-secondary--color: var(--si-sys-color-text-danger, var(--ix-color-danger-text));
  --ix-button-danger-secondary--color--active: var(--si-sys-color-text-on-danger, var(--ix-color-on-danger, #fff));
  --ix-button-danger-secondary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-danger-secondary--color--hover: var(--si-sys-color-text-on-danger, var(--ix-color-on-danger, #fff));
  --ix-button-danger-tertiary--background: rgba(0, 0, 0, 0);
  --ix-button-danger-tertiary--background--active: var(--si-sys-color-background-danger-active, color-mix(in srgb, var(--ix-color-danger) 72%, var(--ix-color-text)));
  --ix-button-danger-tertiary--background--disabled: rgba(0, 0, 0, 0);
  --ix-button-danger-tertiary--background--hover: var(--si-sys-color-background-danger-hover, color-mix(in srgb, var(--ix-color-danger) 85%, var(--ix-color-text)));
  --ix-button-danger-tertiary--border-color: rgba(0, 0, 0, 0);
  --ix-button-danger-tertiary--border-color--active: rgba(0, 0, 0, 0);
  --ix-button-danger-tertiary--border-color--disabled: rgba(0, 0, 0, 0);
  --ix-button-danger-tertiary--border-color--hover: rgba(0, 0, 0, 0);
  --ix-button-danger-tertiary--color: var(--si-sys-color-text-danger, var(--ix-color-danger-text));
  --ix-button-danger-tertiary--color--active: var(--si-sys-color-text-on-danger, var(--ix-color-on-danger, #fff));
  --ix-button-danger-tertiary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-danger-tertiary--color--hover: var(--si-sys-color-text-on-danger, var(--ix-color-on-danger, #fff));
  --ix-button-primary--background: var(--si-sys-color-background-accent, var(--ix-color-primary));
  --ix-button-primary--background--active: var(--si-sys-color-background-accent-active, color-mix(in srgb, var(--ix-color-primary) 72%, var(--ix-color-text)));
  --ix-button-primary--background--disabled: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-button-primary--background--hover: var(--si-sys-color-background-accent-hover, color-mix(in srgb, var(--ix-color-primary) 85%, var(--ix-color-text)));
  --ix-button-primary--border-color: rgba(0, 0, 0, 0);
  --ix-button-primary--border-color--active: rgba(0, 0, 0, 0);
  --ix-button-primary--border-color--disabled: rgba(0, 0, 0, 0);
  --ix-button-primary--border-color--hover: rgba(0, 0, 0, 0);
  --ix-button-primary--color: var(--si-sys-color-text-on-accent, var(--ix-color-on-primary));
  --ix-button-primary--color--active: var(--si-sys-color-text-on-accent, var(--ix-color-on-primary));
  --ix-button-primary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-primary--color--hover: var(--si-sys-color-text-on-accent, var(--ix-color-on-primary));
  --ix-button-secondary--background: var(--si-sys-color-background-accent-secondary, rgba(0, 0, 0, 0));
  --ix-button-secondary--background--active: var(--si-sys-color-background-accent-secondary-active, color-mix(in srgb, var(--ix-color-primary) 24%, transparent));
  --ix-button-secondary--background--disabled: rgba(0, 0, 0, 0);
  --ix-button-secondary--background--hover: var(--si-sys-color-background-accent-secondary-hover, color-mix(in srgb, var(--ix-color-primary) 14%, transparent));
  --ix-button-secondary--border-color: var(--si-sys-color-border-accent, var(--ix-color-primary));
  --ix-button-secondary--border-color--active: var(--si-sys-color-border-accent-active, color-mix(in srgb, var(--ix-color-primary) 72%, var(--ix-color-text)));
  --ix-button-secondary--border-color--disabled: var(--si-sys-color-border-3, color-mix(in srgb, var(--ix-color-border) 60%, var(--ix-color-surface-1)));
  --ix-button-secondary--border-color--hover: var(--si-sys-color-border-accent-hover, color-mix(in srgb, var(--ix-color-primary) 85%, var(--ix-color-text)));
  --ix-button-secondary--color: var(--si-sys-color-text-accent, var(--ix-color-secondary));
  --ix-button-secondary--color--active: var(--si-sys-color-text-accent-active, color-mix(in srgb, var(--ix-color-secondary) 72%, var(--ix-color-text)));
  --ix-button-secondary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-secondary--color--hover: var(--si-sys-color-text-accent-hover, color-mix(in srgb, var(--ix-color-secondary) 85%, var(--ix-color-text)));
  --ix-button-subtle-primary--background: var(--si-sys-color-background-2, var(--ix-color-surface-2));
  --ix-button-subtle-primary--background--active: var(--si-sys-color-background-selected, color-mix(in srgb, var(--ix-color-text) 18%, var(--ix-color-surface-2)));
  --ix-button-subtle-primary--background--disabled: var(--si-sys-color-background-1, var(--ix-color-surface-1));
  --ix-button-subtle-primary--background--hover: var(--si-sys-color-background-hover, color-mix(in srgb, var(--ix-color-text) 10%, var(--ix-color-surface-2)));
  --ix-button-subtle-primary--border-color: rgba(0, 0, 0, 0);
  --ix-button-subtle-primary--border-color--active: rgba(0, 0, 0, 0);
  --ix-button-subtle-primary--border-color--disabled: rgba(0, 0, 0, 0);
  --ix-button-subtle-primary--border-color--hover: rgba(0, 0, 0, 0);
  --ix-button-subtle-primary--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-primary--color--active: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-primary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-subtle-primary--color--hover: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-secondary--background: rgba(0, 0, 0, 0);
  --ix-button-subtle-secondary--background--active: var(--si-sys-color-background-selected, color-mix(in srgb, var(--ix-color-text) 18%, var(--ix-color-surface-2)));
  --ix-button-subtle-secondary--background--disabled: rgba(0, 0, 0, 0);
  --ix-button-subtle-secondary--background--hover: var(--si-sys-color-background-hover, color-mix(in srgb, var(--ix-color-text) 10%, var(--ix-color-surface-2)));
  --ix-button-subtle-secondary--border-color: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-button-subtle-secondary--border-color--active: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-button-subtle-secondary--border-color--disabled: var(--si-sys-color-border-3, color-mix(in srgb, var(--ix-color-border) 60%, var(--ix-color-surface-1)));
  --ix-button-subtle-secondary--border-color--hover: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-button-subtle-secondary--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-secondary--color--active: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-secondary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-subtle-secondary--color--hover: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-tertiary--background: rgba(0, 0, 0, 0);
  --ix-button-subtle-tertiary--background--active: var(--si-sys-color-background-selected, color-mix(in srgb, var(--ix-color-text) 18%, var(--ix-color-surface-2)));
  --ix-button-subtle-tertiary--background--disabled: rgba(0, 0, 0, 0);
  --ix-button-subtle-tertiary--background--hover: var(--si-sys-color-background-hover, color-mix(in srgb, var(--ix-color-text) 10%, var(--ix-color-surface-2)));
  --ix-button-subtle-tertiary--border-color: rgba(0, 0, 0, 0);
  --ix-button-subtle-tertiary--border-color--active: rgba(0, 0, 0, 0);
  --ix-button-subtle-tertiary--border-color--disabled: rgba(0, 0, 0, 0);
  --ix-button-subtle-tertiary--border-color--hover: rgba(0, 0, 0, 0);
  --ix-button-subtle-tertiary--color: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-tertiary--color--active: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-subtle-tertiary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-subtle-tertiary--color--hover: var(--si-sys-color-text-primary, var(--ix-color-text));
  --ix-button-tertiary--background: rgba(0, 0, 0, 0);
  --ix-button-tertiary--background--active: var(--si-sys-color-background-accent-secondary-active, color-mix(in srgb, var(--ix-color-primary) 24%, transparent));
  --ix-button-tertiary--background--disabled: rgba(0, 0, 0, 0);
  --ix-button-tertiary--background--hover: var(--si-sys-color-background-accent-secondary-hover, color-mix(in srgb, var(--ix-color-primary) 14%, transparent));
  --ix-button-tertiary--border-color: rgba(0, 0, 0, 0);
  --ix-button-tertiary--border-color--active: rgba(0, 0, 0, 0);
  --ix-button-tertiary--border-color--disabled: rgba(0, 0, 0, 0);
  --ix-button-tertiary--border-color--hover: rgba(0, 0, 0, 0);
  --ix-button-tertiary--color: var(--si-sys-color-text-accent, var(--ix-color-secondary));
  --ix-button-tertiary--color--active: var(--si-sys-color-text-accent-active, color-mix(in srgb, var(--ix-color-secondary) 72%, var(--ix-color-text)));
  --ix-button-tertiary--color--disabled: var(--si-sys-color-text-disabled, var(--ix-color-text-disabled, var(--ix-color-text-secondary)));
  --ix-button-tertiary--color--hover: var(--si-sys-color-text-accent-hover, color-mix(in srgb, var(--ix-color-secondary) 85%, var(--ix-color-text)));
  --ix-button--border-radius: var(--si-sys-sizing-border-radius-xs, var(--ix-radius));
  --ix-button-border-radius-left: var(--ix-button--border-radius);
  --ix-button-border-radius-right: var(--ix-button--border-radius);
  --ix-button--border-width: var(--si-sys-sizing-border-width-default, 1px);
  --ix-button--outline-width--focus: var(--si-sys-sizing-border-width-default, 1px);
  --ix-button--focus--outline-offset: var(--si-sys-sizing-focus-ring-offset, 2px);
  --ix-button--padding: 0 var(--si-sys-sizing-spacing-x-40, 8px);
  --ix-button--height: var(--si-sys-sizing-size-80, var(--_ix-button-height, 32px));
  --ix-button--margin-right: var(--si-sys-sizing-spacing-x-20, 4px);
  --ix-button-icon--margin-right: var(--si-sys-sizing-spacing-x-20, 4px);
  --ix-button-icon-right--margin-left: var(--si-sys-sizing-spacing-x-20, 4px);
  --ix-button--min-width: var(--si-sys-sizing-size-120, 80px);
  appearance: none;
  -webkit-appearance: none;
  box-sizing: border-box;
  display: inline-flex;
  flex-direction: row;
  flex-wrap: nowrap;
  overflow: hidden;
  align-items: center;
  justify-content: center;
  vertical-align: middle;
  gap: 0;
  height: var(--ix-button--height);
  min-height: 0;
  min-width: var(--ix-button--min-width);
  padding: var(--ix-button-padding, var(--ix-button--padding));
  border-width: var(--ix-button--border-width);
  border-style: solid;
  border-top-left-radius: var(--ix-button-border-radius-left);
  border-bottom-left-radius: var(--ix-button-border-radius-left);
  border-top-right-radius: var(--ix-button-border-radius-right);
  border-bottom-right-radius: var(--ix-button-border-radius-right);
  font: var(--si-sys-typography-body-sbold, 600 14px/22px var(--ix-font));
  font-feature-settings: "clig" off, "liga" off;
  font-style: normal;
  letter-spacing: var(--si-ref-typography-letter-spacing-normal, normal);
  text-decoration: none;
  text-align: start;
  white-space: nowrap;
  -webkit-font-smoothing: antialiased;
  cursor: pointer;
  filter: none;
  box-shadow: none;
  opacity: 1;
  transition: none;
}
.ix-theme .ix-button.ix-button-standard:hover:not(:disabled):not([aria-disabled=true]),
.ix-theme .ix-button.ix-button-standard:active:not(:disabled) {
  filter: none;
  box-shadow: none;
  transition: none;
}
.ix-theme .ix-button.ix-button-standard .ix-button-loading {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  margin-right: var(--ix-button--margin-right);
}
.ix-theme .ix-button.ix-button-standard .ix-button-icon-start {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  pointer-events: none;
  margin-right: var(--ix-button-icon--margin-right);
}
.ix-theme .ix-button.ix-button-standard .ix-button-icon-end {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  pointer-events: none;
  margin-left: var(--ix-button-icon-right--margin-left);
}
.ix-theme .ix-button.ix-button-standard .ix-button-content {
  display: inline-block;
  position: relative;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.ix-theme .ix-button.ix-button-standard.ix-align-start .ix-button-content,
.ix-theme .ix-button.ix-button-standard .ix-button-content.content-start,
.ix-theme .ix-button.ix-button-standard .ix-button-content.ix-button-content-start {
  width: 100%;
}
.ix-theme .ix-button.ix-button-standard.ix-focused,
.ix-theme .ix-button.ix-button-standard:focus-visible {
  outline: var(--ix-button--outline-width--focus) solid var(--ix-button--outline-color--focus);
  outline-offset: var(--ix-button--focus--outline-offset);
}
.ix-theme .ix-button.ix-button-standard:disabled,
.ix-theme .ix-button.ix-button-standard[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.disabled {
  pointer-events: none;
  cursor: default;
  opacity: 1;
}
.ix-theme .ix-button.ix-button-standard.ix-button-primary {
  background-color: var(--ix-button-primary--background);
  color: var(--ix-button-primary--color);
  --ix-button-color: var(--ix-button-primary--color);
  border-color: var(--ix-button-primary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-primary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-primary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-primary--border-color--hover);
  background-color: var(--ix-button-primary--background--hover);
  color: var(--ix-button-primary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-primary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-primary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-primary--border-color--active);
  background-color: var(--ix-button-primary--background--active);
  color: var(--ix-button-primary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-primary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-primary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-primary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-primary--border-color--disabled);
  background-color: var(--ix-button-primary--background--disabled);
  color: var(--ix-button-primary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-primary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-secondary {
  background-color: var(--ix-button-secondary--background);
  color: var(--ix-button-secondary--color);
  --ix-button-color: var(--ix-button-secondary--color);
  border-color: var(--ix-button-secondary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-secondary--border-color--hover);
  background-color: var(--ix-button-secondary--background--hover);
  color: var(--ix-button-secondary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-secondary--border-color--active);
  background-color: var(--ix-button-secondary--background--active);
  color: var(--ix-button-secondary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-secondary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-secondary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-secondary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-secondary--border-color--disabled);
  background-color: var(--ix-button-secondary--background--disabled);
  color: var(--ix-button-secondary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-secondary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary {
  background-color: var(--ix-button-tertiary--background);
  color: var(--ix-button-tertiary--color);
  --ix-button-color: var(--ix-button-tertiary--color);
  border-color: var(--ix-button-tertiary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-tertiary--border-color--hover);
  background-color: var(--ix-button-tertiary--background--hover);
  color: var(--ix-button-tertiary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-tertiary--border-color--active);
  background-color: var(--ix-button-tertiary--background--active);
  color: var(--ix-button-tertiary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-tertiary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-tertiary--border-color--disabled);
  background-color: var(--ix-button-tertiary--background--disabled);
  color: var(--ix-button-tertiary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-tertiary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary {
  background-color: var(--ix-button-subtle-primary--background);
  color: var(--ix-button-subtle-primary--color);
  --ix-button-color: var(--ix-button-subtle-primary--color);
  border-color: var(--ix-button-subtle-primary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-subtle-primary--border-color--hover);
  background-color: var(--ix-button-subtle-primary--background--hover);
  color: var(--ix-button-subtle-primary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-subtle-primary--border-color--active);
  background-color: var(--ix-button-subtle-primary--background--active);
  color: var(--ix-button-subtle-primary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-primary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-subtle-primary--border-color--disabled);
  background-color: var(--ix-button-subtle-primary--background--disabled);
  color: var(--ix-button-subtle-primary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-subtle-primary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary {
  background-color: var(--ix-button-subtle-secondary--background);
  color: var(--ix-button-subtle-secondary--color);
  --ix-button-color: var(--ix-button-subtle-secondary--color);
  border-color: var(--ix-button-subtle-secondary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-subtle-secondary--border-color--hover);
  background-color: var(--ix-button-subtle-secondary--background--hover);
  color: var(--ix-button-subtle-secondary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-subtle-secondary--border-color--active);
  background-color: var(--ix-button-subtle-secondary--background--active);
  color: var(--ix-button-subtle-secondary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-secondary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-subtle-secondary--border-color--disabled);
  background-color: var(--ix-button-subtle-secondary--background--disabled);
  color: var(--ix-button-subtle-secondary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-subtle-secondary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary {
  background-color: var(--ix-button-subtle-tertiary--background);
  color: var(--ix-button-subtle-tertiary--color);
  --ix-button-color: var(--ix-button-subtle-tertiary--color);
  border-color: var(--ix-button-subtle-tertiary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-subtle-tertiary--border-color--hover);
  background-color: var(--ix-button-subtle-tertiary--background--hover);
  color: var(--ix-button-subtle-tertiary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-subtle-tertiary--border-color--active);
  background-color: var(--ix-button-subtle-tertiary--background--active);
  color: var(--ix-button-subtle-tertiary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-subtle-tertiary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-subtle-tertiary--border-color--disabled);
  background-color: var(--ix-button-subtle-tertiary--background--disabled);
  color: var(--ix-button-subtle-tertiary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-subtle-tertiary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary {
  background-color: var(--ix-button-danger-primary--background);
  color: var(--ix-button-danger-primary--color);
  --ix-button-color: var(--ix-button-danger-primary--color);
  border-color: var(--ix-button-danger-primary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-danger-primary--border-color--hover);
  background-color: var(--ix-button-danger-primary--background--hover);
  color: var(--ix-button-danger-primary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-danger-primary--border-color--active);
  background-color: var(--ix-button-danger-primary--background--active);
  color: var(--ix-button-danger-primary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-danger-primary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-danger-primary--border-color--disabled);
  background-color: var(--ix-button-danger-primary--background--disabled);
  color: var(--ix-button-danger-primary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-danger-primary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary {
  background-color: var(--ix-button-danger-secondary--background);
  color: var(--ix-button-danger-secondary--color);
  --ix-button-color: var(--ix-button-danger-secondary--color);
  border-color: var(--ix-button-danger-secondary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-danger-secondary--border-color--hover);
  background-color: var(--ix-button-danger-secondary--background--hover);
  color: var(--ix-button-danger-secondary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-danger-secondary--border-color--active);
  background-color: var(--ix-button-danger-secondary--background--active);
  color: var(--ix-button-danger-secondary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-danger-secondary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-danger-secondary--border-color--disabled);
  background-color: var(--ix-button-danger-secondary--background--disabled);
  color: var(--ix-button-danger-secondary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-danger-secondary--color--disabled);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary {
  background-color: var(--ix-button-danger-tertiary--background);
  color: var(--ix-button-danger-tertiary--color);
  --ix-button-color: var(--ix-button-danger-tertiary--color);
  border-color: var(--ix-button-danger-tertiary--border-color);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled):hover,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled).hover {
  border-color: var(--ix-button-danger-tertiary--border-color--hover);
  background-color: var(--ix-button-danger-tertiary--background--hover);
  color: var(--ix-button-danger-tertiary--color--hover);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled):active,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary:not(:disabled):not([aria-disabled=true]):not(.disabled).active {
  border-color: var(--ix-button-danger-tertiary--border-color--active);
  background-color: var(--ix-button-danger-tertiary--background--active);
  color: var(--ix-button-danger-tertiary--color--active);
}
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary:disabled,
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary[aria-disabled=true],
.ix-theme .ix-button.ix-button-standard.ix-button-danger-tertiary.disabled {
  pointer-events: none;
  cursor: default;
  border-color: var(--ix-button-danger-tertiary--border-color--disabled);
  background-color: var(--ix-button-danger-tertiary--background--disabled);
  color: var(--ix-button-danger-tertiary--color--disabled);
  opacity: 1;
  --ix-button-color: var(--ix-button-danger-tertiary--color--disabled);
}
"#;

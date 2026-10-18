//! Scoped Radio token recipe from pinned iX radio.scss and radio.vars.scss.

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-check.ix-radio.ix-radio-native {
  --ix-radio-checkmark--background: var(
    --si-sys-color-text-on-accent,
    var(--ix-color-on-primary)
  );
  --ix-radio--outline-color--focus: var(
    --si-sys-color-effects-focus,
    var(--ix-color-focus)
  );
  --ix-radio-checked--background: var(
    --si-sys-color-background-accent,
    var(--ix-color-primary)
  );
  --ix-radio-checked--background--active: var(
    --si-sys-color-background-accent-active,
    var(--ix-color-primary)
  );
  --ix-radio-checked--background--disabled: var(
    --si-sys-color-text-disabled,
    var(--ix-color-text-disabled, var(--ix-color-text-secondary))
  );
  --ix-radio-checked--background--hover: var(
    --si-sys-color-background-accent-hover,
    var(--ix-color-primary)
  );
  --ix-radio-checked--background--info: var(
    --si-sys-color-background-information,
    var(--ix-color-info)
  );
  --ix-radio-checked--background--info--active: var(
    --si-sys-color-background-information-active,
    var(--ix-color-info)
  );
  --ix-radio-checked--background--info--hover: var(
    --si-sys-color-background-information-hover,
    var(--ix-color-info)
  );
  --ix-radio-checked--background--invalid: var(
    --si-sys-color-background-danger,
    var(--ix-color-danger)
  );
  --ix-radio-checked--background--invalid--active: var(
    --si-sys-color-background-danger-active,
    var(--ix-color-danger)
  );
  --ix-radio-checked--background--invalid--hover: var(
    --si-sys-color-background-danger-hover,
    var(--ix-color-danger)
  );
  --ix-radio-checked--background--warning: var(
    --si-sys-color-background-warning,
    var(--ix-color-warning)
  );
  --ix-radio-checked--background--warning--active: var(
    --si-sys-color-background-warning-active,
    var(--ix-color-warning)
  );
  --ix-radio-checked--background--warning--hover: var(
    --si-sys-color-background-warning-hover,
    var(--ix-color-warning)
  );
  --ix-radio-checked--border-color: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--active: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--disabled: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--hover: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--info: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--info--active: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--info--hover: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--invalid: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--invalid--active: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--invalid--hover: rgba(0, 0, 0, 0);
  --ix-radio-checked--border-color--warning: var(
    --si-sys-color-border-warning,
    var(--ix-color-warning-text)
  );
  --ix-radio-checked--border-color--warning--active: var(
    --si-sys-color-border-warning,
    var(--ix-color-warning-text)
  );
  --ix-radio-checked--border-color--warning--hover: var(
    --si-sys-color-border-warning,
    var(--ix-color-warning-text)
  );
  --ix-radio-checked--color: var(
    --si-sys-color-text-on-accent,
    var(--ix-color-on-primary)
  );
  --ix-radio-checked--color--active: var(
    --si-sys-color-text-on-accent,
    var(--ix-color-on-primary)
  );
  --ix-radio-checked--color--disabled: var(
    --si-sys-color-text-inverse,
    var(--ix-color-on-primary)
  );
  --ix-radio-checked--color--hover: var(
    --si-sys-color-text-on-accent,
    var(--ix-color-on-primary)
  );
  --ix-radio-checked--color--info: var(
    --si-sys-color-text-on-information,
    var(--ix-color-on-info)
  );
  --ix-radio-checked--color--info--active: var(
    --si-sys-color-text-on-information,
    var(--ix-color-on-info)
  );
  --ix-radio-checked--color--info--hover: var(
    --si-sys-color-text-on-information,
    var(--ix-color-on-info)
  );
  --ix-radio-checked--color--invalid: var(
    --si-sys-color-text-on-danger,
    var(--ix-color-on-danger)
  );
  --ix-radio-checked--color--invalid--active: var(
    --si-sys-color-text-on-danger,
    var(--ix-color-on-danger)
  );
  --ix-radio-checked--color--invalid--hover: var(
    --si-sys-color-text-on-danger,
    var(--ix-color-on-danger)
  );
  --ix-radio-checked--color--warning: var(
    --si-sys-color-text-on-warning,
    var(--ix-color-on-warning)
  );
  --ix-radio-checked--color--warning--active: var(
    --si-sys-color-text-on-warning,
    var(--ix-color-on-warning)
  );
  --ix-radio-checked--color--warning--hover: var(
    --si-sys-color-text-on-warning,
    var(--ix-color-on-warning)
  );
  --ix-radio-label--color: var(
    --si-sys-color-text-primary,
    var(--ix-color-text)
  );
  --ix-radio-label--color--disabled: var(
    --si-sys-color-text-disabled,
    var(--ix-color-text-disabled, var(--ix-color-text-secondary))
  );
  --ix-radio-unchecked--background: var(
    --si-sys-color-background-1,
    var(--ix-color-surface-1)
  );
  --ix-radio-unchecked--background--active: var(
    --si-sys-color-background-selected,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--background--disabled: rgba(0, 0, 0, 0);
  --ix-radio-unchecked--background--hover: var(
    --si-sys-color-background-hover,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--background--info: var(
    --si-sys-color-background-1,
    var(--ix-color-surface-1)
  );
  --ix-radio-unchecked--background--info--active: var(
    --si-sys-color-background-selected,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--background--info--hover: var(
    --si-sys-color-background-hover,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--background--invalid: var(
    --si-sys-color-background-1,
    var(--ix-color-surface-1)
  );
  --ix-radio-unchecked--background--invalid--active: var(
    --si-sys-color-background-selected,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--background--invalid--hover: var(
    --si-sys-color-background-hover,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--background--warning: var(
    --si-sys-color-background-1,
    var(--ix-color-surface-1)
  );
  --ix-radio-unchecked--background--warning--active: var(
    --si-sys-color-background-selected,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--background--warning--hover: var(
    --si-sys-color-background-hover,
    var(--ix-color-surface-2)
  );
  --ix-radio-unchecked--border-color: var(
    --si-sys-color-border-2,
    var(--ix-color-border)
  );
  --ix-radio-unchecked--border-color--active: var(
    --si-sys-color-border-2,
    var(--ix-color-border)
  );
  --ix-radio-unchecked--border-color--disabled: var(
    --si-sys-color-text-disabled,
    var(--ix-color-text-disabled, var(--ix-color-text-secondary))
  );
  --ix-radio-unchecked--border-color--hover: var(
    --si-sys-color-border-2,
    var(--ix-color-border)
  );
  --ix-radio-unchecked--border-color--info: var(
    --si-sys-color-border-information,
    var(--ix-color-info-text)
  );
  --ix-radio-unchecked--border-color--info--active: var(
    --si-sys-color-border-information,
    var(--ix-color-info-text)
  );
  --ix-radio-unchecked--border-color--info--hover: var(
    --si-sys-color-border-information,
    var(--ix-color-info-text)
  );
  --ix-radio-unchecked--border-color--invalid: var(
    --si-sys-color-border-danger,
    var(--ix-color-danger-text)
  );
  --ix-radio-unchecked--border-color--invalid--active: var(
    --si-sys-color-border-danger,
    var(--ix-color-danger-text)
  );
  --ix-radio-unchecked--border-color--invalid--hover: var(
    --si-sys-color-border-danger,
    var(--ix-color-danger-text)
  );
  --ix-radio-unchecked--border-color--warning: var(
    --si-sys-color-border-warning,
    var(--ix-color-warning-text)
  );
  --ix-radio-unchecked--border-color--warning--active: var(
    --si-sys-color-border-warning,
    var(--ix-color-warning-text)
  );
  --ix-radio-unchecked--border-color--warning--hover: var(
    --si-sys-color-border-warning,
    var(--ix-color-warning-text)
  );
  --ix-radio-checkmark--border-color: white;
  --ix-radio--border-width: var(--si-sys-sizing-border-width-default, 1px);
  --ix-radio--focus--outline-offset: var(
    --si-sys-sizing-focus-ring-offset,
    3px
  );
  --ix-radio-checkmark--border-width-initial: var(
    --si-sys-sizing-border-width-default,
    1px
  );
  --ix-radio-checkmark--border-radius: var(
    --si-sys-sizing-border-radius-full,
    50%
  );
  --ix-radio--outline-width--focus: var(
    --si-sys-sizing-border-width-default,
    1px
  );
  --ix-radio-checkmark--width: var(--si-sys-sizing-size-60, 20px);
  --ix-radio-checkmark--min-width: var(--si-sys-sizing-size-60, 20px);
  --ix-radio-checkmark--max-width: var(--si-sys-sizing-size-60, 20px);
  --ix-radio-checkmark--height: var(--si-sys-sizing-size-60, 20px);
  --ix-radio-checkmark--min-height: var(--si-sys-sizing-size-60, 20px);
  --ix-radio-checkmark--max-height: var(--si-sys-sizing-size-60, 20px);
  --ix-radio-button--height: var(--si-sys-sizing-size-60, 20px);
  --ix-radio-button--width: var(--si-sys-sizing-size-60, 20px);
  --ix-radio--target-size: var(--si-sys-sizing-size-70, 24px);
  --ix-radio-selection-indicator--width: var(--si-sys-sizing-size-30, 8px);
  --ix-radio-selection-indicator--height: var(--si-sys-sizing-size-30, 8px);
  --ix-radio--margin: var(--si-sys-sizing-spacing-y-10, 2px) 0
    var(--si-sys-sizing-spacing-y-10, 2px)
    var(--si-sys-sizing-spacing-x-40, 8px);
}

.ix-theme .ix-check.ix-radio.ix-radio-native {
  box-sizing: border-box;
  display: inline-flex;
  position: relative;
  justify-content: flex-start;
  align-items: center;
  gap: 0;
  font: inherit;
  min-height: 0;
  opacity: 1;
  cursor: default;
}

.ix-theme .ix-check.ix-radio.ix-radio-native *,
.ix-theme .ix-check.ix-radio.ix-radio-native *::before,
.ix-theme .ix-check.ix-radio.ix-radio-native *::after {
  box-sizing: border-box;
}

.ix-theme .ix-check.ix-radio.ix-radio-native > input[type="radio"] {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
  margin: 0;
  pointer-events: none;
  outline: 0;
}

.ix-theme .ix-check.ix-radio.ix-radio-native > .ix-radio-button {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  height: var(--ix-radio-button--height);
  width: var(--ix-radio-button--width);
}

.ix-theme .ix-check.ix-radio.ix-radio-native > .ix-radio-button::before {
  content: "";
  position: absolute;
  top: calc((var(--ix-radio-button--height) - var(--ix-radio--target-size)) / 2);
  bottom: calc(
    (var(--ix-radio-button--height) - var(--ix-radio--target-size)) / 2
  );
  left: calc(
    (var(--ix-radio-button--height) - var(--ix-radio--target-size)) / 2
  );
  right: calc(
    (var(--ix-radio-button--height) - var(--ix-radio--target-size)) / 2
  );
}

.ix-theme .ix-check.ix-radio.ix-radio-native > .ix-radio-button > .ix-check-shape {
  all: unset;
  box-sizing: border-box;
  display: inline-flex;
  position: relative;
  justify-content: center;
  align-items: center;
  width: var(--ix-radio-checkmark--width);
  min-width: var(--ix-radio-checkmark--min-width);
  max-width: var(--ix-radio-checkmark--max-width);
  height: var(--ix-radio-checkmark--height);
  min-height: var(--ix-radio-checkmark--min-height);
  max-height: var(--ix-radio-checkmark--max-height);
  border-width: var(--ix-radio--border-width);
  border-style: solid;
  border-color: var(--ix-radio-unchecked--border-color);
  border-radius: var(--ix-radio-checkmark--border-radius);
  background: var(--ix-radio-unchecked--background);
  box-shadow: none;

}

.ix-theme
  .ix-check.ix-radio.ix-radio-native
  > .ix-radio-button
  > .ix-check-shape
  > .ix-radio-dot {
  border-radius: var(--ix-radio-checkmark--border-radius);
  background-color: var(--ix-radio-checkmark--background);
  width: var(--ix-radio-selection-indicator--width);
  height: var(--ix-radio-selection-indicator--height);
  visibility: hidden;
}

.ix-theme .ix-check.ix-radio.ix-radio-native > .ix-radio-label {
  cursor: default;
  margin: var(--ix-radio--margin);
}



.ix-theme .ix-check.ix-radio.ix-radio-native:hover {
  cursor: default;
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:hover
  > .ix-radio-button
  > .ix-check-shape {
  border-color: var(--ix-radio-unchecked--border-color--hover);
  background-color: var(--ix-radio-unchecked--background--hover);
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:active
  > .ix-radio-button
  > .ix-check-shape {
  border-color: var(--ix-radio-unchecked--border-color--active);
  background-color: var(--ix-radio-unchecked--background--active);
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:is(
    .checked,
    :has(> input:checked)
  )
  > .ix-radio-button
  > .ix-check-shape {
  background-color: var(--ix-radio-checked--background);
  border-color: var(--ix-radio-checked--border-color);
  box-shadow: none;
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:is(
    .checked,
    :has(> input:checked)
  ):hover
  > .ix-radio-button
  > .ix-check-shape {
  background-color: var(--ix-radio-checked--background--hover);
  border-color: var(--ix-radio-checked--border-color--hover);
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:is(
    .checked,
    :has(> input:checked)
  ):active
  > .ix-radio-button
  > .ix-check-shape {
  background-color: var(--ix-radio-checked--background--active);
  border-color: var(--ix-radio-checked--border-color--active);
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:is(
    :focus-visible,
    :has(> input:focus-visible)
  ) {
  outline: var(--ix-radio--outline-width--focus) solid
    var(--ix-radio--outline-color--focus);
  outline-offset: var(--ix-radio--focus--outline-offset);
}



.ix-theme
  .ix-check.ix-radio.ix-radio-native:is(
    [data-disabled="true"],
    .disabled,
    :has(> input:disabled)
  ) {
  cursor: default;
  pointer-events: none;
  opacity: 1;
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:is(
    [data-disabled="true"],
    .disabled,
    :has(> input:disabled)
  )
  > .ix-radio-button
  > .ix-check-shape {
  border-color: var(--ix-radio-unchecked--border-color--disabled);
  background: var(--ix-radio-unchecked--background--disabled);
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:is(
    [data-disabled="true"],
    .disabled,
    :has(> input:disabled)
  ):is(.checked, :has(> input:checked))
  > .ix-radio-button
  > .ix-check-shape {
  background-color: var(--ix-radio-checked--background--disabled);
  border-color: var(--ix-radio-checked--border-color--disabled);
}



.ix-theme
  .ix-check.ix-radio.ix-radio-native:not(
    :is([data-disabled="true"], .disabled, :has(> input:disabled))
  ):is(.ix-info, .ix-field-info)
  > .ix-radio-button
  > .ix-check-shape {
  --ix-radio-unchecked--background: var(
    --ix-radio-unchecked--background--info
  );
  --ix-radio-unchecked--background--hover: var(
    --ix-radio-unchecked--background--info--hover
  );
  --ix-radio-unchecked--background--active: var(
    --ix-radio-unchecked--background--info--active
  );
  --ix-radio-unchecked--border-color: var(
    --ix-radio-unchecked--border-color--info
  );
  --ix-radio-unchecked--border-color--hover: var(
    --ix-radio-unchecked--border-color--info--hover
  );
  --ix-radio-unchecked--border-color--active: var(
    --ix-radio-unchecked--border-color--info--active
  );
  --ix-radio-checked--background: var(--ix-radio-checked--background--info);
  --ix-radio-checked--background--hover: var(
    --ix-radio-checked--background--info--hover
  );
  --ix-radio-checked--background--active: var(
    --ix-radio-checked--background--info--active
  );
  --ix-radio-checked--border-color: var(
    --ix-radio-checked--border-color--info
  );
  --ix-radio-checked--border-color--hover: var(
    --ix-radio-checked--border-color--info--hover
  );
  --ix-radio-checked--border-color--active: var(
    --ix-radio-checked--border-color--info--active
  );
  --ix-radio-mixed--background: var(--ix-radio-mixed--background--info);
  --ix-radio-mixed--background--hover: var(
    --ix-radio-mixed--background--info--hover
  );
  --ix-radio-mixed--background--active: var(
    --ix-radio-mixed--background--info--active
  );
  --ix-radio-mixed--border-color: var(--ix-radio-mixed--border-color--info);
  --ix-radio-mixed--border-color--hover: var(
    --ix-radio-mixed--border-color--info--hover
  );
  --ix-radio-mixed--border-color--active: var(
    --ix-radio-mixed--border-color--info--active
  );
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:not(
    :is([data-disabled="true"], .disabled, :has(> input:disabled))
  ):is(.ix-warning, .ix-field-warning)
  > .ix-radio-button
  > .ix-check-shape {
  --ix-radio-unchecked--background: var(
    --ix-radio-unchecked--background--warning
  );
  --ix-radio-unchecked--background--hover: var(
    --ix-radio-unchecked--background--warning--hover
  );
  --ix-radio-unchecked--background--active: var(
    --ix-radio-unchecked--background--warning--active
  );
  --ix-radio-unchecked--border-color: var(
    --ix-radio-unchecked--border-color--warning
  );
  --ix-radio-unchecked--border-color--hover: var(
    --ix-radio-unchecked--border-color--warning--hover
  );
  --ix-radio-unchecked--border-color--active: var(
    --ix-radio-unchecked--border-color--warning--active
  );
  --ix-radio-checked--background: var(--ix-radio-checked--background--warning);
  --ix-radio-checked--background--hover: var(
    --ix-radio-checked--background--warning--hover
  );
  --ix-radio-checked--background--active: var(
    --ix-radio-checked--background--warning--active
  );
  --ix-radio-checked--border-color: var(
    --ix-radio-checked--border-color--warning
  );
  --ix-radio-checked--border-color--hover: var(
    --ix-radio-checked--border-color--warning--hover
  );
  --ix-radio-checked--border-color--active: var(
    --ix-radio-checked--border-color--warning--active
  );
  --ix-radio-mixed--background: var(--ix-radio-mixed--background--warning);
  --ix-radio-mixed--background--hover: var(
    --ix-radio-mixed--background--warning--hover
  );
  --ix-radio-mixed--background--active: var(
    --ix-radio-mixed--background--warning--active
  );
  --ix-radio-mixed--border-color: var(--ix-radio-mixed--border-color--warning);
  --ix-radio-mixed--border-color--hover: var(
    --ix-radio-mixed--border-color--warning--hover
  );
  --ix-radio-mixed--border-color--active: var(
    --ix-radio-mixed--border-color--warning--active
  );
}

.ix-theme
  .ix-check.ix-radio.ix-radio-native:not(
    :is([data-disabled="true"], .disabled, :has(> input:disabled))
  ):is(
    .ix-invalid,
    .ix-invalid--required,
    .ix-field-invalid
  )
  > .ix-radio-button
  > .ix-check-shape {
  --ix-radio-unchecked--background: var(
    --ix-radio-unchecked--background--invalid
  );
  --ix-radio-unchecked--background--hover: var(
    --ix-radio-unchecked--background--invalid--hover
  );
  --ix-radio-unchecked--background--active: var(
    --ix-radio-unchecked--background--invalid--active
  );
  --ix-radio-unchecked--border-color: var(
    --ix-radio-unchecked--border-color--invalid
  );
  --ix-radio-unchecked--border-color--hover: var(
    --ix-radio-unchecked--border-color--invalid--hover
  );
  --ix-radio-unchecked--border-color--active: var(
    --ix-radio-unchecked--border-color--invalid--active
  );
  --ix-radio-checked--background: var(--ix-radio-checked--background--invalid);
  --ix-radio-checked--background--hover: var(
    --ix-radio-checked--background--invalid--hover
  );
  --ix-radio-checked--background--active: var(
    --ix-radio-checked--background--invalid--active
  );
  --ix-radio-checked--border-color: var(
    --ix-radio-checked--border-color--invalid
  );
  --ix-radio-checked--border-color--hover: var(
    --ix-radio-checked--border-color--invalid--hover
  );
  --ix-radio-checked--border-color--active: var(
    --ix-radio-checked--border-color--invalid--active
  );
  --ix-radio-mixed--background: var(--ix-radio-mixed--background--invalid);
  --ix-radio-mixed--background--hover: var(
    --ix-radio-mixed--background--invalid--hover
  );
  --ix-radio-mixed--background--active: var(
    --ix-radio-mixed--background--invalid--active
  );
  --ix-radio-mixed--border-color: var(--ix-radio-mixed--border-color--invalid);
  --ix-radio-mixed--border-color--hover: var(
    --ix-radio-mixed--border-color--invalid--hover
  );
  --ix-radio-mixed--border-color--active: var(
    --ix-radio-mixed--border-color--invalid--active
  );
}

.ix-theme .ix-check.ix-radio.ix-radio-native > input:checked + .ix-radio-button > .ix-check-shape > .ix-radio-dot {
  visibility: visible;
}
"#;

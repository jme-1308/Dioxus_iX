//! NumberInput-specific recipes from the MIT-licensed pinned iX 5.2.1 source.
//! Shared field and genuine IconButton recipes provide all other presentation.

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-number-field {
  display: contents;
}
.ix-theme .ix-number-input-native.ix-number-has-steppers .ix-input-wrap > input[type=number] {
  min-width: var(--ix-input-show-stepper-buttons--min-width);
}
.ix-theme .ix-number-input-native .ix-input-end > .ix-number-steppers {
  display: flex;
  position: relative;
  flex-direction: row;
  flex-wrap: nowrap;
}
.ix-theme .ix-number-input-native .ix-input-end > .ix-number-steppers[data-hidden=true] {
  display: none;
}
"#;

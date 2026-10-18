//! Group layout and field feedback recipes from pinned iX 5.2.1.

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-group-native {
  display: inline-block;
  position: relative;
  border: 0;
  padding: 0;
  margin: 0;
  min-inline-size: 0;
  font: inherit;
}
.ix-theme .ix-checkbox-group {
  --ix-checkbox-group-checkbox-container--gap: var(--si-sys-sizing-spacing-y-40, .5rem);
  --ix-checkbox-group-checkbox-container--margin: var(--si-sys-sizing-spacing-y-30, .375rem) 0;
}
.ix-theme .ix-radio-group {
  --ix-radio-group-checkbox-container--gap: var(--si-sys-sizing-spacing-y-40, .5rem) var(--si-sys-sizing-spacing-x-60, 1rem);
  --ix-radio-group-checkbox-container--margin: var(--si-sys-sizing-spacing-y-30, .375rem) 0;
}
.ix-theme .ix-group-native > .ix-group-wrapper {
  --ix-field-wrapper-icon--color--info: var(--si-sys-color-text-information, var(--ix-color-info-text));
  --ix-field-wrapper-icon--color--invalid: var(--si-sys-color-text-danger, var(--ix-color-danger-text));
  --ix-field-wrapper-icon--color--valid: var(--si-sys-color-text-success, var(--ix-color-success));
  --ix-field-wrapper-icon--color--warning: var(--si-sys-color-text-warning, var(--ix-color-warning-text));
  --ix-field-wrapper-bottom-text--gap: var(--si-sys-sizing-spacing-y-20, .25rem);
  --ix-field-wrapper-bottom-text--margin-right: var(--si-sys-sizing-spacing-x-20, .25rem);
  --ix-field-wrapper-slot-wrapper--gap: var(--si-sys-sizing-spacing-y-20, .25rem);
  --ix-field-wrapper-field-top--gap: var(--si-sys-sizing-spacing-x-60, 1rem);
  --ix-field-wrapper-bottom-text--margin-top: var(--si-sys-sizing-spacing-y-20, .25rem);
  --ix-field-wrapper-bottom-text--margin-bottom: var(--si-sys-sizing-spacing-y-20, .25rem);
  --ix-field-wrapper-text-icon--margin: var(--si-sys-sizing-spacing-y-10, .125rem);
  display: flex;
  position: relative;
  flex-direction: column;
}
.ix-theme .ix-group-wrapper > .ix-group-slot {
  display: flex;
  position: relative;
  align-items: center;
  justify-content: flex-start;
  gap: var(--ix-field-wrapper-slot-wrapper--gap);
}
.ix-theme .ix-group-wrapper > .ix-group-top,
.ix-theme .ix-group-wrapper > .ix-group-bottom {
  display: flex;
  flex-direction: row;
  position: relative;
  justify-content: space-between;
  gap: var(--ix-field-wrapper-field-top--gap);
  width: 100%;
  min-width: 100%;
}
.ix-theme .ix-group-label {
  --ix-field-label--margin-top: var(--si-sys-sizing-spacing-y-40, .5rem);
  --ix-field-label--margin-bottom: var(--si-sys-sizing-spacing-y-20, .25rem);
  display: inline-block;
  position: relative;
  margin-top: var(--ix-field-label--margin-top);
  margin-bottom: var(--ix-field-label--margin-bottom);
}
.ix-theme .ix-group-choices {
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
}
.ix-theme .ix-checkbox-group > .ix-group-wrapper > .ix-group-slot > .ix-group-choices {
  gap: var(--ix-checkbox-group-checkbox-container--gap);
  margin: var(--ix-checkbox-group-checkbox-container--margin);
}
.ix-theme .ix-radio-group > .ix-group-wrapper > .ix-group-slot > .ix-group-choices {
  gap: var(--ix-radio-group-checkbox-container--gap);
  margin: var(--ix-radio-group-checkbox-container--margin);
}
.ix-theme .ix-group-slot > .ix-group-choices.ix-group-row { flex-direction: row; }
.ix-theme .ix-group-bottom > .ix-group-bottom-right { margin-left: auto; margin-right: 0; }
.ix-theme .ix-group-bottom > .ix-group-bottom-text {
  display: flex;
  position: relative;
  align-items: flex-start;
  justify-content: flex-start;
  gap: var(--ix-field-wrapper-bottom-text--gap);
  margin-right: var(--ix-field-wrapper-bottom-text--margin-right);
  margin-top: var(--ix-field-wrapper-bottom-text--margin-top);
  margin-bottom: var(--ix-field-wrapper-bottom-text--margin-bottom);
}
.ix-theme .ix-group-bottom-text > .ix-group-text-icon { margin: var(--ix-field-wrapper-text-icon--margin); }
.ix-theme .ix-group-text-icon.ix-field-invalid { color: var(--ix-field-wrapper-icon--color--invalid); }
.ix-theme .ix-group-text-icon.ix-field-info { color: var(--ix-field-wrapper-icon--color--info); }
.ix-theme .ix-group-text-icon.ix-field-valid { color: var(--ix-field-wrapper-icon--color--valid); }
.ix-theme .ix-group-text-icon.ix-field-warning { color: var(--ix-field-wrapper-icon--color--warning); }
"#;

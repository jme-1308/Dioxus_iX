//! Spinner recipes translated from pinned MIT-licensed iX 5.2.1.
//! SPDX-FileCopyrightText: 2023 Siemens AG
//! SPDX-License-Identifier: MIT
//! Public system tokens override provisional public density fallbacks.

pub(crate) const STYLES: &str = r#"
.ix-theme {
  --_ix-spinner-size-40: 12px;
  --_ix-spinner-size-50: 16px;
  --_ix-spinner-size-60: 20px;
  --_ix-spinner-size-80: 32px;
  --_ix-spinner-size-130: 96px;
}
.ix-theme[data-ix-density=compact] {
  --_ix-spinner-size-40: 8px;
  --_ix-spinner-size-50: 12px;
  --_ix-spinner-size-60: 16px;
  --_ix-spinner-size-80: 24px;
  --_ix-spinner-size-130: 80px;
}
.ix-theme .ix-spinner {
  --ix-spinner--border-color: var(--si-sys-color-border-2, var(--ix-color-border));
  --ix-spinner-track--border-color: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-spinner-primary--border-color: var(--si-sys-color-border-accent-hover, var(--ix-color-primary));
  --ix-spinner-primary-track--border-color: var(--si-sys-color-border-4, var(--ix-color-border));
  --ix-spinner-track--border-width-default: var(--si-sys-sizing-border-width-default, 1px);
  --ix-spinner-track--border-width-selected: var(--si-sys-sizing-border-width-selected, 2px);
  --ix-spinner-track--border-width-emphasis: var(--si-sys-sizing-border-width-emphasis, 4px);
  --ix-spinner-medium--size: var(--si-sys-sizing-size-80, var(--_ix-spinner-size-80));
  --ix-spinner-extra-extra-small--size: var(--si-sys-sizing-size-40, var(--_ix-spinner-size-40));
  --ix-spinner-extra-small--size: var(--si-sys-sizing-size-50, var(--_ix-spinner-size-50));
  --ix-spinner-small--size: var(--si-sys-sizing-size-60, var(--_ix-spinner-size-60));
  --ix-spinner-large--size: var(--si-sys-sizing-size-130, var(--_ix-spinner-size-130));
  --ix-spinner--rotation-duration: 2s;
  --ix-spinner--mask-duration: 3s;
  display: block;
  position: relative;
  box-sizing: content-box;
}
.ix-theme .ix-spinner *,
.ix-theme .ix-spinner *::before,
.ix-theme .ix-spinner *::after { box-sizing: border-box; }
.ix-theme .ix-button .ix-spinner { box-sizing: border-box; }
.ix-theme .ix-spinner > .ix-spinner-ring {
  border-radius: 100%;
  position: relative;
  animation: ix-spinner-rotate var(--ix-spinner--rotation-duration) linear infinite;
}
.ix-theme .ix-spinner > .ix-spinner-ring.xxs {
  width: var(--ix-spinner-extra-extra-small--size);
  height: var(--ix-spinner-extra-extra-small--size);
  min-width: var(--ix-spinner-extra-extra-small--size);
  min-height: var(--ix-spinner-extra-extra-small--size);
  max-width: var(--ix-spinner-extra-extra-small--size);
  max-height: var(--ix-spinner-extra-extra-small--size);
}
.ix-theme .ix-spinner > .ix-spinner-ring.xs {
  width: var(--ix-spinner-extra-small--size);
  height: var(--ix-spinner-extra-small--size);
  min-width: var(--ix-spinner-extra-small--size);
  min-height: var(--ix-spinner-extra-small--size);
  max-width: var(--ix-spinner-extra-small--size);
  max-height: var(--ix-spinner-extra-small--size);
}
.ix-theme .ix-spinner > .ix-spinner-ring.sm {
  width: var(--ix-spinner-small--size);
  height: var(--ix-spinner-small--size);
  min-width: var(--ix-spinner-small--size);
  min-height: var(--ix-spinner-small--size);
  max-width: var(--ix-spinner-small--size);
  max-height: var(--ix-spinner-small--size);
}
.ix-theme .ix-spinner > .ix-spinner-ring.md {
  width: var(--ix-spinner-medium--size);
  height: var(--ix-spinner-medium--size);
  min-width: var(--ix-spinner-medium--size);
  min-height: var(--ix-spinner-medium--size);
  max-width: var(--ix-spinner-medium--size);
  max-height: var(--ix-spinner-medium--size);
}
.ix-theme .ix-spinner > .ix-spinner-ring.xxl {
  width: var(--ix-spinner-large--size);
  height: var(--ix-spinner-large--size);
  min-width: var(--ix-spinner-large--size);
  min-height: var(--ix-spinner-large--size);
  max-width: var(--ix-spinner-large--size);
  max-height: var(--ix-spinner-large--size);
}
.ix-theme .ix-spinner > .ix-spinner-ring::after,
.ix-theme .ix-spinner > .ix-spinner-ring:not(.hide-track)::before {
  content: '';
  box-sizing: border-box;
  position: absolute;
  inset: 8.33%;
  border-radius: 100%;
  border: var(--ix-spinner-track--border-width-selected) solid;
}
.ix-theme .ix-spinner > .ix-spinner-ring.xxs::after,
.ix-theme .ix-spinner > .ix-spinner-ring.xxs:not(.hide-track)::before {
  border-width: var(--ix-spinner-track--border-width-default);
}
.ix-theme .ix-spinner > .ix-spinner-ring.xxl::after,
.ix-theme .ix-spinner > .ix-spinner-ring.xxl:not(.hide-track)::before {
  border-width: var(--ix-spinner-track--border-width-emphasis);
}
.ix-theme .ix-spinner > .ix-spinner-ring::after {
  border-color: var(--ix-button-color, var(--ix-spinner--border-color));
  animation: ix-spinner-clip-mask var(--ix-spinner--mask-duration) linear infinite;
}
.ix-theme .ix-spinner > .ix-spinner-ring:is(.xxs, .xs, .sm, .md, .xxl):not(.hide-track)::before { border-color: var(--ix-spinner-track--border-color); }
.ix-theme .ix-spinner > .ix-spinner-ring.primary::after { border-color: var(--ix-spinner-primary--border-color); }
.ix-theme .ix-spinner > .ix-spinner-ring.primary::before { border-color: var(--ix-spinner-primary-track--border-color); }
@keyframes ix-spinner-rotate { 100% { transform: rotate(360deg); } }
@keyframes ix-spinner-clip-mask {
  0% { clip-path: polygon(50% 50%, 0 0, 0 0, 0 0, 0 0, 0 0); }
  12.5% { clip-path: polygon(50% 50%, 0 0, 100% 0, 100% 0, 100% 0, 100% 0); }
  25% { clip-path: polygon(50% 50%, 0 0, 100% 0, 100% 100%, 100% 100%, 100% 100%); }
  37.5% { clip-path: polygon(50% 50%, 0 0, 100% 0, 100% 100%, 0 100%, 0 100%); }
  50% { clip-path: polygon(50% 50%, 0 0, 100% 0, 100% 100%, 0 100%, 0 0); }
  62.5% { clip-path: polygon(50% 50%, 100% 0, 100% 0, 100% 100%, 0 100%, 0 0); }
  75% { clip-path: polygon(50% 50%, 100% 100%, 100% 100%, 100% 100%, 0 100%, 0 0); }
  87.5% { clip-path: polygon(50% 50%, 0 100%, 0 100%, 0 100%, 0 100%, 0 0); }
  100% { clip-path: polygon(50% 50%, 0 0, 0 0, 0 0, 0 0, 0 0); }
}
"#;

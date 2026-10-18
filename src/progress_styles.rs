//! Progress presentation informed by the pinned MIT-licensed iX 5.2.1 recipes.
//! SPDX-FileCopyrightText: 2025 Siemens AG
//! SPDX-License-Identifier: MIT
//! Public Brand system tokens can replace the established provisional colors.

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-progress {
  --ix-progress-color: var(--ix-progress-indicator-fill--background, var(--si-sys-color-border-accent-hover, var(--ix-color-primary)));
  --_ix-progress-track: var(--ix-progress-indicator-track--background, var(--si-sys-color-border-4, var(--ix-color-surface-2)));
  --_ix-progress-helper: var(--ix-progress-indicator-helper--color, var(--ix-color-text-secondary));
  --_ix-progress-icon: var(--ix-progress-indicator-helper-icon--color, var(--ix-color-text-secondary));
  --_ix-progress-label: var(--ix-progress-indicator-label--color, var(--ix-color-text-secondary));
  --_ix-progress-height: var(--ix-progress-indicator-md--height, var(--si-sys-sizing-size-30, 8px));
  display: block;
  width: var(--ix-progress-indicator--width, 100%);
  max-width: 100%;
  min-width: 0;
  color: var(--ix-color-text);
  font: 14px/20px var(--ix-font);
}
.ix-theme .ix-progress-status-error {
  --ix-progress-color: var(--ix-progress-indicator-fill-error--background, var(--si-sys-color-border-danger, var(--ix-color-danger)));
  --_ix-progress-track: var(--ix-progress-indicator-track-error--background, var(--si-sys-color-background-danger-subtle, color-mix(in srgb, var(--ix-color-danger) 16%, var(--ix-color-surface-1))));
  --_ix-progress-helper: var(--ix-progress-indicator-helper-error--color, var(--ix-color-danger-text));
  --_ix-progress-icon: var(--ix-progress-indicator-helper-icon-error--color, var(--ix-color-danger-text));
  --_ix-progress-label: var(--ix-progress-indicator-label-error--color, var(--ix-color-danger-text));
}
.ix-theme .ix-progress-status-info {
  --ix-progress-color: var(--ix-progress-indicator-fill-info--background, var(--si-sys-color-border-information, var(--ix-color-info)));
  --_ix-progress-track: var(--ix-progress-indicator-track-info--background, var(--si-sys-color-background-information-subtle, color-mix(in srgb, var(--ix-color-info) 16%, var(--ix-color-surface-1))));
  --_ix-progress-helper: var(--ix-progress-indicator-helper-info--color, var(--ix-color-text));
  --_ix-progress-icon: var(--ix-progress-indicator-helper-icon-info--color, var(--ix-color-info-text));
}
.ix-theme .ix-progress-status-warning {
  --ix-progress-color: var(--ix-progress-indicator-fill-warning--background, var(--si-sys-color-border-warning, var(--ix-color-critical)));
  --_ix-progress-track: var(--ix-progress-indicator-track-warning--background, var(--si-sys-color-background-warning-subtle, color-mix(in srgb, var(--ix-color-critical) 16%, var(--ix-color-surface-1))));
  --_ix-progress-helper: var(--ix-progress-indicator-helper-warning--color, var(--ix-color-text));
  --_ix-progress-icon: var(--ix-progress-indicator-helper-icon-warning--color, var(--si-sys-color-text-warning, var(--ix-color-critical)));
}
.ix-theme .ix-progress-status-success {
  --ix-progress-color: var(--ix-progress-indicator-fill-success--background, var(--si-sys-color-border-success, var(--ix-color-success)));
  --_ix-progress-track: var(--ix-progress-indicator-track-success--background, var(--si-sys-color-background-success-subtle, color-mix(in srgb, var(--ix-color-success) 16%, var(--ix-color-surface-1))));
  --_ix-progress-helper: var(--ix-progress-indicator-helper-success--color, var(--ix-color-text));
  --_ix-progress-icon: var(--ix-progress-indicator-helper-icon-success--color, var(--si-sys-color-text-success, var(--ix-color-success)));
}
.ix-theme .ix-progress-status-paused {
  --ix-progress-color: var(--ix-progress-indicator-fill-paused--background, var(--si-sys-color-border-1, var(--ix-color-text-secondary)));
  --_ix-progress-track: var(--ix-progress-indicator-track-paused--background, var(--si-sys-color-border-4, var(--ix-color-surface-2)));
  --_ix-progress-helper: var(--ix-progress-indicator-helper-paused--color, var(--ix-color-text));
  --_ix-progress-icon: var(--ix-progress-indicator-helper-icon-paused--color, var(--ix-color-text-secondary));
}
.ix-theme .ix-progress-xs { --_ix-progress-height: var(--ix-progress-indicator-xs--height, var(--si-sys-sizing-size-10, 2px)); }
.ix-theme .ix-progress-sm { --_ix-progress-height: var(--ix-progress-indicator-sm--height, var(--si-sys-sizing-size-20, 4px)); }
.ix-theme .ix-progress-lg { --_ix-progress-height: var(--ix-progress-indicator-lg--height, var(--si-sys-sizing-size-50, 16px)); }
.ix-theme .ix-progress-xl { --_ix-progress-height: var(--ix-progress-indicator-xl--height, var(--si-sys-sizing-size-70, 24px)); }
.ix-theme .ix-progress-label {
  color: var(--_ix-progress-label);
  margin: var(--ix-progress-indicator-label--margin, 0 0 8px);
  overflow-wrap: anywhere;
}
.ix-theme .ix-progress-content {
  display: flex;
  align-items: center;
  gap: var(--ix-progress-indicator-progress-container--gap, 8px);
  min-width: 0;
}
.ix-theme .ix-progress-track {
  flex: 1 1 0;
  min-width: 0;
  height: var(--_ix-progress-height);
  overflow: hidden;
  background: var(--_ix-progress-track);
  border-radius: var(--ix-progress-indicator-linear-progress-container--border-radius, var(--si-sys-sizing-border-radius-xs, 2px));
}
.ix-theme .ix-progress-fill {
  height: 100%;
  background: var(--ix-progress-color);
  transition: width .2s ease-out;
}
.ix-theme .ix-progress-value {
  flex: 0 1 auto;
  min-width: 0;
  max-width: 100%;
  font-variant-numeric: tabular-nums;
  overflow-wrap: anywhere;
}
.ix-theme .ix-progress-linear .ix-progress-value {
  min-width: min(40px, 25%);
  max-width: 50%;
}
.ix-theme .ix-progress-helper {
  display: flex;
  align-items: flex-start;
  gap: var(--ix-progress-indicator-helper-text--gap, 8px);
  margin: var(--ix-progress-indicator-helper-text--margin, 8px 0 0);
  min-width: 0;
  color: var(--_ix-progress-helper);
  font-size: 12px;
  line-height: 18px;
}
.ix-theme .ix-progress-helper > .ix-icon {
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--_ix-progress-icon);
}
.ix-theme .ix-progress-helper-text { min-width: 0; overflow-wrap: anywhere; }
.ix-theme .ix-progress-circle {
  position: relative;
  display: inline-flex;
  width: var(--ix-progress-size);
  height: var(--ix-progress-size);
  flex-shrink: 0;
}
.ix-theme .ix-progress-circle svg {
  display: block;
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
  overflow: visible;
}
.ix-theme .ix-progress-circle circle {
  fill: none;
  stroke-width: 2.4px;
  vector-effect: non-scaling-stroke;
}
.ix-theme .ix-progress-circle-track { stroke: var(--_ix-progress-track); }
.ix-theme .ix-progress-circle-fill {
  stroke: var(--ix-progress-color);
  stroke-linecap: butt;
  transition: stroke-dashoffset .2s ease-out;
}
.ix-theme .ix-progress-circle-value {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px;
  min-width: 0;
  overflow: hidden;
  font-size: 12px;
  line-height: 16px;
  font-variant-numeric: tabular-nums;
}
.ix-theme .ix-progress-text-left .ix-progress-helper,
.ix-theme .ix-progress-circular.ix-progress-text-left .ix-progress-content { justify-content: flex-start; }
.ix-theme .ix-progress-text-center .ix-progress-helper,
.ix-theme .ix-progress-circular.ix-progress-text-center .ix-progress-content { justify-content: center; }
.ix-theme .ix-progress-text-right .ix-progress-helper,
.ix-theme .ix-progress-circular.ix-progress-text-right .ix-progress-content { justify-content: flex-end; }
@media (prefers-reduced-motion: reduce) {
  .ix-theme .ix-progress-fill,
  .ix-theme .ix-progress-circle-fill { transition: none; }
}
"#;

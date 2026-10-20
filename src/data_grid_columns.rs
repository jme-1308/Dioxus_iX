// SPDX-License-Identifier: MIT

//! Pure horizontal track-sizing and column-window geometry for `DataGrid`.
//!
//! These helpers model the leading 48px selection column and CSS Grid's
//! `minmax(min_width.max(24), 1fr)` / fixed-pixel track sizing so horizontal
//! virtualisation can select a bounded column window while the full CSS grid
//! template remains the visual layout authority.

use std::ops::Range;

const SELECTION_WIDTH: f64 = 48.0;
const MIN_TRACK_WIDTH: u32 = 24;
const DEFAULT_VIEWPORT_WIDTH: f64 = 720.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ColumnTrack {
    pub min_width: u32,
    pub fixed_width: Option<u32>,
}

fn effective_viewport_width(viewport_width: f64) -> f64 {
    if viewport_width.is_finite() && viewport_width > 0.0 {
        viewport_width
    } else {
        DEFAULT_VIEWPORT_WIDTH
    }
}

fn sanitize_track_width(width: f64) -> f64 {
    if width.is_finite() && width > 0.0 {
        width
    } else {
        f64::from(MIN_TRACK_WIDTH)
    }
}

/// Resolve per-column pixel widths for a horizontal viewport that includes the
/// leading 48px selection track.
///
/// Fixed tracks consume `fixed_width.max(min_width.max(24))`. Flexible tracks
/// share remaining space using CSS Grid `minmax(min, 1fr)` water-filling. When
/// minimum and fixed tracks exceed the viewport, flexible tracks use their
/// minimums; when all tracks are fixed, unused viewport space remains blank.
pub(crate) fn column_widths(tracks: &[ColumnTrack], viewport_width: f64) -> Vec<f64> {
    let mut widths: Vec<f64> = tracks
        .iter()
        .map(|track| {
            let min = track.min_width.max(MIN_TRACK_WIDTH);
            f64::from(track.fixed_width.map_or(min, |fixed| fixed.max(min)))
        })
        .collect();
    let available = (effective_viewport_width(viewport_width) - SELECTION_WIDTH).max(0.0);
    let base_total: f64 = widths.iter().sum();
    if available <= base_total {
        return widths;
    }
    let mut flexible: Vec<bool> = tracks
        .iter()
        .map(|track| track.fixed_width.is_none())
        .collect();
    let mut remaining = (available
        - tracks
            .iter()
            .zip(&widths)
            .filter(|(track, _)| track.fixed_width.is_some())
            .map(|(_, width)| *width)
            .sum::<f64>())
    .max(0.0);
    loop {
        let unfrozen = flexible.iter().filter(|&&is_flex| is_flex).count();
        if unfrozen == 0 {
            break;
        }
        let share = remaining / unfrozen as f64;
        let mut froze_any = false;
        for (is_flex, &width) in flexible.iter_mut().zip(&widths) {
            if *is_flex && width > share {
                *is_flex = false;
                remaining = (remaining - width).max(0.0);
                froze_any = true;
            }
        }
        if !froze_any {
            for (is_flex, width) in flexible.iter().zip(&mut widths) {
                if *is_flex {
                    *width = share.max(*width);
                }
            }
            break;
        }
    }
    widths
}

/// Compute the visible column index range with `buffer` overscan columns on
/// each side for data tracks starting at `x = 48` after the selection column.
pub(crate) fn column_window(
    widths: &[f64],
    scroll_left: f64,
    viewport_width: f64,
    buffer: usize,
) -> Range<usize> {
    if widths.is_empty() {
        return 0..0;
    }
    let viewport = effective_viewport_width(viewport_width);
    let mut intervals = Vec::with_capacity(widths.len());
    let mut cursor = SELECTION_WIDTH;
    for &width in widths {
        let start = cursor;
        let end = start + sanitize_track_width(width);
        intervals.push((start, end));
        cursor = end;
    }
    let max_scroll = (cursor - viewport).max(0.0);
    let scroll = if scroll_left.is_finite() {
        scroll_left.clamp(0.0, max_scroll)
    } else {
        0.0
    };
    let view_start = scroll;
    let view_end = scroll + viewport;
    let first = intervals
        .iter()
        .position(|&(_, end)| end > view_start)
        .unwrap_or(widths.len() - 1);
    let last_inclusive = intervals
        .iter()
        .rposition(|&(start, _)| start < view_end)
        .unwrap_or(first)
        .max(first);
    first.saturating_sub(buffer)
        ..(last_inclusive + 1)
            .saturating_add(buffer)
            .min(widths.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_widths_allocates_mixed_fixed_and_flexible_tracks_with_water_filling() {
        let tracks = [
            ColumnTrack {
                min_width: 80,
                fixed_width: Some(100),
            },
            ColumnTrack {
                min_width: 12,
                fixed_width: None,
            },
            ColumnTrack {
                min_width: 100,
                fixed_width: None,
            },
            ColumnTrack {
                min_width: 250,
                fixed_width: None,
            },
        ];
        // 748px viewport - 48px selection = 700px available; 100px fixed leaves 600px.
        // Initial 600 / 3 = 200 freezes the 250px min track; remaining 350 / 2 = 175.
        assert_eq!(
            column_widths(&tracks, 748.0),
            vec![100.0, 175.0, 175.0, 250.0]
        );

        // When minimum + fixed widths exceed the viewport, flexible tracks use minimums
        // (with <24px clamped to 24px) and fixed tracks clamp to at least their minimum.
        let tight = [
            ColumnTrack {
                min_width: 10,
                fixed_width: None,
            },
            ColumnTrack {
                min_width: 90,
                fixed_width: Some(60),
            },
            ColumnTrack {
                min_width: 120,
                fixed_width: None,
            },
        ];
        assert_eq!(column_widths(&tight, 200.0), vec![24.0, 90.0, 120.0]);

        // When all tracks are fixed, extra viewport width stays blank.
        let all_fixed = [
            ColumnTrack {
                min_width: 80,
                fixed_width: Some(110),
            },
            ColumnTrack {
                min_width: 80,
                fixed_width: Some(140),
            },
        ];
        assert_eq!(column_widths(&all_fixed, 900.0), vec![110.0, 140.0]);
    }

    #[test]
    fn column_window_resolves_selection_offset_boundaries_overscan_and_clamped_scroll() {
        // Tracks at [48..148), [148..248), [248..348), [348..448), [448..548).
        let widths = [100.0, 100.0, 100.0, 100.0, 100.0];
        assert_eq!(column_window(&widths, 0.0, 248.0, 0), 0..2);
        assert_eq!(column_window(&widths, 0.0, 248.0, 1), 0..3);
        assert_eq!(column_window(&widths, 100.0, 200.0, 0), 0..3);
        assert_eq!(column_window(&widths, 148.0, 200.0, 0), 1..3);
        assert_eq!(column_window(&widths, 148.0, 200.0, 1), 0..4);

        // Total content width is 548px; for a 248px viewport, max scroll is 300px,
        // placing the clamped window at [300..548), which intersects columns 2..5.
        assert_eq!(column_window(&widths, 9_999.0, 248.0, 0), 2..5);
        assert_eq!(column_window(&widths, 9_999.0, 248.0, 1), 1..5);
    }

    #[test]
    fn geometry_helpers_normalize_zero_nan_infinite_negative_and_empty_inputs() {
        assert!(column_widths(&[], 720.0).is_empty());
        assert_eq!(column_window(&[], 0.0, 720.0, 2), 0..0);

        let tracks = [
            ColumnTrack {
                min_width: 80,
                fixed_width: None,
            },
            ColumnTrack {
                min_width: 80,
                fixed_width: None,
            },
        ];
        // Non-positive and non-finite viewports use the 720px pre-measurement fallback
        // ((720 - 48) / 2 = 336).
        for invalid_viewport in [0.0, -200.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(column_widths(&tracks, invalid_viewport), vec![336.0, 336.0]);
        }

        let widths = [200.0, 200.0, 200.0, 200.0, 200.0];
        for invalid_scroll in [-50.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                column_window(&widths, invalid_scroll, 0.0, 0),
                column_window(&widths, 0.0, 720.0, 0)
            );
        }
        assert_eq!(
            column_window(&[0.0, f64::NAN, -10.0], f64::NAN, f64::NAN, usize::MAX),
            0..3
        );
    }

    #[test]
    fn thousand_columns_produce_bounded_premeasurement_and_scrolled_windows() {
        let tracks = vec![
            ColumnTrack {
                min_width: 80,
                fixed_width: None,
            };
            1000
        ];
        let widths = column_widths(&tracks, 0.0);
        assert_eq!(widths.len(), 1000);
        assert!(widths.iter().all(|&width| width == 80.0));

        // At scroll 0 with default 720px viewport, [0..720) covers 48px selection plus
        // 672px of data columns (ceil(672 / 80) = 9 columns: 0..9), plus 2 overscan = 0..11.
        let initial = column_window(&widths, 0.0, 0.0, 2);
        assert_eq!(initial, 0..11);
        assert!(initial.len() < 20);

        let middle = column_window(&widths, 40_000.0, 720.0, 2);
        assert_eq!(middle, 497..511);
        assert!(middle.len() < 20);

        let end = column_window(&widths, 1_000_000.0, 720.0, 2);
        assert_eq!(end, 989..1000);
    }
}

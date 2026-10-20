//! Column value interpretation. Cells and edit requests remain strings;
//! numeric columns opt into number ordering/comparison without a DOM bridge.
use crate::{DataGridFilterCondition, DataGridFilterOperator, DataGridSortDirection};
use std::cmp::Ordering;

/// Sorting/filtering interpretation, independent of the editor. Text preserves
/// case-insensitive behavior. Number accepts trimmed, finite `f64` decimals and
/// scientific notation; locale separators, currency and arbitrary precision
/// are not inferred. Cell display remains unchanged.
/// Host-side projections and selections use `DataGridState`'s column-aware
/// methods; legacy methods intentionally retain text semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataGridValueType {
    #[default]
    Text,
    /// Blank/invalid values sort last in either direction. Equal numbers and
    /// unavailable values retain the host's stable row order.
    /// Numeric comparisons exclude invalid values and queries. `Blank` matches
    /// trimmed empty cells; `NotBlank` also includes nonempty unknown values.
    Number,
}

fn finite_number(value: &str) -> Option<f64> {
    value
        .trim()
        .parse()
        .ok()
        .filter(|number: &f64| number.is_finite())
}

impl DataGridValueType {
    pub(crate) fn compare(
        self,
        left: &str,
        right: &str,
        direction: DataGridSortDirection,
    ) -> Ordering {
        let order = match self {
            Self::Text => left.to_lowercase().cmp(&right.to_lowercase()),
            Self::Number => match (finite_number(left), finite_number(right)) {
                (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
                (Some(_), None) => return Ordering::Less,
                (None, Some(_)) => return Ordering::Greater,
                (None, None) => return Ordering::Equal,
            },
        };
        if direction == DataGridSortDirection::Descending {
            order.reverse()
        } else {
            order
        }
    }

    pub(crate) fn matches(self, value: &str, condition: &DataGridFilterCondition) -> bool {
        if !condition.is_active() {
            return true;
        }
        use DataGridFilterOperator::*;
        match self {
            Self::Text => {
                let text = value.to_lowercase();
                let query = condition.value.to_lowercase();
                match condition.operator {
                    Contains => text.contains(&query),
                    NotContains => !text.contains(&query),
                    Equals => text == query,
                    NotEqual => text != query,
                    StartsWith => text.starts_with(&query),
                    EndsWith => text.ends_with(&query),
                    Blank => text.trim().is_empty(),
                    NotBlank => !text.trim().is_empty(),
                    _ => false,
                }
            }
            Self::Number => {
                match condition.operator {
                    Blank => return value.trim().is_empty(),
                    NotBlank => return !value.trim().is_empty(),
                    _ => {}
                }
                let (Some(number), Some(query)) =
                    (finite_number(value), finite_number(&condition.value))
                else {
                    return false;
                };
                match condition.operator {
                    Equals => number == query,
                    NotEqual => number != query,
                    LessThan => number < query,
                    LessThanOrEqual => number <= query,
                    GreaterThan => number > query,
                    GreaterThanOrEqual => number >= query,
                    _ => false,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_sort_by_value_and_keep_unavailable_values_last_in_both_directions() {
        let values = ["10", "2", "", "-0.5", "2.0", "pending", "2e0", "NaN", "inf"];
        for (direction, expected) in [
            (
                DataGridSortDirection::Ascending,
                ["-0.5", "2", "2.0", "2e0", "10", "", "pending", "NaN", "inf"],
            ),
            (
                DataGridSortDirection::Descending,
                ["10", "2", "2.0", "2e0", "-0.5", "", "pending", "NaN", "inf"],
            ),
        ] {
            let mut actual = values;
            actual.sort_by(|left, right| DataGridValueType::Number.compare(left, right, direction));
            assert_eq!(actual, expected);
        }
        assert_eq!(
            DataGridValueType::Number.compare(" -0 ", "0", DataGridSortDirection::Ascending),
            Ordering::Equal
        );
        assert_eq!(
            DataGridValueType::Text.compare("10", "2", DataGridSortDirection::Ascending),
            Ordering::Less
        );
    }

    #[test]
    fn numeric_comparisons_reject_unknown_values_without_treating_them_as_zero() {
        let condition = |operator, value: &str| DataGridFilterCondition {
            operator,
            value: value.into(),
        };
        use DataGridFilterOperator::*;
        let matches = |operator, query, value| {
            DataGridValueType::Number.matches(value, &condition(operator, query))
        };
        assert!(matches(Equals, "2.0", " 2e0 "));
        assert!(matches(Equals, "0", "-0"));
        assert!(matches(NotEqual, "2", "2.5"));
        assert!(matches(LessThan, "-1", "-2"));
        assert!(matches(LessThanOrEqual, "2", "2.0"));
        assert!(matches(GreaterThan, "2", "2.5"));
        assert!(matches(GreaterThanOrEqual, "2", "2e0"));
        for value in ["", " ", "pending", "NaN", "inf", "1e999"] {
            assert!(!matches(Equals, "0", value), "{value}");
            assert!(!matches(NotEqual, "0", value), "{value}");
        }
        for query in ["NaN", "inf", "unknown", "1,000"] {
            assert!(!matches(NotEqual, query, "2"), "{query}");
        }
        assert!(matches(Blank, "", " \t"));
        assert!(!matches(Blank, "", "pending"));
        assert!(matches(NotBlank, "", "pending"));
        assert!(!matches(Contains, "2", "2"));
        assert!(matches(Equals, "", "pending")); // Inactive condition.
    }
}

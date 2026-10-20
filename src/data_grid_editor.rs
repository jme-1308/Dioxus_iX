//! Native editor choices for the controlled Rust grid. Values stay strings so
//! consumers keep the same stable-ID edit request and conflict checks.

/// A configured grid editor. `DataGridColumn::editor` supplies the shared
/// required/length constraints; this choice adds type-specific validation.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum DataGridCellEditor {
    #[default]
    Text,
    Number(DataGridNumberEditor),
    /// Use unique option values. Unknown or disabled values cannot be saved.
    Select(Vec<DataGridSelectOption>),
    /// Boolean cells use the literal strings `true` and `false`.
    Checkbox,
}

impl DataGridCellEditor {
    /// Validate a draft before requesting a controlled row update.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            Self::Text => Ok(()),
            Self::Number(config) => config.validate(value),
            Self::Select(options) => {
                if options
                    .iter()
                    .any(|option| !option.disabled && option.value == value)
                {
                    Ok(())
                } else {
                    Err("Choose an available option before saving.".into())
                }
            }
            Self::Checkbox if matches!(value, "true" | "false") => Ok(()),
            Self::Checkbox => Err("Choose a checked or unchecked value before saving.".into()),
        }
    }
}

/// Inclusive bounds and an optional positive increment for a native number
/// editor. No step means any finite decimal is accepted. With a step, the
/// increment starts at `min`, or zero when there is no minimum.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataGridNumberEditor {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

impl DataGridNumberEditor {
    /// Empty input is allowed here; the column's shared `required` constraint
    /// decides whether it can be saved. Invalid configuration is never ignored.
    pub fn validate(&self, value: &str) -> Result<(), String> {
        if self.min.is_some_and(|value| !value.is_finite())
            || self.max.is_some_and(|value| !value.is_finite())
            || matches!((self.min, self.max), (Some(min), Some(max)) if min > max)
            || self
                .step
                .is_some_and(|step| !step.is_finite() || step <= 0.0)
        {
            return Err("The number limits are invalid.".into());
        }
        let value = value.trim();
        if value.is_empty() {
            return Ok(());
        }
        let number = value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or("Enter a finite number before saving.")?;
        if let Some(min) = self.min {
            if number < min {
                return Err(format!("Enter {min} or more before saving."));
            }
        }
        if let Some(max) = self.max {
            if number > max {
                return Err(format!("Enter {max} or less before saving."));
            }
        }
        if let Some(step) = self.step {
            let base = self.min.unwrap_or(0.0);
            let increments = (number - base) / step;
            // Decimal steps such as 0.1 need a small arithmetic tolerance;
            // genuinely off-step drafts remain invalid and editable.
            if !increments.is_finite() || (increments - increments.round()).abs() > 1e-7 {
                return Err(format!(
                    "Use increments of {step} from {base} before saving."
                ));
            }
        }
        Ok(())
    }
}

/// One native select option, preserving a stable stored value and visible label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataGridSelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl DataGridSelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_constraints_reject_nonfinite_out_of_bounds_and_off_step_drafts() {
        let number = DataGridNumberEditor {
            min: Some(0.0),
            max: Some(100.0),
            step: Some(0.5),
        };
        for value in ["", "0", "2.5", "1e1", "100"] {
            assert!(number.validate(value).is_ok(), "{value}");
        }
        for value in [
            "NaN", "inf", "-inf", "1e999", "kit", "-0.5", "100.5", "2.25",
        ] {
            assert!(number.validate(value).is_err(), "{value}");
        }
        let decimal = DataGridNumberEditor {
            min: Some(0.1),
            max: None,
            step: Some(0.1),
        };
        assert!(decimal.validate("0.3").is_ok());
        assert!(decimal.validate("0.35").is_err());
        assert!(DataGridNumberEditor::default().validate("-0.125").is_ok());
        for config in [
            DataGridNumberEditor {
                min: Some(2.0),
                max: Some(1.0),
                step: None,
            },
            DataGridNumberEditor {
                min: Some(f64::NAN),
                max: None,
                step: None,
            },
            DataGridNumberEditor {
                min: None,
                max: Some(f64::INFINITY),
                step: None,
            },
            DataGridNumberEditor {
                min: None,
                max: None,
                step: Some(0.0),
            },
            DataGridNumberEditor {
                min: None,
                max: None,
                step: Some(-1.0),
            },
        ] {
            assert!(config.validate("1").is_err());
        }
    }

    #[test]
    fn choice_and_boolean_values_validate_without_coercing_consumer_data() {
        let mut retired = DataGridSelectOption::new("retired", "Retired");
        retired.disabled = true;
        let choices =
            DataGridCellEditor::Select(vec![DataGridSelectOption::new("ready", "Ready"), retired]);
        assert!(choices.validate("ready").is_ok());
        for value in ["", "Ready", "retired", "unknown"] {
            assert!(choices.validate(value).is_err(), "{value}");
        }
        for value in ["true", "false"] {
            assert!(DataGridCellEditor::Checkbox.validate(value).is_ok());
        }
        for value in ["", "1", "TRUE", "false "] {
            assert!(
                DataGridCellEditor::Checkbox.validate(value).is_err(),
                "{value}"
            );
        }
    }
}

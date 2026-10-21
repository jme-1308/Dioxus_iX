use super::*;
use crate::{
    DataGridFilterCondition, DataGridFilterOperator, DataGridSortDirection, DataGridTextFilter,
};

fn columns() -> Vec<DataGridColumn> {
    let mut number = DataGridColumn::new("load", "Load");
    number.value_type = DataGridValueType::Number;
    vec![
        DataGridColumn::new("asset", "Asset"),
        number,
        DataGridColumn::new("note", "Note"),
    ]
}
#[test]
fn csv_scopes_reuse_typed_sort_managed_order_filters_and_stable_selection() {
    let columns = columns();
    let rows = vec![
        DataGridRow::new("a")
            .cell("asset", "Pump A")
            .cell("load", "10"),
        DataGridRow::new("b")
            .cell("asset", "Pump B")
            .cell("load", "2"),
        DataGridRow::new("c")
            .cell("asset", "Valve C")
            .cell("load", "2.0"),
        DataGridRow::new("d")
            .cell("asset", "Pump D")
            .cell("load", "unknown"),
    ];
    let mut state = DataGridState {
        sort: Some(("load".into(), DataGridSortDirection::Ascending)),
        row_order: vec!["c".into(), "b".into(), "a".into()],
        column_order: vec!["load".into(), "asset".into()],
        ..Default::default()
    };
    state
        .selected
        .extend(["a".into(), "c".into(), "stale".into()]);
    state.filters.insert(
        "asset".into(),
        DataGridTextFilter {
            first: DataGridFilterCondition {
                operator: DataGridFilterOperator::Contains,
                value: "Pump".into(),
            },
            ..Default::default()
        },
    );
    for (row_scope, expected) in [
        (DataGridCsvRowScope::Displayed, vec!["b", "a", "d"]),
        (DataGridCsvRowScope::SelectedDisplayed, vec!["a"]),
        (DataGridCsvRowScope::SelectedAll, vec!["c", "a"]),
        (DataGridCsvRowScope::All, vec!["c", "b", "a", "d"]),
    ] {
        let csv = state.export_csv(
            &rows,
            &columns,
            &DataGridCsvOptions {
                row_scope,
                ..Default::default()
            },
        );
        assert_eq!(csv.row_ids, expected);
        assert_eq!(csv.column_ids, ["load", "asset", "note"]);
        assert!(csv.content.starts_with("Load,Asset,Note\r\n"));
        if row_scope == DataGridCsvRowScope::All {
            assert!(
                csv.content
                    .starts_with("Load,Asset,Note\r\n2.0,Valve C,\r\n2,Pump B,\r\n10,Pump A,\r\n")
            );
        }
    }
    assert_eq!(
        state.filters.len(),
        1,
        "export leaves current filtering untouched"
    );
}
#[test]
fn csv_preserves_unicode_quotes_multiline_missing_cells_and_subset_display_order() {
    let columns = columns();
    let rows = vec![
        DataGridRow::new("r")
            .cell("asset", "Møtor, \"α\"")
            .cell("note", "Line one\r\nLine two"),
    ];
    let state = DataGridState {
        column_order: vec!["note".into(), "asset".into(), "load".into()],
        ..Default::default()
    };
    let csv = state.export_csv(&rows, &columns, &Default::default());
    assert_eq!(
        csv.content,
        "Note,Asset,Load\r\n\"Line one\r\nLine two\",\"Møtor, \"\"α\"\"\",\r\n"
    );
    let csv = state.export_csv(
        &rows,
        &columns,
        &DataGridCsvOptions {
            include_headers: false,
            columns: Some(vec![
                "asset".into(),
                "missing".into(),
                "note".into(),
                "note".into(),
            ]),
            ..Default::default()
        },
    );
    assert_eq!(csv.column_ids, ["note", "asset"]);
    assert_eq!(
        csv.content,
        "\"Line one\r\nLine two\",\"Møtor, \"\"α\"\"\"\r\n"
    );
    let empty = state.export_csv(&[], &columns, &Default::default());
    assert_eq!(empty.content, "Note,Asset,Load\r\n");
    let no_columns = state.export_csv(
        &rows,
        &columns,
        &DataGridCsvOptions {
            columns: Some(vec![]),
            ..Default::default()
        },
    );
    assert!(no_columns.content.is_empty());
    assert!(no_columns.column_ids.is_empty());
    assert_eq!(no_columns.row_ids, ["r"]);
    assert!(
        state
            .export_csv(
                &[],
                &columns,
                &DataGridCsvOptions {
                    include_headers: false,
                    ..Default::default()
                }
            )
            .content
            .is_empty()
    );
}
#[test]
fn csv_default_formula_policy_escapes_text_headers_and_invalid_numbers_but_preserves_finite_numeric_cells()
 {
    let mut columns = columns();
    columns[0].title = " =SUM(A1)".into();
    let values = [
        ("=1+1", "-42"),
        ("  +SUM(A1)", "+2.5"),
        ("\tordinary", "1e2"),
        ("\rtext", "-1e999"),
        ("@cmd", "NaN"),
        ("-12", " -0.5 "),
        ("+7", "+SUM(A1)"),
        ("\u{2003}=1", "0"),
    ];
    let rows: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, (text, number))| {
            DataGridRow::new(index.to_string())
                .cell("asset", *text)
                .cell("load", *number)
        })
        .collect();
    let escaped = DataGridState::default().export_csv(&rows, &columns, &Default::default());
    assert_eq!(
        escaped.content,
        "' =SUM(A1),Load,Note\r\n'=1+1,-42,\r\n'  +SUM(A1),+2.5,\r\n'\tordinary,1e2,\r\n\"'\rtext\",'-1e999,\r\n'@cmd,NaN,\r\n'-12, -0.5 ,\r\n'+7,'+SUM(A1),\r\n'\u{2003}=1,0,\r\n"
    );
    let raw = DataGridState::default().export_csv(
        &rows,
        &columns,
        &DataGridCsvOptions {
            formula_policy: DataGridCsvFormulaPolicy::Preserve,
            ..Default::default()
        },
    );
    assert!(
        raw.content
            .starts_with(" =SUM(A1),Load,Note\r\n=1+1,-42,\r\n")
    );
    assert!(
        raw.content
            .contains("\tordinary,1e2,\r\n\"\rtext\",-1e999,")
    );
    assert!(raw.content.contains("+7,+SUM(A1),\r\n"));
}
#[test]
fn csv_snapshot_is_owned_committed_data_and_download_bytes_are_bom_prefixed_utf8() {
    let mut rows = vec![
        DataGridRow::new("asset")
            .cell("asset", "Møtor")
            .cell("load", "-2"),
    ];
    let mut columns = columns();
    let mut state = DataGridState::default();
    let snapshot = state.export_csv(&rows, &columns, &Default::default());
    rows[0]
        .cells
        .insert("asset".into(), "Later committed edit".into());
    columns[0].title = "Changed column".into();
    state.row_order.push("different".into());
    state.selected.insert("asset".into());
    assert_eq!(snapshot.content, "Asset,Load,Note\r\nMøtor,-2,\r\n");
    assert_eq!(snapshot.row_ids, ["asset"]);
    let encoded = snapshot.data_url();
    let encoded = encoded
        .strip_prefix("data:text/csv;charset=utf-8;base64,")
        .unwrap();
    let bytes = STANDARD.decode(encoded).unwrap();
    assert_eq!(&bytes[..3], b"\xef\xbb\xbf");
    assert_eq!(std::str::from_utf8(&bytes[3..]).unwrap(), snapshot.content);
}

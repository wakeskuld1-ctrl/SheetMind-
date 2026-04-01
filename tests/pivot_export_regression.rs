use calamine::{Data, Reader, open_workbook_auto};
use excel_skill::domain::handles::TableHandle;
use excel_skill::frame::loader::LoadedTable;
use excel_skill::ops::export::export_excel;
use excel_skill::ops::pivot::{PivotAggregation, pivot_table};
use excel_skill::ops::preview::preview_table;
use polars::prelude::{DataFrame, NamedFrom, Series};

mod common;

use crate::common::create_test_output_path;

#[test]
fn pivot_table_export_writes_blank_cells_and_numeric_values_to_excel() {
    let loaded = LoadedTable {
        // 2026-03-23: 这里构造带缺失透视交叉格的最小样例，原因是先稳定复现“空值被写成 null 文本、数值被写成字符串”的导出问题。
        // 2026-03-23: 目的在于锁定导出后的 Excel 单元格要么为空白，要么为真正可统计的数值类型。
        handle: TableHandle::new_confirmed(
            "memory://pivot-export",
            "Sheet1",
            vec!["region".into(), "month".into(), "sales".into()],
        ),
        dataframe: DataFrame::new(vec![
            Series::new("region".into(), vec![Some("East"), Some("West")]).into(),
            Series::new("month".into(), vec![Some("Jan"), Some("Feb")]).into(),
            Series::new("sales".into(), vec![100.0_f64, 80.5_f64]).into(),
        ])
        .unwrap(),
    };

    let pivoted = pivot_table(
        &loaded,
        &["region"],
        &["month"],
        &["sales"],
        PivotAggregation::Sum,
    )
    .unwrap();
    let preview = preview_table(&pivoted.dataframe, pivoted.dataframe.height()).unwrap();
    let output_path = create_test_output_path("pivot_table_export_frame", "xlsx");

    // 2026-03-23: 这里先锁定预览里的缺失交叉格为空字符串，原因是用户不希望再看到 null 文本。
    assert_eq!(preview.rows[0]["Feb"], "");

    export_excel(&pivoted, output_path.to_str().unwrap(), "Pivot").unwrap();

    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("Pivot").unwrap();
    let east_feb = range.get((1, 1));
    let east_jan = range.get((1, 2)).unwrap();
    let west_feb = range.get((2, 1)).unwrap();

    // 2026-03-23: 这里锁定空单元格保持为空白，原因是避免 Excel 里出现字符串 null 干扰后续统计。
    assert!(east_feb.is_none() || matches!(east_feb, Some(Data::Empty)));
    // 2026-03-23: 这里锁定导出单元格为数值类型，原因是用户导出后要能继续直接求和/透视。
    assert!(matches!(east_jan, Data::Float(value) if (*value - 100.0).abs() < 1e-9));
    assert!(matches!(west_feb, Data::Float(value) if (*value - 80.5).abs() < 1e-9));
}

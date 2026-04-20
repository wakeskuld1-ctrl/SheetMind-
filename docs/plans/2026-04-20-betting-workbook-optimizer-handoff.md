# Betting Workbook Optimizer Handoff

## What Is Delivered

- Rust solver binary entry: `src/bin/betting_solver.rs`
- Workbook bridge: `src/ops/betting_workbook_bridge.rs`
- Optimizer core: `src/ops/betting_optimizer.rs`
- Delivery macro asset path: `assets/excel_templates/betting_optimizer/vbaProject.bin`

## Current Business Contract

- Formula contract:
  - `赔付额 = 下注额 * 47`
  - `可赔付本金 = 总下注额 * 0.98`
  - `盈亏额 = 赔付额 - 可赔付本金`
- Risk interpretation:
  - `盈亏额 > 0` means the number is a risk number
  - `盈亏额 <= 0` means the number is a safe number
- Refund constraint:
  - only current risk numbers may be adjusted downward
  - current safe numbers must remain unchanged

## Workbook Contract

- Sheet 1: `当前盘面`
  - rows `2:50`: numbers `01..49`
  - column `B`: original stake input
  - cell `F2`: max loss target
  - cell `F3`: loss count target
- Sheet 2: `优化建议`
  - summary paragraph
  - summary metric table
  - adjustment detail rows

## CLI Usage

```powershell
betting_solver.exe template .\计算器s6.1_稳交付版.xlsm
betting_solver.exe solve .\计算器s6.1_稳交付版.xlsm
betting_solver.exe solve .\计算器s6.1_稳交付版.xlsm .\计算器s6.1_稳交付版-结果.xlsm
```

## Remaining Risk

- The bundled `vbaProject.bin` is still the spike-stage macro asset used to keep the workbook macro-enabled.
- The Rust delivery path is complete for template generation and workbook solving.
- The final customer-facing button behavior still needs a formal VBA project replacement if the Excel button must launch the solver directly without a manual CLI step.

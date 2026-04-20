# Betting Workbook Delivery

Place `betting_solver.exe` beside the delivered workbook.

Recommended customer package layout:

- `计算器s6.1_稳交付版_页面还原.xlsm`
- `betting_solver.exe`
- `README.md`

Current delivery note:

- The Rust solver is the formal calculation engine.
- The workbook keeps the first-sheet input experience and the second-sheet suggestion view.
- The embedded VBA asset is now the formal workbook launcher:
  `say_hello` saves the workbook, shows a modeless progress form, starts `betting_solver.exe`, writes trace logs, and refreshes sheet 2 in place.
- Runtime traces are written into the workbook directory:
  - `logs/betting_macro_*.log`
  - `logs/betting_solver_*.log`
- The current run status is mirrored into `计算器!AH14`.
- If the operator clicks `取消`, the macro requests process termination and stops the current run.

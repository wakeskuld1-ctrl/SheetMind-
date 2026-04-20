# Betting Workbook Delivery

Place `betting_solver.exe` beside the delivered workbook.

Recommended customer package layout:

- `计算器s6.1_稳交付版.xlsm`
- `betting_solver.exe`
- `README.md`

Current delivery note:

- The Rust solver is the formal calculation engine.
- The workbook keeps the first-sheet input experience and the second-sheet suggestion view.
- The embedded `vbaProject.bin` currently comes from the repository's macro workbook spike asset, so the workbook shape is stable, but the final customer-facing VBA trigger still needs a formal business macro replacement if you want the in-Excel button to launch the solver automatically.
- Until that final VBA asset is replaced, the standard operating path is:
  1. open the workbook and fill sheet 1
  2. run `betting_solver.exe solve <workbook-path>`
  3. reopen or refresh the generated workbook result

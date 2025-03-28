# Excel TUI

Replicating most of Excel's functionality in a simple terminal app.

## Formulas

In theory, all Excel formulas should function in this TUI as well.

### Function RPN Expectations

- `MEAN(1,2,3)+MEAN(4,5,6)` -> `1 2 3 MEAN 4 5 6 MEAN +`

## To Do

- [ ] Menu bar
- [ ] Status bar
- [x] Formula suggestions
- [x] Multi-cell selections
- [x] Support reference operators in formulas
- [x] Scrolling of the table
- [x] Make it an actual CLI
- [ ] Application of functions and operations along a range of cells (ie. `A1:B2+3`)
- [x] Cache the application's function rendering state and only update it when cells change
- [x] Undo and redo
- [x] Mouse support
- [ ] Multi-sheet spreadsheets and XLSX support
- [ ] Theming and changing the color scheme
- [ ] Inserting cells into formula by clicking in spreadsheet
- [x] Highlighting cell ranges from active formula in spreadsheet
- [ ] $ for cells when copy-pasting
- [ ] Make horizontal scrolling a little bit smoother
- [ ] Pasting blocks of text
- [ ] A dozen other things

## Known Issues

*Not missing features, but issues/caveats.*

- Horizontal scrolling through the touchpad is only supported on some terminals.

## Specifications

These are the specs of Excel that eventually need to be met:

- https://support.microsoft.com/en-us/office/overview-of-formulas-in-excel-ecfdc708-9162-49e8-b993-c311f47ca173
- https://support.microsoft.com/en-us/office/calculation-operators-and-precedence-in-excel-48be406d-4975-4d31-b2b8-7af9e0e2878a
- https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
- https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/f780b2d6-8252-4074-9fe3-5d7bc4830968

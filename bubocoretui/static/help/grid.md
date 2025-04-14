The Grid provides a matrix interface for creating and manipulating sequences and scripts that compose the scene loaded on the server.

## Navigation & Selection

*   `↑` / `↓` / `←` / `→` : Move the cursor (single cell selection).
*   `Shift` + `Arrows` : Extend the selection range.
*   `Esc` : If multiple cells are selected, reset to a single cell selection at the start of the previous range.

## Sequence Manipulation

*   `a`: Add a new line column.
*   `d`: Remove the *last* line column.
*   `c`: Copy the selected cells to the clipboard.
*   `p`: Paste the copied step under cursor.

## Step Manipulation (within Sequences)

*   `+`: Add a new step (length 1.0) to the *end* of the line under the cursor.
*   `-`: Remove the *last* step from the line under the cursor.
*   `Space` : Toggle the enabled/disabled state of the selected step(s).
*   `Enter` : Request the script for the selected step (opens in Editor).
*   `>` or `.`: Increase the length of selected step(s) by 0.25.
*   `<` or `,`: Decrease the length of selected step(s) by 0.25 (minimum length 0.01).
*   `b`: Toggle the selected step as the *start marker* for its line.
*   `e`: Toggle the selected step as the *end marker* for its line.

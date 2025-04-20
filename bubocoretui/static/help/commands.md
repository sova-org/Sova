Type `Ctrl+P` to open the command palette, then enter commands:

## General

*   `quit` (aliases: `q`, `exit`) - Quit the application
*   `navigation` (alias: `nav`) - Toggle the navigation overlay

## Views

*   `editor` (aliases: `edit`, `script`) - Switch to the script editor view
*   `grid` (alias: `scene`) - Switch to the scene grid view
*   `options` (alias: `settings`) - Switch to the options view
*   `devices` (alias: `devs`) - Switch to the connected devices view
*   `logs` - Switch to the application logs view
*   `files` (aliases: `projects`, `save`, `load`) - Switch to the save/load projects view
*   `help` (aliases: `?`, `docs`) - Switch to the help view

## Network / Client

*   `setname [<name>]` (alias: `name`) - Set username (e.g., 'setname BuboBubo')
*   `chat [<message>]` (alias: `say`) - Send chat message to other peers (e.g., 'chat Hello how are you?')

## Clock / Transport

*   `tempo [<bpm> [now|end|<beat>]]` (aliases: `t`, `bpm`) - Set tempo (e.g., 'tempo 120', 20-999 BPM)
*   `quantum [<beats>]` - Set Link quantum (e.g., 'quantum 4', >0 <=16)
*   `play [[now|end|<beat>]]` - Start the transport
*   `stop` (alias: `pause`) `[[now|end|<beat>]]` - Stop the transport

## Project Management

*   `save [<name>]` - Save current project state. If no name, uses current or prompts.
*   `load [<name> [now|end|<beat>]]` - Load a project state.

## Editor

*   `mode [normal|vim]` - Switch editor keymap mode (e.g., 'mode vim')

## Scene / Line Control

*   `scenelength [<length> [now|end|<beat>]]` (alias: `sl`) - Set scene length
*   `linelength [<line> <len|scene> [now|end|<beat>]]` (alias: `ll`) - Set line length (line is 1-indexed)
*   `linespeed [<line> <factor> [now|end|<beat>]]` (alias: `ls`) - Set line speed factor (line is 1-indexed)

Unknown commands will be logged as errors.
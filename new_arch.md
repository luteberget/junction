# New architecture

## MODEL (MVVM "inner" model)
No derived information.  Only editable by user, and only user-editable part.

0. View  (Move to derived?)
  a. selection
  b. command builder
  c. windows? 
  d. graph/instant: scenario + time
  e. synthesis builder)
1. infrastructure
2. interlocking options
3. scenarios (vehicles?)

These are stored in im::Vector (persistent vector) so that 
undo/redo is solved with snapshots.

## DERIVED / CALC (MVVM "outer" model)
All sorts of derived information, calculated by background processes.

Currently running jobs and finished jobs with stats are in a list
for special "background jobs" menu in GUI.

1. Schematic. (infrastructure -> schematic)
2. Interlocking. (infrastructure + interl. opts. -> interlocking)
3. Dispatches. (if + is + scenarios -> dispatches + graph/instant)
4. Synthesized.

## APP (MVVM view model)
Wraps model and derived. Allows read-only access to both.
Allows *integrate* function which submits a command.
Commands need to invalidate and reconstruct derived information. How?

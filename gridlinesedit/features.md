# glrail reboot juni 2019


## Infrastructure document model
Topology = discrete-coord lines 
           + specific nodes' (specified by coord) properties 
               (lost when deleted or connection-degree has changed)
           + optional length info
Objects = coord+angle, optional mileage, optional "pos" (?),
          function (TODO multiple functions, customize drawing, name/id?)

TODO: (1) polygon areas? (2) delimited areas? (3) track properties


(derives node types from line pieces,
 derives km / pos from graphical positions unless specified,
 dervies DGraph for schematic+customdata+deriveroutes+dispatch)

## Infrastructure editor

 * Draw tracks
 * Place objects
 * Select lines / objects
   - Erase lines / objects
   - Move lines+nodes / objects
   - Context menu on selection
     a. Modify nodes (node data)
     b. Modify objects (object menu)
     c. Lengths on tracks
 * Scroll (wheel and CTRL-drag)
 * Copy/paste (copy=relative to cursor posision?)


## Static interlocking model

Derived from 

## Static interlocking editor

 * Route criteria? (or Customdata scripts)
 * Interlocking window:
   - List (recognized) interlocking data and hover-to-show



## Dispatch model

Vehicles
Dispatch list of timed events 
Currently selected dispatch (opens timeline window)

## Dispatch editor

 * Edit vehicles in separate window/menu?
 * Add new / select between history/dispatch/scenario
 * Timeline view of currently active dispatch.
   - Extends context menu on selection:
     a. border: start train here
     b. signal: train route from here // overlap swing here?
   - Extends overdrawn view layer (with tooltips)
     a. train positions
     b. switch and detection section state



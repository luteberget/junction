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

 *  x  Draw tracks
 * (?) Place objects
 *  x  Select lines / objects
   -  x  Erase lines / objects
   -  x  Move lines+nodes / objects
   -  x Context menu on selection
     a.  x  Modify nodes (node data)
     b. ( ) Modify objects (object menu)
     c. ( ) Lengths on tracks
 *  x  Scroll (wheel and CTRL-drag)
 * Copy/paste (copy=relative to cursor posision?)


## Static interlocking model


language: 
 - assignment/equality
 - objects of type, objects in sets
   types are a fixed set, uppercase. 
   entities, path, area, and sets 
 - set builder, builds Lua-tables-like obj
    1. base set, can 
    2. "nested" product of bindings and filters
 - filters (membership, equality)
 - operations (of many arities) 
 - tuples (ordered, and/or maps?)

types are enumerable and  non-enumerable types.
sets are enumerable types.

toplevel sets/types: Location (track+pos) (not enumerable)
		     Entity contains all entities (enumerable)
                     Path (not enumerable)
		     Area (not enumerable)

predefined sets: each feature ("type") of entities, s.a. MainSig, AxleCounter, etc.

end = MainSig \union BufferStop
detectionSections = delimitedAreas AxleCounter
routePaths = { (start=s, end=e, path=p) | MainSig s <- Entity,
				          Path p, Entity e <- directedNeighbors s end }
routes = { *routePath*, 
	    sections = { a <- detectionSections | intersects a routePath.path }
         | routePath <- routePaths }


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



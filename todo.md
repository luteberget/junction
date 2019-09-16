# Junction TODOs

## cleanup
* x refactor modules (ui, model, analysis)
* code comments documentation
* end user documentation (README.md)
* web page
* user test
* rewrite viewport/canvas plotting (canvas object with wrapped addLine, etc., functions)
* wrap all unsafe / imgui

## minor
* x  rename to junction
* x  hotkeys (save,load)
* x save/load config / colors
* x better color defaults
* change fonts+size
* save/load with extension .junc

## infrastructure model + sim
* more objects
  * derailers?
  * X pre-signalling (editor + simulator)
  * atc? or something more low level
         or only rep.balise
  * ertms?
  * train starting velocity for concrete dispatch
  * velocity signs / restrictions

* mileage reversed detection / fix
* X undo classification (avoid excessive undos)
* track lengths
* mileages on nodes / objects
* pos on objects
* specify signal sight distance, warn if not attainable because of facing switches
* gradient (radius?)

* X rename node references in model when "extending" boundary node
* X rename object references in model when moving objects.

## interlocking
* inspectable interlocking
* configurable interlocking
* datalog-based interlocking?
* overlaps
* flank protection

## dispatch
* X better dispatch representation
* X edit dispatch

* auto-dispatch/movements/constraints-based-train
 * X plan datastructure
 * X add new plan from infrastructure + dispatch menu
 * X add new train, show trains as rows
 * X add new visits to train from infrastructure context menu
 * X rearrange visits, merge visits, split visits
 * X add constraints
 * X topo sort view by constraints
 * X draw constraints
 * X bug: VisitKey invalidated by moving visit from one train to another
 * X delete constraints
 *   delete trains
 * X delete plans and dispatches
 * X rename trains, dispatches, plans
 *   disallow bad constraints, drop non-working constraints when rearranging visits
 *   highlight visits location in infrastructure
 * X visit types: boundary vs. middle
 *   dwell time
 *   constraint maximum time diff

* X gui for dispatch/auto-dispatch

*   copy autodispatch as manual

## railml

* import railml 2.x 
  * schematic config
  * X  convert tracks
  *    convert objects

* import railml 2.x nor?
* import railml 3
* export railml 2.x
* export railml 2.x nor?
* export railml 3

## schematic
* auto-layout whole model
* auto-layout selection
* with given mileages / positions / fixed symbols

## testing

* code tests
* gui tests, including screenshots?

## synthesis

* X whole-station proof of concept
* suggestions proof of concept
* limit to selection

## static analysis

* datalog properties in errors window


## bugs
1. can undo creation of manual dispatch, but still displayed and crashes when adding new commands.

# file format

* defined, versioned file format 
* how to handle backwards compatiblity when using json/cbor serialization?





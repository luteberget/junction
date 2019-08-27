# immuted TODOs

## cleanup
* refactor modules (ui, model, analysis)
* code comments documentation
* end user documentation (README.md)
* web page
* user test
* rewrite viewport/canvas plotting (canvas object with wrapped addLine, etc., functions)

## minor
* x  rename to junction
* x  hotkeys (save,load)
* save/load config / colors
* better color defaults
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

* mileage reversed detection / fix
* X undo classification (avoid excessive undos)
* track lengths
* velocity signs / restrictions
* mileages on nodes / objects
* pos on objects
* specify signal sight distance, warn if not attainable because of facing switches
* gradient (radius?)

## interlocking
* inspectable interlocking
* configurable interlocking
* datalog-based interlocking?
* overlaps
* flank protection

## dispatch
* better dispatch representation
* edit dispatch
* auto-dispatch/movements/constraints-based-train

* gui for dispatch/auto-dispatch

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

* whole-station proof of concept
* suggestions proof of concept
* limit to selection

## static analysis

* datalog properties in errors window


## bugs
1. can undo creation of manual dispatch, but still displayed and crashes when adding new commands.

# file format

* defined, versioned file format 
* how to handle backwards compatiblity when using json/cbor serialization?





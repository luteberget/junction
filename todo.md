# Junction TODOs

## cleanup
* x refactor modules (ui, model, analysis)
* x end user documentation (README.md)
* x web page

* code comments documentation
* user test
* rewrite viewport/canvas plotting (canvas object with wrapped addLine, etc., functions)
* wrap all unsafe / imgui

## minor
* x  rename to junction
* x  hotkeys (save,load)
* x save/load config / colors
* x better color defaults
*   change fonts+size
*   save/load with extension .junc

*   view (center/zoom) better preserved when opening/closing dispatch
*   polyline trains (and tracks, and tvd sections?)

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

*   rename open end -> model boundary

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
 * x delete trains
 * X delete plans and dispatches
 * X rename trains, dispatches, plans
 *   disallow bad constraints, drop non-working constraints when rearranging visits
 * x highlight visits location in infrastructure
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
* X can undo creation of manual dispatch, but still displayed and crashes when adding new commands.
*   synthesis with multiple properties does not work?


# file format

* defined, versioned file format 
* how to handle backwards compatiblity when using json/cbor serialization?




# issues found in user test 2019-08-27

TOP pri
 * planning: visits to signals etc. is not implemented
 * close popup menus with escape key
 * sort dispatches from planning so they are stable through minor changes
 * add constraint -> then click a location fails (because the location is a button?)
 * Show Error message if loading file fails (make vec of error messages)
 * Synthesis example failed (remove detectors may be at fault)
 * a delete all buttons (Edit menu bar)
 * Minimize window crashed application

MIDDLE pri
 * user wants to pan view while the infrastructure context menu is open
 * the box size is not the actual grip size in commands in dispatch view
 * when highlighting a command in the dispatch, higlight not only the 
   signal but the whole route
 * the name of a new vehicle is the same as the default vehicle
 * the add vehicle button at the bottom of the vehicles window may give the impression
   that clicking it is necessary to save changes made in the window
 * the dispatch dropdown menu should close when clicking (add auto) or (add manual)
 * playlist of dipatches (run through all dispatches)
 * planning: locations look like buttons, but they are not
 * plan: t1: a-> b->, t2: b<-, a<- with constraints only on the last two, force a crossing, 
   but they probably should not. 
 * Save as type: \*.junc


LOW pri
 * draw trains as polylines
 * (maybe) when hovering a signal in the infrastructure, highlight any routes starting
   from that signal 


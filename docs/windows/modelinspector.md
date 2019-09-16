---
title: Model inspector
parent: Tool windows
nav_order: 5
---


The model inspector shows you a tree representation of the analysis model 
that is being edited, and also all of the derived information about
the model that is used for dispatching.

The model itself consists of:

 * Line segments representing tracks.
 * Node properties, overriding the defaults.
 * Objects locations and functions (main signal, detector, etc.)
 * Vehicle type specifications.
 * Dispatch specifications.
 * Plan specifications.

The analysis output consists of:

 * Topology: the inferred track and node types gathered from the line segments.
 * DGraph: the railway network double-node graph used for simulation.
 * Interlocking: elementary routes gathered from consecutive main signals and the
   detection sections that the train must pass over to get from the begin signal 
   to the end signal.
 * Dispatches: one simulation output for each manual dispatch, providing the
   timeline used for the dispatch diagram view. For each plan specification
   (auto dispatch), there is a (possibly empty) set of simulation outputs.
   

![Model inspector window](imgs/viewdata_1.png)

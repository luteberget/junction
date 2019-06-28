# Useful concepts for producing interlocking specifications

0. dgraph representation + "table" structure (underlying route model)

struct DGraph { node-a :Node, node-b :Node }
struct Location { node :Node } // DNode + direction


emit-table()
emit-route { start = x, end = x, ... }

objects: dnode, partnode/location (partnode = dnode+dir), 
         path, area.

1. building blocks for imperative specification


2. building blocks for declarative specification

a. next-relation on filtered graph

let mainsignal = function(x) do return x.type == signal and x.function == "main" end
let mainsig = function(a,b,d) do return a.is(mainsignal) and b.is(mainsignal)
neighbors(mainsig)


3. train route / shunting route

custom language:
routes := { Route(a,b) | a <- model.get(mainsignal), 
			 b <- a.next(mainsignal) }

moonscript:
routes = [ { start = a, end = b, path = p } 
           for a in model.get(mainsignal) 
           for (b,p) in a.next(mainsignal+samedir(a.dir)) ]

for r in routes {
  r.sections = { sec for sec in il:section(model) when sec.intersect(path) }
  r.facingswitches = { sw for sw in model:filter(switch) 
                       when path.contains(sw) and sw. }
}


emit-table(route)
for r in routes: emit-route(r) 
  -- tries to interpret r table into Route struct on Rust side.
  -- produces warning for unknown fields?



4. repl and custom syntax?

s1 = model.get(mainsignal).first()

s1









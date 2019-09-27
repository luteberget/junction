Stier som dekker alle sekvenser av sporgeometrisegmenter som et tog kan oppleve.

Denne analysen er gjort på bakgrunn av en antagelse om at kontroll av sporgeometri 
ikke kan gjøres per segment eller med et fast antall påfølgende segmenter, fordi 
sekvensen av sporgeometrisegmenter er avhengig av sporvekselstillinger, dvs. 
hvilken sti toget tar (path dependent).

Brute force er å ta alle mulige stier som finnes i infrastrukturen. Dette
gir en worst-case asymptotisk kjøretid på b\*2^n, hvor b er antall modellgrenser
og n er det høyeste antall sporveksler som kan være motrettet på samme kjøretur.

På typiske infrastrukturer så vil de fleste av disse stiene overlappe med mange
andre stier, og mye av jobben blir unødvendig for kontroll av sporgeometri. 
Vi kan begrense sti-avhengigheten (path dependency) til en maks-lengde slik at
togets stivalg etter å ha kjørt denne makslengden ikke lenger påvirker kontroll
av sporgeometri. Med en slik begrenset sti-avhengighet med lengde l vil
worst-case asymptotisk kjøretid være b\*n\*2^m, hvor b er antall modellgrenser,
n er antall sporveksler, og m er antall sporveksler som kan være motrettet
på samme kjøretur OG ligger innenfor l meter fra hverandre. Typiske
jernbaneanlegg har m som ligger på 2-4, og man unngår da helt effekten av 
eksponensiell kjøretid når infrastrukturen blir stor.


Algoritmen er rett-fram: gjør dybde-først søk og ta vare på hele stien, men 
når stien deler seg til to stier, behold den ene som den er og behold kun 
halen av den andre (hale-lengde l). Visited-settet inneholder kun haler av 
stier, og søket avsluttes når halen av gjeldende sti er i visited-settet.
Pseudo-kode:

```rust
type Edge = (Node,Node,double); // Two nodes and the length between them.
type Path = vector<Edge>; // List of edges

vector<Path> paths(Infrastructure inf, double path_length_equality_margin) {
   visited = new set;
   output = new vector;
   stack = new stack;
   for each boundary {
      add the first edge from boundary to stack;
   }
   while current_path = stack.pop() {
      if path_length of current_path >= path_length_equality_margin {
         tail = current_path shortened to path_length_equality_margin from the end;
         if visited contains tail {
            add current_path to output;
            continue;
         } else {
            insert tail in visited;
         }
      }

      // other_side is the opposite node in the double node graph
      current_node = opposite dgraph node from the last node if current_path;

      switch on edges from current_node {
         case Single(target, length): // non-branching edge
            current_path.push(edge from current_node to target, length)
            push current_path to stack;
         case Switch(left_edge, right_edge): // branching edge
            new_path = current_path
            current_path.push(edge from current_node to left_edge.target, left_edge.length);
            new_path.push(edge from current_node to right_edge.target, right_edge.length);
            push current_path and new_path to stack;
         default: // Either a dead-end or a model boundary
            add current_path to output;
      }
   }
}

```

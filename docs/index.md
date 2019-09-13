---
title: Overview
---

![Junction logo](imgs/logo1.png)

[⇓ Download (win/linux/macos)](https://github.com/luteberget/junction/releases/latest){: .btn .btn-green } [Repository](http://github.com/luteberget/junction/){: .btn .btn-outline }


Junction is a railway operations analysis tool for small-scale infrastructure,
such as construction projects. Its main features are:

 * Quickly build the **infrastructure** of tracks and signaling equipment by 
   drawing lines on a grid. Switches and crossings geometry and topology
   are automatically inferred from the visual drawing.

   ![Inf1](imgs/inf_draw_1.png) ![Inf2](imgs/inf_draw_2.png)

   See [Infrastructure](infrastructure.md).

 * **Dispatch** individual trains using train routes by pointing to the starting location
   of a train route and selecting a route from the menu.

   ![Inf1](imgs/dispatch_2.png)

   See [Dispatching](dispatch.md).

 * Build **plans** representing train operations such as crossing, overtaking, train frequency, etc.,
   and get a list of possible dispatch patterns that solve.
   When you continue adjusting the infrastructure, the plans will be updated and
   you can check at a glance that operations are still working.

   ![Inf1](imgs/autodispatch_1.png)

   See [Planning](planning.md).


![Overview](imgs/ss_overview.png)






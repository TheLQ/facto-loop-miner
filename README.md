# LoopMiner - Factorio Map Generator Mod

Mine every ore patch within hundreds of chunks with a massive generated rail loop network.
Jump start your base with effectively unlimited throughput resource collection
on base game vanilla-like maps.

!["Large random map of resources, with rails between them and center box"](history/bad-pathing6-2.png)

## Why

A point2point rail network of loops without intersections avoids bottleneck problems in other designs at scale.
Cost is significant, repetitive spaghetti building time.

LoopMiner generates megabase-scale and stress-test-scale resource collection rail networks.
Capable of mining every patch within at least 1,000 chunk diameter and routing to a central base.
Millions of ore per minute can be delivered

Puzzle is building a base capable of processing millions of ore per minute.
best network design parameters suitable for base,
megabase stress testing the Factorio engine,
and benchmarking designs.
Internally, puzzle is optimal path finding, optimal outpost design, and performance.

* Rail Loop for each connection between Miner > Base, Miner > Aggregation, Aggregation > Base
    * Dedicated rail eliminates late-game contention problems on shared rail (Best Throughput).
      Intersections without bottlenecks is difficult.
    * Dedicated rail avoids more complex concepts like buffer yards up to LTN, Logistic Train Network
    * A loop of unintersected rail has the simplest game pathing (Best UPS)
* Mining outposts
    * Drill > Train, Direct Insertion for lowest handling costs (Best UPS)
    * Fixed loading and optionally unloading train timers to avoid holding up train from empty drill patches and unequal
      unloading
* Optional aggregation outposts
    * Aggregation outposts combine multiple <3x wagon mining trains into a single significantly longer train
    * Longer trains reduce effect of _train swap delay_ (Best Throughput) and simplify routing on very large maps
    * Increases handling cost
* Optional smelt-on-site mining outposts
    * Drill > Smelter > Train, Direct Insertion roughly halves handling costs smelting elsewhere (Best UPS)
    * Doubles amount of ore patches required to replace ore inaccessible under smelters
* Design testing and benchmarking
    * Supports replacing existing networks to iterate on designs. Saves hours changing manually laid designs. 

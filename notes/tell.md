*Mining 100 patches with dedicated rail. 250x250 chunks.*

I've spent several months writing a base generator mod.
It creates massive rail loop networks between every ore patch within hundreds of chunks,
feeding a massive megabase. Then turned into a Factorio generator library in Rust.

*Core concept:* A dedicated loop of rail, point 2 point without intersections, has the highest throughput
yet simplest engine pathing. No 8x8 rail intersections or fancy LTN buffering needed.
However manual placing is very time intensive at scale.

This pathfinds rail loops from 100s of mines to the base.
Builds the mine (1+ ore patches) drills, belts, inserters, electrics, wall, and defense.
Builds base unload inserters and belts.
Builds the

Goal is to autogenerate one of the largest megabases with only default entities (no bobs mods).
Including mining, rail loading, rail offloading, belts, and smelting.

Goal is very ambitious, creating a perfect Factorio map framework was hard enough,
so I'm releasing this WIP map now because Factorio 2 will be released soon with new overpass rail mechanics,
making my complicated pathing algorithm unnessesary.
Note even one of the FFF posts mentioned rail was very complciated.

16,211 lines of rust, 300,000 entities,

Engine details (can skip)
==

Started with long bikeshedding a custom Surface implementation.

----


For a sense of scale, this is a mine in game. Back to this image, that mine is this tiny blue blob

The high level puzzle is this:

* draw a line from the base to each mine
* with the fewest turns
* without intersecting
* with the shortest route

Around 15k lines

Challenges
==

You have to define these concepts. A map, tiles on a map (the floor), entities on the map (which may cover tiles), a
patch. Basically a light game core.

You have to store and process a 32gb file 64k x 64k = 4.2 billion entries. mmap, io_uring, cloning an mmap in 50ms vs
vec.clone() in 2.5 seconds

Line is actually a series of rails chunks or steps. There's 2 side by side, there's 64 positions to check (8 rails * 4
pixels * 2 dual). Turn is more complicated sweeping turn.

You have to pathfind, which is a whole topic of optimal algorithm, trying multiple combinations (9! is 362,000)

===


Overview
===

*Mining 144 patches with dedicated rail each. 250x250 chunks.*

For the past year I've worked on a base generator mod.
Using only base game entities (no bobs mods etc).
Goal is highest processed ore/minute and one of the largest by area megabases.

*Core concept:* A dedicated loop of rail, point 2 point without intersections, has the highest throughput
and simplest pathing. No 8x8 rail intersections or fancy LTN buffering needed.
However manual placing is very time intensive.

Additionally, generating the best mine. Loading and unloading trains using
belts instead of bots.

This is version 1 mod, released before Factorio 2 comes out completely
changing rail mechanics and sort of obsoleteing the need for such a complex mod.


Details
==

This became an interesting computer science problem.
And one of the hardest projects I've ever done.
Entire University PhD Dissertations are dedicated to optimal pathfinding.

For sanity I wrote in Rust instead of Lua.
The cost is implementing a mini Factorio game engine.

Rail wsa very complex to implement.

Attempt to mine every ore patch in a 250 chunk square. 





  




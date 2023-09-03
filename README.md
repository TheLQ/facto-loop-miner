# LoopMiner - Factorio Map Generator Mod

Generate massive rail loop networks between mining outposts and bases.
Provide effectively unlimited throughput ore collection
on _vanilla_ maps.

Train loops avoid bottleneck problems with other designs at scale. 
Cost is enormous, repetitive building time. 
Instead automation easily solves it for us.

`loop-miner-exporter` Factorio mod collects map data from a save.
`loop-miner` running locally generates loop network from map data.
Output is a standalone mod creating
network via Lua API. Mod works on any save with same map seed and base generation settings. 
Mods may be disabled after.

## Automatic Rail Network

Scaled up application of known megabase best practices

* Dedicated rail eliminates late-game contention problems on shared rail. (Best Throughput) 
Intersections without bottlenecks is difficult. 
* Dedicated rail avoids complexity from buffer yards up to LTN, Logistic Train Network 
* Continuous rail has simplest game pathing (best UPS)
* Tuned train count avoids over production (Lower entity overhead)
* Fixed ore loading timer (avoid trains wasting time without all drills feeding)
* Pathfinds train to edge of base. 
  * `Avoid Water` default on. Limits available density from difficult pathfinding
  * Could be optimized
* Configurable loop design when 3+ rail loops are required. Like big ore patches. 
  * `Nested Loops` works only with logistic networks.
    Does not work with `Direct Insertion` or `Train 2 Train` consumers
  * `Single` adds 2nd rail over ore to maintain loop

## Unlimited Throughput Network

Ideally, a resource aggregator provides a continuous queue of trains filled with ore 
to deliver to the consumer furnace/assembly train stop.
Train unloads never pausing from filled destination/box.
Factory consumes resource fast enough to never block train.

**Treat ore as a fixed unlimited input tuned by number of Cargo Wagons.**

Unlimited comes from the 
[effectively unlimited map size](https://wiki.factorio.com/Map_generator#Maximum_map_size_and_used_memory) 
to mine. 

### Parameters

Determines resource requirements. Drives the necessary
aggregation/furnace outpost count >
drill count >
mining outpost count

* Per-Ore options
  * `Wagon Count` - Each wagon adds X,xxx ore/minute. 
  More wagons increase density and reduce train change delay impact.
  Too many wagons are difficult to unload and pathfind.
  Estimate 10-50.
  * `Train Count` - Each train has configured wagon count and sufficient engines.
  Estimate 5-20.
* Base Aggregator options
* `Use Aggregator Outposts` with it's options
* `Use Furnace Outposts` with it's options
* Per-ore `Furnace Outposts Percent` if both Aggregator and Furnace are enabled.
Aggregator Outposts deliver ore to base for certain recipes. 
* Per aggregation and base types, Station Design relative to delivery edge
  * `Parallel` stacks stations. Optimal for many wagon, train 2 logistic 2 train.
  * `Perpendicular` puts rail side by side. 
  Optimal for short wagon trains.
  Not for long trains because of logistic bot travel distance.

Over mining wastes UPS and increases map size. Esttmate with known 60 UPS @ X,XXX SPM limits.

This [charts](https://wiki.factorio.com/Map_generator#Charting_(removing_fog_of_war)) 
a significant amount of map chunks

Unless for Benchmarks and record setting, see section below 

### Base Aggregator

Distributes resources at edge of base. Player collects ore for processing

* Mandatory Options
  * `Base Size` - X and Y sizes in chunks.
  Delivery rails pathfind to border.
  Estimate 40(?) or from your saves.
  * `Base Center` - X and Y Center. Default 0x0 where ore is least.    
* Options
  * `One Resource per Side` routes different resources to different sides.
    Minimizes bot travel distance. Complicates pathfinding.
* Designs
  * `Train 2 Logistic 2 Train` generates input and output trains.
  `Output Wagons` configures train length. 
  Player DIY connects unterminated output rail to factory.  
  * `Train 2 Logistic` lets player DIY ore loading stations
  * `Unterminated` lets player DIY everything past the base edge.
  Useful for Progression scenarios.
    * Deleting pregenerated stations simulates this as needed. 
    Useful for already unlimited throughput loops 


### Mining Outposts

Places drills on ore patches. Create output train.

* `Best Drill 2 Train` - Direct Insertion (best UPS, fewest entities). 
Default minimum 8 drills over full ore going to 2 wagons are required to add lane.
Missed ore from fitting a rectangle inside a jagged shape are ignored as unproductive.
Gain is optimal rail solver and in-game train mechanics.
Cost is increased outposts to meet demand.
  * `Drill Effect Tiles` 1-25 - Tiles that must have ore. 
  Below threshold drill isn't created.
  Lower thresholds may increase drill or train lane count.
  Default 25 (5x5 effect size) focuses on plentiful tiles closer to the center.
  * `Minimum Drills per Wagon` - Default 4. 
  * `Minimum Wagons` - Default 2.
  * `Drill Minimum Ore` >= 1. Amount of ore in 5x5 effect area. Default 1.
  Total available ore less from overlapping effect areas. 
* `Logistic Drill 2 Train` Full Coverage Drill into logistic boxes.
Output loop next to ore patch. Short 1-3(?) wagons trains for
* `Fixed Train Load Time` - Adds to schedule "OR X seconds elapsed".
Elapsed is how long all connected drills should take to fill wagon.
Default enabled to ensure empty drills don't block train. 
* `Fixed Train Store Time` - Same as above for inserters. 
Default enabled for better aggregation. 
Cost affects undersized Player factories with unnecessary train traffic



Direct Insertion Drill Thresholds tuning

* Higher thresholds push away outposts to more plentiful patches. 
May increase patch count to meet demand. 
* Lower thresholds may create non-full wagon loads (assuming timeout mode) that waste UPS.  

### Aggregator Outposts

Route miner loops to nearby aggregation outposts. 
Transfer ore from shorter drill-optimized trains 
to a single longer consumer-optimized train.

* 5+:1 aggregation into 20+ wagon trains vastly increases destination's density
* `Unlimited Logistic Train`
  * `Wagon Count` - Scale parameter
  * Sufficient input miner loops smooth over intermittent delivery
  * Generates Roboport farm
  * 1x unlimited output loop suitable for direct use in base factory
* (Abandoned, what about the loop back???) `Direct Insertion`
  * Trains meet on alternating sides going down train 
  to give sufficient turning room
  * Con: Slowest feeder (eg ore patch empty, train change) blocks train waiting for full.
  Fixed time means generally delivering less than full wagons.
  * Currently unclear advantages

### Furnace Outposts

Automate smelting at more useful outposts.
Create dedicated smelter outposts or smelt-on-site mining outposts

Built in designs

* `Drill 2 Furnace 2 Train` smelts ore on top of patch via Direct Insertion (Best UPS).
Cost is less than half drillable ore per patch.
Extends ore patch solver with additional options.
* `Train 2 Logistic Furnace 2 Train` for bot powered dedicated smelter outpost.  
Like an Aggregator Outpost with furnaces.
Takes input only from multiple miners.
Output rail runs through center at full throughput.
  * `Wagon Count` - Scale parameter 
* (Abandoned) `Drill 2 Train 2 Furnace 2 Train` for Direct Insertion dedicated smelter outpost.
  * Direct insertion miners output short trains. 
  So we generate a skinny wide outpost outputting a gigantic parallel loop array.
  To maintain direct insertion no aggregation is available.
  * Logistic-powered input from miner outpost or aggregator defeats the point of direct insertion.
  * Unclear advantages over current designs

## Reconfigure Mid-Game

Supports deconstructing existing network and constructing new one. Useful for tuning

Reads the existing build mod, replaces create with destroy, appends the new network after.

* If player base expands into loop-miner controlled area
  * `Destroy Existing` removes conflicting entities
  * `Stop` - On conflict teleports player next to entity. Run /unlock to continue.

## Records

* X million Ore per minute

## Base Generation

Generate chunks of factory with configurable parameters

Base Generation is a separate product but very related. 

Available design parameters

* Production unit is the Micro-Factory. Creates exactly 1 product
* Buildings
  * `0 Beacon - Side by side` - 2 furnaces, boxes between them 
  * `6 Beacon - Side by side` - Lanes of beacons
  * `8 Beacon - Side by side` - Full Grid of Beacons
  * `12 Beacon` - Max Beacon
* Roboports
  * `1 Wide`
  * `3 Wide` - Introduce power polls for center rows
  * `2-4 Wide` - 2 wide roboports, with full grid cell size of 4 buildings
* Logistic Network bot count, evenly distributed among Roboports on init
* Connections
  * Each Micro-Factory supports being split in half for trains
  * Each Micro-Factory side supports a dependency/successor micro-factory next to it 
  with a loop encompassing them. Gives the optimal loop.
  * Remaining trains loop inside


### Benchmarking

Solving "optimal" quickly becomes NP-Hard.
Instead parameterize designs and run through a range,
gathering metrics at every step.

* Parameterize several designs and test ranges. 
* Create graphs.
  * UPS vs Bot count
  * Maximum bots needed
  * Gigantic bases taking 100 wagons vs multiple 20 wagon vs multiple 8 wagon.
    * UPS impact?
    * Throughput impact?
* Alarms
  * Boxes are 75% full
  * Boxes are not 100% full by train arriving
    * Don't think logistic networks work that fast
  * 0 bots available?
* Post on /r/Factorio and forum

Research areas include

* `Ore Per Minute` - Consumed ore per minute record
* `Unlimited Factory Template` - Isolated micro-factory test
  * 2 back to back trains on basic empty/filled schedule
  * Shortest possible rail loop that includes 2 station. 2nd station not inside a facto block
  * On 2nd station arrival event, immediately fill train. Train moves on

# How it works

1. Export map data from in-game map
2. Build a rough equivelent to LuaSurface
3. 

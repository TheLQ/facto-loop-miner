# LoopMiner - Factorio Map Generator Mod

Generate massive rail loop networks between mining outposts and bases.
Provide effectively unlimited throughput ore collection
on _vanilla_ maps.

Train loops avoid bottleneck problems with other designs at scale. 
Cost is enormous, repetitive building time. 
Instead automation solves it for us.

Included mod exports map resource data. Standalone `bin/loop-miner` generates loop
network from map data. Output is a standalone mod creating
network via Lua API for any save with same map seed and base generation settings. 
Mods can be disabled after.

## Automatic Rail Loops

Scaled up application of known megabase best practices

* Rail without intersections eliminates bottlenecks waiting for intersection to be free.
* Continuous rail has simplest game pathing (best UPS)
* Fixed ore loading timer (avoid trains wasting time without all drills feeding)
* Pathfinds train to edge of base. 
  * `Avoid Water` default on. Limits available density
* `3+ Lane Mode` for wide ore patches 
  * `Nested Loops` works only with logistic network consumers. 
  Does not work with `Direct Insertion` or `Train 2 Train` consumers
  * `Single` adds 2nd rail over ore to maintain loop 

## Unlimited Throughput Network

Ideally, resource aggregator provides a continuous queue of trains filled with ore 
deliver to the consumer furnace/assembly train stop.
Train unloads never pausing from filled destination/box.
Factory consumes resource fast enough to never block the train.

**Treat ore as a fixed unlimited input tuned by number of Cargo Wagons.**

Unlimited comes from the effectively unlimited map size.

### Base Aggregator

Delivers ore to edge of base. Player collects ore for processing

* `Train 2 Logistic 2 Train` fully generates the aggregator. 
2nd train is of configurable length.
Train is unterminated for Player to DIY to Factory.
* `Train 2 Logistic` lets player DIY create sized ore loading stations
* `Unterminated` for Basic Loop mode or Outposts. Useful for Progression scenarios.

### Aggregator Outposts

Route miner loops to nearby aggregation outposts. 
Transfer ore from shorter drill-optimized trains to longer consumer-optimized trains.

Logistic bots only 

* 5+:1 aggregation into 20+ wagon trains vastly increases consumer density
* Scaled to fill output train without pausing
* Optional unterminated base aggregator side for Player to deliver straight to factory
* Generates sufficient logistic network farm

### Mining Outposts

Ore Producer. 

* Direct insertion miner to train (best UPS, fewest entities)

### Furnace Outposts

Useful outposts.
Create dedicated smelter outposts or smelt-on-site mining outposts

Built in designs

* `Drill 2 Furnace 2 Train` smelts ore on top of the ore patch via Direct Insertion.  
Best UPS. 
Reduced ore patch area lowers throughput and total available ore.
Extends ore patch solver with additional options.
* `Train 2 Logistic Furnace 2 Train` for bot powered dedicated smelter outpost.  
Takes input from multiple miners or 1+ aggregators.
Output rail runs through center at full throughput.
See base generation for options

* `Train 2 Furnace 2 Train` makes an outpost
  consuming from one or multiple mining outposts
  direct inserting miner to train
  somehow aggregating...
  Abandoned, doesn't make sense compared to just logistic


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
* Alarms
  * Boxes are 75% full
  * Boxes are not 100% full by train arriving
    * Don't think logistic networks work that fast
  * 0 bots available?
* Post on /r/Factorio and forum

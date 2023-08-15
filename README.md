# LoopMiner - Factorio Map Generator Mod

Create massive rail loops between mining outposts and bases.
Provides effectively unlimited throughput ore collection
on _vanilla_ generated maps.

## Automatic Rail Loops

A practical implementation of megabase mining best practices on vanilla-generated maps

* Continuous loop trains without intersections have least pauses, simplest game pathing (best UPS)
* Direct insertion miner to train (best UPS)
* Automatic ore onload timer (avoid trains wasting time without all drills feeding)
* Pathfinds train to edge of base

## Unlimited Throughput Consumers

Ideally, a continuous queue of trains filled with ore wait at the consumer train stop.
Train unloads without pausing from filled boxes. 
Repeat.

Ore trains should never run dry. Treat ore as a fixed input tuned by number of Cargo Wagons.

## Aggregation Outposts

Optionally route miner loops to nearby aggregation outposts. 

* Transfers ore from shorter drill-optimized trains to longer consumer-optimized trains.
* 5:1 or more aggregation increases central base density 

## Base Generation

TODO: Apply loop concept to base internals. Define input and output resource tree, 
tilable 8x beaconed assembly machines. 

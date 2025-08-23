 # FX Simulator and Aggregator - fx_sim_agg

 `fx_sim_agg` simulates FX market data streams and aggregates them into a real-time book of buys and sells. It is written in Rust to allow concurrent, fast, memory-safe  programming without garbage collection pauses.

 - `main.rs` combines all the individual asynchronous market data streams from each liquidity provider into a single merged stream
 that yields values in the order they arrive from the source market data streams. Also initiates log4rs logging framework
 - `simulator.rs` generates simulated FX market data and sends the data as asynchronous market data streams
 - `aggregator.rs` updates and aggregates the asynchronous data streams into a real-time FX book of buys and sells
 - `lib.rs` various utilities used by the other modules

Configuration of the different liquidity providers is via an input config file:

![config.txt](resources/config.txt.png)


 "FIX" like market data is generated for the different liquidity providers and saved in a FIX.log:

![FIX.log](resources/FIX.log.png)

The aggregated FX book is updated and displayed in real-time as a ladder:

![FX_ladder](resources/FX_ladder.png)

**TODO** 
1. Real-time graphical display of the aggregated FX book
2. Real-time graphical display of generated FX data
3. Real-time trades from the aggregated FX book
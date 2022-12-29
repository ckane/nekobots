# Simple Rust Ecological Simulator

This is a rudimentary simulator of an ecosystem that I wrote as a small
project in Rust to help teach myself Rust, as well as demonstrate an example
of using it to develop something interesting.

## Demo GIF

```sh
nekobots -s 10 -t 50 -b 150 -f 3
```

![Demonstration GIF of Simulation](/img/demo.gif)

This simulator (right now) maintains the following entities:

1. "Nekobots" - animals inhabiting the environment and consuming what grows here
2. "Vegetattion" - plants that grow in the world - provide food to what consumes it
3. "Map" - ASCII world created from the dimensions of the terminal it runs in.
4. "Ticks" - World Clock that ticks by at a configurable rate

It is a colorized ASCII simulation using the
[crossterm](https://docs.rs/crossterm/latest/crossterm/index.html) library for
terminal output. This should make it compile & run on any UNIX or Windows systems.

Upon loading, a map will be created that takes up the entire dimensions of the
visible Terminal window. At this time, resizing the terminal while the simulation
is running will lead to undefined behavior (so don't do that, but it's a hard problem
to fix well). A percentage of the map will be populated with vegetation (configurable
via the command line). Next, the nekobots will be populated at random locations on the
map. The simulation will start immediately.

The nekobots will wander around aimlessly, until they get hungry (and will turn
orange/brown). Once hungry, if there is food within sight (configurable, again, via the
command line) they will move toward it, otherwise they will wander aimlessly. If they
are hungry and walk on food, they will eat it, repopulating their energy level (and no
longer be hungry).

If they deplete their energy level, they will die (turn red).

Once eaten, the vegetation will disappear and will regrow after a period of time (this
time is also configurable via the command line), in the same spot.

# Options

Here is an output of the supported command-line arguments:

```
Usage: nekobots [OPTIONS]

Options:
  -b, --bots <BOTS>          Number of bots to create [default: 10]
  -t, --tick-delay <MSEC>    Tick delay in msec (inverse of speed) [default: 250]
  -s, --sight <SQUARES>      Sight (how many squares ahead a bot can "see") [default: 10]
  -r, --regrow-time <TICKS>  Vegetation regrowth time (in ticks) [default: 100]
  -f, --food-prob <PERCENT>  Map vegetation probability (in percent) [default: 5]
  -h, --help                 Print help information
  -V, --version              Print version information
```


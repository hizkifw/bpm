# bpm - basic process manager

## How to use

Create a folder `bpm.d` next to the `bpm` executable. Place executable shell
scripts in the `bpm.d` folder. On startup, `bpm` will scan the folder, launch
all executable files, and makes sure they keep running. Standard output and
error will be captured and printed out to `bpm`'s standard output.

Processes that fork will not be monitored. `bpm` will only relaunch exited
processes if they exit with a non-success code (i.e. processes that exit with
code 0 will not be relaunched).

## Building

```sh
cargo build --release
```

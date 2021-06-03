# simple-backroll

Trying out the [Backroll](https://github.com/HouraiTeahouse/backroll-rs) networking library
with a very simple "game" with two moving squares.
This compiles with the current latest main branch in backroll-rs (commit 7d3a714fc5eec42804026d374aa4f8df29909f4c),
but it gives runtime errors.

## Running

To run two separate clients that connect to each other, run these commands in two separate terminals:

    cargo run -- 0   # "0" is the default and can be omitted
    cargo run -- 1

You can also run both clients in the same process, but that makes the log output harder to read:

    cargo run -- 0 1

You can specify the number of players with the `-n` argument, so for single-player, use:

    cargo run -- -n 1

Instead of running all clients in one process, create a more realistic simulation with the multi-client script. E.g. to run 3 client processes:

    ./multi-clients 3

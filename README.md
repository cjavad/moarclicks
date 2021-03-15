# moarclicks
Proof of concept click enhancer using Rust / MPSC

## --help

Too lazy to write a readme so take this CLAP generated output.

```
moarclicks 2.0.0
Javad S. <javadscript@gmx.com>
Does awesome things

USAGE:
    moarclicks.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --clicks <NUMBER>    Amount of clicks to prepend every click [default: 3]
        --maxD <NUMBER>      Max delay in MS [default: 500]
    -C, --cps <NUMBER>       Minimum activaction CPS for enhance to start [default: 2]
        --minD <NUMBER>      Min delay in MS [default: 100]
    -r, --rand <FLOAT>       Chance to randomize delay [default: 0.5]
```

# i3-autolayout
[![crates.io](https://img.shields.io/crates/v/i3-autolayout.svg)](https://crates.io/crates/i3-autolayout)


## Introduction
`i3-autolayout` is a simple service which helps keep a reasonable windows layout for your *i3* manager.

without i3-autolayout      |  with i3-autolayout
:-------------------------:|:-------------------------:
 ![DEMO GIF without autolayout](https://github.com/BiagioFesta/i3-autolayout/blob/main/img/i3-autolayout-without.gif) |  ![DEMO GIF with autolayout](https://github.com/BiagioFesta/i3-autolayout/blob/main/img/i3-autolayout-with.gif)

Without autolayout, you have to manually decide whether to split the windows horizontally or vertically. 

Instead, when `i3-autolayout` service is enabled, the split mode is automatically selected to better distribute the width and the height of windows uniformly. 

Of course, you can still perform a manual split: indeed, `i3-autolayout` aims to be as less invasive as possible.

`i3-autolayout` is written in *Rust* (programming language) for best performances and minimal system resource usage.


## Install

See the [installation guide](https://github.com/BiagioFesta/i3-autolayout/wiki#install).

## Usage

See the [usage guide](https://github.com/BiagioFesta/i3-autolayout/wiki/Usage).


## Alternatives

### [`i3-auto-layout`](https://github.com/chmln/i3-auto-layout)

I have discovered `i3-auto-layout` project only after I have created mine. 
So credit to that older project.

Some improvements over it:
 * Less overhead (dependencies and runtime).
   * `i3-autolayout` (*this project*) uses a simple single blocking thread runtime and simple "stdout" minimal logging (less CPU overhead).
   * `i3-autolayout` (*this project*) takes 2% of the virtual memory instead used by `i3-auto-layout` (*alternative project*).
 ```
    COMMAND      |     VIRT (kb)
  --------------------------------
 i3-autolayout   |   3423
 i3-auto-layout  |   140140
 ```
 * `i3-autolayout` (*this project*) checks the workspace and avoid horizontal splitting on monitor with a vertical configuration.
 * `i3-autolayout` (*this project*) provides an optional systemd unit file, if you prefer a more "service-oriented approach".

# i3-autolayout
[![crates.io](https://img.shields.io/crates/v/i3-autolayout.svg)](https://crates.io/crates/i3-autolayout)
[![Rust](https://github.com/BiagioFesta/i3-autolayout/actions/workflows/ci.yml/badge.svg)](https://github.com/BiagioFesta/i3-autolayout/actions/workflows/ci.yml)

## Introduction
`i3-autolayout` is a simple service which helps keep a reasonable windows layout for your *i3* manager.

without i3-autolayout      |  with i3-autolayout
:-------------------------:|:-------------------------:
 ![DEMO GIF without autolayout](https://github.com/BiagioFesta/i3-autolayout/blob/main/img/i3-autolayout-without.gif) |  ![DEMO GIF with autolayout](https://github.com/BiagioFesta/i3-autolayout/blob/main/img/i3-autolayout-with.gif)

Without autolayout, you have to manually decide whether to split the windows horizontally or vertically. 

Instead, when `i3-autolayout` service is enabled, the split mode is automatically selected to better distribute the width and the height of windows uniformly. 

Of course, you can still perform a manual split: indeed, `i3-autolayout` aims to be as less invasive as possible.

`i3-autolayout` is written in *Rust* (programming language) for best performances and minimal system resource usage.

## Key Features
 * Easy to install (see the installation guide).
 * [`tabmode`](https://github.com/BiagioFesta/i3-autolayout/wiki/Usage#tabmode): real tabbed layout with a single command.
 * Written in Rust. Minimum resources overhead.
 * Systemd Unit (if you like it).

## Install

See the [installation guide](https://github.com/BiagioFesta/i3-autolayout/wiki#install).

## Usage

See the [usage guide](https://github.com/BiagioFesta/i3-autolayout/wiki/Usage).

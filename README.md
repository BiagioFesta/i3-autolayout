# i3-autolayout

## Introduction
`i3-autolayout` is a simple service which helps keep a reasonable windows layout for your i3 manager.

without i3-autolayout      |  with i3-autolayout
:-------------------------:|:-------------------------:
 ![DEMO GIF without autolayout](https://github.com/BiagioFesta/i3-autolayout/blob/main/img/i3-autolayout-without.gif) |  ![DEMO GIF with autolayout](https://github.com/BiagioFesta/i3-autolayout/blob/main/img/i3-autolayout-with.gif)

Without autolayout, you have to manually decide whether to split the windows horizontally or vertically. 

Instead, when `i3-autolayout` service is enabled, the split mode is automatically selected to better distribute the width and the height of windows uniformly. 

Of course, you can still perform a manual split: indeed, `i3-autolayout` aims to be as less invasive as possible.

---

## Build & Usage

### Requirements

* [i3wm](https://i3wm.org/): of course, you need the i3 window manager as `i3-autolayout` is more "like a plugin for it".
  * If using Wayland [sway](https://swaywm.org/) is an i3 alternative and it should be compatible with this service.
* [Rust](https://www.rust-lang.org/) toolchain: for compiling this code base.
  * You can easily install rust via [`rustup`](https://rustup.rs/).
  
### Compiling

* Clone the repository:
```
git clone https://github.com/BiagioFesta/i3-autolayout.git
```

* Check out the project directory:
```
cd i3-autolayout
```

* Build the project:
```
cargo build --release
```

### Install
What you essentially need is just the binary of the service. 

By default, the binary can be found the directory `target/release/i3-autolayout` (no additional dependencies should be needed). 

You can copy it in a convenient place within your filesystem and run it as daemon (e.g., run at i3 startup with the `exec` command in your i3 configuration).

If you want, you can find a simple bash script in the project which installs the binary beside a systemd service unit file.

You need to specify the prefix where the binary and the systemd unit will be placed. For example:

```
sudo ./install.sh --prefix /usr
```

### Starting the service

If you have installed the systemd service unit, you can start the service within your i3 configuration.

Reload the list of installed units:
```
systemctl --user daemon-reload
```

Append the following statement to your i3 configuration file (i.e. `~/.config/i3/config`):
```
exec_always systemctl --user start i3-autolayout
```

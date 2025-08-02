<div align='center'>

<h2>HyprViz</h2>

**Fork** of [HyprGUI](https://github.com/MarkusVolk/hyprgui) — an unofficial GUI for configuring Hyprland, built with GTK4 and Rust. 🚀🦀<br>
Comes with a custom [hyprparser](https://crates.io/crates/hyprparser) for Hyprland's configuration file. (Rust btw) 🦀

![Preview](.github/preview.png)

</div>

## Installation

### Arch Linux
**Currently unavailable**

There are 2 different [AUR](https://aur.archlinux.org) packages available:

- [hyprviz](https://aur.archlinux.org/packages/hyprviz) - Latest release built from source
- [hyprviz-git](https://aur.archlinux.org/packages/hyprviz-git) - Latest development version built from source
- [hyprviz-bin](https://aur.archlinux.org/packages/hyprviz-bin) - Latest release in binary form

Install the preferred package with:
```bash
git clone https://aur.archlinux.org/<package>.git
cd <package>
makepkg -si
```

Or, if you're using an [AUR Helper](https://wiki.archlinux.org/title/AUR_helpers), it's even simpler (using [paru](https://github.com/Morganamilo/paru) as an example):
```bash
paru -S <package>
```

## Building from source
1. Install Rust (preferably `rustup`) through your distro's package or [the official script](https://www.rust-lang.org/tools/install)
2. Install `git`, `pango` and `gtk4`
3. Clone this repository:
`git clone https://github.com/timasoft/hyprviz.git && cd hyprviz`
4. Compile the app with `cargo build --release` or run it directly with `cargo run --release`

## TODO:
- [x] Improve value parser
- [x] Improve colour options
- [ ] Add default values
- [ ] Add bind section
- [ ] Add windowrule section
- [ ] Add other config sections from Hyprland
- [ ] Improve GUI

## Credits:
- [Nyx](https://github.com/MarkusVolk) - Implementing the parser, rest of the GUI, and maintaining the hyprgui project
- [Adam](https://github.com/adamperkowski) - Implementing the base GUI, maintaining the AUR packages and the project alongside Nyx
- [Vaxry](https://github.com/vaxerski) - Hyprland
- [rust-gtk](https://github.com/gtk-rs/gtk4-rs) - The GTK4 library
- [Hyprland](https://github.com/hyprwm/Hyprland) - The window manager

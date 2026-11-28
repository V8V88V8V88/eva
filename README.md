# Eva
<p align="center"><img src="data/eva.svg" alt="Eva icon" width="160"></p>

Contents
========
* [Introduction](#introduction)
* [Features](#features)
* [Keybindings](#keybindings)
* [Installing](#installing)
* [Building](#building)
## Introduction
Eva was created by Nathan Fisher. I picked it up after it stopped being updated, and keep it working with current GTK4 and Rust.

Eva is a [gemini protocol](https://gemini.circumlunar.space/) browser written in
[Rust](https://rust-lang.org) using the [gtk+](https://gtk-rs.org/) toolkit. Eva
is focused on delivering a simple yet powerful interface and clear, readable
and beautiful page rendering. It has no designs on being the most fully featured
Gemini browser, but instead wants to be the most polished native Gtk+ Gemini
client that stays out of the user's way.

Eva's gemtext rendering has been designed to be clean yet visually appealing,
using modern css styling to visually separate elements such as block quotes and
preformatted sections, using colored cards with rounded edges and box shadows.
The default color scheme has been chosen to provide great readability, while
giving the user a great deal of control over appearance.

Eva is currently alpha quality software and is under heavy development. Some
features are not yet implemented and there may be bugs. However, rest assured
that when Eva sees an official release it will be a polished user experience.
## Features
 - [x] tabbed interface
 - [x] bookmarks
 - [x] keyboard shortcuts
   - [x] user configurable
 - [x] user controlled styling
   - [x] user specified fonts
   - [x] user specified colors
 - [x] back-forward list
 - [x] history
 - [x] Gemini protocol
   - [x] Render gmi pages
   - [x] Display text
   - [x] Display images
   - [x] View gemtext source
   - [x] Tofu
   - [x] Client certificates
   - [x] Send user input
   - [x] Sensitive user input
   - [x] Downloads
   - [x] Save content
 - [ ] Titan protocol
 - [x] Gopher protocol
   - [x] Render gophermaps
   - [x] Display text
   - [x] Display images
   - [x] Downloads
   - [x] Save content
 - [x] Finger protocol
 - [x] Data url's
   - [x] Display text
   - [x] Render gemtext
   - [x] Display images
   - [x] Save content
 - [x] Spartan protocol
   - [x] Render all content supported by gemini
   - [x] Upload Spartan content

## Keybindings
> Note: See [KEYS.md](KEYS.md) for information on changing keybindings

| Key | Action |
| --- | --- |
| Ctrl/T | New tab |
| Ctrl/N | New window |
| Ctrl/W | Close tab |
| Ctrl/Q | Close window |
| Ctrl/R | Reload page |
| Alt/Home | Go to homepage |
| Alt/Left | Go back |
| Alt/Right | Go next |
| Ctrl/PageDown | Next tab |
| Ctrl/PageUp | Previous tab |
| Alt/[1-9] | nth tab |
| Ctrl/Shift/O | Open bookmarks |
| Ctrl/D | Bookmark page |
| Ctrl/H | Open History |
| Ctrl/S | Save Page |
| Ctrl/Shift/P | Open preferences |
| Ctrl/Shift/A | Open about dialog |

## Installing
### Arch Linux
Install the [eva-browser](https://aur.archlinux.org/packages/eva-browser) package
from the aur. Alternatively, beginning with the 0.3.0 release, releases on Codeberg
and Github include an Arch `PKGBUILD` file.
### FreeBSD
Beginning with the 0.3.0 release, there is a binary package for FreeBSD 13 and the
x86_64 architecture included in the releases on Codeberg and Github.
### All others
See the [Building](#building) section below.

## Building
```sh
# clone the source
git clone https://codeberg.org/jeang3nie/eva.git
cd eva
cargo build --release
# To gather all release files into target/dist
cargo xtask dist
```

# GemView
Contents
========
- [Introduction](#introduction)
- [Features](#features)
- [Usage](#usage)
## Introduction
GemView is a [gemini protocol](https://gemini.circumlunar.space/) browser widget
for gtk+ (version 4) implemented in Rust.
## Features
- [x] Browse and render gemini gemtext content
- [x] Display plain text over gemini
- [x] Display images over gemini
- [x] Display text and images from `data://` url's
- [x] Browse and render gopher maps, plain text and images over gopher
- [x] Display finger protocol content
- [x] Browse local files and directories via 'file://' url's
- [x] Spartan protocol support
- [x] Open http(s) links in a *normal* browser
- [x] User customizable fonts
- [x] User customizable colors (via CSS)
- [x] Back/forward list
- [ ] History

## Usage
```Yaml
[dependencies]
gemview = 0.2.0

[dependencies.gtk]
version = "~0.4"
package = "gtk4"
```
```Rust
use gemview::GemView;
use gtk::prelude::*;

let browser = GemView::default();
let scroller = gtk::builders::ScrolledWindowBuilder::new()
    .child(&browser)
    .hexpand(true)
    .vexpand(true)
    .build();
let window = gtk::builders::WindowBuilder::new()
    .child(&scroller)
    .title("GemView")
    .build()
window.show();
browser.visit("gemini://gemini.circumlunar.space");
```

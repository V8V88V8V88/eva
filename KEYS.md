## Defaults
As described in the project README, Eva comes with the following default
keybindings.
| Key | Action |
| --- | --- |
| Ctrl/T | New tab |
| Ctrl/Shift/E | View open tabs |
| Ctrl/L | Select the address bar |
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
| Ctrl/Shift/P | Open preferences |
| Ctrl/Shift/A | Open about dialog |

It is possible to configure these keybindings to suit one's own needs by creating
a *keys.toml* file. The location of this file should be in the same directory as
the program's configuration file, usually in `~/.config/eva`. This file is in
[toml](https://toml.io/en/) format and consists of a simple key/value store.

### Sample *keys.toml* file
```
[keys]
go_previous = "<primary>B"
go_next = "<primary>F"
open_bookmarks = "<primary><Shift>B"
```
The first field is the action name, while the value is the key combination
consisting of one or more modifiers and a key value. If the action name is
misspelled or does not refer to a valid action it will be ignored, while if the
keybinding specified does not parse to a valid keybinding the program will fall
back to it's default for that action.
### Actions
| Action name | Description |
| --- | --- |
| new_tab | Opens a new tab |
| tab_overview | Shows a grid of all open tabs |
| focus_address | Focuses the address bar and selects the address |
| close_tab | Closes the current tab |
| next_tab | Switches to the tab to the right or below the current tab |
| prev_tab | Switches to the tab to the left or above the current tab |
| tab1 | Switches to the first tab |
| tab2 | Switches to the second tab |
| tab3 | Switches to the third tab |
| tab4 | Switches to the fourth tab |
| tab5 | Switches to the fifth tab |
| tab6 | Switches to the sixth tab |
| tab7 | Switches to the seventh tab |
| tab8 | Switches to the eighth tab |
| tab9 | Switches to the ninth tab |
| reload | Reloads the current page |
| go_home | Navigates to the homepage in the current tab |
| go_previous | Navigates to the previous url in the current tab's history |
| go_next | Navigates to the next url in the current tab's history |
| new_window | Opens a new window |
| open_bookmarks | Opens the bookmarks page in the current tab |
| bookmark_page | Pops up the bookmark editor |
| open_history | View history |
| view_source | View the source of the current gemtext document |
| save_page | Save the raw source of the current document |
| open_prefs | Open the preferences dialog |
| open_about | Open the About dialog |
| quit | Close the current window |

### Modifier keys
| Common name | String for *keys.toml* |
| --- | --- |
| Control | &lt;primary&gt; |
| Alt | &lt;Alt&gt; |
| Shift | &lt;Shift&gt; |
| Super (Windows) | &lt;Super&gt; |

### Other key names
All alphabetic keys are simply the key value, in upper or lower case. Number keys
are their numerical value. The arrow keys are "Up", "Down", "Left", and "Right",
which the PgUp and PgDn keys will be "Page_Up" and "Page_Down". The gdk header
file [gdkkeysyms.h](https://gitlab.gnome.org/GNOME/gtk/-/blob/main/gdk/gdkkeysyms.h)
can be consulted for the full list, removing the "GDK_KEY_" portion of the string.

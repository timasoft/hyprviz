## Quirks

Dwindle splits are NOT PERMANENT. The split is determined dynamically with the
W/H ratio of the parent node. If W > H, it's side-by-side. If H > W, it's
top-and-bottom. You can make them permanent by enabling `preserve_split`.

## Bind Dispatchers

| dispatcher | description | params |
| --- | --- | --- |
| pseudo | toggles the given window's pseudo mode | left empty / `active` for current, or `window` for a specific window |

## Layout messages

Dispatcher `layoutmsg` params:

| param | description | args |
| --- | --- | --- |
| togglesplit | toggles the split (top/side) of the current window. `preserve_split` must be enabled for toggling to work. | none |
| swapsplit | swaps the two halves of the split of the current window. | none |
| preselect | A one-time override for the split direction. (valid for the next window to be opened, only works on tiled windows) | direction |
| movetoroot | moves the selected window (active window if unspecified) to the root of its workspace tree. The default behavior maximizes the window in its current subtree. If `unstable` is provided as the second argument, the window will be swapped with the other subtree instead. It is not possible to only provide the second argument, but `movetoroot active unstable` will achieve the same result. | [window, [ string ]] |

e.g.:

```ini
bind = SUPER, A, layoutmsg, preselect l
```

## Layer Rules

Some things in Wayland are not windows, but layers. That includes, for example:
app launchers, status bars, or wallpapers.

Those have specific rules, separate from windows. Their syntax is the exact same,
but they have different props and effects.

### Props

| Field | Argument | Description |
| -------------- | --------------- | --- |
| match:namespace | \[RegEx\] | namespace of the layer, check `hyprctl layers`. |

### Effects

| effect | argument | description |
| ---- | ----------- | --- |
| no_anim | \[on\] | Disables animations. |
| blur | \[on\] | Enables blur for the layer. |
| blur_popups | \[on\] | Enables blur for the popups. |
| ignore_alpha | \[a\] | Makes blur ignore pixels with opacity of `a` or lower. `a` is float value from `0` to `1`. `a = 0` if unspecified. |
| dim_around | \[on\] | Dims everything behind the layer. |
| xray | \[on\] | Sets the blur xray mode for a layer. `0` for off, `1` for on, `unset` for default. |
| animation | \[style\] | Allows you to set a specific animation style for this layer. |
| order | \[n\] | Sets the order relative to other layers. A higher `n` means closer to the edge of the monitor. Can be negative. `n = 0` if unspecified. |
| above_lock | \[0/1/2\] | If non-zero, renders the layer above the lockscreen when the session is locked. If set to `2`, you can interact with the layer on the lockscreen, otherwise it will only be rendered above it. |
| no_screen_share | \[on\] | Hides the layer from screen sharing by drawing a black rectangle over it. |

## Layer Rules

Some things in Wayland are not windows, but layers. That includes, for example:
app launchers, status bars, or wallpapers.

Those have specific rules, separate from windows:

```ini
layerrule = rule, namespace
# or
layerrule = rule, address
```

where `rule` is the rule and `namespace` is the namespace RegEx (find
namespaces in `hyprctl layers`) or `address` is an address in the form of
`address:0x[hex]`.

### Rules

| rule | description |
| ---- | ----------- |
| unset | Removes all layerRules previously set for a select namespace RegEx. Please note it has to match _exactly_. |
| noanim | Disables animations. |
| blur | Enables blur for the layer. |
| blurpopups | Enables blur for the popups. |
| ignorealpha \[a\] | Makes blur ignore pixels with opacity of `a` or lower. `a` is float value from `0` to `1`. `a = 0` if unspecified. |
| ignorezero | Makes blur ignore fully transparent pixels. Same as `ignorealpha 0`. |
| dimaround | Dims everything behind the layer. |
| xray \[on\] | Sets the blur xray mode for a layer. `0` for off, `1` for on, `unset` for default. |
| animation \[style\] | Allows you to set a specific animation style for this layer. |
| order \[n\] | Sets the order relative to other layers. A higher `n` means closer to the edge of the monitor. Can be negative. `n = 0` if unspecified. |
| abovelock \[interactable\] | Renders the layer above the lockscreen when the session is locked. If set to `true`, you can interact with the layer on the lockscreen, otherwise it will only be rendered above it. |
| noscreenshare \[on\] | Hides the layer from screen sharing by drawing a black rectangle over it. |

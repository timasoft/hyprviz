![Showcase](https://user-images.githubusercontent.com/43317083/179357849-321f042c-f536-44b3-9e6f-371df5321836.gif)

## Dispatchers

`layoutmsg` commands:

| command | description | params |
| --- | --- | --- |
| swapwithmaster | swaps the current window with master. If the current window is the master, swaps it with the first child. | either `master` (new focus is the new master window), `child` (new focus is the new child) or `auto` (which is the default, keeps the focus of the previously focused window). Adding `ignoremaster` will ignore this dispatcher if master is already focused. |
| focusmaster | focuses the master window. | either `master` (focus stays on master), `auto` (default; focus first non-master window if already on master) or `previous` (remember current window when focusing master, if already on master, focus previous or fallback to `auto`). |
| cyclenext | focuses the next window respecting the layout | either `loop` (allow looping from the bottom of the pile back to master) or `noloop` (force stop at the bottom of the pile, like in DWM). `loop` is the default if left blank. |
| cycleprev | focuses the previous window respecting the layout | either `loop` (allow looping from master to the bottom of the pile) or `noloop` (force stop at master, like in DWM). `loop` is the default if left blank. |
| swapnext | swaps the focused window with the next window respecting the layout | either `loop` (allow swapping the bottom of the pile and master) or `noloop` (do not allow it, like in DWM). `loop` is the default if left blank. |
| swapprev | swaps the focused window with the previous window respecting the layout | either `loop` (allow swapping master and the bottom of the pile) or `noloop` (do not allow it, like in DWM). `loop` is the default if left blank. |
| addmaster | adds a master to the master side. That will be the active window, if it's not a master, or the first non-master window. | none |
| removemaster | removes a master from the master side. That will be the active window, if it's a master, or the last master window. | none |
| orientationleft | sets the orientation for the current workspace to left (master area left, slave windows to the right, vertically stacked) | none |
| orientationright | sets the orientation for the current workspace to right (master area right, slave windows to the left, vertically stacked) | none |
| orientationtop | sets the orientation for the current workspace to top (master area top, slave windows to the bottom, horizontally stacked) | none |
| orientationbottom | sets the orientation for the current workspace to bottom (master area bottom, slave windows to the top, horizontally stacked) | none |
| orientationcenter | sets the orientation for the current workspace to center (master area center, slave windows alternate to the left and right, vertically stacked) | none |
| orientationnext | cycle to the next orientation for the current workspace (clockwise) | none |
| orientationprev | cycle to the previous orientation for the current workspace (counter-clockwise) | none |
| orientationcycle | cycle to the next orientation from the provided list, for the current workspace | allowed values: `left`, `top`, `right`, `bottom`, or `center`. The values have to be separated by a space. If left empty, it will work like `orientationnext` |
| mfact | change mfact, the master split ratio | the new split ratio, a relative float delta (e.g `-0.2` or `+0.2`) or `exact` followed by a the exact float value between 0.0 and 1.0 |
| rollnext | rotate the next window in stack to be the master, while keeping the focus on master | none |
| rollprev | rotate the previous window in stack to be the master, while keeping the focus on master | none |

Parameters for the commands are separated by a single space.

{{< callout type=info >}}

Example usage:

```ini
bind = MOD, KEY, layoutmsg, cyclenext
# behaves like xmonads promote feature (https://hackage.haskell.org/package/xmonad-contrib-0.17.1/docs/XMonad-Actions-Promote.html)
bind = MOD, KEY, layoutmsg, swapwithmaster master
```

{{< /callout >}}

## Workspace Rules

`layoutopt` rules:

| rule | description | type |
| --- | --- | --- |
| orientation:[o] | Sets the orientation of a workspace. For available orientations, see [Config->orientation](#config) | string |

Example usage:

```ini
workspace = 2, layoutopt:orientation:top
```

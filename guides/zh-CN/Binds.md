---
weight: 5
title: 键位绑定
---

> [!NOTE]
> 翻译自英文版，使用 Qwen3.5-Plus

## 基本

```ini
bind = MODS, key, dispatcher, params
```

例如，

```ini
bind = SUPER_SHIFT, Q, exec, firefox
```

将绑定打开 Firefox 到 <key>SUPER</key> + <key>SHIFT</key> + <key>Q</key>

> [!NOTE]
> 对于没有修饰键的绑定，将其留空：
>
> ```ini
> bind = , Print, exec, grim
> ```

_完整修饰键列表，请参阅 [Variables](../Variables/#variable-types)。_

_调度器列表可在 [Dispatchers](../Dispatchers/#list-of-dispatchers) 中找到。_

## 非常见符号 / 使用按键码绑定

请参阅 [xkbcommon-keysyms.h 头文件](https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h) 获取所有 keysyms。您应使用的名称是 `XKB_KEY_` 之后的部分。

如果您想通过按键码绑定，可以在 KEY 位置使用 `code:` 前缀，例如：

```ini
bind = SUPER, code:28, exec, amongus
```

这将绑定 <key>SUPER</key> + <key>t</key>，因为 <key>t</key> 的按键码是 28。

> [!NOTE]
> 如果您不确定按键的名称或按键码，可以使用 [`wev`](https://github.com/jwrdegoede/wev) 来查找。

## 其他

### 非 QWERTY 布局的工作区绑定

用于键位绑定的键需要在您的布局中无需任何修饰键即可访问。  
例如，[法语 AZERTY](https://zh.wikipedia.org/wiki/AZERTY) 布局使用 <key>SHIFT</key> + _`未修饰键`_ 来输入 `0-9` 数字。因此，此布局的工作区键位绑定需要使用 _`未修饰键`_ 的名称，而不能在使用 `0-9` 数字时工作。

> [!NOTE]
> 要获取 `unmodified_key` 的正确名称，请参阅 [非常见符号部分](#uncommon-syms--binding-with-a-keycode)

```ini
# 在法语布局中，而不是：
# bind = $mainMod, 1, workspace,  1

# 使用
bind = $mainMod, ampersand, workspace,  1
```

有关配置法语 AZERTY 布局的帮助，请参阅此 [文章](https://rherault.dev/articles/hyprland-fr-layout)。

### 解除绑定

您也可以使用 `unbind` 关键字解除键位绑定，例如：

```ini
unbind = SUPER, O
```

这对于使用 `hyprctl` 动态键位绑定可能很有用，例如：

```bash
hyprctl keyword unbind SUPER, O
```

> [!NOTE]
> 在 `unbind` 中，键是区分大小写的。它必须与您要解除绑定的 `bind` 中的大小写完全匹配。
>
> ```ini
> bind = SUPER, TAB, workspace, e+1
> unbind = SUPER, Tab # 这将不会解除绑定
> unbind = SUPER, TAB # 这将解除绑定
> ```

## 绑定标志

`bind` 支持以下格式的标志：

```ini
bind[flags] = ...
```

例如：

```ini
bindrl = MOD, KEY, exec, amongus
```

可用标志：

| 标志 | 名称 | 说明 |
|------|------|-------------|
| `l` | locked | 即使在输入抑制器 (例如锁屏) 激活时也能工作。 |
| `r` | release | 将在按键释放时触发。 |
| `c` | click | 只要鼠标光标保持在 `binds:drag_threshold` 内，将在按键或按钮释放时触发。 |
| `g` | drag | 只要鼠标光标移动到 `binds:drag_threshold` 之外，将在按键或按钮释放时触发。 |
| `o` | long press | 将在长按按键时触发。 |
| `e` | repeat | 按住时会重复触发。 |
| `n` | non-consuming | 键/鼠标事件将传递给活动窗口，同时触发调度器。 |
| `m` | mouse | 请参阅专门的 [鼠标绑定](#mouse-binds) 部分。 |
| `t` | transparent | 不能被其他绑定覆盖。 |
| `i` | ignore mods | 将忽略修饰键。 |
| `s` | separate | 将任意组合每个修饰键/键之间的键，请参阅 [Keysym 组合](#keysym-combos)。 |
| `d` | has description | 允许您为绑定编写描述。 |
| `p` | bypass | 绕过应用程序请求抑制键位绑定。 |
| `u` | submap universal | 无论当前子映射为何，都将保持激活。 |

示例用法：

```ini
# 示例音量按钮，允许按住并保持，音量限制为 150%
binde = , XF86AudioRaiseVolume, exec, wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 5%+

# 示例音量按钮，即使在输入抑制器激活时也会激活
bindl = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-

# 首次按下打开 wofi，第二次关闭
bindr = SUPER, SUPER_L, exec, pkill wofi || wofi

# 描述一个绑定
bindd = SUPER, Q, 打开我最喜欢的终端，exec, kitty

# 长按时跳过播放器，正常按下时仅跳过 5 秒
bindo = SUPER, XF86AudioNext, exec, playerctl next
bind = SUPER, XF86AudioNext, exec, playerctl position +5
```

### 鼠标按钮

您也可以通过在鼠标按键码前加上 `mouse:` 来绑定或解除绑定鼠标按钮，例如：

```ini
bind = SUPER, mouse:272, exec, amongus  # 将 `exec amogus` 绑定到 SUPER + 左键。
```

### 仅绑定修饰键

要仅绑定修饰键，您需要使用 TARGET 修饰掩码 (带激活修饰键) 和 `r` 标志，例如：

```ini
bindr = SUPER ALT, Alt_L, exec, amongus  # 将 `exec amongus` 绑定到 SUPER + ALT。
```

### Keysym 组合

对于多个键的任意组合，在每个修饰键/键之间用 `&` 分隔，并使用 `s` 标志，例如：

```ini
# 您可以使用单个修饰键和多个键。
binds = Control_L, A&Z, exec, kitty
# 您也可以指定多个特定修饰键。
binds = Control_L&Shift_L, K, exec, kitty
# 您也可以同时使用两者！
binds = Control_R&Super_R&Alt_L, J&K&L, exec, kitty
# 如果您想尝试更疯狂的...可以使用其他键进行绑定...
binds = Escape&Apostrophe&F7, T&O&A&D, exec, battletoads 2: retoaded
```

> [!NOTE]
> 请注意，这仅对 keysym 有效，并且会使所有修饰键变为 keysym。  
> 如果您不知道什么是 keysym，请使用 `wev` 并按下您要使用的键。

### 鼠标滚轮

您也可以使用 `mouse_up` 和 `mouse_down` (或如果您的鼠标支持水平滚动，则使用 `mouse_left` 和 `mouse_right`) 绑定鼠标滚轮事件：

```ini
bind = SUPER, mouse_down, workspace, e-1
```

> [!NOTE]
> 您可以使用 `binds:scroll_event_delay` 控制重置时间。

### 开关

开关对于绑定事件很有用，例如关闭和打开笔记本电脑的盖子：

```ini
# 当开关切换时触发。
bindl = , switch:[开关名称], exec, swaylock
# 当开关开启时触发。
bindl = , switch:on:[开关名称], exec, hyprctl keyword monitor "eDP-1, disable"
# 当开关关闭时触发。
bindl = , switch:off:[开关名称], exec, hyprctl keyword monitor "eDP-1, 2560x1600, 0x0, 1"
```

> [!WARNING]
> Systemd `HandleLidSwitch` 设置在 `logind.conf` 中可能与 Hyprland 的笔记本电脑盖子开关配置冲突。

> [!NOTE]
> 您可以使用 `hyprctl devices` 查看您的开关。

### 一个键多个绑定

您可以通过多次分配同一个键绑定，使用不同的 `disapatcher` 和 `param` 来触发多个操作：

```ini
# 在浮动工作区中切换窗口：
bind = SUPER, Tab, cyclenext         # 更改焦点到另一个窗口
bind = SUPER, Tab, bringactivetotop  # 将其带到顶部
```

> [!WARNING]
> 键绑定将自上而下按编写顺序执行。

### 描述

您可以使用 `d` 标志描述您的键绑定。  
您的描述始终位于 `dispatcher` 前面，并且不得包含逗号 (`,`)！

```ini
bindd = MODS, key, description, dispatcher, params
```

例如：

```ini
bindd = SUPER, Q, 打开我最喜欢的终端，exec, kitty
```

如果您想访问您的描述，可以使用 `hyprctl binds`。  
有关更多信息，请查看 [使用 Hyprctl](../Using-hyprctl)。

## 鼠标绑定

这些是依赖鼠标移动的绑定。它们将少一个参数。  
`binds:drag_threshold` 可用于区分相同按钮的点击和拖动：

```ini
binds {
    drag_threshold = 10  # 仅在拖动超过 10px 后触发拖动事件
}
bindm = ALT, mouse:272, movewindow      # ALT + 左键：通过拖动超过 10px 移动窗口。
bindc = ALT, mouse:272, togglefloating  # ALT + 左键：通过点击浮动窗口
```

可用鼠标绑定：

| 名称 | 说明 | 参数 |
| ---- | ----------- | ------ |
| movewindow | 移动活动窗口 | 无 |
| resizewindow | 调整活动窗口大小 | `1` -> 调整大小并保持窗口宽高比。 <br> `2` -> 调整大小并忽略 `keepaspectratio` 窗口规则/属性。 <br> 无或任何其他值表示正常调整大小 |

常见鼠标按钮按键码 (检查 `wev` 以获取其他按钮)：

```txt
左键 -> 272
右键 -> 273
中键 -> 274
```

> [!NOTE]
> 鼠标绑定，尽管其名称，其行为类似于普通绑定。  
> 您可以自由使用任何键 / 修饰键。当按下时，鼠标功能将被
> 激活。

### 触摸板

由于在触摸板上点击和移动鼠标不舒适，您也可以使用键盘键代替鼠标点击。

```ini
bindm = SUPER, mouse:272, movewindow
bindm = SUPER, Control_L, movewindow
bindm = SUPER, mouse:273, resizewindow
bindm = SUPER, ALT_L, resizewindow
```

## 全局键绑定

### 经典方式

是的，您没听错，Hyprland 确实支持_所有_应用程序的全局键绑定，
包括 OBS、Discord、Firefox 等。

请参阅 [`pass`](../Dispatchers/#list-of-dispatchers) 和
[`sendshortcut`](../Dispatchers/#list-of-dispatchers) 调度器。

以 OBS 为例："开始/停止录制" 键绑定设置为
<key>SUPER</key> + <key>F10</key>，要使其全局工作，只需添加：

```ini
bind = SUPER, F10, pass, class:^(com\.obsproject\.Studio)$
```

到您的配置中即可。

`pass` 将自行传递 `PRESS` 和 `RELEASE` 事件，无需 `bindr`。  
这也意味着一键通将与一个 `pass` 完美配合，例如：

```ini
bind = , mouse:276, pass, class:^(TeamSpeak 3)$  # 将 MOUSE5 传递给 TeamSpeak3。
```

您也可以添加快捷方式，将其他键传递给窗口。

```ini
bind = SUPER, F10, sendshortcut, SUPER, F4, class:^(com\.obsproject\.Studio)$  # 当按下 SUPER + F10 时，将 SUPER + F4 发送给 OBS。
```

> [!WARNING]
> 这与所有原生 Wayland 应用程序完美配合，但 XWayland 有点不稳定。  
> 确保您传递的是"全局 Xorg 键绑定"，否则从其他 XWayland 应用程序传递可能无法工作。

### DBus 全局快捷键

某些应用程序可能已经支持 xdg-desktop-portal 中的 GlobalShortcuts 门户。  
如果是这种情况，建议使用以下方法而不是 `pass`：

打开您想要的应用程序并在终端中运行 `hyprctl globalshortcuts`。  
这将为您提供当前注册的快捷键列表及其描述。

选择您喜欢的，例如 `coolApp:myToggle`，并使用 `global` 调度器将其绑定到您想要的任何内容：

```ini
bind = SUPERSHIFT, A, global, coolApp:myToggle
```

> [!NOTE]
> 请注意，此功能_仅_与
> [XDPH](../../Hypr-Ecosystem/xdg-desktop-portal-hyprland) 一起工作。

## 子映射

键位绑定子映射，也称为 _模式_ 或 _组_，允许您激活
一组单独的键位绑定。  
例如，如果您想进入一个 `resize` _模式_，允许您使用箭头键调整窗口大小，您可以这样操作：

```ini
# 切换到名为 `resize` 的子映射。
bind = ALT, R, submap, resize

# 开始名为 "resize" 的子映射。
submap = resize

# 设置可重复绑定以调整活动窗口大小。
binde = , right, resizeactive, 10 0
binde = , left, resizeactive, -10 0
binde = , up, resizeactive, 0 -10
binde = , down, resizeactive, 0 10

# 使用 `reset` 返回到全局子映射
bind = , escape, submap, reset

# 重置子映射，将返回到全局子映射
submap = reset

# 后续键位绑定将再次为全局...
```

> [!WARNING]
> 不要忘记在内部时重置键映射的键绑定 (`escape`，在本例中)！
>
> 如果您卡在键映射内部，可以使用 `hyprctl dispatch submap reset` 返回。  
> 如果您没有打开终端，那就倒霉了。您已被警告。

您也可以设置相同的键绑定执行多个操作，例如调整大小
并关闭子映射，如下所示：

```ini
bind = ALT, R, submap, resize

submap = resize

bind = , right, resizeactive, 10 0
bind = , right, submap, reset
# ...

submap = reset
```

这之所以有效，是因为绑定按出现顺序执行，并且
每个绑定可以分配多个操作。

您可以使用 submap universal 绑定标志设置一个无论当前子映射为何都将保持激活的键位绑定。

```ini
bindu = $mainMod, K, exec, kitty
```

### 嵌套

子映射可以嵌套，见以下示例：

```ini
bind = $mainMod, M, submap, main_submap
submap = main_submap

# ...

# nested_one
bind = , 1, submap, nested_one
submap = nested_one

# ...

bind = SHIFT, escape, submap, reset
bind =      , escape, submap, main_submap
submap = main_submap
# /nested_one

# nested_two
bind = , 2, submap, nested_two
submap = nested_two

# ...

bind = SHIFT, escape, submap, reset
bind =      , escape, submap, main_submap
submap = main_submap
# /nested_two

bind = , escape, submap, reset
submap = reset
```

### 自动关闭子映射

通过在后面附加 ``,`` 后跟子映射或 _reset_，可以在调度后自动关闭子映射或发送到另一个子映射。

```ini
bind = SUPER,a, submap, submapA

# 按下 a 后将子映射设置为 submapB。
submap = submapA, submapB
bind = ,a,exec, someCoolThing.sh
submap = reset

# 按下 a 后将子映射重置为默认。
submap = submapB, reset
bind = ,a,exec, someOtherCoolThing.sh
submap = reset
```

### 通配捕获

您还可以通过特殊的 `catchall` 关键字定义键绑定，该关键字
在按下任何键时都会激活。  
这可用于在子映射中防止任何键传递到活动应用程序，
或在按下任何未知键时立即退出：

```ini
bind = , catchall, submap, reset
```

## 示例绑定

### 媒体

这些绑定设置了常规键盘媒体音量键的预期行为，
包括屏幕锁定时：

```ini
bindel = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bindel = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bindl = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
# 需要 playerctl
bindl = , XF86AudioPlay, exec, playerctl play-pause
bindl = , XF86AudioPrev, exec, playerctl previous
bindl = , XF86AudioNext, exec, playerctl next
```

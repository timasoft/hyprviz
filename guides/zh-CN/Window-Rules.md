---
weight: 7
title: 窗口规则
---

> [!NOTE]
> 翻译自英文版，使用 Qwen3.5-Plus

> [!WARNING]
> 规则从上到下依次评估，因此它们的书写顺序确实很重要！
> 更多信息请参阅 [注意事项](#notes)

## 窗口规则

您可以根据窗口属性设置窗口规则来实现不同的窗口行为。

### 语法

基本命名规则语法：
```ini
windowrule {
  name = apply-something
  match:class = my-window

  border_size = 10
}
```

基本匿名规则语法：
```ini
windowrule = match:class my-window, border_size 10
```

规则分为两类参数：_props_ 和 _effects_。Props
是 `match:` 部分，用于确定窗口是否应获得该
规则。Effects 是应用的内容。

_所有_ props 必须全部匹配才能应用规则。

您可以每条规则设置任意数量的 props 和 effects，顺序任意，只要：
- 每种类型只有一个（例如指定 `match:class` 两次是无效的）
- 至少有一个 _prop_

### Props

支持的 props 字段：

| Field | Argument | Description |
| -------------- | --------------- | --- |
| match:class | \[RegEx\] | `class` 与 `RegEx` 匹配的窗口。 |
| match:title | \[RegEx\] | `title` 与 `RegEx` 匹配的窗口。 |
| match:initial_class | \[RegEx\] | `initialClass` 与 `RegEx` 匹配的窗口。 |
| match:initial_title |\[RegEx\] | `initialTitle` 与 `RegEx` 匹配的窗口。 |
| match:tag | \[name\] | 具有匹配 `tag` 的窗口。 |
| match:xwayland | \[bool\] | Xwayland 窗口。 |
| match:float | \[bool\] | 浮动窗口。 |
| match:fullscreen | \[bool\] | 全屏窗口。 |
| match:pin | \[bool\] | 固定窗口。 |
| match:focus | \[bool\] | 当前聚焦窗口。 |
| match:group | \[bool\] | 分组窗口。 |
| match:modal | \[bool\] | 模态窗口（例如 "Are you sure" 弹窗） |
| match:fullscreen_state_client | \[client\] | 具有匹配 `fullscreenstate` 的窗口。`client` 可以是 `0` - 无，`1` - 最大化，`2` - 全屏，`3` - 最大化和全屏。 |
| match:fullscreen_state_internal | \[internal\] | 具有匹配 `fullscreenstate` 的窗口。`internal` 可以是 `0` - 无，`1` - 最大化，`2` - 全屏，`3` - 最大化和全屏。 |
| match:workspace | \[workspace\] | 在匹配工作区上的窗口。`w` 可以是 `id`、`name:string` 或 `workspace selector`。 |
| match:content | \[int\] | 具有指定内容类型的窗口 (none = 0, photo = 1, video = 2, game = 3) |
| match:xdg_tag | \[RegEx\] | 通过其 xdgTag 匹配窗口（参见 `hyprctl clients` 检查是否具有） |

请记住，您_必须_至少声明一个字段，但不需要全部声明。

> [!NOTE]
> 要获取有关窗口的 class、title、XWayland 状态或其
> 大小的更多信息，您可以使用 `hyprctl clients`。

> [!NOTE]
> 在 `hyprctl clients` 命令的输出中：
> `fullscreen` 指的是 `fullscreen_state_internal` 和
> `fullscreenClient` 指的是 `fullscreen_state_client`

### 正则表达式编写

请注意 Hyprland 使用 [Google 的 RE2](https://github.com/google/re2) 来解析正则表达式。这意味着所有需要多项式
时间计算的操作将无法工作。请参阅 [RE2 wiki](https://github.com/google/re2/wiki/Syntax) 了解支持的扩展。

如果您想_否定_一个正则表达式，即仅当正则表达式_失败_时才通过，您可以在其前面加上 `negative:`，例如：`negative:kitty`。

## Effects

### 静态 effects

Static effects 在窗口打开时仅评估一次，之后不再重新评估。这实际上意味着匹配 `title` 和 `class` 时，将始终找到 `initialTitle` 和 `initialClass`。

> [!WARNING]
> 不可能基于窗口创建后 `title` 的变化来 `float`（或此处列出的任何其他静态规则）窗口。这适用于此处列出的所有静态 effects。

| Effect | argument | Description |
| ---- | ----------- | --- |
| float | \[on\] | 使窗口浮动。 |
| tile | \[on\] |使窗口平铺。 |
| fullscreen | \[on\] | 使窗口全屏。 |
| maximize | \[on\] | 使窗口最大化。 |
| fullscreen_state | \[internal\] \[client\]  | 设置聚焦窗口的全屏模式及发送给客户端的模式，其中 internal 和 client 可以是 `0` - 无，`1` - 最大化，`2` - 全屏，`3` - 最大化和全屏。 |
| move  | \[expr\] \[expr\] | 将浮动窗口移动到给定坐标，显示器局部。两个表达式以空格分隔。 |
| size | \[expr\] \[expr\] | 将浮动窗口调整为给定大小。两个表达式以空格分隔。 |
| center | \[on\] | 如果窗口是浮动的，将其中心置于显示器上。 |
| pseudo | \[on\] | 伪平铺窗口。 |
| monitor | \[id\] | 设置窗口应打开的显示器。`id` 可以是 id 编号或名称（例如 `1` 或 `DP-1`）。 |
| workspace | \[w\] | 设置窗口应打开的工作区（有关工作区语法，请参见 [调度器->工作区](../Dispatchers#workspaces)）。 <br> 您还可以将 \[w\] 设置为 `unset`。这将取消应用于此窗口的所有先前工作区规则。此外，您可以在工作区后添加 `silent` 以使窗口静默打开。 |
| no_initial_focus | \[on\] | 禁用窗口的初始聚焦 |
| pin | \[on\] | 固定窗口（即在所有工作区显示）。_注意：仅限浮动窗口_。 |
| group | \[options\] | 设置窗口组属性。请参阅下方注释。 |
| suppress_event | \[types...\] | 忽略来自窗口的特定事件。事件以空格分隔，可以是：`fullscreen`、`maximize`、`activate`、`activatefocus`、`fullscreenoutput`。 |
| content | \[none\|photo\|video\|game\] | 设置内容类型。 |
| no_close_for | \[ms\] | 在打开时，使窗口在给定的毫秒数内无法通过 `killactive` 调度器关闭。 |

#### 表达式

表达式以空格分隔，因此您的数学表达式不能有空格。它们是常规的数学表达式，公开了一些变量，
名称不言自明。所有位置变量都是显示器局部的。

- `monitor_w` 和 `monitor_h` 用于显示器大小
- `window_x` 和 `window_y` 用于窗口位置
- `window_w` 和 `window_h` 用于窗口大小
- `cursor_x` 和 `cursor_y` 用于光标位置

示例表达式：
- `window_w*0.5`
- `(monitor_w/2)+17`

用括号包围表达式以提高清晰度是个好主意，使用空格分隔：
- `(monitor_w*0.5) (monitor_h*0.5)`
- `((monitor_w*0.5)+17) (monitor_h*0.2)`

### 动态 effects

Dynamic effects 在每次属性更改时重新评估。

| Effect | argument | Description |
| ---- | ----------- | --- |
| persistent_size | \[on\] | 允许浮动窗口在应用程序启动之间保持大小。 |
| no_max_size | \[on\] | 移除最大大小限制。对于报告无效最大大小的窗口特别有用（例如 winecfg）。 |
| stay_focused | \[on\] | 只要窗口可见，就强制聚焦在该窗口上。 |
| animation | \[style\] (\[opt\]) | 强制将动画应用于窗口，使用选定的 opt。Opt 是可选的。 |
| border_color | \[c\] | 强制窗口的边框颜色。 <br> c 的选项：`color`/`color ... color angle` -> 设置活动边框颜色/渐变 OR `color color`/`color ... color angle color ... color [angle]` -> 设置窗口的活动和非活动边框颜色/渐变。颜色定义请参见 [变量->颜色](../Variables#variable-types)。 |
| idle_inhibit  | \[mode\] | 为窗口设置空闲抑制规则。如果激活，`hypridle` 等应用将不会触发。模式：`none`、`always`、`focus`、`fullscreen`。 |
| opacity  | \[a\] | 附加不透明度乘数。a 的选项：`float` -> 设置整体不透明度，`float float` -> 分别设置 active_opacity 和 inactive_opacity，`float float float` -> 分别设置 active_opacity、inactive_opacity 和 fullscreen_opacity。 |
| tag | \[name\] | 将标签 `name` 应用于窗口，使用前缀 `+`/`-` 来设置/取消设置标志，或不使用前缀来切换标志。 |
| max_size | \[w\] \[h\] | 设置最大大小（x,y -> 整数）。适用于浮动窗口。（使用 `misc:size_limits_tiled` 以包含平铺窗口。） |
| min_size | \[w\] \[h\] | 设置最小大小（x,y -> 整数）。适用于浮动窗口。（使用 `misc:size_limits_tiled` 以包含平铺窗口。） |
| border_size | \[int\] | 设置边框大小。 |
| rounding | \[int\] | 强制应用程序具有 X 像素的圆角，忽略默认设置（在 `decoration:rounding` 中）。必须是整数。 |
| rounding_power | \[float\] | 覆盖窗口的圆角功率（参见 `decoration:rounding_power`）。 |
| allows_input | \[on\] | 强制 XWayland 窗口接收输入，即使它请求不这样做。（可能解决某些游戏启动器因某种原因无法接收焦点的问题） |
| dim_around | \[on\] | 使窗口周围的所有内容变暗。请注意，此规则适用于浮动窗口，将其用于平铺窗口可能会导致奇怪的行为。 |
| decorate | \[on\] | (默认是 _true_) 是否绘制窗口装饰 |
| focus_on_activate | \[on\] | Hyprland 是否应聚焦请求聚焦的应用程序（`activate` 请求）。 |
| keep_aspect_ratio | \[on\] | 用鼠标调整窗口大小时强制保持宽高比。 |
| nearest_neighbor | \[on\] | 强制窗口使用 [最近邻](https://zh.wikipedia.org/wiki/%E6%9C%80%E8%BF%91%E9%82%BB%E6%8F%92%E5%80%BC) 过滤。 |
| no_anim | \[on\] | 禁用窗口的动画。 |
| no_blur | \[on\] |  禁用窗口的模糊效果。 |
| no_dim | \[on\] |  禁用窗口变暗。 |
| no_focus | \[on\] |  禁用窗口聚焦。 |
| no_follow_mouse | \[on\] |  当设置 `input:follow_mouse=1` 时，防止鼠标移动到其上时聚焦窗口。 |
| no_shadow | \[on\] |  禁用窗口阴影。 |
| no_shortcuts_inhibit | \[on\] |  不允许应用程序 [抑制您的快捷键](https://wayland.app/protocols/keyboard-shortcuts-inhibit-unstable-v1)。 |
| no_screen_share | \[on\] | 通过在其位置绘制黑色矩形，使窗口及其弹窗在屏幕共享中隐藏。即使其他窗口在上方，也会绘制矩形。 |
| no_vrr | \[on\] | 为窗口禁用 VRR。仅当 [`misc:vrr`](../Variables/#Misc) 设置为 `2` 或 `3` 时才有效。 |
| opaque | \[on\] |  强制窗口不透明。 |
| force_rgbx | \[on\] |  强制 Hyprland 忽略整个窗口表面的 alpha 通道，使其_实际上、完全 100% 不透明_。 |
| sync_fullscreen | \[on\] |  全屏模式是否应始终与发送给窗口的模式相同（仅在下一次全屏模式更改时生效）。 |
| immediate | \[on\] |  强制窗口允许撕裂。参见 [撕裂页面](../Tearing)。 |
| xray | \[on\] |  为窗口设置模糊 X 射线模式。 |
| render_unfocused | \[on\] |  强制窗口认为它在不可见时正在渲染。另请参见 [变量 - 杂项](../Variables/#Misc) 了解设置 `render_unfocused_fps`。 |
| scroll_mouse | \[float\] | 强制窗口覆盖变量 `input:scroll_factor`。 |
| scroll_touchpad  | \[float\] | 强制窗口覆盖变量 `input:touchpad:scroll_factor`。

所有动态 effects 都可以用 `setprop` 设置，参见 [`setprop`](../Dispatchers#setprop)。

### `group` 窗口规则选项

- `set` [`always`] - 将窗口作为组打开。
- `new` - `barred set` 的简写。
- `lock` [`always`] - 锁定添加此窗口的组。与 `set` 或
  `new` 一起使用（例如 `new lock`）以创建新的锁定组。
- `barred` - 不要自动将窗口分组到聚焦的未锁定组中。
- `deny` - 不允许窗口被切换为组或添加到组中（参见 `denywindowfromgroup` 调度器）。
- `invade` - 强制在锁定组中打开窗口。
- `override` [其他选项] - 覆盖其他 `group` 规则，例如，您可以使特定工作区中的所有窗口作为组打开，并使用 `group override barred` 使具有特定标题的窗口作为普通窗口打开。
- `unset` - 清除所有 `group` 规则。

> [!NOTE]
> 没有选项的 `group` 规则是 `group set` 的简写。
>
> 默认情况下，`set` 和 `lock` 仅对新窗口生效一次。`always`
> 限定符使它们始终有效。

### 标签

窗口可能有多个标签，可以是静态的或动态的。动态标签将带有 `*` 后缀。
您可以使用 `hyprctl clients` 检查窗口标签。

使用 `tagwindow` 调度器向窗口添加静态标签：

```bash
hyprctl dispatch tagwindow +code       # 为当前窗口添加标签。
hyprctl dispatch tagwindow -- -code    # 从当前窗口移除标签（使用 `--` 保护前导 `-`）。
hyprctl dispatch tagwindow code        # 切换当前窗口的标签。

# 或者您可以使用窗口正则表达式标记匹配的窗口：
hyprctl dispatch tagwindow +music deadbeef
hyprctl dispatch tagwindow +media title:Celluloid
```

使用 `tag` 规则向窗口添加动态标签：

```ini
windowrule = tag +term, match:class footclient  # 为窗口 footclient 添加动态标签 `term*`。
windowrule = tag term, match:class footclient   # 切换窗口 footclient 的动态标签 `term*`。
windowrule = tag +code, match:tag cpp           # 为具有标签 `cpp` 的窗口添加动态标签 `code*`。

windowrule = opacity 0.8, match:tag code        # 为具有标签 `code` 或 `code*` 的窗口设置不透明度。
windowrule = opacity 0.7, match:tag cpp         # 具有标签 `cpp` 的窗口将同时匹配 `code` 和 `cpp`，后者将覆盖先前的匹配。
windowrule = opacity 0.6, match:tag term*       # 仅为具有标签 `term*` 的窗口设置不透明度，`term` 不会匹配。

windowrule = tag -code, match:tag  term          # 为具有标签 `term` 或 `term*` 的窗口移除动态标签 `code*`。
```

或者使用键绑定以方便使用：

```ini
bind = $mod Ctrl, 2, tagwindow, alpha_0.2
bind = $mod Ctrl, 4, tagwindow, alpha_0.4

windowrule = opacity 0.2 override, match:tag alpha_0.2
windowrule = opacity 0.4 override, match:tag alpha_0.4
```

`tag` 规则只能操作动态标签，而 `tagwindow` 调度器仅适用于静态标签（即调用调度器后，动态标签将被清除）。

### 示例规则

```ini
# 将 kitty 移动到 100 100 并添加动画样式
windowrule {
  name = move-kitty
  match:class = kitty

  move = 100 100
  animation = popin
}

windowrule = no_blur on, match:class firefox                                              # 为 firefox 禁用模糊
windowrule = move (cursor_x-(window_w*0.5)) (cursor_y-(window_h*0.5)), match:class kitty  # 将 kitty 移动到光标中心
windowrule = border_color rgb(FF0000) rgb(880808), match:fullscreen 1                     # 如果窗口全屏，则将边框颜色设置为红色
windowrule = border_color rgb(FFFF00), match:title .*Hyprland.*                           # 当标题包含 Hyprland 时，将边框颜色设置为黄色
windowrule = opacity 1.0 override 0.5 override 0.8 override, match:class kitty            # 为 kitty 设置活动不透明度为 1.0，非活动不透明度为 0.5，全屏不透明度为 0.8
windowrule = match:class kitty, rounding 10                                               # 为 kitty 设置圆角为 10
windowrule = match:class (pinentry-)(.*), stay_focused on                                  # 修复 pinentry 失去焦点的问题
```

### 注意事项

标记为_动态_的 effects 将在窗口的匹配属性更改时重新评估。<br>
例如，如果定义了一个规则，在窗口浮动时更改其 `border_color`，则当设置为浮动时，`border_color` 将更改为请求的颜色，当再次平铺时恢复为默认颜色。

Effects 将从上到下处理，_最后一个_ 匹配将优先。即：
```ini
windowrule = opacity 0.8 0.8, match:class kitty
windowrule = opacity 0.5 0.5, match:float yes
```

在这里，所有非全屏的 kitty 窗口将具有 `opacity 0.8`，除非
它们是浮动的。否则，它们将具有 `opacity 0.5`。其余
非全屏浮动窗口将具有 `opacity 0.5`。
```ini
windowrule = opacity 0.5 0.5, match:float true
windowrule = opacity 0.8 0.8, match:class kitty
```

在这里，所有 kitty 窗口将具有 `opacity 0.8`，即使它们是浮动的。
其余浮动窗口将具有 `opacity 0.5`。

> [!NOTE]
> 默认情况下，不透明度是所有不透明度的乘积。例如，设置
> `active_opacity` 为 `0.5` 并将 `opacity` 设置为 `0.5` 将导致总不透明度为
> `0.25`。 <br>
> 允许将不透明度设置为超过 `1.0`，但任何超过 `1.0` 的不透明度乘积将导致图形故障。 <br>
> 例如，使用 `0.5 * 2 = 1` 没问题，但 `0.5 * 4 = 2` 将导致图形故障。 <br>
> 您可以在不透明度值后放置 `override` 以覆盖为精确值而不是乘数。
> 例如，要将活动和非活动不透明度设置为 0.8，并使全屏窗口
> 完全不透明，不受其他不透明度规则影响：
>
> ```ini
> windowrule = match:class kitty, opacity 0.8 override 0.8 override 1.0 override
> ```

### 动态启用/禁用/更改规则

只有命名规则可以动态更改、启用或禁用。匿名规则不能。

对于启用或禁用，公开了关键字 `enable`：

```sh
hyprctl keyword 'windowrule[my-rule-name]:enable false'
```

对于更改属性，可以使用相同的语法：

```sh
hyprctl keyword 'windowrule[my-rule-name]:match:class kitty'
```

_单引号是必需的，否则您的 shell 可能会尝试解析该字符串_

---
weight: 7
title: 窗口规则
---

{{< callout type=info >}}

Translated from en by qwen3

{{</ callout >}}

{{< callout type=warning >}}

窗口规则是**区分大小写**的。（例如 `firefox` ≠ `Firefox`）

从 Hyprland v0.46.0 开始，正则表达式需要完全匹配窗口值。例如，对于 `kitty`：

- `kitty`/`(kitty)`/`^(kitty)$`：匹配成功。
- `tty`：以前可以匹配，现在不行。使用 `.*tty.*` 使其表现如前，
  或考虑使用更具体的正则表达式。

{{< /callout >}}

{{< callout type=warning >}}

规则从上到下依次评估，因此它们的书写顺序确实很重要！
更多信息请参阅[注意事项](#注意事项)

{{< /callout >}}

## 窗口规则

您可以设置窗口规则，根据窗口属性实现不同的窗口行为。

### 语法

```ini
windowrule=RULE,PARAMETERS
```

- `RULE` 是[规则](#规则)（如果适用，还包括参数）
- `PARAMETERS` 是可通过各种窗口属性匹配的逗号分隔列表。请参阅下方字段。

示例规则：
```ini
windowrule = float, class:kitty, title:kitty
```

单行中可以指定多个规则，用逗号分隔。但必须至少跟一个参数。

示例：
```ini
windowrule = float, pin, size 400 400, move 0 0, class:kitty, initialTitle:kitty
```
其中 float、pin、size 和 move 是 `RULES`，而 class 和 initialTitle 是 `PARAMETERS`。

{{< callout type=info >}}

对于浏览器窗口等具有动态窗口标题的情况，请记住正则表达式功能强大。

例如，窗口规则：
`windowrule = opacity 0.3 override 0.3 override,title:(.*)(- Youtube)` 将匹配
_任何_ 包含 "- Youtube" 字符串的窗口（该字符串位于其他文本之后）。这可能是
多个浏览器窗口或其他因任何原因包含该字符串的应用程序。

对于 `windowrule = float,class:kitty,title:kitty` 示例，
`class:(kitty)` `WINDOW` 字段使窗口规则专门针对 kitty 终端。

{{< /callout >}}

支持的参数字段：

| 字段 | 说明 |
| -------------- | --------------- |
| class:[RegEx] | `class` 与 `RegEx` 匹配的窗口。 |
| title:[RegEx] | `title` 与 `RegEx` 匹配的窗口。 |
| initialClass:[RegEx] | `initialClass` 与 `RegEx` 匹配的窗口。 |
| initialTitle:[RegEx] | `initialTitle` 与 `RegEx` 匹配的窗口。 |
| tag:[name] | 具有匹配 `tag` 的窗口。 |
| xwayland:[0/1] | Xwayland 窗口。 |
| floating:[0/1] | 浮动窗口。 |
| fullscreen:[0/1] | 全屏窗口。 |
| pinned:[0/1] | 固定窗口。 |
| focus:[0/1] | 当前聚焦窗口。 |
| group:[0/1] | 分组窗口。 |
| fullscreenstate:[internal] [client] | 具有匹配 `fullscreenstate` 的窗口。`internal` 和 `client` 可以是 `*` - 任意，`0` - 无，`1` - 最大化，`2` - 全屏，`3` - 最大化和全屏。 |
| workspace:[w] | 在匹配工作区上的窗口。`w` 可以是 `id` 或 `name:string`。 |
| onworkspace:[w] | 在匹配工作区上的窗口。`w` 可以是 `id`、`name:string` 或 `workspace selector`。 |
| content:[none|photo|video|game] | 具有指定内容类型的窗口 |
| xdgtag:[string] | 通过其 xdgTag 匹配窗口（参见 `hyprctl clients` 检查是否具有） |

请注意，您_必须_至少声明一个字段，但不需要全部声明。

{{< callout type=info >}}

要获取有关窗口的 class、title、XWayland 状态或其大小的更多信息，
可以使用 `hyprctl clients`。

{{< /callout >}}


{{< callout type=info >}}

在 `hyprctl clients` 命令的输出中：
`fullscreen` 指的是 `fullscreenstate.internal`，
`fullscreenClient` 指的是 `fullscreenstate.client`

{{< /callout >}}

### 正则表达式编写

请注意，Hyprland 使用 [Google 的 RE2](https://github.com/google/re2) 来解析正则表达式。这意味着所有需要多项式时间计算的操作将无法工作。请参阅 [RE2 wiki](https://github.com/google/re2/wiki/Syntax) 了解支持的扩展。

如果您想_否定_一个正则表达式，即仅当正则表达式_失败_时才通过，可以在其前面加上 `negative:`，例如：`negative:kitty`。

## 规则

### 静态规则

静态规则在窗口打开时仅评估一次，之后不再重新评估。这实际上意味着匹配 `title` 和 `class` 时，将始终使用 `initialTitle` 和 `initialClass`。

{{< callout type=warning >}}

不可能基于窗口创建后 `title` 的变化来`float`（或此处列出的任何其他静态规则）窗口。这适用于此处列出的所有静态规则。

{{< /callout >}}

| 规则 | 说明 |
| ---- | ----------- |
| float | 使窗口浮动。 |
| tile | 使窗口平铺。 |
| fullscreen | 使窗口全屏。 |
| maximize | 使窗口最大化。 |
| persistentsize | 允许浮动窗口在应用程序启动之间保持大小。 |
| fullscreenstate [internal] [client] | 设置聚焦窗口的全屏模式及发送给客户端的模式，其中 internal 和 client 可以是 `0` - 无，`1` - 最大化，`2` - 全屏，`3` - 最大化和全屏。 |
| move [x] [y] | 移动浮动窗口（`x, y` -> 整数或百分比，例如 `100` 或 `20%`。<br>您也可以使用 `100%-` 表示右/下边缘锚点，例如 `100%-20`。此外，选项还支持使用 `100%-w-` 减去窗口大小，例如 `100%-w-20`。这将在屏幕右/下边缘与窗口之间创建一个间隙，间隙大小为指定的减去值）。<br>此外，您还可以使用 `cursor [x] [y]`，其中 x 和 y 是像素或百分比。百分比基于窗口大小计算。在其他参数前指定 `onscreen` 以强制窗口进入屏幕（例如 `move onscreen cursor 50% 50%`） |
| size [w] [h] | 调整浮动窗口大小（`w, h` -> 整数或百分比，例如 `1280, 720` 或 `50%, 50%`。<br>`<` 和 `>` 也可以前置组合使用，分别指定允许的最大或最小大小。（例如 `<1280` 或 `<40%` -> 最大大小，`>300` 或 `>10%` -> 最小大小）。<br>注意：像素的整数值将根据显示器的缩放因子进行缩放。 |
| center ([opt]) | 如果窗口是浮动的，将其中心置于显示器上。将 opt 设置为 `1` 以尊重显示器保留区域。 |
| pseudo | 伪平铺窗口。 |
| monitor [id] | 设置窗口应打开的显示器。`id` 可以是 id 编号或名称（例如 `1` 或 `DP-1`）。 |
| workspace [w] | 设置窗口应打开的工作区（有关工作区语法，请参见 [调度器->工作区](../Dispatchers#工作区)）。<br>您还可以将 [w] 设置为 `unset`。这将取消应用于此窗口的所有先前工作区规则。此外，您可以在工作区后添加 `silent` 以使窗口静默打开。 |
| noinitialfocus | 禁用窗口的初始聚焦 |
| pin | 固定窗口（即在所有工作区显示）。_注意：仅限浮动窗口_。 |
| unset [rule] | 取消匹配 `PARAMETERS` 的规则（需要精确匹配）或特定 `RULE`。无规则默认为 `all`。 |
| nomaxsize | 移除最大大小限制。对于报告无效最大大小的窗口特别有用（例如 winecfg）。 |
| stayfocused | 只要窗口可见，就强制聚焦在该窗口上。 |
| group [options] | 设置窗口组属性。请参阅下方注释。 |
| suppressevent [types...] | 忽略来自窗口的特定事件。事件以空格分隔，可以是：`fullscreen`、`maximize`、`activate`、`activatefocus`、`fullscreenoutput`。 |
| content [none|photo|video|game] | 设置内容类型。 |
| noclosefor [ms] | 在打开时，使窗口在给定的毫秒数内无法通过 `killactive` 调度器关闭。 |

### 动态规则

动态规则在每次属性更改时重新评估。

| 规则 | 说明 |
| ---- | ----------- |
| animation [style] ([opt]) | 强制将动画应用于窗口，使用选定的 opt。Opt 是可选的。 |
| bordercolor [c] | 强制窗口的边框颜色。<br>c 的选项：`color`/`color ... color angle` -> 设置活动边框颜色/渐变 OR `color color`/`color ... color angle color ... color [angle]` -> 设置窗口的活动和非活动边框颜色/渐变。颜色定义请参见 [变量->颜色](../Variables#variable-types)。 |
| idleinhibit [mode] | 为窗口设置空闲抑制规则。如果激活，`hypridle` 等应用将不会触发。模式：`none`、`always`、`focus`、`fullscreen`。 |
| opacity [a] | 附加不透明度乘数。a 的选项：`float` -> 设置整体不透明度，`float float` -> 分别设置活动不透明度和非活动不透明度，`float float float` -> 分别设置活动不透明度、非活动不透明度和全屏不透明度。 |
| tag [name] | 将标签 `name` 应用于窗口，使用前缀 `+`/`-` 来设置/取消设置标志，或不使用前缀来切换标志。 |
| maxsize [w] [h] | 设置最大大小（x,y -> 整数）。 |
| minsize [w] [h] | 设置最小大小（x,y -> 整数）。|

以下规则也可以通过 [`setprop`](../Dispatchers#setprop) 设置：

| 规则 | 说明 |
| ---- | ----------- |
| bordersize [int] | 设置边框大小。 |
| rounding [int] | 强制应用程序具有 X 像素的圆角，忽略默认设置（在 `decoration:rounding` 中）。必须是整数。 |
| roundingpower [float] | 覆盖窗口的圆角功率（参见 `decoration:rounding_power`）。 |
| allowsinput [on] | 强制 XWayland 窗口接收输入，即使它请求不这样做。（可能解决某些游戏启动器因某种原因无法接收焦点的问题） |
| dimaround [on] | 使窗口周围的所有内容变暗。请注意，此规则适用于浮动窗口，将其用于平铺窗口可能会导致奇怪的行为。 |
| decorate [on] | 是否绘制窗口装饰 |
| focusonactivate [on] | Hyprland 是否应聚焦请求聚焦的应用程序（`activate` 请求）。 |
| keepaspectratio [on] | 用鼠标调整窗口大小时强制保持宽高比。 |
| nearestneighbor [on] | 强制窗口使用[最近邻](https://en.wikipedia.org/wiki/Image_scaling#Nearest-neighbor_interpolation)过滤。 |
| noanim [on] | 禁用窗口的动画。 |
| noblur [on] | 禁用窗口的模糊效果。 |
| noborder [on] | 禁用窗口的边框。 |
| nodim [on] | 禁用窗口变暗。 |
| nofocus [on] | 禁用窗口聚焦。 |
| nofollowmouse [on] | 当设置 `input:follow_mouse=1` 时，防止鼠标移动到其上时聚焦窗口。 |
| nomaxsize [on] | 禁用窗口的最大大小。 |
| norounding [on] | 禁用窗口圆角。 |
| noshadow [on] | 禁用窗口阴影。 |
| noshortcutsinhibit [on] | 不允许应用程序[抑制您的快捷键](https://wayland.app/protocols/keyboard-shortcuts-inhibit-unstable-v1)。 |
| opaque [on] | 强制窗口不透明。 |
| forcergbx [on] | 强制 Hyprland 忽略整个窗口表面的 alpha 通道，使其_实际上、完全 100% 不透明_。 |
| syncfullscreen [on] | 全屏模式是否应始终与发送给窗口的模式相同（仅在下一次全屏模式更改时生效）。 |
| immediate [on] | 强制窗口允许撕裂。参见[撕裂页面](../Tearing)。 |
| xray [on] | 为窗口设置模糊 X 射线模式。 |
| renderunfocused | 强制窗口认为它在不可见时正在渲染。另请参见 [变量 - 杂项](../Variables/#Misc) 了解设置 `render_unfocused_fps`。 |
| scrollmouse [float] | 强制窗口覆盖变量 `input:scroll_factor`。 |
| scrolltouchpad [float] | 强制窗口覆盖变量 `input:touchpad:scroll_factor`。 |
| noscreenshare [on] | 通过在其位置绘制黑色矩形，使窗口及其弹窗在屏幕共享中隐藏。即使其他窗口在上方，也会绘制矩形。 |
| novrr [on] | 为窗口禁用 VRR。仅当 [`misc:vrr`](../Variables/#Misc) 设置为 `2` 或 `3` 时才有效。 |

{{< callout type=info >}}

使用窗口规则时，[on] 可以设置为 `0` 表示_禁用_，`1` 表示_启用_，或留空使用默认值。

使用 `setprop` 时，[on] 可以设置为 `0` 表示_禁用_，`1` 表示_启用_，
`toggle` 表示切换状态，或 `unset` 表示取消先前值。

使用 `setprop` 时，[int] 也可以设置为 `unset` 以取消先前值。

{{< /callout >}}

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

{{< callout type=info >}}

没有选项的 `group` 规则是 `group set` 的简写。

默认情况下，`set` 和 `lock` 仅对新窗口生效一次。`always`
限定符使它们始终有效。

{{< /callout >}}

### 标签

窗口可能有多个标签，可以是静态的或动态的。动态标签将带有 `*` 后缀。
您可以使用 `hyprctl clients` 检查窗口标签。

使用 `tagwindow` 调度器向窗口添加静态标签：

```bash
hyprctl dispatch tagwindow +code     # 为当前窗口添加标签。
hyprctl dispatch tagwindow -- -code  # 从当前窗口移除标签（使用 `--` 保护前导 `-`）。
hyprctl dispatch tagwindow code      # 切换当前窗口的标签。

# 或者您可以使用窗口正则表达式标记匹配的窗口：
hyprctl dispatch tagwindow +music deadbeef
hyprctl dispatch tagwindow +media title:Celluloid
```

使用 `tag` 规则向窗口添加动态标签：

```ini
windowrule = tag +term, class:footclient  # 为窗口 footclient 添加动态标签 `term*`。
windowrule = tag term, class:footclient   # 切换窗口 footclient 的动态标签 `term*`。
windowrule = tag +code, tag:cpp           # 为具有标签 `cpp` 的窗口添加动态标签 `code*`。

windowrule = opacity 0.8, tag:code        # 为具有标签 `code` 或 `code*` 的窗口设置不透明度。
windowrule = opacity 0.7, tag:cpp         # 具有标签 `cpp` 的窗口将同时匹配 `code` 和 `cpp`，后者将覆盖先前的匹配。
windowrule = opacity 0.6, tag:term*       # 仅为具有标签 `term*` 的窗口设置不透明度，`term` 不会匹配。

windowrule = tag -code, tag:term          # 为具有标签 `term` 或 `term*` 的窗口移除动态标签 `code*`。
```

或者使用键绑定以方便使用：

```ini
bind = $mod Ctrl, 2, tagwindow, alpha_0.2
bind = $mod Ctrl, 4, tagwindow, alpha_0.4

windowrule = opacity 0.2 override, tag:alpha_0.2
windowrule = opacity 0.4 override, tag:alpha_0.4
```

`tag` 规则只能操作动态标签，而 `tagwindow` 调度器仅适用于静态标签（即调用调度器后，动态标签将被清除）。

### 示例规则

```ini
windowrule = move 100 100, class:kitty                                    # 将 kitty 移动到 100 100
windowrule = animation popin, class:kitty                                 # 为 kitty 设置动画样式
windowrule = noblur, class:firefox                                        # 为 firefox 禁用模糊
windowrule = move cursor -50% -50%, class:kitty                           # 将 kitty 移动到光标中心
windowrule = bordercolor rgb(FF0000) rgb(880808), fullscreen:1            # 如果窗口全屏，则将边框颜色设置为红色
windowrule = bordercolor rgb(00FF00), fullscreenstate:* 1                 # 如果窗口的客户端全屏状态为 1（最大化），则将边框颜色设置为绿色（内部状态可以是任意值）
windowrule = bordercolor rgb(FFFF00), title:.*Hyprland.*                  # 当标题包含 Hyprland 时，将边框颜色设置为黄色
windowrule = opacity 1.0 override 0.5 override 0.8 override, class:kitty  # 为 kitty 设置活动不透明度为 1.0，非活动不透明度为 0.5，全屏不透明度为 0.8
windowrule = rounding 10, class:kitty                                     # 为 kitty 设置圆角为 10
windowrule = stayfocused,  class:(pinentry-)(.*)                          # 修复 pinentry 失去焦点的问题
```

### 注意事项

标记为_动态_的规则将在窗口的匹配属性更改时重新评估。<br>
例如，如果定义了一个规则，在窗口浮动时更改其`bordercolor`，则当设置为浮动时，`bordercolor`将更改为请求的颜色，当再次平铺时恢复为默认颜色。

规则将从上到下处理，_最后一个_匹配将优先。即：

```ini
windowrule = opacity 0.8 0.8, class:kitty
windowrule = opacity 0.5 0.5, floating:1
```

在这里，所有非全屏的 kitty 窗口将具有 `opacity 0.8`，除非它们是浮动的。否则，它们将具有 `opacity 0.5`。其余非全屏浮动窗口将具有 `opacity 0.5`。

```ini
windowrule = opacity 0.5 0.5,floating:1
windowrule = opacity 0.8 0.8,class:kitty
```

在这里，所有 kitty 窗口将具有 `opacity 0.8`，即使它们是浮动的。
其余浮动窗口将具有 `opacity 0.5`。

{{< callout type=info >}}

默认情况下，不透明度是所有不透明度的乘积。例如，将
`activeopacity` 设置为 `0.5` 并将 `opacity` 设置为 `0.5` 将导致总不透明度为
`0.25`。<br>
允许将不透明度设置为超过 `1.0`，但任何超过 `1.0` 的不透明度乘积将导致图形故障。<br>
例如，使用 `0.5 * 2 = 1` 没问题，但 `0.5 * 4 = 2` 将导致图形故障。<br>
您可以在不透明度值后放置 `override` 以覆盖为精确值而不是乘数。
例如，要将活动和非活动不透明度设置为 0.8，并使全屏窗口完全不透明，不受其他不透明度规则影响：

```ini
windowrule = opacity 0.8 override 0.8 override 1.0 override, class:kitty
```

{{< /callout >}}

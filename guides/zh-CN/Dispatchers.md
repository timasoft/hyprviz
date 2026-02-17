---
weight: 6
title: 调度器
---

> [!NOTE]
> 由 Qwen3.5-Plus 从 en 翻译

请注意，某些特定布局的调度器将在布局页面中列出（请参阅侧边栏）。

## 参数说明

| 参数类型 | 说明 |
| --- | --- |
| window | 一个窗口。可以是以下任意一种：类正则表达式（默认，可选`class:`）、`initialclass:` 初始类正则表达式、`title:` 标题正则表达式、`initialtitle` 初始标题正则表达式、`tag:` 窗口标签正则表达式、`pid:` 进程 ID、`address:` 地址、`activewindow` 活动窗口、`floating` 当前工作区的第一个浮动窗口、`tiled` 当前工作区的第一个平铺窗口 |
| workspace | 见[下文]({{< relref "#workspaces" >}})。 |
| direction | `l` `r` `u` `d` 左 右 上 下 |
| monitor | 以下之一：方向、ID、名称、`current`、相对值（例如 `+1` 或 `-1`） |
| resizeparams | 相对像素增量 vec2（例如 `10 -10`），可选窗口大小的百分比（例如 `20 25%`）或`exact`后跟精确 vec2（例如 `exact 1280 720`），可选屏幕大小的百分比（例如 `exact 50% 50%`） |
| floatvalue | 相对浮点增量（例如 `-0.2` 或 `+0.2`）或`exact`后跟精确浮点值（例如 `exact 0.5`） |
| zheight | `top` 或 `bottom` |
| mod | `SUPER`, `SUPER_ALT` 等。 |
| key | `g`, `code:42`, `42` 或鼠标点击（`mouse:272`） |

## 调度器列表

| 调度器 | 说明 | 参数 |
| --- | --- | --- |
| exec | 执行 shell 命令 | 命令（支持规则，参见[下方]({{< relref "#executing-with-rules" >}})） |
| execr | 执行原始 shell 命令（不支持规则） | 命令 |
| pass | 将按键（带修饰键）传递给指定窗口。可用于解决 Wayland 上全局键绑定不工作的问题。 | window |
| sendshortcut | 将指定按键（带修饰键）发送给可选的指定窗口。功能类似于 pass | mod, key[, window] |
| sendkeystate | 将具有特定状态（按下/重复/释放）的按键发送给指定窗口（窗口必须保持焦点以继续接收事件）。 | mod, key, state, window |
| killactive | 关闭（非杀死）活动窗口 | 无 |
| forcekillactive | 杀死活动窗口 | 无 |
| closewindow | 关闭指定窗口 | window |
| killwindow | 杀死指定窗口 | window |
| signal | 向活动窗口发送信号 | signal |
| signalwindow | 向指定窗口发送信号 | `window,signal`，例如`class:Alacritty,9` |
| workspace | 更改工作区 | workspace |
| movetoworkspace | 将焦点窗口移动到工作区 | workspace 或 `workspace,window` 用于指定特定窗口 |
| movetoworkspacesilent | 同上，但不切换到该工作区 | workspace 或 `workspace,window` 用于指定特定窗口 |
| togglefloating | 切换当前窗口的浮动状态 | 留空 / `active` 表示当前窗口，或 `window` 表示特定窗口 |
| setfloating | 将当前窗口的浮动状态设置为 true | 留空 / `active` 表示当前窗口，或 `window` 表示特定窗口 |
| settiled | 将当前窗口的浮动状态设置为 false | 留空 / `active` 表示当前窗口，或 `window` 表示特定窗口 |
| fullscreen | 设置焦点窗口的全屏模式 | `mode action`，其中 mode 可以是 0 - 全屏（占据整个屏幕）或 1 - 最大化（保留间隙和栏），而 action 是可选的，可以是 `toggle`（默认）、`set` 或 `unset`。 |
| fullscreenstate | 设置焦点窗口的全屏模式及发送给客户端的模式 | `internal client action`，其中 internal（Hyprland 窗口）和 client（应用程序）可以是 `-1` - 当前，`0` - 无，`1` - 最大化，`2` - 全屏，`3` - 最大化和全屏。action 是可选的，可以是 `toggle`（默认）或 `set`。 |
| dpms | 设置所有显示器的 DPMS 状态。不要直接与键绑定一起使用。 | `on`, `off`, 或 `toggle`。对于特定显示器，在空格后添加显示器名称 |
| forceidle | 设置所有空闲计时器的经过时间，忽略空闲抑制器。计时器在下次活动时恢复正常行为。不要直接与键绑定一起使用。 | floatvalue（秒数） |
| pin | 固定窗口（即在所有工作区显示）_注意：仅限浮动窗口_ | 留空 / `active` 表示当前窗口，或 `window` 表示特定窗口 |
| movefocus | 在指定方向移动焦点 | direction |
| movewindow | 在指定方向移动活动窗口或将窗口移动到显示器。对于浮动窗口，将窗口移动到该方向的屏幕边缘 | direction 或 `mon:` 后跟显示器，可选后跟空格和 `silent` 以防止焦点随窗口移动 |
| swapwindow | 交换活动窗口与给定方向的另一个窗口或与特定窗口 | direction 或 `window` |
| centerwindow | 将活动窗口居中 _注意：仅限浮动窗口_ | 无（用于显示器中心）或 1（以尊重显示器保留区域） |
| resizeactive | 调整活动窗口大小 | resizeparams |
| moveactive | 移动活动窗口 | resizeparams |
| resizewindowpixel | 调整选定窗口大小 | `resizeparams,window`，例如 `100 100,^(kitty)$` |
| movewindowpixel | 移动选定窗口 | `resizeparams,window` |
| cyclenext | 聚焦下一个窗口（在工作区上，如果未提供`visible`） | 无（表示下一个）或 `prev`（表示上一个），另外 `tiled` 仅限平铺窗口，`floating` 仅限浮动窗口。`prev tiled` 是有效的。`visible` 用于所有显示器循环。`visible prev floating` 是有效的。如果提供 `hist` 参数 - 焦点顺序将取决于焦点历史。所有其他修饰符也适用于此，`visible next floating hist` 是有效的。 |
| swapnext | 将焦点窗口与工作区上的下一个窗口交换 | 无（表示下一个）或 `prev`（表示上一个） |
| tagwindow | 将标签应用于当前或第一个匹配的窗口 | `tag [window]`，例如 `+code ^(foot)$`, `music` |
| focuswindow | 聚焦第一个匹配的窗口 | window |
| focusmonitor | 聚焦显示器 | monitor |
| splitratio | 更改分割比例 | floatvalue |
| movecursortocorner | 将光标移动到活动窗口的角落 | direction, 0 - 3, 左下 - 0, 右下 - 1, 右上 - 2, 左上 - 3 |
| movecursor | 将光标移动到指定位置 | `x y` |
| renameworkspace | 重命名工作区 | `id name`，例如 `2 work` |
| exit | 退出合成器，不询问任何问题。 | 无 |
| forcerendererreload | 强制渲染器重新加载所有资源和输出 | 无 |
| movecurrentworkspacetomonitor | 将活动工作区移动到显示器 | monitor |
| focusworkspaceoncurrentmonitor | 聚焦当前显示器上的请求工作区，必要时将当前工作区交换到其他显示器。如果您想要 XMonad/Qtile 风格的工作区切换，请在配置中将`workspace`替换为此项。 | workspace |
| moveworkspacetomonitor | 将工作区移动到显示器 | workspace 和 monitor 用空格分隔 |
| swapactiveworkspaces | 交换两个显示器之间的活动工作区 | 两个显示器用空格分隔 |
| bringactivetotop | _已弃用_，建议使用 alterzorder。将当前窗口带到堆栈顶部 | 无 |
| alterzorder | 修改活动或指定窗口的窗口堆栈顺序。注意：不能用于将浮动窗口移到平铺窗口后面。 | zheight[,window] |
| togglespecialworkspace | 切换特殊工作区的开关 | 无（表示第一个）或名称（表示命名的，名称必须是特殊工作区的名称） |
| focusurgentorlast | 聚焦紧急窗口或上一个窗口 | 无 |
| togglegroup | 将当前活动窗口切换到组中 | 无 |
| changegroupactive | 切换到组中的下一个窗口。 | b - 后退，f - 前进，或从 1 开始的索引 |
| focuscurrentorlast | 从当前焦点切换到上一个焦点窗口 | 无 |
| lockgroups | 锁定组（所有组将不接受新窗口） | `lock` 表示锁定，`unlock` 表示解锁，`toggle` 表示切换 |
| lockactivegroup | 锁定聚焦组（当前组将不接受新窗口或移动到其他组） | `lock` 表示锁定，`unlock` 表示解锁，`toggle` 表示切换 |
| moveintogroup | 将活动窗口移动到指定方向的组中。如果指定方向没有组，则无操作。 | direction |
| moveoutofgroup | 将活动窗口移出组。如果不在组中，则无操作 | 留空 / `active` 表示当前窗口，或 `window` 表示特定窗口 |
| movewindoworgroup | 如果给定方向有组，则行为类似于`moveintogroup`。如果相对于活动组的给定方向没有组，则行为类似于`moveoutofgroup`。否则行为类似于`movewindow`。 | direction |
| movegroupwindow | 交换活动窗口与组中的下一个或上一个窗口 | `b` 表示后退，其他表示前进 |
| denywindowfromgroup | 禁止活动窗口成为组或被插入到组中 | `on`, `off` 或 `toggle` |
| setignoregrouplock | 临时启用或禁用 binds:ignore_group_lock | `on`, `off`, 或 `toggle` |
| global | 使用 GlobalShortcuts 门户执行全局快捷键。参见[此处](../Binds/#global-keybinds) | name |
| submap | 更改当前映射组。参见[子映射](../Binds/#submaps) | `reset` 或名称 |
| event | 以`custom>>yourdata`形式向 socket2 发出自定义事件 | 要发送的数据 |
| setprop | 设置窗口属性 | `window property value` |
| toggleswallow | 如果窗口被聚焦窗口吞没，则取消吞没。再次执行以重新吞没 | 无 |

> [!WARNING]
> [uwsm](../../Useful-Utilities/Systemd-start) 用户应避免使用 `exit` 调度器，或直接终止 Hyprland 进程，因为以这种方式退出 Hyprland 会将其从客户端下移除并干扰有序关机序列。使用 `exec, uwsm stop`（或[其他变体](https://github.com/Vladimir-csp/uwsm#how-to-stop)），这将优雅地关闭图形会话（以及绑定的登录会话，如果有的话）。如果您遇到单元进入不一致状态的问题，影响后续会话，请改用 `exec, loginctl terminate-user ""`（终止用户的所有单元）。
>
> 同样强烈建议相应地替换 `hyprland.conf` 键绑定部分中的 `exit` 调度器。

> [!WARNING]
> **不建议**直接通过键绑定设置 DPMS 或 forceidle，因为这可能会导致未定义行为。相反，考虑使用类似以下的方式
>
> ```ini
> bind = MOD, KEY, exec, sleep 1 && hyprctl dispatch dpms off
> ```

### 分组（选项卡式）窗口

Hyprland 允许您使用 `togglegroup` 键绑定调度器从当前活动窗口创建组。

组类似于 i3wm 的"选项卡式"容器。它占用一个窗口的空间，您可以使用 `changegroupactive` 键绑定调度器将窗口更改为选项卡式"组"中的下一个窗口。

新组的边框颜色可以通过`group`配置部分中的相应`col.`设置进行配置。

您可以使用 `lockactivegroup` 调度器锁定组，以阻止新窗口进入此组。此外，`lockgroups` 调度器可用于切换独立的全局组锁定，这将阻止新窗口进入任何组，无论其本地组锁定状态如何。

您可以使用 `denywindowfromgroup` 调度器阻止窗口添加到组中或成为组。如果当前活动窗口或方向上的窗口设置了此属性，则 `movewindoworgroup` 将表现得像 `movewindow`。

## 工作区

您有九种选择：

- ID：例如 `1`, `2`, 或 `3`

- 相对 ID：例如 `+1`, `-3` 或 `+100`

- 显示器上的工作区，相对值使用 `+` 或 `-`，绝对值使用 `~`：例如 `m+1`、`m-2` 或 `m~3`

- 包含空工作区的显示器上的工作区，相对值使用 `+` 或 `-`，绝对值使用 `~`：例如 `r+1` 或 `r~3`

- 打开的工作区，相对值使用 `+` 或 `-`，绝对值使用 `~`：例如 `e+1`、`e-10` 或 `e~2`

- 名称：例如 `name:Web`, `name:Anime` 或 `name:Better anime`

- 上一个工作区：`previous`，或 `previous_per_monitor`

- 第一个可用的空工作区：`empty`，后缀 `m` 表示仅在显示器上搜索，和/或 `n` 表示下一个可用的空工作区。例如 `emptynm`

- 特殊工作区：`special` 或 `special:name` 用于命名的特殊工作区。

> [!WARNING]
> `special` 仅在 `movetoworkspace` 和 `movetoworkspacesilent` 上受支持。
> 其他任何调度器都会导致未记录的行为。

> [!WARNING]
> 数字工作区（例如 `1`, `2`, `13371337`）**仅**允许在 1 到 2147483647（含）之间
> 不允许使用 `0` 或负数。

## 特殊工作区

特殊工作区在其他地方被称为"便签板"。一个可以在任何显示器上切换开关的工作区。

> [!NOTE]
> 您可以定义多个命名的特殊工作区，但同时这些工作区的数量限制为 97 个。

例如，要将窗口/应用程序移动到特殊工作区，您可以使用以下语法：

```ini
bind = SUPER, C, movetoworkspace, special
# 上面的语法将在按下'SUPER'+'C'时将窗口移动到特殊工作区。
# 要查看隐藏的窗口，可以使用上面提到的 togglespecialworkspace 调度器。
```

## 使用规则执行

`exec` 调度器支持添加规则。请注意，某些窗口可能工作得更好，某些可能更差。它会记录生成进程的 PID 并使用它。
例如，如果您的进程分叉然后分叉打开一个窗口，这将不起作用。

语法是：

```ini
bind = mod, key, exec, [rules...] command
```

例如：

```ini
bind = SUPER, E, exec, [workspace 2 silent; float; move 0 0] kitty
```

### setprop

属性列表：

| 属性 | 说明 |
| --- | --- |
| alpha | 浮点数 0.0 - 1.0 |
| alphaoverride | 0/1，使下一个设置成为覆盖而不是乘法 |
| alphainactive | 浮点数 0.0 - 1.0 |
| alphainactiveoverride | 0/1，使下一个设置成为覆盖而不是乘法 |
| alphafullscreen | 浮点数 0.0 - 1.0 |
| alphafullscreenoverride | 0/1，使下一个设置成为覆盖而不是乘法 |
| animationstyle | 字符串，不能锁定 |
| activebordercolor | 渐变，-1 表示未设置 |
| inactivebordercolor | 渐变，-1 表示未设置 |
| maxsize | vec2 (`x y`) |
| minsize | vec2 (`x y`) |

可以在[窗口规则](../Window-Rules#dynamic-rules)部分找到更多属性。

例如：

```sh
address:0x13371337 noanim 1
address:0x13371337 nomaxsize 0
address:0x13371337 opaque toggle
address:0x13371337 immediate unset
address:0x13371337 bordersize relative -2
address:0x13371337 roundingpower relative 0.1
```

### Fullscreenstate

`fullscreenstate internal client`

`fullscreenstate` 调度器将 Hyprland 为窗口维护的状态与传达给客户端的全屏状态解耦。

`internal` 是指 Hyprland 维护的状态。

`client` 是指应用程序接收的状态。

| 值 | 状态 | 说明 |
| --- | --- | --- |
| -1 | 当前 | 维持当前全屏状态。 |
| 0 | 无 | 窗口分配当前布局定义的空间。 |
| 1 | 最大化 | 窗口占据整个工作空间，保留边距。 |
| 2 | 全屏 | 窗口占据整个屏幕。 |
| 3 | 最大化和全屏 | 全屏最大化窗口的状态。与全屏工作方式相同。 |

例如：

`fullscreenstate 2 0` 使应用程序全屏，但客户端保持非全屏模式。

这可用于防止基于 Chromium 的浏览器在检测到已全屏时进入演示模式。

`fullscreenstate 0 2` 保持窗口非全屏，但客户端在窗口内进入全屏模式。

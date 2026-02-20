---
weight: 4
title: 显示器
---

> [!NOTE]
> 翻译自英文版，使用 Qwen3.5-Plus

## 常规

显示器的常规配置格式如下：

```ini
monitor = 名称，分辨率，位置，缩放比例
```

一个常见的示例：

```ini
monitor = DP-1, 1920x1080@144, 0x0, 1
```

这将使 `DP-1` 上的显示器成为 `1920x1080` 显示，位于
144Hz，`0x0` 偏移自左上角，缩放比例为 1（无缩放）。

要列出所有可用显示器（活动和非活动）：

```bash
hyprctl monitors all
```

显示器定位在虚拟“布局”上。`position` 是该显示器在布局中的位置，
以像素为单位。（从左上角计算）

例如：

```ini
monitor = DP-1, 1920x1080, 0x0, 1
monitor = DP-2, 1920x1080, 1920x0, 1
```

将告诉 Hyprland 将 DP-1 放在 DP-2 的_左侧_，而

```ini
monitor = DP-1, 1920x1080, 1920x0, 1
monitor = DP-2, 1920x1080, 0x0, 1
```

将告诉 Hyprland 将 DP-1 放在_右侧_。

`position` 可以包含_负_值，所以上面的示例也可以
写成

```ini
monitor = DP-1, 1920x1080, 0x0, 1
monitor = DP-2, 1920x1080, -1920x0, 1
```

Hyprland 使用反向 Y 轴笛卡尔坐标系。因此，负 y 坐标
会使显示器位置更高，而正 y 坐标会使它位置更低。

例如：

```ini
monitor = DP-1, 1920x1080, 0x0, 1
monitor = DP-2, 1920x1080, 0x-1080, 1
```

将告诉 Hyprland 将 DP-2 放在 DP-1 _上方_，而

```ini
monitor = DP-1, 1920x1080, 0x0, 1
monitor = DP-2, 1920x1080, 0x1080, 1
```

将告诉 Hyprland 将 DP-2 放在_下方_。

> [!NOTE]
> 位置是根据缩放（和转换）后的分辨率计算的，这意味着
> 如果您想将 4K 显示器（缩放比例为 2）放在 1080p 显示器的左侧，您会
> 对第二个屏幕使用位置 `1920x0`（3840 / 2）。如果显示器
> 还旋转了 90 度（垂直），您会使用 `1080x0`。

> [!WARNING]
> 显示器不能重叠。这意味着如果您设置的位置使任何显示器
> 重叠，您将收到警告。

将名称留空将定义一个回退规则，当没有其他规则
匹配时使用。

分辨率有几个特殊值：

- `preferred` - 使用显示器的首选尺寸和刷新率。
- `highres` - 使用最高支持的分辨率。
- `highrr` - 使用最高支持的刷新率。
- `maxwidth` - 使用最宽支持的分辨率。

位置也有几个特殊值：

- `auto` - 让 Hyprland 决定位置。默认情况下，它将每个新显示器放在现有显示器的右侧，
  使用显示器的左上角作为根点。
- `auto-right/left/up/down` - 将显示器放在其他显示器的右侧/左侧、上方或下方，
  同样基于每个显示器的左上角作为根点。
- `auto-center-right/left/up/down` - 将显示器放在其他显示器的右侧/左侧、上方或下方，
  但计算位置时使用每个显示器的中心而不是左上角。

_**请注意：**_ 虽然允许为第一个显示器指定显示器方向，但这没有效果，它将
被定位在 (0,0)。此外，方向总是从中心向外，所以您可以指定 `auto-up` 然后 `auto-left`，
但左侧的显示器将仅位于原点左侧并在原点上方。您也可以指定重复的方向，并且
显示器将继续沿该方向排列。

您也可以使用 `auto` 作为缩放比例，让 Hyprland 为您决定缩放比例。
这些取决于显示器的 PPI。

快速插入随机显示器的推荐规则：

```ini
monitor = , preferred, auto, 1
```

这将使任何未通过显式规则指定的显示器自动
放置在其他显示器的右侧，并使用其首选
分辨率。

对于更具体的规则，您还可以使用输出的描述（参见
`hyprctl monitors` 获取更多详细信息）。如果 `hyprctl monitors` 的输出
如下所示：

```yaml
Monitor eDP-1 (ID 0):
        1920x1080@60.00100 at 0x0
        description: Chimei Innolux Corporation 0x150C (eDP-1)
        make: Chimei Innolux Corporation
        model: 0x150C
        [...]
```

那么 `description` 值（不包括端口名称 `(eDP-1)`）可以
用于指定显示器：

```ini
monitor = desc:Chimei Innolux Corporation 0x150C, preferred, auto, 1.5
```

记得删除 `(portname)`！

### 自定义模型线

您可以通过将分辨率字段更改为模型线来设置自定义模型线，
例如：

```ini
monitor = DP-1, modeline 1071.101 3840 3848 3880 3920 2160 2263 2271 2277 +hsync -vsync, 0x0, 1
```

### 禁用显示器

要禁用显示器，请使用

```ini
monitor = 名称，disable
```

> [!WARNING]
> 禁用显示器将字面意义上将其从布局中移除，移动所有窗口
> 和工作区到任何剩余的显示器上。如果您想以屏保样式禁用显示器（仅关闭显示器），请使用 `dpms`
> [调度器](../Dispatchers)。

## 自定义保留区域

保留区域是平铺窗口不会占用的区域。
如果您的工作流程需要自定义保留区域，您可以添加它：

```ini
monitor = 名称，addreserved, TOP, BOTTOM, LEFT, RIGHT
```

其中 `TOP` `BOTTOM` `LEFT` `RIGHT` 是整数，即要添加的保留区域的像素数。这会叠加在计算出的保留区域
（例如任务栏）之上，但您在配置中每个显示器只能使用一个此类规则。

## 额外参数

您可以在显示器规则末尾组合额外参数，示例：

```ini
monitor = eDP-1, 2880x1800@90, 0x0, 1, transform, 1, mirror, DP-2, bitdepth, 10
```

参见下文以获取每个参数的更多详细信息。

### 镜像显示器

如果您想镜像显示器，在显示器规则末尾添加 `, mirror, <NAME>`，示例：

```ini
monitor = DP-3, 1920x1080@60, 0x0, 1, mirror, DP-2
monitor = , preferred, auto, 1, mirror, DP-1
```

请记住，镜像显示器不会为第二个显示器“重新渲染”所有内容，所以
如果将 1080p 屏幕镜像到 4K 屏幕上，分辨率
在 4K 显示器上仍为 1080p。这也意味着在不同宽高比（例如 16:9 和 16:10）上会发生
挤压和拉伸。

### 10 位支持

如果您想为显示器启用 10 位支持，在显示器规则末尾添加 `, bitdepth, 10`，
例如：

```ini
monitor = eDP-1, 2880x1800@90, 0x0, 1, bitdepth, 10
```

> [!WARNING]
> 在 Hyprland 中注册的颜色（例如边框颜色）_不_支持
> 10 位。
> 某些应用程序_不_支持在启用 10 位时进行屏幕捕获。

### 颜色管理预设

添加 `, cm, X` 以更改默认 sRGB 输出预设

```ini
monitor = eDP-1, 2880x1800@90, 0x0, 1, bitdepth, 10, cm, wide
```

```plain
auto    - 8bpc 使用 srgb，如果支持 10bpc 则使用 wide（推荐）
srgb    - sRGB 原色（默认）
dcip3   - DCI P3 原色
dp3     - Apple P3 原色
adobe   - Adobe RGB 原色
wide    - 广色域，BT2020 原色
edid    - 来自 edid 的原色（已知不准确）
hdr     - 广色域和 HDR PQ 传输函数（实验性）
hdredid - 与 hdr 相同，但使用 edid 原色（实验性）
```

如果启用了 `render:cm_fs_passthrough`，则无需 hdr `cm` 设置即可实现全屏 HDR。

使用 `sdrbrightness, B` 和 `sdrsaturation, S` 控制 HDR 模式下的 SDR 亮度和饱和度。两个值的默认值均为 `1.0`。典型亮度值应在 `1.0 ... 2.0` 范围内。

```ini
monitor = eDP-1, 2880x1800@90, 0x0, 1, bitdepth, 10, cm, hdr, sdrbrightness, 1.2, sdrsaturation, 0.98
```

SDR 显示器上 sRGB 内容默认使用的传输函数由 `, sdr_eotf, X` 定义。默认值（`0`）是遵循 `render:cm_sdr_eotf`。这可以更改为分段 sRGB（`1`），或 Gamma 2.2（`2`）。

### VRR

每显示器 VRR 可以通过添加 `, vrr, X` 完成，其中 `X` 是 [变量页面](../Variables) 中的模式。

## 旋转

如果您想旋转显示器，在显示器规则末尾添加 `, transform, X`，其中 `X` 对应转换编号，例如：

```ini
monitor = eDP-1, 2880x1800@90, 0x0, 1, transform, 1
```

转换列表：

```plain
0 -> 正常（无转换）
1 -> 90 度
2 -> 180 度
3 -> 270 度
4 -> 翻转
5 -> 翻转 + 90 度
6 -> 翻转 + 180 度
7 -> 翻转 + 270 度
```

## 显示器 v2

替代语法。`monitor = DP-1,1920x1080@144,0x0,1,transform,2` 等同于

```ini
monitorv2 {
  output = DP-1
  mode = 1920x1080@144
  position = 0x0
  scale = 1
  transform = 2
}
```

`disable` 标志变为 `disabled = true`，但其他命名设置保持其名称：`名称，值` &rarr; `名称 = 值`（例如 `bitdepth,10` &rarr; `bitdepth = 10`）

EDID 覆盖和 SDR &rarr; HDR 设置：

| 名称 | 说明 | 类型 |
|---|---|---|
| supports_wide_color | 强制广色域支持（0 - 自动，1 - 强制开启，-1 - 强制关闭） | int |
| supports_hdr | 强制 HDR 支持。需要广色域（0 - 自动，1 - 强制开启，-1 - 强制关闭） | int |
| sdr_min_luminance | 用于 SDR &rarr; HDR 映射的 SDR 最小亮度。设置为 0.005 以匹配 HDR 黑色的真正黑色 | float |
| sdr_max_luminance | SDR 最大亮度。可用于调整整体 SDR &rarr; HDR 亮度。80 - 400 是合理范围。所需值可能在 200 和 250 之间 | int |
| min_luminance | 显示器的最小亮度 | float |
| max_luminance | 显示器的最大可能亮度 | int |
| max_avg_luminance | 显示器在典型帧上的平均最大亮度 | int |

注意：如果显示器固件缺乏某些安全检查，这些值可能会传递给显示器本身并导致增加烧屏或其他损坏。

## 默认工作区

参见 [工作区规则](../Workspace-Rules)。

### 将工作区绑定到显示器

参见 [工作区规则](../Workspace-Rules)。

---
weight: 9
title: 动画
---

{{< callout type=info >}}

Translated from en by qwen3

{{</ callout >}}

## 基本信息

动画通过 `animation` 关键字进行声明。

```ini
animation = 名称, 开关, 速度, 曲线 [,样式]
```

`开关` 使用 `0` 表示禁用，`1` 表示启用。_注意：_ 如果为 `0`，可以省略后续参数。

`速度` 是动画将持续的时间（以 ds 为单位，1ds = 100ms）。

`曲线` 是贝塞尔曲线名称，参见[曲线](#曲线)部分。

`样式` (可选) 是动画样式。

动画是以树形结构组织的。如果某个动画未设置，它将继承其父级的值。参见[动画树](#动画树)。

### 示例

```ini
animation = workspaces, 1, 8, default
animation = windows, 1, 10, myepiccurve, slide
animation = fade, 0
```

### 动画树

```txt
全局
  ↳ 窗口 - 样式: slide, popin, gnomed
    ↳ windowsIn - 窗口打开 - 样式: 与窗口相同
    ↳ windowsOut - 窗口关闭 - 样式: 与窗口相同
    ↳ windowsMove - 中间过程，移动、拖动、调整大小
  ↳ 图层 - 样式: slide, popin, fade
    ↳ layersIn - 图层打开
    ↳ layersOut - 图层关闭
  ↳ 淡入淡出
    ↳ fadeIn - 窗口打开时的淡入效果
    ↳ fadeOut - 窗口关闭时的淡出效果
    ↳ fadeSwitch - 切换活动窗口及其不透明度时的淡入淡出效果
    ↳ fadeShadow - 切换活动窗口时阴影的淡入淡出效果
    ↳ fadeDim - 非活动窗口变暗的缓动效果
    ↳ fadeLayers - 控制图层的淡入淡出效果
      ↳ fadeLayersIn - 图层打开时的淡入效果
      ↳ fadeLayersOut - 图层关闭时的淡出效果
    ↳ fadePopups - 控制 Wayland 弹窗的淡入淡出效果
      ↳ fadePopupsIn - Wayland 弹窗打开时的淡入效果
      ↳ fadePopupsOut - Wayland 弹窗关闭时的淡出效果
    ↳ fadeDpms - 控制切换 dpms 时的淡入淡出效果
  ↳ 边框 - 用于动画化边框颜色切换速度
  ↳ borderangle - 用于动画化边框渐变角度 - 样式: once (默认), loop
  ↳ 工作区 - 样式: slide, slidevert, fade, slidefade, slidefadevert
    ↳ workspacesIn - 样式: 与工作区相同
    ↳ workspacesOut - 样式: 与工作区相同
    ↳ specialWorkspace - 样式: 与工作区相同
      ↳ specialWorkspaceIn - 样式: 与工作区相同
      ↳ specialWorkspaceOut - 样式: 与工作区相同
  ↳ zoomFactor - 动画化屏幕缩放
  ↳ monitorAdded - 显示器添加时的缩放动画
```

{{< callout type=warning >}}

对 `borderangle` 使用 `loop` 样式要求 Hyprland _持续_ 以与屏幕刷新率相等的频率渲染新帧（例如，对于 60Hz 显示器，每秒 60 次），这可能会给 CPU/GPU 带来压力并影响电池寿命。<br>
即使动画被禁用或边框不可见，此设置仍会生效。

{{</ callout >}}

## 曲线

通过 `bezier` 关键字可以定义自己的[贝塞尔曲线](https://en.wikipedia.org/wiki/B%C3%A9zier_curve)：

```ini
bezier = 名称, X0, Y0, X1, Y1
```

其中 `名称` 是您选择的名称，`X0, Y0, X1, Y1` 是三次贝塞尔曲线的两个控制点。<br>
设计自己的贝塞尔曲线可以使用[cssportal.com](https://www.cssportal.com/css-cubic-bezier-generator/)网站。<br>
如果您想从预设的贝塞尔曲线中选择，可以查看[easings.net](https://easings.net)。

### 示例

```ini
bezier = overshoot, 0.05, 0.9, 0.1, 1.1
```

### 额外设置

对于 `windows` 中的动画样式 `popin`，您可以指定最小百分比作为起始值。例如，以下设置将使动画从 80% -> 100% 的大小变化：

```ini
animation = windows, 1, 8, default, popin 80%
```

对于 `workspaces` 中的动画样式 `slide`、`slidevert`、`slidefade` 和 `slidefadevert`，您可以指定移动百分比。例如，以下设置将使窗口移动屏幕宽度的 20%：

```ini
animation = workspaces, 1, 8, default, slidefade 20%
```

对于 `windows` 和 `layers` 中的 `slide` 动画样式，您可以指定强制方向。<br>
您可以选择 `top`（顶部）、`bottom`（底部）、`left`（左侧）或 `right`（右侧）。

```ini
animation = windows, 1, 8, default, slide left
```

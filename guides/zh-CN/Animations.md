---
weight: 9
title: 动画
---

> [!NOTE]
> 由 Qwen3.5-Plus 从 en 翻译

## 一般

动画通过 `animation` 关键字声明。

```ini
animation = NAME, ONOFF, SPEED, CURVE [,STYLE]
```

`ONOFF` 使用 `0` 禁用，`1` 启用。_注意：_ 如果为 `0`，您
可以省略后续参数。

`SPEED` 是动画将持续的 ds 数量（1ds = 100ms）。

`CURVE` 是贝塞尔曲线名称，参见 [曲线](#curves)。

`STYLE`（可选）是动画样式。

动画是以树形结构组织的。如果某个动画未设置，它将继承其
父级的值。参见 [动画树](#animation-tree)。

### 示例

```ini
animation = workspaces, 1, 8, default
animation = windows, 1, 10, myepiccurve, slide
animation = fade, 0
```

### 动画树

```txt
global
  ↳ windows - 样式：slide, popin, gnomed
    ↳ windowsIn - 窗口打开 - 样式：与 windows 相同
    ↳ windowsOut - 窗口关闭 - 样式：与 windows 相同
    ↳ windowsMove - 中间所有过程，移动、拖动、调整大小。
  ↳ layers - 样式：slide, popin, fade
    ↳ layersIn - 图层打开
    ↳ layersOut - 图层关闭
  ↳ fade
    ↳ fadeIn - 窗口打开时的淡入效果
    ↳ fadeOut - 窗口关闭时的淡出效果
    ↳ fadeSwitch - 切换 activewindow 及其不透明度时的淡入淡出效果
    ↳ fadeShadow - 切换 activewindow 时阴影的淡入淡出效果
    ↳ fadeDim - 非活动窗口变暗的缓动效果
    ↳ fadeLayers - 控制图层的淡入淡出效果
      ↳ fadeLayersIn - 图层打开时的淡入效果
      ↳ fadeLayersOut - 图层关闭时的淡出效果
    ↳ fadePopups - 控制 Wayland 弹窗的淡入淡出效果
      ↳ fadePopupsIn - Wayland 弹窗打开时的淡入效果
      ↳ fadePopupsOut - Wayland 弹窗关闭时的淡出效果
    ↳ fadeDpms - 控制切换 dpms 时的淡入淡出效果
  ↳ border - 用于动画化边框颜色切换速度
  ↳ borderangle - 用于动画化边框渐变角度 - 样式：once (默认), loop
  ↳ workspaces - 样式：slide, slidevert, fade, slidefade, slidefadevert
    ↳ workspacesIn - 样式：与 workspaces 相同
    ↳ workspacesOut - 样式：与 workspaces 相同
    ↳ specialWorkspace - 样式：与 workspaces 相同
      ↳ specialWorkspaceIn - 样式：与 workspaces 相同
      ↳ specialWorkspaceOut - 样式：与 workspaces 相同
  ↳ zoomFactor - 动画化屏幕缩放
  ↳ monitorAdded - 显示器添加时的缩放动画
```

> [!WARNING]
> 对 `borderangle` 使用 `loop` 样式要求 Hyprland _持续_ 以与屏幕刷新率相等的频率渲染新帧（例如，对于 60Hz 显示器，每秒 60 次），这可能会给 CPU/GPU 带来压力并影响电池寿命。<br>
> 即使动画被禁用或边框不可见，此设置仍会生效。

## 曲线

定义自己的 [贝塞尔曲线](https://zh.wikipedia.org/wiki/%E8%B4%9D%E5%A1%9E%E5%B0%94%E6%9B%B2%E7%BA%BF) 可以通过 `bezier` 关键字完成：

```ini
bezier = NAME, X0, Y0, X1, Y1
```

其中 `NAME` 是您选择的名称，`X0, Y0, X1, Y1` 是三次贝塞尔曲线的两个控制点。<br>
设计自己的贝塞尔曲线可以使用 [cssportal.com](https://www.cssportal.com/css-cubic-bezier-generator/) 网站。<br>
如果您想从预设的贝塞尔曲线中选择，可以查看 [easings.net](https://easings.net)。

### 示例

```ini
bezier = overshoot, 0.05, 0.9, 0.1, 1.1
```

### 额外设置

对于 `windows` 中的动画样式 `popin`，您可以指定最小百分比
作为起始值。例如，以下设置将使动画从 80% -> 100% 的
大小变化：

```ini
animation = windows, 1, 8, default, popin 80%
```

对于 `workspaces` 中的动画样式 `slide`、`slidevert`、`slidefade` 和 `slidefadevert`，您可以
指定移动百分比。例如，以下设置将使窗口移动
屏幕宽度的 20%：

```ini
animation = workspaces, 1, 8, default, slidefade 20%
```

对于 `windows` 和 `layers` 中的 `slide` 动画样式，您可以指定强制方向。<br>
您可以选择 `top`、`bottom`、`left` 或 `right`。

```ini
animation = windows, 1, 8, default, slide left
```

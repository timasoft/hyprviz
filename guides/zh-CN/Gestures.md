---
weight: 10
title: 手势
---

> [!NOTE]
> 由 Qwen3.5-Plus 从 en 翻译

## 通用

Hyprland 支持触控板的 1:1 手势操作，用于某些功能。基本语法如下：

```ini
gesture = fingers, direction, action, options
```

各种动作可能有各自的选项，也可能没有。如果动作不需要选项，您可以直接在动作参数处结束。

您还可以通过在 `direction` 后添加 `, mod: [MODMASK]` 来将手势限制在特定修饰键上，
或通过添加 `scale: [SCALE]` 来按浮点数缩放动画速度。

示例：

```ini
gesture = 3, horizontal, workspace
gesture = 3, down, mod: ALT, close
gesture = 3, up, mod: SUPER, scale: 1.5, fullscreen
gesture = 3, left, scale: 1.5, float
```

### 方向

支持以下方向：
| `direction` | 描述 |
| -- | -- |
| `swipe` | 任意滑动 |
| `horizontal` | 水平滑动 |
| `vertical` | 垂直滑动 |
| `left`, `right`, `up`, `down` | 滑动方向 |
| `pinch` | 任意捏合 |
| `pinchin`, `pinchout` | 方向性捏合 |


### 动作

将 `unset` 指定为动作将取消之前设置的特定手势。请注意，它需要与原始手势的所有内容完全匹配，包括方向、修饰键、手指数量和缩放比例。

| `action` | 描述 | 参数 |
| -- | -- | -- |
| `dispatcher` | 最基本的动作，在手势结束时执行调度器 | `dispatcher, params` |
| `workspace` | 工作区滑动手势，用于切换工作区 | |
| `move` | 移动活动窗口 | 无 |
| `resize` | 调整活动窗口大小 | 无 |
| `special` | 切换特殊工作区 | 特殊工作区名称（不含 `special:` 前缀），例如 `mySpecialWorkspace` |
| `close` | 关闭活动窗口 | 无 |
| `fullscreen` | 将活动窗口全屏 | 无表示全屏，`maximize` 表示最大化 |
| `float` | 将活动窗口浮动 | 无表示切换，`float` 或 `tile` 表示单向操作 |

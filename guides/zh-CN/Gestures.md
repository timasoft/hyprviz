---
weight: 10
title: 手势
---

> [!NOTE]
> 翻译自英文版，使用 Qwen3.5-Plus

## 通用

Hyprland 支持 触控 板 的 1:1 手势 操作，用于 某些 功能。基本 语法 如下：

```ini
gesture = fingers, direction, action, options
```

各种 动作 可能 有 各自 的 选项，或 可能 没有。您 可以 丢弃 选项 完全 并 结束
在 动作 参数 处 如果 动作 需要 无。

您 可以 也 限制 手势 到 一个 修饰 键 通过 添加 `, mod: [MODMASK]` 在 `direction` 后，
或 缩放 动画 的 速度 通过 一个 浮点 数 通过 添加 `scale: [SCALE]`。

示例：

```ini
gesture = 3, horizontal, workspace
gesture = 3, down, mod: ALT, close
gesture = 3, up, mod: SUPER, scale: 1.5, fullscreen
gesture = 3, left, scale: 1.5, float
```

### 方向

以下 方向 是 支持 的：
| `direction` | 描述 |
| -- | -- |
| `swipe` | 任意 滑动 |
| `horizontal` | 水平 滑动 |
| `vertical` | 垂直 滑动 |
| `left`, `right`, `up`, `down` | 滑动 方向 |
| `pinch` | 任意 捏合 |
| `pinchin`, `pinchout` | 方向 性 捏合 |



### 动作

指定 `unset` 作为 动作 将 取消 一个 特定 手势 那 是 之前 设置。请 注意 它 需要 完全 匹配 一切
从 原始 手势 包括 方向、修饰 键、手指 数量 和 缩放 比例。

| `action` | 描述 | 参数 |
| -- | -- | -- |
| `dispatcher` | 最 基本，执行 一个 调度 器 一旦 手势 结束 | `dispatcher, params` |
| `workspace` | 工作 区 滑动 手势，用于 切换 工作 区 | |
| `move` | 移动 活动 窗口 | 无 |
| `resize` | 调整 活动 窗口 大小 | 无 |
| `special` | 切换 一个 特殊 工作 区 | 特殊 工作 区 名称 不含 `special:` 前缀，例如 `mySpecialWorkspace` |
| `close` | 关闭 活动 窗口 | 无 |
| `fullscreen` | 将 活动 窗口 全屏 | 无 表示 全屏，`maximize` 表示 最大化 |
| `float` | 将 活动 窗口 浮动 | 无 表示 切换，`float` 或 `tile` 表示 单向 操作 |

### 标志

手势 支持 标志 通过 以下 语法：

```ini
gesture[flags] = ...
```

支持 的 标志：

| 标志 | 名称 | 描述 |
| -- | -- | -- |
| `p` | bypass | 允许 手势 绕过 快捷 键 抑制 器。 |

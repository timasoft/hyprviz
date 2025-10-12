---
weight: 8
title: 工作区规则
---

{{< callout type=info >}}

Translated from en by qwen3

{{</ callout >}}

您可以设置工作区规则以实现特定于工作区的行为。例如，
您可以定义一个所有窗口都无边框和间隙的工作区。

对于特定布局的规则，请参阅特定布局页面。例如：
[主布局->工作区规则](../Master-Layout#workspace-rules)。

### 工作区选择器

已创建的工作区可以通过工作区选择器进行定位，例如 `r[2-4] w[t1]`。

选择器的属性由空格分隔。属性内部不允许有空格。

属性：

- `r[A-B]` - 从 A 到 B（包含）的 ID 范围
- `s[bool]` - 工作区是否为特殊工作区
- `n[bool]`, `n[s:string]`, `n[e:string]` - 命名操作。`n[bool]` ->
  工作区是否为命名工作区，`s` 和 `e` 分别表示以指定字符串开头和结尾
- `m[monitor]` - 显示器选择器
- `w[(flags)A-B]`, `w[(flags)X]` - 工作区上窗口计数的属性。
  A-B 是包含范围，X 是特定数字。标志可以省略。
  可以是 `t` 表示仅平铺窗口，`f` 表示仅浮动窗口，`g` 表示计数组而非窗口，
  `v` 表示仅计数可见窗口，`p` 表示仅计数固定窗口。
- `f[-1]`, `f[0]`, `f[1]`, `f[2]` - 工作区的全屏状态。`-1`: 无
  全屏，`0`: 全屏，`1`: 最大化，`2`, 全屏但不向窗口发送全屏状态。

### 语法

```ini
workspace = WORKSPACE, RULES
```

- WORKSPACE 是一个有效的工作区标识符（参见
  [调度器->工作区](../Dispatchers#workspaces)）。此字段是必需的。
  这_可以是_一个工作区选择器，但请注意
  工作区选择器只能匹配_已存在_的工作区。
- RULES 是此处[规则](#规则)中描述的一个（或多个）规则。

### 示例

```ini
workspace = name:myworkspace, gapsin:0, gapsout:0
workspace = 3, rounding:false, bordersize:0
workspace = w[tg1-4], shadow:false
```

#### 智能间隙

要从其他窗口管理器/合成器中复制"智能间隙"/"仅在单窗口时无间隙"功能，请使用以下配置：

```ini
workspace = w[tv1], gapsout:0, gapsin:0
workspace = f[1], gapsout:0, gapsin:0
windowrule = bordersize 0, floating:0, onworkspace:w[tv1]
windowrule = rounding 0, floating:0, onworkspace:w[tv1]
windowrule = bordersize 0, floating:0, onworkspace:f[1]
windowrule = rounding 0, floating:0, onworkspace:f[1]
```

#### 智能间隙（忽略特殊工作区）

您可以组合工作区选择器以进行更精细的控制，例如，忽略特殊工作区：

```ini
workspace = w[tv1]s[false], gapsout:0, gapsin:0
workspace = f[1]s[false], gapsout:0, gapsin:0
windowrule = bordersize 0, floating:0, onworkspace:w[tv1]s[false]
windowrule = rounding 0, floating:0, onworkspace:w[tv1]s[false]
windowrule = bordersize 0, floating:0, onworkspace:f[1]s[false]
windowrule = rounding 0, floating:0, onworkspace:f[1]s[false]
```

## 规则

| 规则 | 说明 | 类型 |
| --- | --- | --- |
| monitor:[m] | 将工作区绑定到显示器。参见[语法](#语法)和[显示器](../Monitors)。 | 字符串 |
| default:[b] | 该工作区是否应作为给定显示器的默认工作区 | 布尔值 |
| gapsin:[x] | 设置窗口之间的间隙（等同于[常规->gaps_in](../Variables#general)） | 整数 |
| gapsout:[x] | 设置窗口与显示器边缘之间的间隙（等同于[常规->gaps_out](../Variables#general)） | 整数 |
| bordersize:[x] | 设置窗口周围的边框大小（等同于[常规->border_size](../Variables#general)） | 整数 |
| border:[b] | 是否绘制边框 | 布尔值 |
| shadow:[b] | 是否绘制阴影 | 布尔值 |
| rounding:[b] | 是否绘制圆角窗口 | 布尔值 |
| decorate:[b] | 是否绘制窗口装饰 | 布尔值 |
| persistent:[b] | 即使工作区为空且非活动，也保持该工作区存活 | 布尔值 |
| on-created-empty:[c] | 一旦工作区被创建为空（即不是通过移动窗口到它而创建的），要执行的命令。参见[命令语法](../Dispatchers#executing-with-rules) | 字符串 |
| defaultName:[s] | 工作区的默认名称 | 字符串 |

### 示例规则

```ini
workspace = 3, rounding:false, decorate:false
workspace = name:coding, rounding:false, decorate:false, gapsin:0, gapsout:0, border:false, monitor:DP-1
workspace = 8,bordersize:8
workspace = name:Hello, monitor:DP-1, default:true
workspace = name:gaming, monitor:desc:Chimei Innolux Corporation 0x150C, default:true
workspace = 5, on-created-empty:[float] firefox
workspace = special:scratchpad, on-created-empty:foot
```

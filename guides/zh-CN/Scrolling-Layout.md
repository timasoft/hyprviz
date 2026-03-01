> [!NOTE]
> 翻译自英文版，使用 Qwen3.5-Plus

## 工作区规则

| 名称 | 描述 | 类型 |
| --- | --- | --- |
| direction | 与 scrolling:direction 相同 | str |

例如

```ini
workspace = 2, layoutopt:direction:right
```

## 布局消息

调度器 `layoutmsg` 参数：

| 名称 | 描述 | 参数 |
| --- | --- | --- |
| move | 水平移动布局，通过相对逻辑像素（`-200`, `+200`）或列（`+col`, `-col`） | 移动数据 |
| colresize | 调整当前列的大小，设置为某个值或相对值，例如 `0.5`, `+0.2`, `-0.2`，或使用 `+conf` 或 `-conf` 循环预设值。也可以是 `all (number)` 用于将所有列调整为特定宽度 | 相对浮点数 / 相对配置 |
| fit | 根据参数执行 fit 操作。可用：`active`, `visible`, `all`, `toend`, `tobeg` | fit 模式 |
| focus | 移动焦点并居中布局，同时环绕而不是移动到相邻显示器。 | 方向 |
| promote | 将窗口移动到其自己的新列 | 无 |
| swapcol | 将当前列与其左侧 (`l`) 或右侧 (`r`) 的邻居交换。交换会环绕（例如，将第一列向左交换会将其移动到末尾）。 | `l` 或 `r` |
| togglefit | 切换 focus_fit_method (center, fit) | 无 |

Hyprland 配置的示例键绑定：

```
bind = $mainMod, period, layoutmsg, move +col
bind = $mainMod, comma, layoutmsg, swapcol l
```

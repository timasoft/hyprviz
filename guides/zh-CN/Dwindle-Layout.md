> [!NOTE]
> 由 Qwen3.5-Plus 从 en 翻译

## 特性

Dwindle 分割不是永久性的。分割方式由父节点的宽高比动态决定，依据
W/H 比例。如果宽度 > 高度，则为左右并排；如果高度 > 宽度，则为
上下排列。您可以通过启用 `preserve_split` 使分割方式保持固定。

## 绑定调度器

| 调度器 | 说明 | 参数 |
| --- | --- | --- |
| pseudo | 切换指定窗口的 pseudo 模式 | 留空 / `active` 表示当前窗口，或 `window` 表示特定窗口 |

## 布局消息

调度器 `layoutmsg` 参数：

| 参数 | 说明 | 参数 |
| --- | --- | --- |
| togglesplit | 切换当前窗口的分割方式（上下/左右）。必须启用 `preserve_split` 才能进行切换。 | 无 |
| swapsplit | 交换当前窗口分割的两部分。 | 无 |
| preselect | 对分割方向的一次性覆盖设置。（仅对下一个要打开的窗口有效，仅适用于平铺窗口） | 方向 |
| movetoroot | 将选定窗口（未指定则为活动窗口）移动到其工作区树的根节点。默认行为是在当前子树中最大化窗口。如果提供 `unstable` 作为第二个参数，窗口将与另一个子树交换。无法仅提供第二个参数，但 `movetoroot active unstable` 可以达到相同效果。 | [window, [ string ]] |

例如：

```ini
bind = SUPER, A, layoutmsg, preselect l
```

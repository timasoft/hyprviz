> [!NOTE]
> 由 Qwen3.5-Plus 从 en 翻译

![master1](https://user-images.githubusercontent.com/43317083/179357849-321f042c-f536-44b3-9e6f-371df5321836.gif)

## 调度器

`layoutmsg` 命令：

| 命令 | 说明 | 参数 |
| --- | --- | --- |
| swapwithmaster | 交换当前窗口与主窗口。如果当前窗口是主窗口，则与第一个子窗口交换。 | 可以是 `master`（新焦点为主窗口）、`child`（新焦点为新子窗口）或 `auto`（默认值，保持先前焦点窗口的焦点）。添加 `ignoremaster` 将在主窗口已聚焦时忽略此调度器。 |
| focusmaster | 聚焦主窗口。 | 可以是 `master`（焦点保持在主窗口）、`auto`（默认；如果已在主窗口上，则聚焦第一个非主窗口）或 `previous`（聚焦主窗口时记住当前窗口，如果已在主窗口上，则聚焦上一个窗口或回退到 `auto`）。 |
| cyclenext | 按照布局聚焦下一个窗口 | 可以是 `loop`（允许从堆栈底部循环回主窗口）或 `noloop`（强制在堆栈底部停止，如 DWM 中）。如果留空，默认为 `loop`。 |
| cycleprev | 按照布局聚焦上一个窗口 | 可以是 `loop`（允许从主窗口循环到堆栈底部）或 `noloop`（强制在主窗口停止，如 DWM 中）。如果留空，默认为 `loop`。 |
| swapnext | 按照布局交换聚焦窗口与下一个窗口 | 可以是 `loop`（允许交换堆栈底部和主窗口）或 `noloop`（不允许，如 DWM 中）。如果留空，默认为 `loop`。 |
| swapprev | 按照布局交换聚焦窗口与上一个窗口 | 可以是 `loop`（允许交换主窗口和堆栈底部）或 `noloop`（不允许，如 DWM 中）。如果留空，默认为 `loop`。 |
| addmaster | 向主区域添加一个主窗口。这将是活动窗口（如果它不是主窗口）或第一个非主窗口。 | 无 |
| removemaster | 从主区域移除一个主窗口。这将是活动窗口（如果它是主窗口）或最后一个主窗口。 | 无 |
| orientationleft | 将当前工作区的方向设置为左侧（主区域在左，从窗口在右侧，垂直堆叠） | 无 |
| orientationright | 将当前工作区的方向设置为右侧（主区域在右，从窗口在左侧，垂直堆叠） | 无 |
| orientationtop | 将当前工作区的方向设置为顶部（主区域在顶部，从窗口在底部，水平堆叠） | 无 |
| orientationbottom | 将当前工作区的方向设置为底部（主区域在底部，从窗口在顶部，水平堆叠） | 无 |
| orientationcenter | 将当前工作区的方向设置为中心（主区域在中心，从窗口交替在左右两侧，垂直堆叠） | 无 |
| orientationnext | 循环到当前工作区的下一个方向（顺时针） | 无 |
| orientationprev | 循环到当前工作区的上一个方向（逆时针） | 无 |
| orientationcycle | 从提供的列表中循环到当前工作区的下一个方向 | 允许的值：`left`、`top`、`right`、`bottom` 或 `center`。值之间用空格分隔。如果留空，其行为将类似于 `orientationnext` |
| mfact | 更改 mfact（主区域分割比例） | 新的分割比例，相对浮点增量（例如 `-0.2` 或 `+0.2`）或 `exact` 后跟 0.0 到 1.0 之间的精确浮点值 |
| rollnext | 将堆栈中的下一个窗口旋转为主窗口，同时保持焦点在主窗口上 | 无 |
| rollprev | 将堆栈中的上一个窗口旋转为主窗口，同时保持焦点在主窗口上 | 无 |

命令的参数用单个空格分隔。

> [!NOTE]
> 示例用法：
> 
> ```ini
> bind = MOD, KEY, layoutmsg, cyclenext
> # 行为类似于 xmonad 的 promote 功能 (https://hackage.haskell.org/package/xmonad-contrib-0.17.1/docs/XMonad-Actions-Promote.html)
> bind = MOD, KEY, layoutmsg, swapwithmaster master
> ```

## 工作区规则

`layoutopt` 规则：

| 规则 | 说明 | 类型 |
| --- | --- | --- |
| orientation:[o] | 设置工作区的方向。可用方向请参见 [配置->方向](#config) | 字符串 |

示例用法：

```ini
workspace = 2, layoutopt:orientation:top
```

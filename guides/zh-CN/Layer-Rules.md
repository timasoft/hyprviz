{{< callout type=info >}}

Translated from en by qwen3

{{</ callout >}}

## 层规则

在 Wayland 中，有些元素不是窗口，而是层（layers）。例如：
应用程序启动器、状态栏或壁纸。

这些元素有特定的规则，与窗口规则分开：

```ini
layerrule = 规则, 命名空间
# 或
layerrule = 规则, 地址
```

其中 `规则` 是规则，`命名空间` 是命名空间正则表达式（可在 `hyprctl layers` 中查找命名空间）或 `地址` 是形式为 `address:0x[十六进制]` 的地址。

### 规则

| 规则 | 说明 |
| ---- | ----------- |
| unset | 移除之前为选定命名空间正则表达式设置的所有层规则。请注意必须_完全匹配_。 |
| noanim | 禁用动画。 |
| blur | 为该层启用模糊效果。 |
| blurpopups | 为弹出窗口启用模糊效果。 |
| ignorealpha [a] | 使模糊效果忽略不透明度为 `a` 或更低的像素。`a` 是从 `0` 到 `1` 的浮点值。如果未指定，则 `a = 0`。 |
| ignorezero | 使模糊效果忽略完全透明的像素。等同于 `ignorealpha 0`。 |
| dimaround | 使该层后面的所有内容变暗。 |
| xray [on] | 为该层设置模糊 X 射线模式。`0` 表示关闭，`1` 表示开启，`unset` 表示默认。 |
| animation [style] | 允许您为此层设置特定的动画样式。 |
| order [n] | 设置相对于其他层的顺序。较高的 `n` 值表示更靠近显示器边缘。可以为负数。如果未指定，则 `n = 0`。 |
| abovelock [interactable] | 当会话锁定时，将该层渲染在锁屏上方。如果设置为 `true`，您可以在锁屏上与该层交互，否则它只会渲染在锁屏上方。 |
| noscreenshare [on] | 通过在其上方绘制黑色矩形来使该层在屏幕共享中隐藏。 |

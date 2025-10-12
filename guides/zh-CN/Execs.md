{{< callout type=info >}}

Translated from en by qwen3

{{</ callout >}}

## 执行

您可以在以下时间执行 shell 脚本：

- 合成器启动时
- 每次配置重新加载时
- 合成器关闭时

`exec-once = 命令` 将仅在启动时执行（[支持规则](../Dispatchers/#executing-with-rules)）

`execr-once = 命令` 将仅在启动时执行

`exec = 命令` 将在每次重新加载时执行（[支持规则](../Dispatchers/#executing-with-rules)）

`execr = 命令` 将在每次重新加载时执行

`exec-shutdown = 命令` 将仅在关闭时执行

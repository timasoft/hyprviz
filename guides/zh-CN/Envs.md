{{< callout type=info >}}

Translated from en by qwen3

{{</ callout >}}

## 设置环境变量

{{< callout type=info >}}

新的环境变量无法传递给已运行的进程。如果您在 Hyprland 运行时更改/添加/删除 `env = ` 条目，只有新启动的应用程序才会应用这些更改。

{{< /callout >}}

您可以使用 `env` 关键字来设置环境变量，例如：

```ini
env = XCURSOR_SIZE,24
```

如果您希望将环境变量导出到 D-Bus（仅限 systemd），还可以添加 `d` 标志：

```ini
envd = XCURSOR_SIZE,24
```

{{< callout >}}

Hyprland 会将原始字符串放入环境变量中。您_不应_在值周围添加引号。
例如：
```ini
env = QT_QPA_PLATFORM,wayland
```
而_**不是**_
```ini
env = QT_QPA_PLATFORM,"wayland"
```

{{< /callout >}}

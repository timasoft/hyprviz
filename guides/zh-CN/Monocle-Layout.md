> [!NOTE]
> 翻译自英文版，使用 Qwen3.5-Plus

## 特性

由于布局的工作方式，`cyclenext` 将无法与 Monocle 一起使用。要循环切换 monocle
窗口，请使用 `layoutmsg, cyclenext` 或 `cyclenext, tiled`。

## 布局消息

Dispatcher `layoutmsg` 参数：

| 名称 | 描述 | 参数 |
| --- | --- | --- |
| cyclenext | 切换到下一个窗口 | 无 | 
| cycleprev | 切换到上一个窗口 | 无 |

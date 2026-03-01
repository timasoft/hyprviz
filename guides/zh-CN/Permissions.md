---
weight: 30
title: 权限
---

> [!NOTE]
> 翻译自英文版，使用 Qwen3.5-Plus

如果您已安装 `hyprland-guiutils`，您可以使用 Hyprland 内置的
权限系统。

目前，它只有少量权限，但未来可能会包含更多。

## 权限

权限的工作方式有点像 Android 的权限。如果一个应用程序尝试对合成器（Hyprland）执行敏感操作，Hyprland 将弹出一个通知，询问您是否
允许它执行该操作。

> [!NOTE]
> 在设置权限之前，请确保通过设置
> `ecosystem:enforce_permissions = true` 来启用它们，因为默认情况下它是禁用的。


### 配置权限

> [!IMPORTANT]
> 在配置中设置的权限**不会**即时重新加载，出于安全原因需要重启 Hyprland

配置它们很简单：

```ini
permission = regex, permission, mode
```

例如：
```ini
permission = /usr/bin/grim, screencopy, allow
```
将允许 `/usr/bin/grim` 始终捕获您的屏幕而无需询问。

```ini
permission = /usr/bin/appsuite-.*, screencopy, allow
```
将允许任何路径以 `/usr/bin/appsuite-` 开头的应用程序捕获您的屏幕而无需询问。


### 权限模式

有 3 种模式：
- `allow`：不询问，直接允许应用程序继续执行。
- `ask`：每次应用程序尝试执行敏感操作时弹出通知。这些弹出窗口允许您拒绝、允许直到应用程序退出，或允许直到 Hyprland 退出。
- `deny`：不询问，始终拒绝应用程序访问。


### 权限列表

`screencopy`：
 - 默认：**ASK**
 - 访问您的屏幕_无需_ 经过 xdg-desktop-portal-hyprland。示例包括：`grim`、`wl-screenrec`、`wf-recorder`。
 - 如果被拒绝，将渲染一个带有 "permission denied" 文本的黑屏。
 - 为什么要拒绝？针对可能恶意尝试通过直接使用 wayland 协议在您不知情的情况下捕获您屏幕的应用程序/脚本。

`plugin`：
 - 默认：**ASK**
 - 访问以加载插件。可以是应用程序二进制文件的正则表达式，或插件路径。
 - _不要_ 默认允许 `hyprctl` 加载您的插件（攻击者可能执行 `hyprctl plugin load /tmp/my-malicious-plugin.so`）- 使用 `deny` 禁用或使用 `ask` 以便被提示。

`keyboard`：
 - 默认：**ALLOW**
 - 访问以连接新键盘。设备名称的正则表达式。
 - 如果您想禁用所有不匹配正则表达式的键盘，请创建一条规则为 `.*` 设置 `DENY` _作为最后一条键盘权限规则_。
 - 为什么要拒绝？Rubber duckies、恶意的虚拟/USB 键盘。

## 注意事项

**xdg-desktop-portal** 实现（包括 xdph）只是普通应用程序。它们也会经过权限检查。您可能需要考虑
添加如下规则：
```ini
permission = /usr/(lib|libexec|lib64)/xdg-desktop-portal-hyprland, screencopy, allow
```
如果您不为所有应用程序允许 screencopy。

<br/>

NixOS 没有为二进制文件提供静态路径，因此必须使用正则表达式。这些示例规则允许 `grim` 和 `xdg-desktop-portal-hyprland` 复制屏幕：
```ini
permission = /nix/store/[a-z0-9]{32}-grim-[0-9.]*/bin/grim, screencopy, allow
permission = /nix/store/[a-z0-9]{32}-xdg-desktop-portal-hyprland-[0-9.]*/libexec/.xdg-desktop-portal-hyprland-wrapped, screencopy, allow
```

当使用 Nix 本身渲染配置时，也可以使用字符串插值（请注意，如果路径包含特殊正则表达式字符（例如 `+`），它们需要被转义）：
```ini
permission = ${lib.getExe pkgs.grim}, screencopy, allow
permission = ${lib.escapeRegex (lib.getExe config.programs.hyprlock.package)}, screencopy, allow
permission = ${pkgs.xdg-desktop-portal-hyprland}/libexec/.xdg-desktop-portal-hyprland-wrapped, screencopy, allow
```

<br/>

在某些 **BSD** 系统上，路径可能不起作用。在这种情况下，您可能希望通过设置完全禁用权限，
```ini
ecosystem {
  enforce_permissions = false
}
```
否则，您将没有_配置_ 控制权限的能力（弹出窗口仍然有效，但不会显示路径，且 "remember" 将不可用）。

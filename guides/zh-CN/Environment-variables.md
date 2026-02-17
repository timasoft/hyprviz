---
weight: 18
title: 环境变量
---

> [!NOTE]
> 由 Qwen3.5-Plus 从 en 翻译

> [!NOTE]
> [uwsm](../../Useful-Utilities/Systemd-start) 用户应避免将环境变量放置在 `hyprland.conf` 文件中。  
> 相反，请使用 `~/.config/uwsm/env` 来设置主题、xcursor、Nvidia 和工具包变量，使用 `~/.config/uwsm/env-hyprland` 来设置 `HYPR*` 和 `AQ_*` 变量。  
> 格式为 `export KEY=VAL`。
> 
> ```plain
> export XCURSOR_SIZE=24
> ```
> 
> 请参阅 [uwsm 自述文件](https://github.com/Vladimir-csp/uwsm?tab=readme-ov-file#4-environments-and-shell-profile) 获取更多信息。

您可以使用 `env` 关键字在显示服务器初始化之前设置环境变量，例如：

```ini
env = GTK_THEME,Nord
```

> [!WARNING]
> 请注意，使用 `env` 关键字时，Hyprland 会将变量值作为**原始字符串**读取，并原样放入环境中。  
> 您**不应**在值周围添加引号 `""`。
> 
> 一些不同格式值的示例：
> 
> ✗ 不要：
> 
> ```py
> env = QT_AUTO_SCREEN_SCALE_FACTOR,"1"
> env = QT_QPA_PLATFORM,"wayland"
> env = QT_QPA_PLATFORM,"wayland;xcb"
> env = AQ_DRM_DEVICES=,"/dev/dri/card1:/dev/dri/card0"
> ```
> 
> ✓ 相反，应该：
> 
> ```py
> env = QT_AUTO_SCREEN_SCALE_FACTOR,1
> env = QT_QPA_PLATFORM,wayland
> env = QT_QPA_PLATFORM,wayland;xcb
> env = AQ_DRM_DEVICES=,/dev/dri/card1:/dev/dri/card0
> ```


> [!WARNING]
> 请避免将这些环境变量放置在 `/etc/environment` 中。  
> 在传统 Linux 发行版上，这会导致所有会话（包括 Xorg 会话）加载您的 Wayland 特定环境变量。

## Hyprland 环境变量

- `HYPRLAND_TRACE=1` - 启用更详细的日志记录。
- `HYPRLAND_NO_RT=1` - 禁用 Hyprland 的实时优先级设置。
- `HYPRLAND_NO_SD_NOTIFY=1` - 如果使用 systemd，禁用 `sd_notify` 调用。
- `HYPRLAND_NO_SD_VARS=1` - 禁用 systemd 和 dbus 激活环境中变量的管理。
- `HYPRLAND_CONFIG` - 指定您希望使用的 Hyprland 配置文件位置。

## Aquamarine 环境变量 <!-- ref https://github.com/hyprwm/aquamarine/blob/main/docs/env.md -->

- `AQ_TRACE=1` - 启用更详细的日志记录。
- `AQ_DRM_DEVICES=` - 设置要使用的 DRM 设备（GPU）的明确列表。这是一个以冒号分隔的路径列表，第一个为主设备。
  例如：`/dev/dri/card1:/dev/dri/card0`
- `AQ_FORCE_LINEAR_BLIT=0` - 禁用在多 GPU 缓冲区上强制使用线性显式修饰符，以潜在地规避 Nvidia 问题。
- `AQ_MGPU_NO_EXPLICIT=1` - 禁用多 GPU 缓冲区上的显式同步。
- `AQ_NO_MODIFIERS=1` - 禁用 DRM 缓冲区的修饰符。

## 工具包后端变量

- `env = GDK_BACKEND,wayland,x11,*` - GTK：如果可用则使用 Wayland；如果不可用，则尝试 X11，然后尝试其他任何 GDK 后端。
- `env = QT_QPA_PLATFORM,wayland;xcb` - Qt：如果可用则使用 Wayland，否则回退到 X11。
- `env = SDL_VIDEODRIVER,wayland` - 在 Wayland 上运行 SDL2 应用程序。如果提供旧版本 SDL 的游戏导致兼容性问题，请移除或设置为 `x11`
- `env = CLUTTER_BACKEND,wayland` - Clutter 软件包已启用 Wayland，此变量将强制 Clutter 应用程序尝试使用 Wayland 后端

## XDG 规范

- `env = XDG_CURRENT_DESKTOP,Hyprland`
- `env = XDG_SESSION_TYPE,wayland`
- `env = XDG_SESSION_DESKTOP,Hyprland`

XDG 特定环境变量通常通过门户和应用程序检测，这些门户和应用程序可能会为您设置这些变量，但显式设置它们也是一个好主意。

如果您的 [桌面门户](https://wiki.archlinux.org/title/XDG_Desktop_Portal) 似乎无缘无故出现故障（无错误），则可能是您的 XDG 环境变量未正确设置。

> [!NOTE]
> [uwsm](../../Useful-Utilities/Systemd-start) 用户无需显式设置 XDG 环境变量，因为 uwsm 会自动设置它们。

## Qt 变量

- `env = QT_AUTO_SCREEN_SCALE_FACTOR,1` - [(来自 Qt 文档)](https://doc.qt.io/qt-5/highdpi.html) 启用基于显示器像素密度的自动缩放
- `env = QT_QPA_PLATFORM,wayland;xcb` - 告诉 Qt 应用程序使用 Wayland 后端，如果 Wayland 不可用则回退到 X11
- `env = QT_WAYLAND_DISABLE_WINDOWDECORATION,1` - 禁用 Qt 应用程序的窗口装饰
- `env = QT_QPA_PLATFORMTHEME,qt5ct` - 告诉基于 Qt 的应用程序从 qt5ct 选择您的主题，与 Kvantum 一起使用。

## NVIDIA 特定设置

要强制使用 GBM 作为后端，请设置以下环境变量：

- `env = GBM_BACKEND,nvidia-drm`
- `env = __GLX_VENDOR_LIBRARY_NAME,nvidia`

> 请参阅 [Archwiki Wayland 页面](https://wiki.archlinux.org/title/Wayland#Requirements) 了解有关这些变量的更多详细信息。

- `env = LIBVA_DRIVER_NAME,nvidia` - NVIDIA GPU 上的硬件加速

> 在设置此变量之前，请参阅 [Archwiki 硬件加速页面](https://wiki.archlinux.org/title/Hardware_video_acceleration) 了解详细信息和必要的值。

- `__GL_GSYNC_ALLOWED` - 控制支持 G-Sync 的显示器是否应使用可变刷新率 (VRR)

> 请参阅 [Nvidia 文档](https://download.nvidia.com/XFree86/Linux-32bit-ARM/375.26/README/openglenvvariables.html) 了解详细信息。

- `__GL_VRR_ALLOWED` - 控制是否应使用自适应同步。建议设置为 "0" 以避免在某些游戏中出现问题。

- `env = AQ_NO_ATOMIC,1` - 使用传统 DRM 接口而非原子模式设置。**不**推荐。

## 主题相关变量

- `GTK_THEME` - 手动设置 GTK 主题，适用于希望避免使用 lxappearance 或 nwg-look 等外观工具的用户。
- `XCURSOR_THEME` - 设置您的光标主题。该主题需要已安装且您的用户可读取。
- `XCURSOR_SIZE` - 设置光标大小。请参阅 [此处](../../FAQ/) 了解您可能需要设置此变量的原因。

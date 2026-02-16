---
weight: 18
title: Переменные окружения
---

> [!NOTE]
> Переведено с en с помощью Qwen3-Max

> [!NOTE]
> [uwsm](../../Useful-Utilities/Systemd-start) пользователям следует избегать размещения переменных окружения в файле `hyprland.conf`.  
> Вместо этого используйте `~/.config/uwsm/env` для переменных темизации, курсора, Nvidia и тулкитов, а также `~/.config/uwsm/env-hyprland` для переменных `HYPR*` и `AQ_*`.  
> Формат: `export KEY=VAL`.
>
> ```plain
> export XCURSOR_SIZE=24
> ```
>
> Дополнительную информацию см. в [readme uwsm](https://github.com/Vladimir-csp/uwsm?tab=readme-ov-file#4-environments-and-shell-profile).

Вы можете использовать ключевое слово `env` для установки переменных окружения до
инициализации сервера отображения, например:

```ini
env = GTK_THEME,Nord
```

> [!WARNING]
> Обратите внимание, что при использовании ключевого слова `env` Hyprland считывает значение переменной как **сырую строку** и помещает её в окружение _как есть_.  
> Вы **НЕ** должны добавлять кавычки `""` вокруг значений.
>
> Несколько примеров с различным форматированием значений:
>
> ✗ НЕЛЬЗЯ:
>
> ```py
> env = QT_AUTO_SCREEN_SCALE_FACTOR,"1"
> env = QT_QPA_PLATFORM,"wayland"
> env = QT_QPA_PLATFORM,"wayland;xcb"
> env = AQ_DRM_DEVICES=,"/dev/dri/card1:/dev/dri/card0"
> ```
>
> ✓ Вместо этого, НУЖНО:
>
> ```py
> env = QT_AUTO_SCREEN_SCALE_FACTOR,1
> env = QT_QPA_PLATFORM,wayland
> env = QT_QPA_PLATFORM,wayland;xcb
> env = AQ_DRM_DEVICES=,/dev/dri/card1:/dev/dri/card0
> ```



> [!WARNING]
> Пожалуйста, избегайте размещения этих переменных окружения в `/etc/environment`.  
> Это приведёт к тому, что все сессии (включая Xorg) будут использовать ваше окружение, специфичное для Wayland,
> в традиционных дистрибутивах Linux.

## Переменные окружения Hyprland

- `HYPRLAND_TRACE=1` - Включает более подробное ведение журнала.
- `HYPRLAND_NO_RT=1` - Отключает установку приоритета реального времени Hyprland.
- `HYPRLAND_NO_SD_NOTIFY=1` - При использовании systemd отключает вызовы `sd_notify`.
- `HYPRLAND_NO_SD_VARS=1` - Отключает управление переменными в окружениях активации systemd и dbus.
- `HYPRLAND_CONFIG` - Указывает расположение конфигурации Hyprland.

## Переменные окружения Aquamarine <!-- ref https://github.com/hyprwm/aquamarine/blob/main/docs/env.md -->

- `AQ_TRACE=1` - Включает более подробное ведение журнала.
- `AQ_DRM_DEVICES=` - Устанавливает явный список устройств DRM (видеокарт) для использования. Это список путей, разделённых двоеточием, где первый путь соответствует основной видеокарте.
  Например: `/dev/dri/card1:/dev/dri/card0`
- `AQ_FORCE_LINEAR_BLIT=0` - Отключает принудительное использование линейных явных модификаторов для буферов с несколькими видеокартами для потенциального обхода проблем Nvidia.
- `AQ_MGPU_NO_EXPLICIT=1` - Отключает явную синхронизацию для буферов с несколькими видеокартами.
- `AQ_NO_MODIFIERS=1` - Отключает модификаторы для буферов DRM.

## Переменные бэкендов тулкитов

- `env = GDK_BACKEND,wayland,x11,*` - GTK: использовать Wayland, если доступен; если нет, попробовать X11, а затем любой другой бэкенд GDK.
- `env = QT_QPA_PLATFORM,wayland;xcb` - Qt: использовать Wayland, если доступен, в противном случае
  переключиться на X11.
- `env = SDL_VIDEODRIVER,wayland` - Запускать приложения SDL2 в Wayland. Удалите или установите в
  `x11`, если игры со старыми версиями SDL вызывают проблемы совместимости.
- `env = CLUTTER_BACKEND,wayland` - Пакет Clutter уже имеет поддержку Wayland, эта
  переменная заставит приложения Clutter попытаться использовать бэкенд Wayland.

## Спецификации XDG

- `env = XDG_CURRENT_DESKTOP,Hyprland`
- `env = XDG_SESSION_TYPE,wayland`
- `env = XDG_SESSION_DESKTOP,Hyprland`

Специфичные для XDG переменные окружения часто определяются через порталы и
приложения, которые могут устанавливать их за вас, однако явная установка
их не будет лишней.

Если ваш [портал рабочего стола](https://wiki.archlinux.org/title/XDG_Desktop_Portal) работает некорректно по,
казалось бы, необъяснимой причине (без ошибок), скорее всего, ваши переменные XDG
установлены неправильно.

> [!NOTE]
> Пользователям [uwsm](../../Useful-Utilities/Systemd-start) не нужно явно устанавливать переменные окружения XDG, так как uwsm устанавливает их автоматически.

## Переменные Qt

- `env = QT_AUTO_SCREEN_SCALE_FACTOR,1` -
  [(Из документации Qt)](https://doc.qt.io/qt-5/highdpi.html) включает
  автоматическое масштабирование на основе плотности пикселей монитора.
- `env = QT_QPA_PLATFORM,wayland;xcb` - Указывает приложениям Qt использовать
  бэкенд Wayland и переключаться на X11, если Wayland недоступен.
- `env = QT_WAYLAND_DISABLE_WINDOWDECORATION,1` - Отключает декорации окон в приложениях Qt.
- `env = QT_QPA_PLATFORMTHEME,qt5ct` - Указывает приложениям на базе Qt использовать тему
  из qt5ct, используйте вместе с Kvantum.

## Специфичные для NVIDIA

Чтобы принудительно использовать GBM в качестве бэкенда, установите следующие переменные окружения:

- `env = GBM_BACKEND,nvidia-drm`
- `env = __GLX_VENDOR_LIBRARY_NAME,nvidia`

> См.
> [страницу Archwiki о Wayland](https://wiki.archlinux.org/title/Wayland#Requirements)
> для получения дополнительных сведений об этих переменных.

- `env = LIBVA_DRIVER_NAME,nvidia` - Аппаратное ускорение на видеокартах NVIDIA

> См.
> [страницу Archwiki об аппаратном ускорении](https://wiki.archlinux.org/title/Hardware_video_acceleration)
> для получения подробной информации и необходимых значений перед установкой этой переменной.

- `__GL_GSYNC_ALLOWED` - Определяет, должны ли мониторы с поддержкой G-Sync использовать
  переменную частоту обновления (VRR).

> См.
> [документацию Nvidia](https://download.nvidia.com/XFree86/Linux-32bit-ARM/375.26/README/openglenvvariables.html)
> для получения подробной информации.

- `__GL_VRR_ALLOWED` - Определяет, следует ли использовать Adaptive Sync. Рекомендуется
  установить значение "0", чтобы избежать проблем в некоторых играх.

- `env = AQ_NO_ATOMIC,1` - использовать устаревший интерфейс DRM вместо атомарной
  настройки режима. **НЕ** рекомендуется.

## Переменные, связанные с темизацией

- `GTK_THEME` - Установка темы GTK вручную для тех, кто хочет избежать использования
  инструментов оформления, таких как lxappearance или nwg-look.
- `XCURSOR_THEME` - Установка темы курсора. Тема должна быть установлена и
  доступна для чтения вашим пользователем.
- `XCURSOR_SIZE` - Установка размера курсора. См. [здесь](../../FAQ/) для объяснения,
  зачем может понадобиться эта переменная.

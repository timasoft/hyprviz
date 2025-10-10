## Setting the environment

{{< callout type=info >}}

A new environment cannot be passed to already running processes. If you change / add / remove an `env = ` entry
when Hyprland is running, only newly spawned apps will pick up the changes.

{{< /callout >}}

You can use the `env` keyword to set environment variables,
e.g:

```ini
env = XCURSOR_SIZE,24
```

You can also add a `d` flag if you want the env var to be exported to D-Bus
(systemd only):

```ini
envd = XCURSOR_SIZE,24
```

{{< callout >}}

Hyprland puts the raw string to the env var. You should _not_ add quotes around
the values.

e.g.:

```ini
env = QT_QPA_PLATFORM,wayland
```

and _**NOT**_

```ini
env = QT_QPA_PLATFORM,"wayland"
```

{{< /callout >}}

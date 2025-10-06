## Executing

You can execute a shell script on:

- startup of the compositor
- every time the config is reloaded.
- shutdown of the compositor

`exec-once = command` will execute only on launch ([support rules](../Dispatchers/#executing-with-rules))

`execr-once = command` will execute only on launch

`exec = command` will execute on each reload ([support rules](../Dispatchers/#executing-with-rules))

`execr = command` will execute on each reload

`exec-shutdown = command` will execute only on shutdown

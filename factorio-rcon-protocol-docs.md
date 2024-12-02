# Factorio RCON Protocol Docs

Factorio supports the RCON (Remote Console) Protocol for
mods to run commands externally.

The official wiki barely documents RCON

The wiki just points to https://developer.valvesoftware.com/wiki/Source_RCON_Protocol

## Special First Connection with ID=0 Requests

The first RCON client has a special ability to retrieve game logs like chat and commands.
Send requests with ID of 0 at a regular interval to retrieve.

Some clients like https://github.com/gorcon/rcon-cli always use ID=0 which may mislead
what the response should look like

This may also break your client! Code parsing command responses
that does not want logs must avoid ID=0 (eg with sequential ids instead of random).
You should not simply skip the first
line either as chat messages may be inserted too.

```
> /time
2024-04-10 02:28:53 [CHAT] <server>: random chat message
1 day, 11 hours, 1 minute and 26 seconds
> /c rcon.print('asdf')
2024-04-10 02:26:58 [CHAT] <server>: random chat message
2024-04-10 02:27:07 [COMMAND] <server> (command): rcon.print('asdf')
asdf
```

## Send Output

Instead of communicating via `script-output` files or parsing `game.log` commands use
the `rcon.print()` method. Sends messages back to the connection.

## Receive Error Output

Normal `/command` requests do not include error messages in RCON Response.
A blank response means both "command success" and "error in logs".
This may be surprising when moving from in-game console experiments to modding.

Instead of a normal command use `/silent-command` which returns most error messages.

For more garantees of success, append `rcon.print('done'')` validation check at the
end of all commands and confirm it's in the output.

```
/c 
/silent-command
```

## Disable achievements may consume first command

On fresh games achievements disable some console commands.   
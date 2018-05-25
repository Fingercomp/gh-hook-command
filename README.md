# gh-hook-command
Sets up a simple HTTP server. The only endpoint it serves is `POST /hook`. When
a GitHub webhook is sent there, the request is validated, and the command
specified in the configuration file is run.

## Installation
```
$ cargo build --release
$ target/release/gh-hook-command
```

## Configuration
The path to the configuration file is set by the `GH_HOOK_CONFIG` environment
variable. If it's unset, `./config.toml` is used instead. The file will be
created if it doesn't exist.

```toml
# The webhook secret key.
secret = "secret secret really secret"

# Address to bind to.
bind = "0.0.0.0:8000"

# The command list.
[commands]
# This runs `cat /dev/stdin && echo ""' on the `push` event.
push = 'cat /dev/stdin && echo ""'
```

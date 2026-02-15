# canis

A single binary, ultra simple media server. Use the browser for playing the videos.

The idea is to run this bound to some closed off interface (e.g. Tailscale) so one doesn't have to worry about authentication (apart from some simple Cookie based checks).

## authentication
Set `AUTH_COOKIE_NAME` and `AUTH_COOKIE_VALUE`. Both need setting or the auth check doesn't run.

## Example

```bash
BIND_ADDRESS=127.0.0.1 PORT=3344 \
AUTH_COOKIE_NAME=X-Hot-Dog AUTH_COOKIE_VALUE=yummy canis
```
to run locally on port 3344 with some basic Cookie authentication.

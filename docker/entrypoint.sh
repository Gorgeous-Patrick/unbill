#!/bin/sh
set -e

/usr/local/bin/unbill-server &

exec caddy run --config /etc/caddy/Caddyfile --adapter caddyfile

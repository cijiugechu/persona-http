#!/usr/bin/env bash
set -euo pipefail

zig_bin="${ZIG_BIN:-zig}"
exec "$zig_bin" ar "$@"


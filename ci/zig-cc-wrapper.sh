#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: zig-cc-wrapper.sh <zig-target> [args...]" >&2
  exit 1
fi

zig_target="$1"
shift
zig_bin="${ZIG_BIN:-zig}"
args=()

original=("$@")
len=${#original[@]}
index=0
target_seen=false

while [[ $index -lt $len ]]; do
  arg="${original[$index]}"
  case "$arg" in
    --target=*)
      args+=("-target" "$zig_target")
      target_seen=true
      ;;
    -target)
      if (( index + 1 < len )); then
        index=$((index + 1))
      fi
      args+=("-target" "$zig_target")
      target_seen=true
      ;;
    *)
      args+=("$arg")
      ;;
  esac
  index=$((index + 1))
done

if [[ "$target_seen" == "false" ]]; then
  args=("-target" "$zig_target" "${args[@]}")
fi

exec "$zig_bin" cc "${args[@]}"


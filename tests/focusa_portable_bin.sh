#!/usr/bin/env bash

focusa_probe_version() {
  local binary="$1"
  local version_output
  local first_line

  if [[ -z "$binary" || ! -x "$binary" ]]; then
    return 1
  fi

  if command -v timeout >/dev/null 2>&1; then
    if ! version_output="$(timeout 4 "$binary" --version 2>&1)"; then
      return 1
    fi
  else
    if ! version_output="$($binary --version 2>&1)"; then
      return 1
    fi
  fi

  if [[ -z "$version_output" ]]; then
    return 1
  fi

  first_line="${version_output%%$'\n'*}"
  [[ "$first_line" == focusa* ]] || return 1

  printf '%s\n' "$version_output"
}

focusa_binary_identity() {
  local binary="$1"
  local identity

  if identity="$(stat -Lc '%d:%i %h %s %y %n' "$binary" 2>/dev/null)"; then
    printf '%s\n' "$identity"
    return 0
  fi

  stat -f '%d:%i %l %z %m %N' "$binary" 2>/dev/null || true
}

focusa_print_binary_evidence() {
  local binary="$1"
  local version
  local file_info
  local identity
  local sha256

  version="$(focusa_probe_version "$binary" 2>/dev/null || true)"
  file_info="$(file -b "$binary" 2>/dev/null || echo "file unavailable")"
  identity="$(focusa_binary_identity "$binary" || echo "identity unavailable")"
  sha256="$(sha256sum "$binary" 2>/dev/null | awk '{print $1}')"
  if [[ -z "$sha256" ]]; then
    sha256="$(shasum -a 256 "$binary" 2>/dev/null | awk '{print $1}')"
  fi

  printf 'selected binary: %s\n' "$binary"
  printf 'selected binary version: %s\n' "${version%%$'\n'*}"
  printf 'selected binary file identity: %s\n' "$identity"
  printf 'selected binary file info: %s\n' "$file_info"
  printf 'selected binary sha256: %s\n' "${sha256:-"sha256 unavailable"}"
}

focusa_is_host_compatible_binary() {
  local binary="$1"
  local host_os
  local host_arch
  local file_info

  [[ -n "$binary" && -f "$binary" && -x "$binary" ]] || return 1

  command -v file >/dev/null 2>&1 || return 1
  file_info="$(file -b "$binary" 2>/dev/null || true)"
  [[ -n "$file_info" ]] || return 1

  host_os="$(uname -s)"
  host_arch="$(uname -m)"

  case "$host_os" in
    Linux)
      [[ "$file_info" == *"ELF"* ]] || return 1
      case "$host_arch" in
        x86_64|amd64)
          [[ "$file_info" == *"x86-64"* || "$file_info" == *"x86_64"* ]] || return 1
          ;;
        aarch64|arm64)
          [[ "$file_info" == *"aarch64"* ]] || return 1
          ;;
        armv7l|armv6l|armv8l|arm*)
          [[ "$file_info" == *"ARM"* ]] || return 1
          ;;
        *)
          :
          ;;
      esac
      ;;
    Darwin)
      [[ "$file_info" == *"Mach-O"* ]] || return 1
      case "$host_arch" in
        x86_64|amd64)
          [[ "$file_info" == *"x86_64"* ]] || return 1
          ;;
        arm64)
          [[ "$file_info" == *"arm64"* ]] || return 1
          ;;
        *)
          :
          ;;
      esac
      ;;
    CYGWIN*|MSYS*|MINGW*|Windows_NT)
      [[ "$file_info" == *"PE32"* ]] || return 1
      ;;
    *)
      :
      ;;
  esac

  if ! focusa_probe_version "$binary" >/dev/null; then
    return 1
  fi

  return 0
}

focusa_resolve_test_cli_binary() {
  local root="$1"
  local -a candidates
  local explicit=0
  local -a tried

  if [[ -n "${FOCUSA_BIN+x}" ]]; then
    explicit=1
    if [[ -z "${FOCUSA_BIN}" ]]; then
      candidates=("")
    else
      candidates=("$FOCUSA_BIN")
    fi
  else
    candidates=(
      "$root/target/debug/focusa"
      "$root/target/release/focusa"
    )
  fi

  for binary in "${candidates[@]}"; do
    [[ -n "$binary" ]] || continue
    tried+=("$binary")
    if [[ ! -x "$binary" ]]; then
      continue
    fi

    if focusa_is_host_compatible_binary "$binary"; then
      printf '%s\n' "$binary"
      return 0
    fi
  done

  if [[ "$explicit" -eq 1 ]]; then
    echo "explicit FOCUSA_BIN is not executable or host-incompatible: ${FOCUSA_BIN}" >&2
  else
    if [[ ${#tried[@]} -gt 0 ]]; then
      echo "no host-compatible focusa binary found in candidates: ${tried[*]}" >&2
    else
      echo "no candidate focusa binary found under: $root/target/{debug,release}/focusa" >&2
    fi
  fi

  return 1
}

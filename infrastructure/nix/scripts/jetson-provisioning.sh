#!/usr/bin/env bash
set -euo pipefail

readonly keyset_name=ringil-jetson
readonly required_files=(
  jetson_rcm_priv.pem
  PK.key PK.crt PK.esl
  KEK.key KEK.crt KEK.esl
  db.key db.crt db.esl
)
key_stage=''

# Private staging files must not survive a failed generation.
cleanup_stage() {
  if [[ -n "$key_stage" ]]; then
    rm -rf -- "$key_stage"
  fi
}
trap cleanup_stage EXIT

# Validation failures must stop before a signing derivation is built.
fail() {
  printf 'Ringil Jetson: %s\n' "$1" >&2
  exit 1
}

# BSD support keeps local preflight tests runnable on macOS signing workstations.
file_mode() {
  stat -c '%a' "$1" 2>/dev/null || stat -f '%Lp' "$1"
}

# A shared or redirected directory could expose new private keys.
require_private_dir() {
  local directory="$1"
  local mode
  [[ "$directory" = /* && -d "$directory" && ! -L "$directory" ]] || fail "vault directory must be an existing absolute directory: $directory"
  mode="$(file_mode "$directory")"
  [[ "$mode" =~ ^[0-7]{3,4}$ ]] || fail "cannot verify vault permissions: $directory"
  (( (8#$mode & 077) == 0 )) || fail "vault directory permits group or other access: $directory"
}

# Missing material permits unsigned fallback; malformed material must fail closed.
check_keyset() {
  local directory="$1"
  local filename
  local mode
  local size
  local fingerprint
  local seen
  local missing=0

  [[ ! -L "$directory" ]] || fail "key vault is a symlink: $directory"
  if [[ ! -e "$directory" ]]; then
    printf 'Ringil Jetson: key vault is missing: %s\n' "$directory" >&2
    return 2
  fi
  require_private_dir "$directory"
  for filename in "${required_files[@]}"; do
    if [[ -L "$directory/$filename" ]]; then
      fail "key vault contains a symlink: $filename"
    fi
    if [[ ! -f "$directory/$filename" || ! -s "$directory/$filename" ]]; then
      printf 'Ringil Jetson: missing key material: %s\n' "$filename" >&2
      missing=1
    fi
  done
  (( missing == 0 )) || return 2

  for filename in jetson_rcm_priv.pem PK.key KEK.key db.key; do
    mode="$(file_mode "$directory/$filename")"
    [[ "$mode" =~ ^[0-7]{3,4}$ ]] || fail "cannot verify key permissions: $filename"
    (( (8#$mode & 077) == 0 )) || fail "private key permits group or other access: $filename"
  done

  openssl rsa -in "$directory/jetson_rcm_priv.pem" -check -noout >/dev/null 2>&1 || fail 'PKC key is invalid'
  [[ "$(openssl pkey -in "$directory/jetson_rcm_priv.pem" -pubout -text_pub -noout 2>/dev/null)" == *'Public-Key: (3072 bit)'* ]] || fail 'PKC key must be RSA-3072 for this key set'
  seen=":$(openssl pkey -in "$directory/jetson_rcm_priv.pem" -pubout -outform DER | sha256sum | cut -d ' ' -f 1):"

  for filename in PK KEK db; do
    openssl rsa -in "$directory/$filename.key" -check -noout >/dev/null 2>&1 || fail "$filename private key must be a valid RSA key"
    [[ "$(openssl pkey -in "$directory/$filename.key" -pubout -text_pub -noout 2>/dev/null)" == *'Public-Key: (2048 bit)'* ]] || fail "$filename private key must be RSA-2048"
    openssl x509 -in "$directory/$filename.crt" -checkend 2592000 -noout >/dev/null 2>&1 || fail "$filename certificate is invalid or expires within 30 days"
    cmp -s \
      <(openssl pkey -in "$directory/$filename.key" -pubout -outform DER 2>/dev/null) \
      <(openssl x509 -in "$directory/$filename.crt" -pubkey -noout 2>/dev/null | openssl pkey -pubin -outform DER 2>/dev/null) \
      || fail "$filename certificate does not match its private key"

    size="$(openssl x509 -in "$directory/$filename.crt" -outform DER 2>/dev/null | wc -c | tr -d ' ')"
    cmp -s \
      <(openssl x509 -in "$directory/$filename.crt" -outform DER 2>/dev/null) \
      <(tail -c "$size" "$directory/$filename.esl") \
      || fail "$filename ESL does not contain its certificate"

    fingerprint="$(openssl pkey -in "$directory/$filename.key" -pubout -outform DER | sha256sum | cut -d ' ' -f 1)"
    [[ "$seen" != *":$fingerprint:"* ]] || fail "$filename reuses another signing key"
    seen="$seen$fingerprint:"
  done
  printf 'Ringil Jetson: key vault preflight passed: %s\n' "$directory" >&2
}

# Staging prevents an interrupted command from exposing a partial key set.
generate_keyset() {
  local parent="$1"
  local destination="$parent/$keyset_name"
  local guid
  local name

  require_private_dir "$parent"
  [[ ! -e "$destination" && ! -L "$destination" ]] || fail "key set already exists: $destination"
  key_stage="$(mktemp -d "$parent/.ringil-jetson.XXXXXXXX")"
  umask 077
  guid="$(uuidgen)"

  openssl genrsa -out "$key_stage/jetson_rcm_priv.pem" 3072 >/dev/null 2>&1
  for name in PK KEK db; do
    openssl genrsa -out "$key_stage/$name.key" 2048 >/dev/null 2>&1
    openssl req -new -x509 -sha256 -days 3650 -subj "/CN=Ringil Jetson $name/" \
      -key "$key_stage/$name.key" -out "$key_stage/$name.crt" >/dev/null 2>&1
    cert-to-efi-sig-list -g "$guid" "$key_stage/$name.crt" "$key_stage/$name.esl"
  done
  check_keyset "$key_stage"
  [[ ! -e "$destination" && ! -L "$destination" ]] || fail "key set appeared during generation: $destination"
  mv "$key_stage" "$destination"
  key_stage=''
  printf 'Ringil Jetson: generated keys in %s\n' "$destination" >&2
  printf 'Ringil Jetson: do not use the new PKC key for a device fused to a different public-key hash.\n' >&2
}

# A partial vault must never produce a partly signed flash artifact.
build_flash() {
  local directory="$1"
  local output="$2"
  local status=0

  check_keyset "$directory" || status=$?
  if (( status == 2 )); then
    printf 'Ringil Jetson: WARNING: key set is incomplete; building unsigned firmware. Ringil PKC signing is inactive for this artifact.\n' >&2
    nix build --accept-flake-config .#nixosConfigurations.prod-swarm.config.system.build.flashScript -o "$output"
    return
  fi
  (( status == 0 )) || fail 'key validation failed; refusing to build'

  export RINGIL_JETSON_KEY_VAULT="$directory"
  export RINGIL_JETSON_KEYSET_ID
  RINGIL_JETSON_KEYSET_ID="$({
    openssl pkey -in "$directory/jetson_rcm_priv.pem" -pubout -outform DER
    cat "$directory/PK.esl" "$directory/KEK.esl" "$directory/db.esl"
  } | sha256sum | cut -d ' ' -f 1)"
  printf 'Ringil Jetson: building PKC-signed firmware with key set %s\n' "$RINGIL_JETSON_KEYSET_ID" >&2
  printf 'Ringil Jetson: verify the board fuse hash matches this PKC key before flashing.\n' >&2
  printf 'Ringil Jetson: UEFI OS payload verification still requires enrollment and signed boot artifacts.\n' >&2
  nix build --accept-flake-config --impure \
    .#nixosConfigurations.prod-swarm-firmware-signed.config.system.build.flashScript -o "$output"
}

(( $# >= 2 )) || fail 'usage: generate VAULT_PARENT | check KEY_DIR | build KEY_DIR [OUTPUT_LINK]'
case "$1" in
  generate)
    (( $# == 2 )) || fail 'usage: generate VAULT_PARENT'
    generate_keyset "$2"
    ;;
  check)
    (( $# == 2 )) || fail 'usage: check KEY_DIR'
    check_keyset "$2"
    ;;
  build)
    (( $# <= 3 )) || fail 'usage: build KEY_DIR [OUTPUT_LINK]'
    build_flash "$2" "${3:-result-flash}"
    ;;
  *) fail 'unknown command' ;;
esac

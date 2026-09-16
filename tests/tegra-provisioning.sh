#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script="$repo_dir/infrastructure/nix/scripts/jetson-provisioning.sh"
fixture="$(mktemp -d)"
trap 'rm -rf "$fixture"' EXIT
chmod 700 "$fixture"
mkdir "$fixture/bin"

cat > "$fixture/bin/cert-to-efi-sig-list" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
test "$1" = -g
openssl x509 -in "$3" -outform DER > "$4"
EOF
cat > "$fixture/bin/nix" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s|%s|%s\n' "${RINGIL_JETSON_KEY_VAULT:-}" "${RINGIL_JETSON_KEYSET_ID:-}" "$*" >> "$RINGIL_TEST_LOG"
EOF
chmod 700 "$fixture/bin/cert-to-efi-sig-list" "$fixture/bin/nix"
export PATH="$fixture/bin:$PATH"
export RINGIL_TEST_LOG="$fixture/nix.log"

bash "$script" build "$fixture/missing" "$fixture/unsigned" 2> "$fixture/warning.log"
rg -q 'unsigned' "$fixture/warning.log"
rg -q 'prod-swarm.config.system.build.flashScript' "$RINGIL_TEST_LOG"

ln -s "$fixture/nonexistent-vault" "$fixture/redirected-vault"
: > "$RINGIL_TEST_LOG"
if bash "$script" build "$fixture/redirected-vault" "$fixture/redirected" > "$fixture/redirected.log" 2>&1; then
  printf 'A redirected vault passed preflight\n' >&2
  exit 1
fi
test ! -s "$RINGIL_TEST_LOG"

bash "$script" generate "$fixture" > "$fixture/generated.log"
vault="$fixture/ringil-jetson"
test -s "$vault/jetson_rcm_priv.pem"
test -s "$vault/PK.esl"
test -s "$vault/KEK.esl"
test -s "$vault/db.esl"
bash "$script" check "$vault"

: > "$RINGIL_TEST_LOG"
bash "$script" build "$vault" "$fixture/signed" 2> "$fixture/signed.log"
rg -q 'prod-swarm-firmware-signed.config.system.build.flashScript' "$RINGIL_TEST_LOG"
rg -q -- '--impure' "$RINGIL_TEST_LOG"
if rg -q -- '--rebuild' "$RINGIL_TEST_LOG"; then
  printf 'Signing build needlessly disabled Nix caching\n' >&2
  exit 1
fi
rg -q '[0-9a-f]{64}' "$RINGIL_TEST_LOG"
rg -q "$vault" "$RINGIL_TEST_LOG"

if bash "$script" generate "$fixture" > "$fixture/overwrite.log" 2>&1; then
  printf 'Key generation overwrote an existing vault\n' >&2
  exit 1
fi

mv "$vault/db.key" "$vault/db.key.hidden"
: > "$RINGIL_TEST_LOG"
bash "$script" build "$vault" "$fixture/partial" 2> "$fixture/partial.log"
rg -q 'unsigned' "$fixture/partial.log"
rg -q 'prod-swarm.config.system.build.flashScript' "$RINGIL_TEST_LOG"
mv "$vault/db.key.hidden" "$vault/db.key"

chmod 644 "$vault/db.key"
: > "$RINGIL_TEST_LOG"
if bash "$script" build "$vault" "$fixture/insecure" > "$fixture/insecure.log" 2>&1; then
  printf 'Insecure key permissions passed preflight\n' >&2
  exit 1
fi
test ! -s "$RINGIL_TEST_LOG"
chmod 600 "$vault/db.key"

cp "$vault/db.key" "$fixture/db.key.backup"
cp "$vault/db.crt" "$fixture/db.crt.backup"
cp "$vault/db.esl" "$fixture/db.esl.backup"
cp "$vault/PK.key" "$vault/db.key"
cp "$vault/PK.crt" "$vault/db.crt"
cp "$vault/PK.esl" "$vault/db.esl"
: > "$RINGIL_TEST_LOG"
if bash "$script" build "$vault" "$fixture/duplicate" > "$fixture/duplicate.log" 2>&1; then
  printf 'Reused UEFI keys passed preflight\n' >&2
  exit 1
fi
test ! -s "$RINGIL_TEST_LOG"
cp "$fixture/db.key.backup" "$vault/db.key"
cp "$fixture/db.crt.backup" "$vault/db.crt"
cp "$fixture/db.esl.backup" "$vault/db.esl"

cp "$vault/PK.crt" "$vault/db.crt"
: > "$RINGIL_TEST_LOG"
if bash "$script" build "$vault" "$fixture/invalid" > "$fixture/invalid.log" 2>&1; then
  printf 'Invalid key material passed preflight\n' >&2
  exit 1
fi
test ! -s "$RINGIL_TEST_LOG"

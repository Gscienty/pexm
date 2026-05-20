#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVICE_CE_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$SERVICE_CE_DIR")"
KEYS_DIR="$ROOT_DIR/keys"

mkdir -p "$KEYS_DIR"

if [ -f "$KEYS_DIR/ed25519_private.pem" ]; then
  echo "Keys already exist in $KEYS_DIR, skipping generation."
  echo "To regenerate, delete $KEYS_DIR first."
  exit 0
fi

echo "Generating Ed25519 key pair..."

openssl genpkey -algorithm Ed25519 -out "$KEYS_DIR/ed25519_private.pem"
openssl pkey -in "$KEYS_DIR/ed25519_private.pem" -pubout -out "$KEYS_DIR/ed25519_public.pem"

chmod 600 "$KEYS_DIR/ed25519_private.pem"
chmod 644 "$KEYS_DIR/ed25519_public.pem"

echo "Keys generated in $KEYS_DIR/"
echo "  - ed25519_private.pem (keep secret!)"
echo "  - ed25519_public.pem"
#!/bin/bash
set -xEeuo pipefail

BOT_DIR="$(realpath "$(dirname "$0")")"
BOT_BINARY="${BOT_DIR}/target/release/cthulhu-roller"

cd "${BOT_DIR}"
git pull -r
cargo build --release

# Create systemd service file dynamically
SERVICE_NAME="cthulhu-roller.service"
SERVICE_PATH="${HOME}/.config/systemd/user/${SERVICE_NAME}"

mkdir -p "$(dirname "${SERVICE_PATH}")"

cat >"${SERVICE_PATH}" <<EOF
[Unit]
Description=Cthulhu Roller
After=network.target

[Service]
ExecStart=${BOT_BINARY}
WorkingDirectory=${BOT_DIR}
Restart=always
RestartSec=3s

StartLimitIntervalSec=30
StartLimitBurst=3

[Install]
WantedBy=default.target
EOF

loginctl enable-linger "${USER}"
systemctl --user daemon-reload
systemctl --user enable "${SERVICE_NAME}"
systemctl --user restart "${SERVICE_NAME}"

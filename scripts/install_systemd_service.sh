#!/usr/bin/env bash
set -euo pipefail

PYTHON_PATH="${1:-python3}"
CONFIG_PATH="${2:-$(realpath ../config.yaml)}"
SERVICE_NAME="${3:-recall-pipeline}"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
UNIT_PATH="$UNIT_DIR/${SERVICE_NAME}.service"

mkdir -p "$UNIT_DIR"

cat >"$UNIT_PATH" <<EOF
[Unit]
Description=Recall Pipeline Service
After=default.target

[Service]
Type=simple
ExecStart=$PYTHON_PATH -m recall_pipeline.cli run --config $CONFIG_PATH
WorkingDirectory=$(realpath ..)
Restart=on-failure
Environment=PYTHONUNBUFFERED=1

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now "${SERVICE_NAME}.service"
echo "Installed user service ${SERVICE_NAME}.service"

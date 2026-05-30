#!/usr/bin/env bash
set -e
chmod +x scripts/*.sh
chmod +x .githooks/*
git config core.hooksPath .githooks
echo "Repository setup complete"

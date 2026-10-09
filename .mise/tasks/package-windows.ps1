#!/usr/bin/env pwsh

$ErrorActionPreference = "Stop"

npm ci
npm run tauri build

#!/usr/bin/env -S nu

const WIX_URL = "https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip"
const NSIS_URL = "https://github.com/tauri-apps/binary-releases/releases/download/nsis-3/nsis-3.zip"

let files_to_get = [
  {
    url: $WIX_URL,
    name: "wix314-binaries.zip",
    extract_path: "WixTools314"
  }
  {
    url: $NSIS_URL,
    name: "nsis-3.zip",
    extract_path: "NSIS"
  }
]

use std/log *

def --env  "windows cleanup" [] {
  log debug --short "Removing temp"
  rm --recursive temp
  let tauri_dir = $env.USERPROFILE | path join "AppData/Local/tauri"
  log info --short $"Removing NSIS and WixToolset from ($tauri_dir)"
  rm --recursive ($tauri_dir | path join "NSIS")
  rm --recursive ($tauri_dir | path join "WixTools314")
  log info --short "Removed all trash"
  if (!(./temp | path exists)) {
    mkdir temp
  }
    cd temp
}



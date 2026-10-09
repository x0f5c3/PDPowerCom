# PDPowerCom Copilot Instructions

## Commands

- Install the pinned Node/Rust/Nushell/PowerShell toolchains with `mise install`.
- Run the browser frontend only with `npm run dev`. Vite uses fixed port `1420`; `tauri dev` starts this command automatically.
- Run the desktop application with `npm run tauri dev`.
- Build the frontend bundle with `npm run build`; package the desktop app with `npm run tauri build`.
- Run the complete repository validation suite with `mise run check`. It installs locked Node dependencies, builds the frontend, and runs Rust formatting, Clippy, and tests.
- Validate Conventional Commit history with `mise run check-commits`.
- Build Windows installers with `mise run package-windows` on Windows.
- Check Rust changes from the repository root with `cargo check -p pdpowercom`.
- Run Rust tests with `cargo test -p pdpowercom`. To run one exact Rust test once it exists, use `cargo test -p pdpowercom test_name -- --exact`.
- There is no configured JavaScript test runner or JavaScript linter. There are currently no test files.
- Windows release packaging may need the WiX/NSIS bootstrap dependencies prepared by `.mise/tasks/windows_build_deps.ps1` when Tauri cannot download them.

## Automation

- Keep project automation in executable `.mise/tasks/*.nu` or `.mise/tasks/*.ps1` files; workflows invoke those tasks with `mise run`, rather than duplicating project commands.
- `.github/workflows/ci.yml` validates Conventional Commit history on every push and pull request, and runs the build checks for pushes and pull requests targeting `master`. `.github/workflows/release.yml` creates a GitHub release from the MSI and NSIS installers when a `v*` tag is pushed.
- This local repository has no configured remote. After adding one, make the first push with `git push -u origin master`; push a version tag such as `v0.1.3` only when it is ready to publish installers.

## Architecture

- This is a Tauri 2 desktop app: `src-tauri/src/main.rs` launches `pdpowercom_lib::run()`, and `src-tauri/src/lib.rs` registers the native commands and owns serial-port access. `src-tauri/tauri.conf.json` makes Tauri run Vite before development/build and load `dist/` in packaged builds.
- The Vue app mounts in `src/main.js`. `src/App.vue` is the application controller; it uses external template and stylesheet files through `<template src="./App.html">` and `<style src="./styles.css">`.
- The Rust backend filters discovered USB serial ports to the target VID/PID and serial-number prefixes, stores the open port in the process-wide `SERIAL_PORT` mutex, and sends `Opened`, `Error`, and raw `Read` events to Vue through a Tauri `Channel`. Keep all serial I/O behind `serial_get_available_ports`, `serial_open_port`, `serial_close_port`, and `serial_write`; register any new command in `tauri::generate_handler!`.
- The frontend calls those commands through `window.__TAURI__.core.invoke`. `SerialEventInit()` receives channel events, and `port_listening()` accumulates partial bytes, frames responses using `PROTOCOL_CONFIG`, then updates device/UI state. Device communication changes normally require matching edits to the command definition, response parser, and the timed requests in `DeviceInfoGet()` or periodic readers.

## Repository Conventions

- Use Conventional Commits for every commit. `cog.toml` is the shared Cocogitto policy and CI rejects non-conforming history; use `koji` to author an interactive compliant commit locally. Do not add emojis to commit subjects.
- Device protocol definitions are centralized in `PROTOCOL_CONFIG` in `src/App.vue`. Commands are byte arrays that end in `0x0a`; parsers identify frames by header, footer, and (where applicable) exact `frameLength`. Preserve the little-endian conversion and units: output voltage/current device values are millivolts in the wire protocol and become volts/amps in UI state; PD voltage uses tenths of a volt.
- Do not send device requests back-to-back. `COMMAND_DELAY` schedules the initial device-info sequence, and the separate 200 ms, 500 ms, and 1000 ms recursive timeouts maintain display, output-state, and input-state polling. Stop these loops when closing or handling a serial error.
- The UI uses a mixture of Vue refs and direct DOM updates/listeners. Follow the existing approach in `App.vue`/`App.html` for affected controls rather than assuming all state is template-reactive.
- The supported device variants are keyed by serial-number prefixes `AC` and `B3`. Update `DEVICE_RANGE_CONFIG` and the variant-specific firmware gates together when supporting another model.
- UI strings live in the four flat dictionaries under `src/locales/`. Add a key with the same spelling to `en.js`, `zh.js`, `ja.js`, and `ru.js`; translation lookup falls back to the key itself. The selected language is persisted as `localStorage.appLanguage`.
- Measurement history is maintained in the ECharts option state and limited to 20 points. If chart labels change, obtain labels through `t()` so applying a language refreshes the chart option consistently.

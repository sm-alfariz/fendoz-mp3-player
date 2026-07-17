
## Project Overview

**SonicWave** — Cross-platform MP3 desktop player built with Tauri 2.0 (Rust backend) + React 19 (TypeScript frontend) + Vite. Targets Windows, macOS, and Linux. Currently at scaffold stage (default Tauri + React template).

planned features: 10-band equalizer, audio visualizer (spectrum/circular/oscilloscope), synchronized .lrc lyrics, SQLite playlist CRUD with drag-and-drop.

## Commands

| Command | Action |
|---|---|
| `pnpm dev` | Start Vite dev server (port 1420) |
| `pnpm build` | `tsc && vite build` |
| `pnpm preview` | Vite preview |
| `pnpm tauri dev` | Run full Tauri desktop app in dev mode |
| `pnpm tauri build` | Build production Tauri bundle |
| `cargo build` (in `src-tauri/`) | Build Rust backend only |

## Architecture

```
mp3-player/
├── src/                    # React frontend (TypeScript)
│   ├── main.tsx            # Entry point
│   ├── App.tsx             # Root component
│   ├── App.css             # Global styles
│   └── vite-env.d.ts       # Vite type declarations
├── src-tauri/              # Rust backend (Tauri 2.0)
│   ├── src/
│   │   ├── main.rs         # Windows entry point
│   │   └── lib.rs          # Tauri app builder, commands
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri config (window, build, bundle)
│   └── capabilities/       # Tauri 2.0 permission capabilities
├── index.html              # HTML entry
├── vite.config.ts          # Vite config (port 1420, Tauri HMR)
├── tsconfig.json           # TypeScript config
└── package.json            # Frontend deps (React, Tauri API, Vite)
```

### Key architectural notes

- **Frontend-static, Rust-backend pattern**: React renders UI; all audio processing (FFT, EQ filters), file system access (MP3 scanning, ID3 tag reading), and SQLite operations run in Rust, exposed via Tauri commands and events.
- **Tauri 2.0 capabilities**: Permissions scoped in `src-tauri/capabilities/default.json`.
- **Vite dev server**: Fixed port 1420, HMR on 1421, ignores `src-tauri/` for watch.
- **pnpm** is the package manager (lockfile: `pnpm-lock.yaml`).
- **No tests exist yet** — the project is at initial scaffold stage.

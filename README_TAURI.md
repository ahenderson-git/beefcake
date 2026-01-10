# Tauri Implementation for beefcake

This project has been updated to use Tauri as the GUI framework with a TypeScript frontend.

## Structure

- `src/` - Rust backend logic (existing beefcake modules).
- `src/tauri_app.rs` - Tauri-specific Rust code (commands and setup).
- `src-frontend/` - TypeScript frontend source code.
- `tauri.conf.json` - Tauri configuration.
- `package.json` - Frontend dependencies and scripts.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (for frontend development)
- [Rust](https://rust-lang.org/) (already installed)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Running in Development

1. **Install frontend dependencies** (do this once):
   ```bash
   npm install
   ```

2. **Run the application**:

> [!IMPORTANT]
> **Do not run the application using `cargo run` or the IDE's default "Run" button on `main.rs`.**
> This will launch the app, but you will see a **"refused to connect"** error because the frontend development server (Vite) isn't running. Always use one of the methods below.

- **Recommended (IDE)**: Select **"Tauri Dev (GUI)"** from the Run configurations dropdown in your IDE and click the **Run** (Play) button. This handles starting both the frontend dev server and the Rust backend.
- **CLI**:
  ```bash
  npm run tauri dev
  ```
  or
  ```bash
  cargo tauri dev
  ```

## Backend Integration

Existing beefcake logic is available to the frontend via Tauri commands. 
See `src/tauri_app.rs` for how commands are implemented and `src-frontend/main.ts` for how they are called from TypeScript.

Example:
```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("analyze_file", { path: "/path/to/file.csv" });
```

## Troubleshooting

### "127.0.0.1 / localhost refused to connect"
This happens if the Vite development server is not running or is unreachable from the Tauri WebView.

**Checklist:**
1. **Did you run the right command?** 
   - ❌ `cargo run` -> **WRONG** (Vite won't start)
   - ❌ IDE Play button on `main.rs` -> **WRONG** (Vite won't start)
   - ✅ `npm run tauri dev` -> **CORRECT**
   - ✅ `cargo tauri dev` -> **CORRECT**
   - ✅ IDE "Tauri Dev (GUI)" configuration -> **CORRECT**

2. **Did you install dependencies?**
   - Run `npm install` in the project root. If you've already done it, try deleting `node_modules` and running it again.

3. **Is Vite actually running?**
   - While the app is open (even if it shows the error), open your web browser and go to `http://127.0.0.1:14206/`.
   - If it works in the browser but NOT in the app, your firewall or a proxy might be blocking the Tauri WebView.
   - If it does NOT work in the browser, check your terminal for errors from Vite.

4. **Port Conflict?**
   - Ensure no other application is using port `14206` or `14207`.

### Manual Fix / Verification
If you suspect dependencies are broken, run:
```bash
npm run reset
```
Then try `npm run tauri dev` again.

### Commands not working
If the UI loads but buttons do nothing, check the browser console (Right click -> Inspect) for errors. 
Common issues include:
- Missing capabilities in `capabilities/default.json`.
- Incorrect command names in `invoke()`.

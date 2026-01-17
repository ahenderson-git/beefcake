# 🛠️ Fixes for Windows Build Issues (EPERM & File Locking)

If you are seeing errors like `EPERM` during `vite build` or `failed to remove file ... beefcake.pdb`, it is usually caused by:
1. **OneDrive Syncing**: OneDrive locks files in the `dist` or `target` folders while they are being updated.
2. **Lingering Processes**: An instance of `beefcake.exe` or its WebView2 components is still running in the background.

## 🚀 Applied Fixes

I have implemented several automated fixes to mitigate these issues:

1.  **Automated Cleanup (`scripts/clean.js`)**: A new script that kills any running `beefcake` processes and attempts to remove locked files with retries (essential for OneDrive).
2.  **`prebuild` Hook**: `npm run build` now automatically runs the cleanup script before starting.
3.  **Vite Configuration**: Disabled Vite's default `emptyOutDir` in `vite.config.ts` because it is prone to `EPERM` on OneDrive. Cleanup is now handled more robustly by our script.
4.  **`build.rs` Protection**: Added logic to `build.rs` to ensure the `dist` directory exists with a placeholder before the Tauri proc-macro runs. This prevents the "frontendDist path doesn't exist" compilation error.

## 🛠️ How to fix if build still fails

### 1. Run the Lock Fixer
If you get a permission error, run:
```powershell
npm run fix-locks
```

### 2. Manual Cleanup (Last Resort)
If the above doesn't work, you can try this PowerShell command to force-kill everything related:
```powershell
Stop-Process -Name "beefcake" -Force; taskkill /F /FI "IMAGENAME eq msedgewebview2.exe" /T
```
*(Warning: This kills all WebView2 processes, including those for other apps.)*

### 3. OneDrive Recommendations
For the best development experience on Windows:
*   **Pause OneDrive** while building.
*   **Exclude `target`, `node_modules`, and `dist`** from OneDrive sync if possible.
*   Move the project to a non-OneDrive folder (e.g., `C:\dev\beefcake`).

## 🏗️ Building for Production
To build the app for production, use:
```powershell
npm run tauri build
```
This will now automatically run the cleanup script first.

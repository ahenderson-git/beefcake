# Port Management Guide

## Overview

Beefcake uses the following ports for development:

| Port  | Service            | Purpose                          |
|-------|--------------------|----------------------------------|
| 14206 | Vite Dev Server    | Main application HTTP server     |
| 14207 | Vite HMR WebSocket | Hot Module Replacement (HMR)     |

These ports are configured in:
- `vite.config.ts` - Server and HMR ports
- `tauri.conf.json` - Dev server URL
- `playwright.config.ts` - E2E test base URL

## Common Issues

### "Port is already in use" Error

**Symptoms:**
```
WebSocket server error: Port is already in use
Error: Port 14206 is already in use
```

**Cause:** A previous dev server didn't shut down cleanly, leaving a zombie process.

**Quick Fix:**

```bash
# Kill ports automatically
npm run kill-ports

# Or manually (PowerShell)
$proc = Get-NetTCPConnection -LocalPort 14206 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess
if ($proc) { Stop-Process -Id $proc -Force }

# Nuclear option: kill all Node processes
taskkill /IM node.exe /F
```

## Port Management Scripts

### Check Port Availability

```bash
npm run check-ports
```

Checks if ports 14206 and 14207 are available. Returns:
- Exit code `0`: All ports available
- Exit code `1`: One or more ports in use
- Shows PIDs and process names

**Custom ports:**
```bash
node scripts/check-ports.js 14206 14207 8080
```

### Kill Stuck Processes

```bash
npm run kill-ports
```

Safely terminates processes using ports 14206 and 14207.

**Custom ports:**
```bash
node scripts/kill-ports.js 14206 14207 8080
```

### Development Scripts with Auto-Cleanup

```bash
# Start dev server with port cleanup
npm run dev:clean

# Start Tauri dev with port cleanup
npm run tauri:dev:clean

# Run E2E tests with port cleanup
npm run test:e2e:clean

# Run all tests with port cleanup
npm run test:all:clean
```

## Configuration

### Vite Server (`vite.config.ts`)

```typescript
server: {
  port: 14206,           // Main dev server port
  strictPort: true,      // Fail if port unavailable (prevents silent failures)
  host: "localhost",
  hmr: {
    protocol: "ws",
    host: "localhost",
    port: 14207,         // HMR WebSocket port
  },
}
```

### Tauri Config (`tauri.conf.json`)

```json
{
  "build": {
    "devUrl": "http://localhost:14206"
  }
}
```

### Playwright Config (`playwright.config.ts`)

```typescript
export default defineConfig({
  use: {
    baseURL: 'http://localhost:14206',
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:14206',
    reuseExistingServer: false,  // Prevents port conflicts
  },
  globalTeardown: async () => {
    await cleanupPorts();  // Auto-cleanup after E2E tests
  },
});
```

## CI/CD Integration

### Pre-Flight Checks

Add port health check before starting servers:

```yaml
# GitHub Actions example
- name: Check ports
  run: npm run check-ports

- name: Start dev server
  run: npm run dev
```

### Cleanup on Failure

Ensure ports are cleaned up even if tests fail:

```yaml
- name: Run E2E tests
  run: npm run test:e2e
  continue-on-error: true

- name: Cleanup ports
  if: always()
  run: npm run kill-ports
```

## Troubleshooting

### Port Still in Use After Cleanup

1. **Check for multiple Node processes:**
   ```bash
   # Windows
   tasklist | findstr node.exe

   # Unix
   ps aux | grep node
   ```

2. **Kill all Node processes:**
   ```bash
   # Windows
   taskkill /IM node.exe /F

   # Unix
   killall node
   ```

3. **Restart your terminal/IDE** - Sometimes shells hold ports

### Changing Default Ports

If 14206/14207 conflict with other applications:

1. Update `vite.config.ts`:
   ```typescript
   server: {
     port: 15206,  // New port
     hmr: {
       port: 15207,  // New HMR port
     },
   }
   ```

2. Update `tauri.conf.json`:
   ```json
   {
     "build": {
       "devUrl": "http://localhost:15206"
     }
   }
   ```

3. Update `playwright.config.ts`:
   ```typescript
   baseURL: 'http://localhost:15206'
   ```

4. Update `scripts/kill-ports.js` and `scripts/check-ports.js`:
   ```javascript
   const DEFAULT_PORTS = [15206, 15207];
   ```

### Playwright Hangs on Teardown

**Symptom:** `npm run test:e2e` completes but terminal hangs

**Cause:** Vite server not terminating cleanly

**Fix:**
1. The `globalTeardown` hook in `playwright.config.ts` handles this
2. If still hanging, manually run: `npm run kill-ports`
3. Check `playwright-report/` for crash logs

### Permission Denied Errors

**Windows:** Run terminal as Administrator

**Unix:** Use `sudo` for port checks:
```bash
sudo lsof -i :14206
sudo kill -9 <PID>
```

## Best Practices

### Development Workflow

1. **Before starting work:**
   ```bash
   npm run check-ports  # Verify ports available
   npm run dev:clean    # Start with cleanup
   ```

2. **After work:**
   - Press `Ctrl+C` to stop dev server
   - Wait for cleanup message
   - If unsure: `npm run kill-ports`

3. **Before committing:**
   ```bash
   npm run test:all:clean  # Run all tests with cleanup
   ```

### Team Guidelines

- **Never commit with ports hardcoded** in code (use config files)
- **Always use `:clean` scripts** when debugging port issues
- **Document any port changes** in this file
- **Include port status** in bug reports

## Architecture Notes

### Why strictPort: true?

Vite's `strictPort: true` causes immediate failure if the port is unavailable. This prevents:
- Silent port conflicts
- Starting on wrong port (e.g., 14207 instead of 14206)
- Debugging false negatives (server on different port than expected)

**Trade-off:** Requires manual port cleanup, but prevents subtle bugs

### Why Two Ports?

- **14206:** Main HTTP server for HTML/JS/CSS
- **14207:** WebSocket for HMR (hot module replacement)

Separating them prevents HMR websocket upgrade conflicts with HTTP traffic.

### Automatic Cleanup

`playwright.config.ts` includes `globalTeardown` to automatically kill ports after E2E tests. This reduces "port in use" errors during development.

## Related Files

- `scripts/kill-ports.js` - Port cleanup utility
- `scripts/check-ports.js` - Port health check utility
- `vite.config.ts` - Port configuration
- `tauri.conf.json` - Dev server URL
- `playwright.config.ts` - E2E test configuration
- `e2e/server-lifecycle.spec.ts` - Port management E2E tests

## Support

If port issues persist after following this guide:

1. Check for conflicting applications (e.g., other dev servers)
2. Restart your computer (clears all ports)
3. File an issue with:
   - Output of `npm run check-ports`
   - Output of `netstat -ano | findstr :14206` (Windows)
   - Output of `lsof -i :14206` (Unix)

# Lifecycle UI Debugging Instructions

## Current Issue
The lifecycle rail shows "No dataset loaded" even though a dataset is analyzed.

## Debug Steps

### 1. Rebuild the Frontend
The TypeScript changes need to be compiled:

```powershell
npm run build
```

Or use the helper script:
```powershell
.\build-frontend.ps1
```

### 2. Restart the Application
- **Completely close Beefcake**
- **Restart it fresh**

### 3. Test with Console Open
1. Open Beefcake
2. Open Developer Tools (usually F12 or Ctrl+Shift+I)
3. Go to the Console tab
4. Analyze a dataset
5. Watch for console messages

### 4. Expected Console Output

When analysis completes, you should see:
```
Creating lifecycle dataset for: your_file.csv
Dataset created with ID: <uuid>
Versions JSON: <json string>
Parsed versions: <array>
Lifecycle dataset created successfully: <object>
Rendering lifecycle rail. currentDataset: <object>
```

### 5. If You See Errors

#### Error: "Failed to create lifecycle dataset"
This means the backend API call failed. Check:
- Is the backend running?
- Check the error details in the console
- The error will also show as a toast notification

#### Error: No console output at all
This means the TypeScript wasn't compiled. Steps:
1. Verify `npm run build` completed successfully
2. Check that `dist/` folder was updated (timestamps)
3. Completely close and restart Beefcake

#### Console shows: "Rendering lifecycle rail. currentDataset: null"
This means the lifecycle dataset creation failed silently. Check:
- The previous console logs for "Failed to create lifecycle dataset"
- The toast notification for error details

### 6. Manual Verification

Check if the build was successful:
```powershell
# Check timestamp of dist folder
ls dist/assets/*.js | Select-Object -First 5 | Format-Table LastWriteTime, Name

# Should show recent timestamps (just now)
```

### 7. Backend Verification

The lifecycle system requires these Tauri commands to be working:
- `lifecycle_create_dataset`
- `lifecycle_list_versions`

If these commands don't exist or fail, the frontend will gracefully degrade (no lifecycle, but analysis still works).

## What Should Happen (Success Case)

After analyzing a file, at the top of the Analyser view you should see:

```
┌─────────────────────────────────────────────────────────────┐
│ your_file.csv                                  1/6 stages   │
├─────────────────────────────────────────────────────────────┤
│ [Raw ✓] → [Profiled] → [Cleaning] → [Advanced] → ...       │
└─────────────────────────────────────────────────────────────┘
```

Instead of:
```
No dataset loaded
```

## Common Issues

### Issue 1: Old Code Running
**Symptom**: No console logs appear
**Solution**: Rebuild frontend, restart app completely

### Issue 2: Backend Not Implemented
**Symptom**: Console shows error calling `lifecycle_create_dataset`
**Solution**: Backend lifecycle commands may not be fully hooked up

### Issue 3: Path Issues
**Symptom**: Dataset created but with wrong path
**Solution**: Check that `path` variable is absolute path to file

## Next Steps After Debugging

Once you can see the console output:
1. Share the console logs
2. Check if there's a toast notification showing the error
3. We can then address the specific API or backend issue

## Files Modified
- `src-frontend/main.ts` - Added lifecycle creation after analysis
- `src-frontend/components/LifecycleRailComponent.ts` - New component
- `src-frontend/components/LifecycleComponent.ts` - New component
- `src-frontend/renderers/lifecycle.ts` - New rendering functions
- `src-frontend/api.ts` - New lifecycle API wrappers
- `src-frontend/types.ts` - New lifecycle types
- `src-frontend/style.css` - New lifecycle styles
- `src-frontend/renderers/layout.ts` - Added Lifecycle nav item

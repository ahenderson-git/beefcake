import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';

const distPath = path.resolve('dist');
const pdbPath = path.resolve('target/debug/beefcake.pdb');

function killProcess(name) {
  try {
    console.log(`Attempting to kill any running ${name} processes...`);
    if (process.platform === 'win32') {
      // Use taskkill with filter to avoid error if process not found
      // /T kills child processes (like WebView2 child processes)
      execSync(`taskkill /F /FI "IMAGENAME eq ${name}.exe" /T`, { stdio: 'ignore' });
    } else {
      execSync(`pkill -9 -f ${name}`, { stdio: 'ignore' });
    }
  } catch (e) {
    // Ignore errors
  }
}

function removeWithRetry(targetPath, retries = 5, delay = 1000) {
  for (let i = 0; i < retries; i++) {
    try {
      if (fs.existsSync(targetPath)) {
        if (fs.lstatSync(targetPath).isDirectory()) {
          fs.rmSync(targetPath, { recursive: true, force: true });
        } else {
          fs.unlinkSync(targetPath);
        }
        console.log(`Successfully removed ${targetPath}`);
      }
      return true;
    } catch (e) {
      if (i === retries - 1) {
        console.warn(`Failed to remove ${targetPath} after ${retries} attempts: ${e.message}`);
        return false;
      }
      console.log(`Retry ${i + 1}: ${targetPath} is locked (possibly by OneDrive or another process), waiting ${delay}ms...`);
      // Synchronous sleep
      const start = Date.now();
      while (Date.now() - start < delay) {}
    }
  }
}

console.log('--- Beefcake Build Cleanup ---');

// 1. Kill the app if it's running
killProcess('beefcake');

// 2. Try to clean dist
console.log('Cleaning dist directory...');
removeWithRetry(distPath);

// 3. Try to clean problematic PDB
console.log('Cleaning problematic PDB file...');
removeWithRetry(pdbPath);

console.log('Cleanup complete.');

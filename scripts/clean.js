import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';

const distPath = path.resolve('dist');
const debugPath = path.resolve('target/debug');

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

function cleanPdbs(dir) {
  if (!fs.existsSync(dir)) return;
  const files = fs.readdirSync(dir);
  for (const file of files) {
    const fullPath = path.join(dir, file);
    if (fs.lstatSync(fullPath).isDirectory()) {
      cleanPdbs(fullPath);
    } else if (file.endsWith('.pdb')) {
      removeWithRetry(fullPath);
    }
  }
}

console.log('--- Beefcake Build Cleanup ---');
const pdbsOnly = process.argv.includes('--pdbs-only');

// 1. Kill the app if it's running
killProcess('beefcake');

if (!pdbsOnly) {
  // 2. Try to clean dist
  console.log('Cleaning dist directory...');
  removeWithRetry(distPath);
}

// 3. Try to clean problematic PDBs
console.log('Cleaning problematic PDB files...');
cleanPdbs(debugPath);

console.log('Cleanup complete.');

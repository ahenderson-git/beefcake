#!/usr/bin/env node

/**
 * Debug script to diagnose Vite dev server caching issues
 * Run with: node scripts/debug-dev-server.js
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🔍 Beefcake Dev Server Debug Tool\n');

// Check if files contain our fixes
console.log('📁 Checking source files...');
const sidebarPath = path.join(__dirname, '../src-frontend/renderers/analyser/sidebar.ts');
const rowPath = path.join(__dirname, '../src-frontend/renderers/analyser/row.ts');

function checkFileContains(filePath, searchString, description) {
  try {
    const content = fs.readFileSync(filePath, 'utf8');
    const found = content.includes(searchString);
    console.log(`  ${found ? '✅' : '❌'} ${description}`);
    return found;
  } catch (err) {
    console.log(`  ❌ ${description} (Error: ${err.message})`);
    return false;
  }
}

const fix1 = checkFileContains(
  sidebarPath,
  "currentStage === 'Raw'",
  'Sidebar fix (Raw stage button logic)'
);

const fix2 = checkFileContains(
  rowPath,
  "currentStage !== 'Profiled'",
  'Row fix (checkbox enabled in Profiled)'
);

if (!fix1 || !fix2) {
  console.log('\n❌ PROBLEM: Source files missing fixes!');
  console.log('   The code changes were not properly saved.');
  process.exit(1);
}

console.log('\n✅ Source files contain correct fixes\n');

// Check for Vite cache
console.log('🗑️  Checking for Vite cache...');
const viteCachePath = path.join(__dirname, '../node_modules/.vite');
if (fs.existsSync(viteCachePath)) {
  console.log('  ⚠️  Vite cache exists at: node_modules/.vite');
  console.log('  💡 This might be serving stale code');

  try {
    console.log('  🧹 Attempting to clear cache...');
    fs.rmSync(viteCachePath, { recursive: true, force: true });
    console.log('  ✅ Vite cache cleared');
  } catch (err) {
    console.log(`  ❌ Failed to clear cache: ${err.message}`);
  }
} else {
  console.log('  ✅ No Vite cache found');
}

// Check for running processes
console.log('\n🔎 Checking for running processes...');
try {
  if (process.platform === 'win32') {
    const processes = execSync('tasklist /FI "IMAGENAME eq node.exe"', { encoding: 'utf8' });
    const nodeCount = (processes.match(/node.exe/gi) || []).length;
    console.log(`  ℹ️  Found ${nodeCount} Node.js processes running`);

    const tauriProcesses = execSync('tasklist /FI "IMAGENAME eq beefcake.exe"', { encoding: 'utf8' });
    const tauriCount = (tauriProcesses.match(/beefcake.exe/gi) || []).length;
    console.log(`  ℹ️  Found ${tauriCount} Beefcake processes running`);

    if (nodeCount > 1 || tauriCount > 0) {
      console.log('  ⚠️  Multiple processes detected - old dev server might be running');
      console.log('  💡 Run: npm run kill-ports');
    }
  }
} catch (err) {
  console.log(`  ⚠️  Could not check processes: ${err.message}`);
}

// Check OneDrive status
console.log('\n☁️  Checking OneDrive sync status...');
const projectPath = path.join(__dirname, '..');
if (projectPath.includes('OneDrive')) {
  console.log('  ⚠️  Project is in OneDrive folder');
  console.log('  💡 OneDrive can interfere with file watching');
  console.log('  💡 Ensure files are fully synced (green checkmark icon)');

  const stats = fs.statSync(sidebarPath);
  const mtime = stats.mtime;
  const now = new Date();
  const ageMinutes = (now - mtime) / 1000 / 60;

  console.log(`  ℹ️  sidebar.ts last modified: ${ageMinutes.toFixed(1)} minutes ago`);
  if (ageMinutes < 1) {
    console.log('  ⚠️  File very recently modified - OneDrive might still be syncing');
  }
} else {
  console.log('  ✅ Project not in OneDrive');
}

// Recommendations
console.log('\n📋 Recommendations:');
console.log('  1. Stop any running Beefcake instances');
console.log('  2. Run: npm run kill-ports');
console.log('  3. Run: npm run tauri:dev:clean');
console.log('  4. When app opens, press Ctrl+Shift+R (hard refresh)');
console.log('  5. If still failing, open DevTools (Ctrl+Shift+I) and check console for errors');

console.log('\n✅ Debug check complete\n');

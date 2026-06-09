const { app, BrowserWindow, ipcMain, shell, dialog } = require('electron');
const path = require('path');
const { spawn, execSync } = require('child_process');
const net = require('net');

let mainWindow = null;
let proxyProcess = null;
let proxyPort = 8080;

// --- Find the bundled Rust binary ---
function getBinaryPath() {
  const isDev = !app.isPackaged;
  if (isDev) {
    return path.join(__dirname, '..', 'target', 'release', 'oplire.exe');
  }
  return path.join(process.resourcesPath, 'oplire.exe');
}

// --- Check if port is free ---
function isPortFree(port) {
  return new Promise(resolve => {
    const server = net.createServer();
    server.listen(port, '127.0.0.1');
    server.on('listening', () => { server.close(); resolve(true); });
    server.on('error', () => resolve(false));
  });
}

// --- Find a free port ---
async function findFreePort(start = 8080) {
  for (let port = start; port < start + 100; port++) {
    if (await isPortFree(port)) return port;
  }
  return start;
}

// --- Start the proxy server ---
function startProxy(port) {
  const bin = getBinaryPath();
  const fs = require('fs');

  if (!fs.existsSync(bin)) {
    dialog.showErrorBox(
      'oplire binary not found',
      `Expected at: ${bin}\n\nPlease build the Rust binary first:\ncd .. && cargo build --release`
    );
    app.quit();
    return null;
  }

  const child = spawn(bin, ['proxy', '--listen', `127.0.0.1:${port}`], {
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  });

  child.stdout.on('data', d => console.log('[oplire]', d.toString().trim()));
  child.stderr.on('data', d => console.error('[oplire]', d.toString().trim()));
  child.on('error', err => console.error('[oplire] spawn error:', err.message));
  child.on('exit', code => {
    console.log(`[oplire] exited with code ${code}`);
    proxyProcess = null;
  });

  return child;
}

// --- Wait for proxy to be ready ---
function waitForProxy(port, timeout = 10000) {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const check = () => {
      const req = net.request(`http://127.0.0.1:${port}/health`);
      req.on('response', res => {
        if (res.statusCode === 200) return resolve(true);
        retry();
      });
      req.on('error', retry);
      req.end();
    };
    const retry = () => {
      if (Date.now() - start > timeout) return reject(new Error('Proxy timeout'));
      setTimeout(check, 300);
    };
    check();
  });
}

// --- TUI mode (opens a real terminal window) ---
function launchTUI() {
  const bin = getBinaryPath();
  const fs = require('fs');

  if (!fs.existsSync(bin)) {
    console.error('\x1b[31moplire binary not found.\x1b[0m');
    console.error(`Expected at: ${bin}`);
    console.error('\nBuild it first: cd .. && cargo build --release\n');
    process.exit(1);
  }

  const isWin = process.platform === 'win32';

  if (isWin) {
    // Open Windows Terminal / cmd with the binary
    const cmd = `start cmd /k "${bin}" status`;
    spawn('cmd', ['/c', cmd], { stdio: 'ignore', detached: true, windowsHide: false });
  } else {
    // Linux/macOS: try to open a terminal emulator
    const term = process.env.TERM || 'x-terminal-emulator';
    spawn(term, ['-e', bin, 'status'], { stdio: 'inherit', detached: true });
  }

  app.quit();
}

// --- Create the main window (Web GUI mode) ---
async function launchGUI() {
  proxyPort = await findFreePort(8080);

  console.log(`[oplire] Starting proxy on port ${proxyPort}...`);
  proxyProcess = startProxy(proxyPort);

  try {
    await waitForProxy(proxyPort);
    console.log(`[oplire] Proxy ready at http://127.0.0.1:${proxyPort}`);
  } catch (e) {
    console.error('[oplire] Proxy failed to start:', e.message);
    dialog.showErrorBox('Proxy failed to start', e.message);
    app.quit();
    return;
  }

  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 900,
    minHeight: 600,
    title: 'oplire',
    backgroundColor: '#18181b',
    titleBarStyle: 'hidden',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.loadURL(`http://127.0.0.1:${proxyPort}`);
  mainWindow.setMenuBarVisibility(false);

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

// --- App lifecycle ---
const gotLock = app.requestSingleInstanceLock();

if (!gotLock) {
  app.quit();
} else {
  app.on('second-instance', () => {
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.focus();
    }
  });

  app.whenReady().then(() => {
    // Check if TUI mode
    const isTUI = process.argv.includes('--tui') || process.argv.includes('-t');
    if (isTUI) {
      launchTUI();
      return;
    }
    launchGUI();
  });

  app.on('window-all-closed', () => {
    if (proxyProcess) {
      proxyProcess.kill();
      proxyProcess = null;
    }
    app.quit();
  });

  app.on('before-quit', () => {
    if (proxyProcess) {
      proxyProcess.kill();
      proxyProcess = null;
    }
  });
}

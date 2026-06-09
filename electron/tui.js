#!/usr/bin/env node
/**
 * oplire TUI - Terminal UI mode
 * Shows instructions and provides full CLI access.
 * Usage: oplire --tui  OR  oplire -t  OR  oplire tui
 */

const { spawn, execSync } = require('child_process');
const path = require('path');
const fs = require('fs');
const readline = require('readline');

// --- Colors ---
const C = {
  reset:   '\x1b[0m',
  bold:    '\x1b[1m',
  dim:     '\x1b[2m',
  red:     '\x1b[31m',
  green:   '\x1b[32m',
  yellow:  '\x1b[33m',
  blue:    '\x1b[34m',
  magenta: '\x1b[35m',
  cyan:    '\x1b[36m',
  white:   '\x1b[37m',
  bg:      '\x1b[48;5;234m',
};

const logo = `
${C.green}      ____                                         
     / __ \\__   _____  _____________  _____   ____
    / / / / | / / _ \\/ ___/ ___/ _ \\/ __ \\/ __ \\
   / /_/ /| |/ /  __/ /  (__  )  __/ /_/ / / / /
   \\____/ |___/\\___/_/  /____/\\___/ .___/_/ /_/ 
                                 /_/${C.reset}              ${C.dim}v2.6.3${C.reset}
`;

const MENU = [
  { key: '1', cmd: ['status'],         label: 'Show status' },
  { key: '2', cmd: ['proxy'],          label: 'Start proxy (foreground)' },
  { key: '3', cmd: ['config', 'show'], label: 'Show config' },
  { key: '4', cmd: ['config', 'set'],  label: 'Set config' },
  { key: '5', cmd: ['install'],        label: 'Install / Repair' },
  { key: '6', cmd: ['reset'],          label: 'Reset WARP license' },
  { key: '7', cmd: ['gui'],            label: 'Launch web GUI' },
  { key: '8', cmd: ['doctor'],         label: 'Run diagnostics' },
  { key: 'q', cmd: null,               label: 'Quit' },
];

function printMenu() {
  console.log(logo);
  console.log(`  ${C.bold}Menu${C.reset}\n`);
  for (const item of MENU) {
    console.log(`  ${C.cyan}${item.key}${C.reset}  ${C.white}${item.label}${C.reset}`);
  }
  console.log();
}

function getBinaryPath() {
  const isDev = !fs.existsSync(path.join(__dirname, '..', 'app.asar'));
  if (isDev) {
    return path.join(__dirname, '..', 'target', 'release', 'oplire.exe');
  }
  return path.join(process.resourcesPath, 'oplire.exe');
}

function runCommand(args) {
  const bin = getBinaryPath();
  if (!fs.existsSync(bin)) {
    console.error(`${C.red}Binary not found: ${bin}${C.reset}`);
    console.error(`${C.dim}Build it: cd .. && cargo build --release${C.reset}\n`);
    return;
  }

  const child = spawn(bin, args, { stdio: 'inherit', detached: false });
  child.on('exit', code => {
    if (code !== 0) {
      console.log(`${C.red}Command exited with code ${code}${C.reset}\n`);
    }
    showPrompt();
  });
}

function showPrompt() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  rl.question(`${C.green}> ${C.reset}`, answer => {
    rl.close();
    const input = answer.trim().toLowerCase();

    if (input === 'q' || input === 'quit' || input === 'exit') {
      console.log(`${C.dim}Goodbye.${C.reset}\n`);
      process.exit(0);
    }

    if (input === 'help' || input === '?') {
      printMenu();
      showPrompt();
      return;
    }

    // Direct command passthrough
    if (input.startsWith('oplire ')) {
      runCommand(input.slice(7).split(/\s+/));
      return;
    }
    if (input === 'opencode' || input.startsWith('opencode ') ||
        input === 'claude' || input.startsWith('claude ') ||
        input === 'proxy' || input.startsWith('proxy ')) {
      runCommand(input.split(/\s+/));
      return;
    }

    // Menu selection
    const selected = MENU.find(m => m.key === input);
    if (selected && selected.cmd) {
      runCommand(selected.cmd);
      return;
    }

    console.log(`${C.yellow}Unknown command. Type 'help' for menu.${C.reset}\n`);
    showPrompt();
  });
}

// --- Main ---
printMenu();
showPrompt();

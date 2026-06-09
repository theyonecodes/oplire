# build.ps1 - oplire build script
# Double-click this file or run: .\build.ps1

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Green
Write-Host "         oplire - Build Script" -ForegroundColor Green
Write-Host "  ========================================" -ForegroundColor Green
Write-Host ""

# --- Load Visual Studio environment ---
function Find-VSCompiler {
    # Check if cl.exe is already available
    if (Get-Command cl -ErrorAction SilentlyContinue) { return $true }

    # Find vcvarsall.bat
    $paths = @(
        "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat",
        "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat",
        "C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat",
        "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat",
        "C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat",
        "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat"
    )

    foreach ($p in $paths) {
        if (Test-Path $p) {
            Write-Host "  Loading Visual Studio environment..." -ForegroundColor Yellow
            $envOut = cmd /c "`"$p`" x64 && set" 2>&1
            foreach ($line in $envOut) {
                if ($line -match '^([^=]+)=(.*)$') {
                    [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
                }
            }
            if (Get-Command cl -ErrorAction SilentlyContinue) {
                Write-Host "  [OK] Visual Studio compiler loaded" -ForegroundColor Green
                return $true
            }
        }
    }

    return $false
}

if (-not (Find-VSCompiler)) {
    Write-Host "  [FAIL] Could not find C compiler" -ForegroundColor Red
    Write-Host ""
    Write-Host "  Options:" -ForegroundColor Yellow
    Write-Host "    1. Open 'Developer PowerShell for VS 2022' and run: .\build.ps1"
    Write-Host "    2. Or install VS Build Tools with C++ workload"
    Write-Host ""
    Read-Host "  Press Enter to exit"
    exit 1
}

# --- Check Rust ---
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "  [FAIL] Rust not found" -ForegroundColor Red
    Write-Host "  Install from: https://rustup.rs"
    Read-Host "  Press Enter to exit"
    exit 1
}
Write-Host "  [OK] Rust found" -ForegroundColor Green

# --- Check Node ---
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "  [FAIL] Node.js not found" -ForegroundColor Red
    Write-Host "  Install from: https://nodejs.org"
    Read-Host "  Press Enter to exit"
    exit 1
}
Write-Host "  [OK] Node.js found" -ForegroundColor Green

# --- Check compiler ---
$hasCompiler = (Get-Command cl -ErrorAction SilentlyContinue) -or (Get-Command gcc -ErrorAction SilentlyContinue)
if (-not $hasCompiler) {
    Write-Host "  [FAIL] C compiler not available" -ForegroundColor Red
    Read-Host "  Press Enter to exit"
    exit 1
}
Write-Host "  [OK] C compiler found" -ForegroundColor Green

Write-Host ""
Write-Host "  Building Rust backend..." -ForegroundColor Cyan
Write-Host "  ------------------------" -ForegroundColor Cyan

cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "  [FAIL] Rust build failed" -ForegroundColor Red
    Read-Host "  Press Enter to exit"
    exit 1
}
Write-Host "  [OK] Rust binary built" -ForegroundColor Green

Write-Host ""
Write-Host "  Building Electron app..." -ForegroundColor Cyan
Write-Host "  ------------------------" -ForegroundColor Cyan

Push-Location "electron"
try {
    if (-not (Test-Path "node_modules")) {
        Write-Host "  Installing dependencies..." -ForegroundColor Yellow
        npm install
        if ($LASTEXITCODE -ne 0) {
            throw "npm install failed"
        }
    }
    npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "Electron build failed"
    }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "  ========================================" -ForegroundColor Green
Write-Host "  Build complete!" -ForegroundColor Green
Write-Host "  ========================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Installer: electron\dist\oplire Setup.exe" -ForegroundColor Cyan
Write-Host ""
Read-Host "  Press Enter to exit"

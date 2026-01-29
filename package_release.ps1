# Orunla Release Packaging Script (Windows)
# Run this to create distribution packages

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "  ORUNLA - Release Packaging Script" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""

# Create dist directories
Write-Host "📁 Creating distribution directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "dist\orunla-windows" | Out-Null
New-Item -ItemType Directory -Force -Path "dist\orunla-mac\ui" | Out-Null

# Build Windows desktop app
Write-Host "🔨 Building Windows desktop app..." -ForegroundColor Yellow
npm run tauri build

# Build Rust binaries
Write-Host "🔨 Building Rust binaries..." -ForegroundColor Yellow
cargo build --release --bin orunla_cli
cargo build --release --bin orunla_mcp

# Package Windows distribution
Write-Host "📦 Packaging Windows distribution..." -ForegroundColor Yellow
Copy-Item "src-tauri\target\release\app.exe" "dist\orunla-windows\Orunla.exe" -ErrorAction SilentlyContinue
Copy-Item "src-tauri\target\release\onnxruntime.dll" "dist\orunla-windows\" -ErrorAction SilentlyContinue
Copy-Item "target\release\orunla_mcp.exe" "dist\orunla-windows\"
Copy-Item "target\release\orunla_cli.exe" "dist\orunla-windows\" -ErrorAction SilentlyContinue
Copy-Item "README-WINDOWS.md" "dist\orunla-windows\README.md"
Copy-Item "docs\CLI.md" "dist\orunla-windows\" -ErrorAction SilentlyContinue
Copy-Item "docs\MCP.md" "dist\orunla-windows\" -ErrorAction SilentlyContinue
Copy-Item "API_REFERENCE.md" "dist\orunla-windows\" -ErrorAction SilentlyContinue

# Package Mac distribution (browser launcher only)
Write-Host "📦 Packaging Mac distribution..." -ForegroundColor Yellow
Copy-Item "launch_orunla.sh" "dist\orunla-mac\"
Copy-Item "ui\index.html" "dist\orunla-mac\ui\"
Copy-Item "ui\main.js" "dist\orunla-mac\ui\"
Copy-Item "README-MAC.md" "dist\orunla-mac\README.md"

# Create zip archives
Write-Host "🗜️  Creating zip archives..." -ForegroundColor Yellow
Compress-Archive -Path "dist\orunla-windows\*" -DestinationPath "dist\orunla-windows.zip" -Force
Compress-Archive -Path "dist\orunla-mac\*" -DestinationPath "dist\orunla-mac.zip" -Force

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host "  ✅ Release packages created successfully!" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Green
Write-Host ""
Write-Host "📦 Windows package: dist\orunla-windows.zip" -ForegroundColor Cyan
Write-Host "📦 Mac package: dist\orunla-mac.zip" -ForegroundColor Cyan
Write-Host ""
Write-Host "Distribution folder structure:" -ForegroundColor Yellow
Get-ChildItem "dist\" -Recurse | Where-Object { !$_.PSIsContainer } | Select-Object -ExpandProperty FullName | ForEach-Object {
    $relativePath = $_.Replace((Get-Location).Path, '.')
    Write-Host "  $relativePath" -ForegroundColor Gray
}

$ErrorActionPreference = "Stop"
$REPO = "ducphanvanntq/tool-change-account"
$BINARY_NAME = "rust-cli.exe"
$INSTALL_NAME = "tool-change-account.exe"

Write-Host "[INFO] Fetching latest release..." -ForegroundColor Green

$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
$tag = $release.tag_name
$ARTIFACT = "tool-change-account-windows-x86_64.zip"
$DOWNLOAD_URL = "https://github.com/$REPO/releases/download/$tag/$ARTIFACT"

Write-Host "[INFO] Version: $tag" -ForegroundColor Green
Write-Host "[INFO] Downloading: $ARTIFACT" -ForegroundColor Green

$TMP_DIR = Join-Path $env:TEMP "tca_install"
if (Test-Path $TMP_DIR) { Remove-Item $TMP_DIR -Recurse -Force }
New-Item -ItemType Directory -Path $TMP_DIR | Out-Null

$zipPath = Join-Path $TMP_DIR $ARTIFACT
Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $zipPath

Write-Host "[INFO] Extracting..." -ForegroundColor Green
Expand-Archive -Path $zipPath -DestinationPath $TMP_DIR -Force

$INSTALL_DIR = Join-Path $env:LOCALAPPDATA "tool-change-account"
if (!(Test-Path $INSTALL_DIR)) { New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null }

Move-Item -Path (Join-Path $TMP_DIR $BINARY_NAME) -Destination (Join-Path $INSTALL_DIR $INSTALL_NAME) -Force

$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$INSTALL_DIR*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$INSTALL_DIR", "User")
    Write-Host "[INFO] Added $INSTALL_DIR to PATH" -ForegroundColor Yellow
}

Remove-Item $TMP_DIR -Recurse -Force

Write-Host "[INFO] Installed successfully!" -ForegroundColor Green
Write-Host ""
& (Join-Path $INSTALL_DIR $INSTALL_NAME) info

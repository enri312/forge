<#
.SYNOPSIS
Instala la última release x86_64 de FORGE para Windows y verifica su SHA-256.
.EXAMPLE
iwr https://raw.githubusercontent.com/enri312/forge/main/install.ps1 -useb | iex
#>

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ForgeRepo = "enri312/forge"
$ForgeTarget = "x86_64-pc-windows-msvc"
$ForgeAsset = "forge-$ForgeTarget.zip"
$ForgeBaseUrl = "https://github.com/$ForgeRepo/releases/latest/download"
$ForgeInstallDir = if ($env:FORGE_INSTALL_DIR) {
    $env:FORGE_INSTALL_DIR
} else {
    Join-Path $env:USERPROFILE ".cargo\bin"
}
$ForgeTempDir = Join-Path ([IO.Path]::GetTempPath()) ("forge-install-" + [Guid]::NewGuid().ToString("N"))

try {
    New-Item -ItemType Directory -Path $ForgeTempDir | Out-Null
    $ForgeArchive = Join-Path $ForgeTempDir $ForgeAsset
    $ForgeChecksumFile = "$ForgeArchive.sha256"

    Write-Host "Instalando FORGE para $ForgeTarget..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri "$ForgeBaseUrl/$ForgeAsset" -OutFile $ForgeArchive -UseBasicParsing
    Invoke-WebRequest -Uri "$ForgeBaseUrl/$ForgeAsset.sha256" -OutFile $ForgeChecksumFile -UseBasicParsing

    $ForgeChecksumText = Get-Content -LiteralPath $ForgeChecksumFile -Raw
    $ForgeChecksumMatch = [Regex]::Match($ForgeChecksumText, "(?i)\b[0-9a-f]{64}\b")
    if (-not $ForgeChecksumMatch.Success) {
        throw "El archivo de checksum publicado no contiene un SHA-256 válido."
    }

    $ForgeExpectedHash = $ForgeChecksumMatch.Value.ToLowerInvariant()
    $ForgeActualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $ForgeArchive).Hash.ToLowerInvariant()
    if ($ForgeActualHash -ne $ForgeExpectedHash) {
        throw "El SHA-256 de $ForgeAsset no coincide; no se instalará."
    }

    $ForgeExtractDir = Join-Path $ForgeTempDir "extracted"
    Expand-Archive -LiteralPath $ForgeArchive -DestinationPath $ForgeExtractDir
    $ForgeBinary = Get-ChildItem -LiteralPath $ForgeExtractDir -Filter "forge.exe" -File -Recurse | Select-Object -First 1
    if (-not $ForgeBinary) {
        throw "El paquete verificado no contiene forge.exe."
    }

    New-Item -ItemType Directory -Force -Path $ForgeInstallDir | Out-Null
    Copy-Item -LiteralPath $ForgeBinary.FullName -Destination (Join-Path $ForgeInstallDir "forge.exe") -Force

    $ForgeUserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    $ForgePathEntries = @($ForgeUserPath -split ";" | Where-Object { $_ })
    if ($ForgePathEntries -notcontains $ForgeInstallDir) {
        $ForgeUpdatedPath = (@($ForgeInstallDir) + $ForgePathEntries) -join ";"
        [Environment]::SetEnvironmentVariable("PATH", $ForgeUpdatedPath, "User")
        Write-Host "Se añadió $ForgeInstallDir al PATH del usuario; abre una terminal nueva." -ForegroundColor Yellow
    }

    Write-Host "FORGE se instaló en $ForgeInstallDir\forge.exe" -ForegroundColor Green
} finally {
    if (Test-Path -LiteralPath $ForgeTempDir) {
        Remove-Item -LiteralPath $ForgeTempDir -Recurse -Force
    }
}

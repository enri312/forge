<#
.SYNOPSIS
    Instalador automatizado de FORGE Build System para Windows.
.DESCRIPTION
    Descarga el binario precompilado x86_64-pc-windows-msvc desde GitHub Releases
    y lo inyecta en la carpeta ~/.cargo/bin (agregándola al PATH del usuario si no existe).
.EXAMPLE
    iwr https://raw.githubusercontent.com/enri312/forge/main/install.ps1 -useb | iex
#>

$ErrorActionPreference = "Stop"

$Repo = "enri312/forge"
$InstallDir = "$env:USERPROFILE\.cargo\bin"
$BinName = "forge.exe"
$Target = "x86_64-pc-windows-msvc"

Write-Host "`n🔥 Instalando FORGE Build System...`n" -ForegroundColor Cyan

# 1. Asegurar directorio
If (!(Test-Path -Path $InstallDir)) {
    Write-Host "Creando directorio de instalacion: $InstallDir"
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# 2. Obtener URL del ZIP de la ultima Release de GitHub
Write-Host "🔍 Buscando ultima version (Target: $Target)..."
try {
    # Request a la GitHub API
    $ReleasesApi = "https://api.github.com/repos/$Repo/releases/latest"
    $ReleaseData = Invoke-RestMethod -Uri $ReleasesApi -UseBasicParsing
    
    # Filtrar el target zip particular
    $Asset = $ReleaseData.assets | Where-Object { $_.name -match "$Target.zip" } | Select-Object -First 1
    
    if (-not $Asset) {
        Write-Host "⚠️ No se encontro una Build precompilada para Windows ($Target) en la version $($ReleaseData.tag_name)." -ForegroundColor Yellow
        Write-Host "🔨 Intenta instalar manualmente mediante Cargo: cargo install forge-cli"
        exit 1
    }

    $DownloadUrl = $Asset.browser_download_url
} catch {
    Write-Host "❌ Error contactando API de GitHub: $_" -ForegroundColor Red
    exit 1
}

Write-Host "⬇️ Descargando binario: $DownloadUrl"

# 3. Descargar y Extraer
$ZipPath = Join-Path -Path $env:TEMP -ChildPath "forge_install.zip"
$ExtractPath = Join-Path -Path $env:TEMP -ChildPath "forge_extracted"

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
    
    if (Test-Path $ExtractPath) { Remove-Item -Recurse -Force $ExtractPath }
    Expand-Archive -Path $ZipPath -DestinationPath $ExtractPath -Force

    # Mover a la ruta final
    $ExtractedBin = Join-Path -Path $ExtractPath -ChildPath $BinName
    if (Test-Path $ExtractedBin) {
        Move-Item -Path $ExtractedBin -Destination (Join-Path $InstallDir $BinName) -Force
    } else {
        # Posiblemente el zip tenga una carpeta interna con el nombre del target
        $NestedBin = Join-Path -Path $ExtractPath -ChildPath "$Target\$BinName"
        Move-Item -Path $NestedBin -Destination (Join-Path $InstallDir $BinName) -Force
    }

} catch {
    Write-Host "❌ Fallo al descargar o extraer el ZIP: $_" -ForegroundColor Red
    exit 1
} finally {
    # Limpieza
    if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }
    if (Test-Path $ExtractPath) { Remove-Item -Recurse -Force $ExtractPath }
}

# 4. Asegurar que .cargo/bin esté en el PATH
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notmatch [regex]::Escape($InstallDir)) {
    Write-Host "🔧 Agregando $InstallDir a tu variable PATH del sistema..." -ForegroundColor Magenta
    [Environment]::SetEnvironmentVariable("PATH", "$InstallDir;$UserPath", "User")
    Write-Host "⚠️  Por favor reinicia tu terminal para aplicar cambios en el PATH." -ForegroundColor Yellow
}

Write-Host "`n✅ FORGE instalado exitosamente en $InstallDir\$BinName`n" -ForegroundColor Green
Write-Host "🚀 Ejecuta 'forge --version' para probar (reinicia tu consola si es necesario).`n"

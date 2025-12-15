# --- 1. SETUP WINDOWS (ns.exe + Icona Ico) ---
$CurrentDir = Get-Location
$ExePath = "$CurrentDir\target\debug\ns.exe"
$IconPath = "$CurrentDir\bin\nodestract.ico" # Windows usa il file .ico fisico
$Extension = ".ns"
$ProgID = "NodeStract.Source"

Write-Host ">>> CONFIGURAZIONE WINDOWS (ns.exe) <<<" -ForegroundColor Cyan

# Pulizia vecchie associazioni
Remove-Item "HKCU:\Software\Classes\$Extension" -Recurse -ErrorAction SilentlyContinue
Remove-Item "HKCU:\Software\Classes\$ProgID" -Recurse -ErrorAction SilentlyContinue

# Nuova Associazione
New-Item -Path "HKCU:\Software\Classes\$Extension" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\$Extension" -Name "(Default)" -Value $ProgID

New-Item -Path "HKCU:\Software\Classes\$ProgID" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\$ProgID" -Name "(Default)" -Value "NodeStract File"

# Icona Windows (Usa il file .ico diretto, più affidabile dell'exe embedded per ora)
New-Item -Path "HKCU:\Software\Classes\$ProgID\DefaultIcon" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\$ProgID\DefaultIcon" -Name "(Default)" -Value "$IconPath"

# Comando Apertura
New-Item -Path "HKCU:\Software\Classes\$ProgID\shell\open\command" -Force | Out-Null
Set-ItemProperty -Path "HKCU:\Software\Classes\$ProgID\shell\open\command" -Name "(Default)" -Value "cmd.exe /k `"$ExePath`" build `"%1`""

# Refresh Explorer
$code = @'
using System;
using System.Runtime.InteropServices;
public class WinRefresher {
    [DllImport("Shell32.dll")]
    public static extern void SHChangeNotify(int eventId, int flags, IntPtr item1, IntPtr item2);
}
'@
Add-Type -TypeDefinition $code -ErrorAction SilentlyContinue
[WinRefresher]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
Write-Host "[OK] Windows configurato su ns.exe." -ForegroundColor Green


# --- 2. SETUP VS CODE (Icona Lock/Sicurezza) ---
Write-Host "`n>>> CONFIGURAZIONE VS CODE <<<" -ForegroundColor Cyan

$VscodeDir = "$CurrentDir\.vscode"
$SettingsFile = "$VscodeDir\settings.json"

if (-not (Test-Path $VscodeDir)) { New-Item -ItemType Directory -Force -Path $VscodeDir | Out-Null }

# Usiamo l'icona 'lock' che è nativa in Material Theme ed è perfetta per un linguaggio "Vault/Safe"
$settingsContent = @{
    "files.associations" = @{
        "*.ns" = "nodestract"
    }
    "material-icon-theme.files.associations" = @{
        "*.ns" = "lock" 
    }
}

$settingsJson = $settingsContent | ConvertTo-Json -Depth 5
Set-Content -Path $SettingsFile -Value $settingsJson

Write-Host "[OK] VS Code configurato. I file .ns useranno l'icona 'Lock' (Sicurezza)." -ForegroundColor Green
Write-Host "Riavvia VS Code per vedere i cambiamenti."
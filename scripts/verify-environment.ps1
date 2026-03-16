$ErrorActionPreference = 'Stop'

Write-Host '==> Verificando entorno de compilación, precisión y empaquetado' -ForegroundColor Cyan

function Test-Tool([string]$Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

$items = @(
    @{ Name = 'cargo'; Required = $true;  Hint = 'Instala Rustup.' },
    @{ Name = 'rustup'; Required = $true; Hint = 'Instala Rustup.' },
    @{ Name = 'rustfmt'; Required = $false; Hint = 'Instala el componente rustfmt: rustup component add rustfmt.' },
    @{ Name = 'cargo-clippy'; Required = $false; Hint = 'Instala el componente clippy: rustup component add clippy.' },
    @{ Name = 'cl'; Required = $false; Hint = 'Instala Visual Studio Build Tools con Desktop development with C++.' },
    @{ Name = 'powershell'; Required = $true; Hint = 'PowerShell debe existir en Windows.' },
    @{ Name = 'wpr'; Required = $false; Hint = 'Instala Windows Performance Toolkit si usarás modo de precisión.' },
    @{ Name = 'wpa'; Required = $false; Hint = 'Instala Windows Performance Analyzer para abrir ETL.' },
    @{ Name = 'tracerpt'; Required = $false; Hint = 'Instala o habilita tracerpt si usarás resumen ETL local.' },
    @{ Name = 'iscc'; Required = $false; Hint = 'Instala Inno Setup si generarás instalador.' }
)

$failed = $false

foreach ($item in $items) {
    $exists = Test-Tool $item.Name
    if ($exists) {
        Write-Host ('[OK]   ' + $item.Name) -ForegroundColor Green
    }
    else {
        if ($item.Required) {
            Write-Host ('[FAIL] ' + $item.Name + ' -> ' + $item.Hint) -ForegroundColor Red
            $failed = $true
        }
        else {
            Write-Host ('[WARN] ' + $item.Name + ' -> ' + $item.Hint) -ForegroundColor Yellow
        }
    }
}

Write-Host ''
Write-Host '==> Versiones útiles' -ForegroundColor Cyan
if (Test-Tool 'cargo') { cargo --version }
if (Test-Tool 'rustup') { rustup --version }
if (Test-Tool 'rustfmt') { rustfmt --version }
if (Test-Tool 'cargo-clippy') { cargo clippy -V }
if (Test-Tool 'wpr') { wpr -status | Out-Host }
if (Test-Tool 'tracerpt') { tracerpt -? | Select-Object -First 5 | Out-Host }

if ($failed) {
    throw 'El entorno no cumple los requisitos mínimos para compilar.'
}

Write-Host 'Entorno base correcto.' -ForegroundColor Green

$ErrorActionPreference = 'Stop'

$packageName = 'rootcause-windows-inspector'
$installerType = 'exe'
$url64 = 'https://github.com/vladimiracunadev-create/rootcause-windows-inspector/releases/latest/download/RootCause-Setup.exe'
$checksum64 = 'UPDATE_SHA256_ON_RELEASE'
$checksumType = 'sha256'

$packageArgs = @{
    packageName    = $packageName
    fileType       = $installerType
    url64bit       = $url64
    checksum64     = $checksum64
    checksumType64 = $checksumType
    silentArgs     = '/SILENT /SUPPRESSMSGBOXES /NORESTART'
    validExitCodes = @(0)
}

Install-ChocolateyPackage @packageArgs

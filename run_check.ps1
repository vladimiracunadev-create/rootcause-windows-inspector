$env:PATH = "C:\Users\vbav\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin;C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.43.34808\bin\Hostx64\x64;" + $env:PATH
Set-Location "C:\dev\rootcause-windows-inspector"
$output = & "C:\Users\vbav\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe" check --message-format=short 2>&1
$output | Set-Content "C:\dev\rootcause-windows-inspector\check_out.txt"
Write-Host "Exit: $LASTEXITCODE"

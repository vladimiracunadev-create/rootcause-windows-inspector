$env:PATH = "C:\Users\vbav\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin;$env:PATH"
Set-Location "C:\dev\rootcause-windows-inspector"
$output = & "C:\Users\vbav\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe" fmt --all 2>&1
$output | Set-Content "C:\dev\rootcause-windows-inspector\fmt_out.txt"
Write-Host "Done"

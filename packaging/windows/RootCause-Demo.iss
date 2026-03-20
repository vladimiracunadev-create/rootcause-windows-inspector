; Instalador público y transparente para RootCause Demo.
; Requiere que target\release\rootcause.exe exista antes de compilar este script.

#define MyAppName "RootCause Demo"
#define MyAppVersion "0.8.0"
#define MyAppPublisher "Vladimir Acuña Dev"
#define MyAppExeName "rootcause.exe"
#define MyAppURL "https://github.com/vladimiracunadev-create/rootcause-windows-inspector"

[Setup]
AppId={{F0C728A0-86FB-4B6D-8D95-9D11D60BDA88}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\RootCause Demo
DefaultGroupName=RootCause Demo
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE
InfoBeforeFile=INFO_BEFORE_DEMO.txt
InfoAfterFile=INFO_AFTER_DEMO.txt
OutputDir=..\..\build\installer
OutputBaseFilename=RootCause-Demo-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
SetupIconFile=..\..\assets\rootcause.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
VersionInfoVersion={#MyAppVersion}
VersionInfoDescription=RootCause Demo Setup
VersionInfoCompany={#MyAppPublisher}
VersionInfoProductName={#MyAppName}
SetupLogging=yes

[Tasks]
Name: "desktopicon"; Description: "Crear acceso directo en el escritorio"; GroupDescription: "Accesos directos:"; Flags: unchecked
Name: "openreadme"; Description: "Abrir LEEME-DEMO.txt al finalizar"; GroupDescription: "Acciones al terminar:"; Flags: checkedonce
Name: "launchapp"; Description: "Ejecutar RootCause Demo al finalizar"; GroupDescription: "Acciones al terminar:"; Flags: unchecked

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\rootcause.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\SECURITY.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\distribution\demo\LEEME-DEMO.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\scripts\wpr-start-general.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion
Source: "..\..\scripts\wpr-stop-general.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion
Source: "..\..\scripts\wpa-open-latest.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion

[Icons]
Name: "{group}\RootCause Demo"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\rootcause.ico"
Name: "{autodesktop}\RootCause Demo"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon; IconFilename: "{app}\assets\rootcause.ico"
Name: "{group}\LEEME RootCause Demo"; Filename: "{app}\LEEME-DEMO.txt"

[Run]
Filename: "{app}\LEEME-DEMO.txt"; Description: "Abrir LEEME-DEMO.txt"; Flags: postinstall shellexec skipifsilent; Tasks: openreadme
Filename: "{app}\{#MyAppExeName}"; Description: "Ejecutar RootCause Demo"; Flags: nowait postinstall skipifsilent; Tasks: launchapp

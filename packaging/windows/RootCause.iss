; Instalador de referencia para RootCause.
; Este proyecto no distribuye binarios precompilados en el repositorio.
; Primero genera target\release\rootcause.exe y luego compila este script.

#define MyAppName "RootCause"
#define MyAppVersion "0.5.0"
#define MyAppPublisher "Vladimir Acuña Dev"
#define MyAppExeName "rootcause.exe"
#define MyAppURL "https://github.com/vladimiracunadev-create/rootcause-windows-inspector"

[Setup]
AppId={{9F8F0676-76EC-462F-89CB-B95C3B76D019}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\RootCause
DefaultGroupName=RootCause
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE
SetupIconFile=..\..\assets\rootcause.ico
OutputDir=..\..\build\installer
OutputBaseFilename=RootCause-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
UninstallDisplayIcon={app}\assets\rootcause.ico
VersionInfoVersion={#MyAppVersion}
VersionInfoDescription=RootCause Setup
VersionInfoCompany={#MyAppPublisher}
VersionInfoProductName={#MyAppName}
SetupLogging=yes

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\SECURITY.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\rootcause.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\..\assets\rootcause-icon-256.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\..\assets\rootcause-icon.svg"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\..\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\scripts\wpr-start-general.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion
Source: "..\..\scripts\wpr-stop-general.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion
Source: "..\..\scripts\wpa-open-latest.ps1"; DestDir: "{app}\scripts"; Flags: ignoreversion

[Icons]
Name: "{group}\RootCause"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\rootcause.ico"
Name: "{autodesktop}\RootCause"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\rootcause.ico"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Ejecutar RootCause"; Flags: nowait postinstall skipifsilent

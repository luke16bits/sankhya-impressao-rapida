; Script do Inno Setup para o Sankhya Impressão Rápida
[Setup]
AppId={{E68A5D6B-4C67-428E-8A51-2292E47BF4F1}
AppName=Sankhya Impressão Rápida
AppVersion=1.0.1
AppPublisher=Comunidade Sankhya Open Source
AppPublisherURL=https://github.com/luke16bits/sankhya-impressao-rapida
AppSupportURL=https://github.com/luke16bits/sankhya-impressao-rapida/issues
AppUpdatesURL=https://github.com/luke16bits/sankhya-impressao-rapida/releases
DefaultDirName={autopf}\Sankhya Impressao Rapida
DefaultGroupName=Sankhya Impressao Rapida
DisableProgramGroupPage=yes
OutputDir=.
OutputBaseFilename=Instalador-Sankhya-Impressao-Rapida
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Files]
Source: "target\release\sankhya-impressao-rapida.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "certs\cert.pem"; DestDir: "{app}"; Flags: ignoreversion


[Run]
; Instalar o certificado SSL local na pasta de Autoridades Raiz Confiáveis
Filename: "certutil.exe"; Parameters: "-addstore -f ""Root"" ""{app}\cert.pem"""; StatusMsg: "Configurando certificados de segurança locais..."; Flags: runhidden
; Registrar e iniciar o serviço do Windows
Filename: "{app}\sankhya-impressao-rapida.exe"; Parameters: "install"; StatusMsg: "Instalando e iniciando o serviço em segundo plano..."; Flags: runhidden

[UninstallRun]
; Parar e remover o serviço na desinstalação antes de remover os arquivos
Filename: "sc.exe"; Parameters: "stop SankhyaImpressaoRapida"; Flags: runhidden
Filename: "sc.exe"; Parameters: "delete SankhyaImpressaoRapida"; Flags: runhidden
Filename: "taskkill.exe"; Parameters: "/F /IM sankhya-impressao-rapida.exe"; Flags: runhidden

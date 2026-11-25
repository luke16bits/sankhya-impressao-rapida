; Script do Inno Setup para o Sankhya Impressão Rápida
[Setup]
AppId={{E68A5D6B-4C67-428E-8A51-2292E47BF4F1}
AppName=Sankhya Impressão Rápida
AppVersion=1.1.0
AppVerName=Sankhya Impressão Rápida v1.1.0
UninstallDisplayName=Sankhya Impressão Rápida v1.1.0
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
CloseApplications=force
RestartApplications=no

[InstallDelete]
Type: files; Name: "{app}\sankhya-impressao-rapida.exe"
Type: files; Name: "{app}\cert.pem"
Type: files; Name: "{app}\JasperToPdf.class"

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Files]
Source: "target\release\sankhya-impressao-rapida.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "certs\cert.pem"; DestDir: "{app}"; Flags: ignoreversion
Source: "helpers\JasperToPdf.class"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKLM; Subkey: "SOFTWARE\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "SankhyaImpressaoRapida"; ValueData: """{app}\sankhya-impressao-rapida.exe"""; Flags: uninsdeletevalue

[Code]
// Para o serviço anterior e finaliza o processo antes de extrair os novos arquivos
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
begin
  if FileExists(ExpandConstant('{app}\sankhya-impressao-rapida.exe')) then
  begin
    Log('Uma versão anterior foi detectada no sistema. Executando atualização automática limpa...');
  end;
  // Remove qualquer serviço legado do Windows das versões anteriores para rodar na sessão do usuário
  Exec('sc.exe', 'stop SankhyaImpressaoRapida', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec('sc.exe', 'delete SankhyaImpressaoRapida', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec('taskkill.exe', '/F /IM sankhya-impressao-rapida.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Sleep(500);
  Result := '';
end;

[Run]
; Instalar o certificado SSL local na pasta de Autoridades Raiz Confiáveis
Filename: "certutil.exe"; Parameters: "-addstore -f ""Root"" ""{app}\cert.pem"""; StatusMsg: "Configurando certificados de segurança locais..."; Flags: runhidden waituntilterminated
; Iniciar o aplicativo diretamente na sessão interativa do usuário
Filename: "{app}\sankhya-impressao-rapida.exe"; Description: "Iniciar Sankhya Impressão Rápida agora"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Remover serviço legado se ainda existir e fechar aplicativo
Filename: "sc.exe"; Parameters: "stop SankhyaImpressaoRapida"; Flags: runhidden waituntilterminated
Filename: "sc.exe"; Parameters: "delete SankhyaImpressaoRapida"; Flags: runhidden waituntilterminated
Filename: "taskkill.exe"; Parameters: "/F /IM sankhya-impressao-rapida.exe"; Flags: runhidden waituntilterminated

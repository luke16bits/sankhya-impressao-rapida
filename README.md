# ⚡ Sankhya Impressão Rápida

> **Substituto ultra-leve, de alta performance e código aberto para o Sankhya Web Connection.**

Com o fim do Navegador Sankhya e a migração oficial do **Sankhya W / Om** para navegadores modernos como o **Google Chrome**, o uso de impressões locais (térmicas, cupom fiscal, etiquetas Zebra e relatórios) tornou-se dependente do aplicativo local *Web Connection*.

No entanto, a implementação original baseada em Java consome **mais de 250 MB a 512 MB de memória RAM**, demora para inicializar e frequentemente causa lentidão ou instabilidade em máquinas mais antigas.

O **Sankhya Impressão Rápida** é uma reengenharia nativa escrita em **Rust**, desenvolvida para fornecer:
* 🚀 **Velocidade extrema:** Tempo de resposta em microssegundos.
* 💾 **Economia de memória:** Menos de **5 MB de RAM** (uma redução de mais de 98% de consumo).
* ⚙️ **Executável Único Portátil:** Sem necessidade de instalar Java Runtime (JRE).
* 🛡️ **Serviço Nativo do Windows:** Pode ser instalado como um serviço do Windows com inicialização automática silenciosa.
* 🔒 **Comunicação Segura:** Suporte HTTPS com os mesmos certificados e endpoints esperados pelo Sankhya W (`port 9096`).

---

## 🏗️ Como Funciona a Arquitetura

```
[ Navegador Google Chrome / Edge ]
        │  (Sankhya W / Om Web)
        │
        │  Chamadas HTTPS para https://localhost:9096
        ▼
[ Sankhya Impressão Rápida (Rust) ] (~4 MB / 5 MB RAM)
        │
        ├── /.info              -> Validação de status
        ├── /.pList             -> Lista de impressoras instaladas
        ├── /.pDefaultPrinter   -> Impressora padrão do Windows
        ├── /.systemInfo        -> Identificação da máquina
        └── /.localPrinting     -> Envio de jobs de impressão
        │
        ▼ (Win32 Spooler API - winspool.drv)
[ Impressora Térmica / Zebra / Laser / PDF ]
```

---

## 📦 Instalação e Uso

### Modo Direto (Portátil para Testes)
Basta dar dois cliques no executável `sankhya-impressao-rapida.exe` ou rodar no prompt:
```cmd
sankhya-impressao-rapida.exe
```

### Instalar como Serviço do Windows (Inicia sozinho no boot)
Abra o Prompt de Comando ou PowerShell como **Administrador**:
```cmd
sankhya-impressao-rapida.exe install
```
*O serviço será registrado no `services.msc` e configurado para iniciar automaticamente com o Windows.*

### Remover o Serviço
```cmd
sankhya-impressao-rapida.exe uninstall
```

---

## 🛠️ Como Compilar

### Opção 1: Compilação Automática no GitHub (Recomendada)
Este repositório já inclui o arquivo `.github/workflows/release.yml`. Ao fazer o push ou criar uma tag no GitHub, o **GitHub Actions** compila o executável automaticamente na nuvem e disponibiliza o `.exe` pronto para download na aba **Releases**.

### Opção 2: Compilar Localmente (Ambiente de Desenvolvimento)
Com a ferramenta Rust instalada:
```bash
cargo build --release
```
O binário final otimizado estará disponível em:
`target/release/sankhya-impressao-rapida.exe`

---

## 📄 Licença
Distribuído sob a licença MIT. Criado para ajudar a comunidade de clientes e parceiros do ecossistema Sankhya a terem uma experiência mais fluida, estável e moderna no Google Chrome.

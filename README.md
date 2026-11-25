# ⚡ Sankhya Impressão Rápida

> **Substituto ultra-leve, de alta performance e código aberto para o Sankhya Web Connection.**

---

### ⚠️ Aviso Legal e Isenção de Responsabilidade (Disclaimer)
> Este projeto é uma iniciativa independente e de código aberto da comunidade, **não sendo afiliado, endossado, mantido ou desenvolvido pela Sankhya Gestão de Negócios Ltda.**
> 
> O software original e oficial **Sankhya Web Connection** é de propriedade e autoria da Sankhya Gestão de Negócios Ltda., **funciona perfeitamente** e pode ser obtido oficialmente no portal [downloads.sankhya.com.br](https://downloads.sankhya.com.br).
> 
> A finalidade desta implementação alternativa em Rust é puramente técnica: fornecer uma opção nativa e ultra-leve voltada para **eliminar o consumo excessivo de hardware e memória RAM** em terminais e computadores com recursos limitados de processamento.

---

## 🎯 Por que este projeto existe?
Com o fim do Navegador Sankhya e a migração oficial do **Sankhya W / Om** para navegadores modernos como o **Google Chrome**, o uso de impressões locais (térmicas, cupom fiscal, etiquetas Zebra e relatórios) tornou-se dependente do aplicativo local *Web Connection*.

No entanto, a implementação baseada em Java consome frequentemente **mais de 250 MB a 512 MB de memória RAM**, demora para inicializar e pode gerar lentidão ou consumo desnecessário em máquinas mais modestas.

O **Sankhya Impressão Rápida** é uma reengenharia nativa escrita em **Rust**, desenvolvida para fornecer:
* 🚀 **Velocidade extrema:** Tempo de resposta em microssegundos.
* 💾 **Economia de memória:** Menos de **5 MB de RAM** (redução de mais de 98% do consumo).
* ⚙️ **Sem necessidade de Java:** Executável 100% nativo em código de máquina.
* 🐧 **Multiplataforma Nativo:** Suporte completo para **Windows** e **Linux** (CachyOS, Arch, Ubuntu, etc.).
* 🛡️ **Serviço Silencioso:** Inicia automaticamente com o sistema operacional sem precisar de janelas abertas ou ícones pesados na bandeja.
* 📄 **Impressão em PDF sem Travar:** Gravação automática instantânea na Área de Trabalho para "Microsoft Print to PDF" e DANFEs.
* 🔒 **Comunicação Segura:** Suporte completo HTTPS (`porta 9196`) e HTTP (`porta 9096`) com certificados e suporte ao Private Network Access (PNA) do Chrome.

---

## 🏗️ Como Funciona a Arquitetura

```
[ Navegador Google Chrome / Edge ]
        │  (Sankhya W / Om Web)
        │
        │  Chamadas HTTPS para https://localhost:9196 / http://localhost:9096
        ▼
[ Sankhya Impressão Rápida (Rust) ] (~3 MB a 5 MB RAM)
        │
        ├── /.info              -> Validação de status do Web Connection
        ├── /.pList             -> Lista de impressoras instaladas
        ├── /.pDefaultPrinter   -> Impressora padrão do sistema
        ├── /.systemInfo        -> Identificação da máquina e usuário
        └── /.localPrinting     -> Envio de jobs de impressão e DANFEs
        │
        ├── [Windows] Win32 Spooler API (winspool.drv) / Salvar PDF
        └── [Linux]   CUPS (lp) / Salvar PDF em ~/Downloads
```

---

## 📦 Instalação

### Opção 1: Instalador Automático Windows com 1 Clique (Recomendado) 🌟
1. Baixe o instalador oficial: **[Instalador-Sankhya-Impressao-Rapida.exe](https://github.com/luke16bits/sankhya-impressao-rapida/releases/latest)**
2. Dê dois cliques no instalador e siga o assistente ("Avançar" -> "Instalar").
3. **Pronto!** O serviço será automaticamente instalado, configurado para inicializar com o Windows e já começará a rodar em segundo plano.
4. Para desinstalar no futuro, basta ir em *Configurações do Windows > Aplicativos > Aplicativos Instalados* e clicar em **Desinstalar**.

---

### Opção 2: Linux (CachyOS, Arch, Ubuntu, Debian, etc.) 🐧
1. Baixe o arquivo **[sankhya-impressao-rapida-linux-x86_64.tar.gz](https://github.com/luke16bits/sankhya-impressao-rapida/releases/latest)**
2. Extraia o arquivo e execute pelo terminal:
   ```bash
   tar -xzvf sankhya-impressao-rapida-linux-x86_64.tar.gz
   ./sankhya-impressao-rapida
   ```
3. Os PDFs gerados são salvos automaticamente na pasta `~/Downloads`.

---

## 💡 Dúvidas Frequentes e Boas Práticas (FAQ)

### 1. O aviso "Web Connection não encontrado" apareceu no login do Sankhya. O que fazer?
Existem duas maneiras simples de lidar com isso:
- **Solução Rápida:** Basta clicar em **"Continuar mesmo assim"**. O sistema abrirá normalmente e todas as funcionalidades de telas e impressões funcionarão perfeitamente.
- **Solução Definitiva (Desativar o aviso no ERP):** Se você não deseja que o Sankhya fique checando a identificação de máquina em cada login:
  1. Acesse o menu sanduíche do Sankhya e vá em: **Configurações > Avançado > Parâmetros da Aplicação**.
  2. Pesquise pelo parâmetro: `USAAPPIDET` (ou `global.usa.app.identificacao`).
  3. Altere o valor para **NÃO** (`N`) e salve.
  4. Pronto! O Sankhya iniciará direto sem exibir a janela amarela de aviso.

### 2. Como permitir o acesso no Google Chrome (Private Network Access)?
Navegadores Chromium modernos bloqueiam requisições de sites externos para o seu computador (`localhost`) por padrão.
- Na barra de endereços do Chrome (ao lado do cadeado ou nome do site), clique no **ícone de configurações do site**.
- Procure por **"Acesso à rede local"** (Local network access) e marque **Permitir**.
- Essa configuração só precisa ser feita **uma única vez** por navegador.

### 3. Como funciona a impressão em PDF?
Quando você seleciona a impressora **"Microsoft Print to PDF"** ou imprime DANFEs/relatórios que geram PDF:
- O arquivo é gerado e gravado instantaneamente na pasta **Área de Trabalho Pública** (`C:\Users\Public\Desktop\Sankhya_DANFE_<data>.pdf`).
- O arquivo aparece imediatamente na sua Área de Trabalho com qualidade original, sem travar o serviço e sem depender de janelas gráficas de diálogo.

---

## 🛠️ Como Compilar

### Compilação Automática no GitHub (CI/CD)
O repositório já inclui o GitHub Actions configurado. Qualquer push na branch `main` compila o projeto nos ambientes Windows e Linux na nuvem e publica os executáveis e instaladores na aba **Releases**.

### Compilação Local
Com o Rust instalado:
```bash
cargo build --release
```

---

## 📄 Licença
Distribuído sob a licença MIT. Criado para ajudar a comunidade de clientes e parceiros do ecossistema Sankhya a terem uma experiência mais fluida, estável e moderna no Google Chrome.

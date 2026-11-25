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
* ⚙️ **Sem necessidade de Java:** Executável 100% nativo em código de máquina para Windows.
* 🛡️ **Serviço Nativo do Windows:** Inicia silenciosamente com o sistema operacional sem precisar de janelas abertas.
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
        ├── /.pDefaultPrinter   -> Impressora padrão do Windows
        ├── /.systemInfo        -> Identificação da máquina e usuário
        └── /.localPrinting     -> Envio de jobs de impressão
        │
        ▼ (Win32 Spooler API - winspool.drv)
[ Impressora Térmica / Zebra / Laser / PDF ]
```

---

## 📦 Instalação

### Opção 1: Instalador Automático com 1 Clique (Recomendado) 🌟
Não precisa abrir terminal nem rodar comandos:
1. Baixe o instalador oficial: **[Instalador-Sankhya-Impressao-Rapida.exe](https://github.com/luke16bits/sankhya-impressao-rapida/releases/latest)**
2. Dê dois cliques no instalador e siga o assistente ("Avançar" -> "Instalar").
3. **Pronto!** O serviço será automaticamente instalado, configurado para inicializar com o Windows e já começará a rodar em segundo plano.
4. Para desinstalar no futuro, basta ir em *Configurações do Windows > Aplicativos > Aplicativos Instalados* e clicar em **Desinstalar**.

---

### Opção 2: Executável Portátil (Linha de Comando)

Para quem prefere rodar manualmente ou gerenciar por linha de comando:

* **Modo Direto (Para Testes na tela):**
  Dê dois cliques em `sankhya-impressao-rapida.exe`. Uma janela se abrirá mostrando as conexões em tempo real.
* **Instalar como Serviço do Windows:**
  No Prompt de Comando (CMD) como **Administrador**:
  ```cmd
  sankhya-impressao-rapida.exe install
  ```
* **Remover o Serviço:**
  ```cmd
  sankhya-impressao-rapida.exe uninstall
  ```

---

## 🛠️ Como Compilar

### Compilação Automática no GitHub (CI/CD)
O repositório já inclui o GitHub Actions configurado. Qualquer push na branch `main` compila o projeto no ambiente Windows na nuvem e publica o executável e o instalador na aba **Releases**.

### Compilação Local
Com o Rust instalado:
```bash
cargo build --release
```

---

## 📄 Licença
Distribuído sob a licença MIT. Criado para ajudar a comunidade de clientes e parceiros do ecossistema Sankhya a terem uma experiência mais fluida, estável e moderna no Google Chrome.

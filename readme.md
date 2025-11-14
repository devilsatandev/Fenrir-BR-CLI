# Fenrir CLI 🐺

Fenrir é um assistente de linha de comando (CLI) experimental que traduz linguagem natural em comandos de terminal executáveis, com foco em segurança e tarefas de desenvolvimento.

Ele usa um modelo de linguagem (atualmente Gemini) como um "Oráculo" para interpretar a intenção do usuário e sugerir comandos. O usuário sempre tem a palavra final, devendo aprovar a execução de qualquer comando sugerido.

---

## Funcionalidades

- **Interpretação de Linguagem Natural:** Diga o que você quer fazer (ex: "liste os arquivos da pasta atual com detalhes") e o Fenrir traduz.
- **Foco em Segurança (WIP):** Integração planejada para facilitar o uso de ferramentas de segurança (ex: nmap, sqlmap) através de linguagem natural.
- **Timeout de Segurança:** Se o Oráculo demorar muito (default: 60s), a operação é cancelada.
- **Confirmação Obrigatória:** O Fenrir nunca executa um comando sugerido pela IA sem a sua aprovação explícita (s/n).
- **Log de Tarefas:** Todas as tarefas propostas pela IA são logadas em `fenrir_tasks.log` para auditoria e consulta posterior.

---

## Instalação e Configuração

Siga estes passos para clonar, construir e configurar o Fenrir no seu ambiente (macOS/Linux).

### 1. Pré-requisitos

- [Rust (cargo)](https://www.rust-lang.org/tools/install)
- O CLI Gemini (ou outro CLI de IA) instalado e configurado no seu `PATH`.
- Sua `GEMINI_API_KEY` (ou equivalente) configurada como variável de ambiente.

### 2. Clonar o Repositório

```sh
git clone <URL_DO_SEU_REPOSITORIO_AQUI>
cd fenrir
```

### 3. Construir o Projeto

Para uma build de desenvolvimento:

```sh
cargo build
```

Para uma build otimizada (Recomendado para uso final):

```sh
cargo build --release
```

### 4. Adicionar ao PATH (Opcional, mas recomendado)

Para poder chamar `fenrir` de qualquer lugar, em vez de `cargo run` ou de navegar até a pasta `target/release/`, adicione o binário compilado ao seu PATH.

O binário estará em `target/release/fenrir` (após a build --release). Mova o binário para um local comum (ex: `~/.local/bin`):

- Crie o diretório se ele não existir:

  ```sh
  mkdir -p ~/.local/bin
  ```

- Copie o binário compilado:

  ```sh
  cp target/release/fenrir ~/.local/bin/
  ```

Adicione `~/.local/bin` ao seu arquivo de perfil do shell (ex: `.zshrc`, `.bashrc`, `.bash_profile`):

- Exemplo para `.zshrc`:

  ```sh
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
  ```

- Recarregue seu shell:

  ```sh
  source ~/.zshrc
  ```

Verifique se funcionou:

```sh
fenrir
```

(Você deve ver a mensagem do modo interativo).

---

## Uso

### Modo Interativo

```sh
fenrir
> liste os arquivos da pasta atual
```

### Comando Direto

```sh
fenrir "escaneie as portas do localhost"
```

Para dúvidas ou consultoria, "10 dólar" e fodase.

Brincadeira. Contato: satandev@proton.me

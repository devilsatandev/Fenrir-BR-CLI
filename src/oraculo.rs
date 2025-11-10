// --- MÓDULO DO ORÁCULO ---
// Toda a lógica de falar com a IA fica aqui.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

// --- CONSTANTES (só do Oráculo) ---
const TIMEOUT_SEGUNDOS: Duration = Duration::from_secs(60);

// --- CONTRATO ---
// (Fica 'pub' pra 'main.rs' poder usar)
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FenrirTask {
    pub task_type: String,
    pub ia_explanation: String,
    pub command_to_run: Option<String>,
    pub target_path: Option<String>,
    pub application: Option<String>,
}

// --- FUNÇÃO PRINCIPAL (pública) ---
// (Fica 'pub' pra 'main.rs' poder usar)
pub async fn chamar_gemini_com_timeout(consulta: &str) -> Result<FenrirTask, String> {

    // O "meta_prompt" agora mora aqui.
    // E já ensina o Oráculo a usar as ferramentas do Kali!
    let meta_prompt = format!(
r#"
Você é um Oráculo para um CLI em Rust chamado Fenrir.
Sua ÚNICA função é traduzir a linguagem natural do usuário em uma FICHA DE TAREFA em formato Markdown.
NÃO responda com explicações. NÃO converse. APENAS A FICHA.
Use "N/A" para campos não aplicáveis.

O formato da Ficha é:
TAREFA: [execute_command | open_editor | unknown]
EXPLICACAO: [O que você entendeu que o usuário quer, em português.]
COMANDO: [O comando shell completo. (N/A se não for 'execute_command')]
ARQUIVO: [O arquivo ou pasta alvo. (N/A se não for 'open_editor')]
APP: [O aplicativo para abrir. (N/A se não for 'open_editor')]

--- Exemplos Padrão ---
Consulta: "liste os arquivos da pasta atual"
Ficha:
TAREFA: execute_command
EXPLICACAO: O usuário quer listar os arquivos na pasta atual.
COMANDO: ls -l
ARQUIVO: N/A
APP: N/A

Consulta: "abre o main.rs no rustrover"
Ficha:
TAREFA: open_editor
EXPLICACAO: O usuário quer abrir o arquivo 'main.rs' no 'rustrover'.
COMANDO: N/A
ARQUIVO: main.rs
APP: rustrover

--- Exemplos de Ferramentas (SecOps) ---
Consulta: "escaneie as portas do localhost"
Ficha:
TAREFA: execute_command
EXPLICACAO: O usuário quer rodar um scan de versão (sV) do Nmap no 'localhost'.
COMANDO: nmap -sV localhost
ARQUIVO: N/A
APP: N/A

Consulta: "inicie o console do metasploit"
Ficha:
TAREFA: execute_command
EXPLICACAO: O usuário quer iniciar o console do Metasploit.
COMANDO: msfconsole
ARQUIVO: N/A
APP: N/A

Consulta: "verifique a versão do sqlmap"
Ficha:
TAREFA: execute_command
EXPLICACAO: O usuário quer verificar a versão do 'sqlmap'.
COMANDO: sqlmap --version
ARQUIVO: N/A
APP: N/A

Consulta: "quantos pau tem uma canoa"
Ficha:
TAREFA: unknown
EXPLICACAO: O usuário fez uma pergunta aleatória que não é um comando.
COMANDO: N/A
ARQUIVO: N/A
APP: N/A

AGORA, A CONSULTA DO USUÁRIO É:
'{consulta}'

GERE APENAS A FICHA DE TAREFA.
"#,
        consulta = consulta
    );

    let cmd_future = Command::new("gemini")
        .arg(meta_prompt)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match tokio::time::timeout(TIMEOUT_SEGUNDOS, cmd_future).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let saida_str = String::from_utf8_lossy(&output.stdout).to_string();

                // O Parser "Caderninho de Fiado" (robusto pra porra)
                let mut task = FenrirTask::default();
                task.task_type = "unknown".to_string();

                for line in saida_str.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim();

                        match key {
                            "TAREFA" => task.task_type = value.to_string(),
                            "EXPLICACAO" => task.ia_explanation = value.to_string(),
                            "COMANDO" if value != "N/A" => task.command_to_run = Some(value.to_string()),
                            "ARQUIVO" if value != "N/A" => task.target_path = Some(value.to_string()),
                            "APP" if value != "N/A" => task.application = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }

                if task.ia_explanation.is_empty() {
                    Err(format!("Oráculo não devolveu uma Ficha Markdown válida. \nSaída crua: '{}'", saida_str))
                } else {
                    Ok(task) // SUCESSO!
                }

            } else {
                let erro_str = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!(
                    "O processo 'gemini' deu erro (stderr): {}",
                    erro_str
                ))
            }
        }
        Ok(Err(e)) => {
            Err(format!(
                "Falha ao executar o processo 'gemini'. Tá instalado? Tá no PATH? Erro: {}",
                e
            ))
        }
        Err(_) => {
            Err("Tente novamente, tempo esgotado.".to_string())
        }
    }
}
// --- MÓDULO EXECUTOR ---
// As "Mãos" do Fenrir.
// Toda a lógica que FAZ alguma coisa (I/O) fica aqui.

use crate::oraculo::FenrirTask; // Precisa saber o que é uma Task
use chrono::Local;
use serde_json;
use std::fs::OpenOptions;
use std::io::{self, Write};
use tokio::process::Command;
use tokio::task;

// --- CONSTANTES (só do Executor) ---
const LOG_FILE: &str = "fenrir_tasks.log";

// --- FUNÇÕES PÚBLICAS ---
// (Ficam 'pub' pra 'main.rs' poder usar)

// Salva a tarefa no 'fenrir_tasks.log'
pub fn log_task(task: &FenrirTask) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_FILE)?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_entry = format!(
        "\n--- [ {} ] ---\n{}\n",
        timestamp,
        serde_json::to_string_pretty(task).unwrap_or("Erro ao serializar tarefa".to_string())
    );

    file.write_all(log_entry.as_bytes())
}

// Pergunta 's' ou 'n' (O Freio de Mão)
pub async fn ask_for_confirmation(acao_proposta: &str) -> bool {
    print!("{}", acao_proposta);
    io::stdout().flush().unwrap();

    let result = task::spawn_blocking(|| {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or(0);
        input.trim().to_lowercase()
    })
        .await;

    match result {
        Ok(input) => input == "s" || input == "sim",
        Err(_) => false,
    }
}

// Executa um comando no shell
pub fn handle_execute_command(comando: &str) {
    println!("Rodando: '{}'...", comando);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(comando)
            .spawn()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(comando)
            .spawn()
    };

    match output {
        Ok(_) => println!("Comando enviado pro terminal."),
        Err(e) => eprintln!("Oxe! Deu erro ao TENTAR rodar o comando: {}", e),
    }
}

// Abre um arquivo no editor
pub fn handle_open_editor(app: &str, path: &str) {
    println!("Tentando abrir '{}' no '{}'...", path, app);

    let cmd_para_rodar = if cfg!(target_os = "macos") && app == "rustrover" {
        format!("open -a RustRover \"{}\"", path)
    } else if cfg!(target_os = "macos") {
        format!("open -a \"{}\" \"{}\"", app, path)
    } else {
        format!("{} \"{}\"", app, path)
    };

    println!("(Usando o comando: '{}')", cmd_para_rodar);
    handle_execute_command(&cmd_para_rodar);
}
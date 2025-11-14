// --- MÓDULO EXECUTOR ---
// As "Mãos" do Fenrir.
// Agora ele recebe 'task_args' (JSON) e se vira.

use crate::oraculo::FenrirTask; // Precisa saber o que é uma Task
use chrono::Local;
use serde_json::Value; // Importa o 'Value' (JSON genérico)
use std::fs::OpenOptions;
use std::io::{self, Write};
use tokio::process::Command;
use tokio::task;

// --- CONSTANTES (só do Executor) ---
const LOG_FILE: &str = "fenrir_tasks.log";

// --- FUNÇÕES PÚBLICAS ---

// Salva a tarefa no 'fenrir_tasks.log'
// (Não muda, logar JSON é bom)
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
// (Não muda)
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
// AGORA ELA RECEBE O JSON DE ARGS
pub fn handle_execute_command(args: Option<Value>) {
    // A gente vai no JSON, acha a chave "cmd", e pega o texto.
    let comando = match args.as_ref().and_then(|a| a.get("cmd")).and_then(|v| v.as_str()) {
        Some(cmd_str) => cmd_str,
        None => {
            eprintln!("Erro: Oráculo mandou 'command' mas não mandou o JSON de 'cmd'!");
            return;
        }
    };

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
// AGORA ELA RECEBE O JSON DE ARGS
pub fn handle_open_editor(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
        Some(map) => map,
        None => {
            eprintln!("Erro: Oráculo mandou 'open_editor' mas não mandou os ARGS!");
            return;
        }
    };

    // Pega "app" e "path" do JSON
    let app = args_map.get("app").and_then(|v| v.as_str());
    let path = args_map.get("path").and_then(|v| v.as_str());

    match (app, path) {
        (Some(app_str), Some(path_str)) => {
            println!("Tentando abrir '{}' no '{}'...", path_str, app_str);
            
            let cmd_para_rodar = if cfg!(target_os = "macos") && app_str == "rustrover" {
                format!("open -a RustRover \"{}\"", path_str)
            } else if cfg!(target_os = "macos") {
                format!("open -a \"{}\" \"{}\"", app_str, path_str)
            } else {
                format!("{} \"{}\"", app_str, path_str)
            };

            println!("(Usando o comando: '{}')", cmd_para_rodar);
            // Re-usa o 'handle_execute_command' (só que com JSON fake)
            let cmd_json = serde_json::json!({ "cmd": cmd_para_rodar });
            handle_execute_command(Some(cmd_json));
        }
        _ => {
            eprintln!("Erro: Oráculo mandou 'open_editor' mas faltou 'app' ou 'path' nos ARGS!");
        }
    }
}// --- MÓDULO EXECUTOR ---
// As "Mãos" do Fenrir.
// Agora ele recebe 'task_args' (JSON) e se vira.

use crate::oraculo::FenrirTask; // Precisa saber o que é uma Task
use chrono::Local;
use serde_json::Value; // Importa o 'Value' (JSON genérico)
use std::fs::OpenOptions;
use std::io::{self, Write};
use tokio::process::Command;
use tokio::task;

// --- CONSTANTES (só do Executor) ---
const LOG_FILE: &str = "fenrir_tasks.log";

// --- FUNÇÕES PÚBLICAS ---

// Salva a tarefa no 'fenrir_tasks.log'
// (Não muda, logar JSON é bom)
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
// (Não muda)
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
// AGORA ELA RECEBE O JSON DE ARGS
pub fn handle_execute_command(args: Option<Value>) {
    // A gente vai no JSON, acha a chave "cmd", e pega o texto.
    let comando = match args.as_ref().and_then(|a| a.get("cmd")).and_then(|v| v.as_str()) {
        Some(cmd_str) => cmd_str,
        None => {
            eprintln!("Erro: Oráculo mandou 'command' mas não mandou o JSON de 'cmd'!");
            return;
        }
    };

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
// AGORA ELA RECEBE O JSON DE ARGS
pub fn handle_open_editor(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
        Some(map) => map,
        None => {
            eprintln!("Erro: Oráculo mandou 'open_editor' mas não mandou os ARGS!");
            return;
        }
    };

    // Pega "app" e "path" do JSON
    let app = args_map.get("app").and_then(|v| v.as_str());
    let path = args_map.get("path").and_then(|v| v.as_str());

    match (app, path) {
        (Some(app_str), Some(path_str)) => {
            println!("Tentando abrir '{}' no '{}'...", path_str, app_str);
            
            let cmd_para_rodar = if cfg!(target_os = "macos") && app_str == "rustrover" {
                format!("open -a RustRover \"{}\"", path_str)
            } else if cfg!(target_os = "macos") {
                format!("open -a \"{}\" \"{}\"", app_str, path_str)
            } else {
                format!("{} \"{}\"", app_str, path_str)
            };

            println!("(Usando o comando: '{}')", cmd_para_rodar);
            // Re-usa o 'handle_execute_command' (só que com JSON fake)
            let cmd_json = serde_json::json!({ "cmd": cmd_para_rodar });
            handle_execute_command(Some(cmd_json));
        }
        _ => {
            eprintln!("Erro: Oráculo mandou 'open_editor' mas faltou 'app' ou 'path' nos ARGS!");
        }
    }
}
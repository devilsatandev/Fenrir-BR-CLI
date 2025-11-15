// --- MÓDULO NMAP ---

use serde_json::Value;
use std::fs;
use tokio::process::Command;

pub fn run(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
        Some(map) => map,
        None => {
            eprintln!("Erro: Oráculo mandou 'nmap' mas faltou os ARGS!");
            return;
        }
    };

    // 1. Pega o 'target' (obrigatório)
    let target = match args_map.get("target").and_then(|v| v.as_str()) {
        Some(t) => t,      
        None => {
            eprintln!("Erro: Oráculo mandou 'nmap' mas faltou o 'target'!");
            return;
        }
    };
    
    // Força output em XML pro relatório
    let output_dir = format!("fenrir_logs/{}", target);
    let output_xml = format!("{}/nmap_scan.xml", output_dir);
    
    // Cria o diretório de log pro alvo
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Aviso: Falha ao criar diretório de log '{}': {}", output_dir, e);
    }

    // 2. Pega as 'flags' (opcional)
    let flags = match args_map.get("flags").and_then(|v| v.as_array()) {
        Some(flags_arr) => flags_arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<&str>>(),
        None => vec![],
    };

    println!("Rodando Nmap (Hardcoded)...");
    let mut cmd = Command::new("nmap");
    
    // Adiciona as flags
    for flag in flags {
        cmd.arg(flag);
    }
    
    // Adiciona nossas flags hardcoded de output
    cmd.arg("-oX");
    cmd.arg(&output_xml);
    
    // Adiciona o target
    cmd.arg(target);

    let spawn_result = cmd.spawn();
    
    match spawn_result {
        Ok(_) => println!("Scan Nmap enviado pro terminal. (Saída em: {})", output_xml),
        Err(e) => eprintln!("Erro ao tentar rodar o 'nmap': {}", e),
    }
}
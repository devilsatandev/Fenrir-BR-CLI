// --- MÓDULO SQLMAP (O PADRÃO) ---
// A gente deixa o "esqueleto" pronto pro futuro.

use serde_json::Value;
use std::fs;
use tokio::process::Command;

pub fn run(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
        Some(map) => map,
        None => {
            eprintln!("Erro: SQLMap precisa de ARGS!");
            return;
        }
    };

    println!("Rodando SQLMap (Hardcoded)...");
    let mut cmd = Command::new("sqlmap");
    
    // Modo batch pra não ficar perguntando
    cmd.arg("--batch");
    
    // Pega o 'url' (quase obrigatório)
    let target_url = match args_map.get("url").and_then(|v| v.as_str()) {
        Some(url) => {
            cmd.arg("-u");
            cmd.arg(url);
            url
        }
        None => ""
    };
    
    // Se temos URL, define pasta de output
    if !target_url.is_empty() {
        let safe_target_name = target_url
            .replace("http://", "")
            .replace("https://", "")
            .replace("/", "_");
            
        let output_dir = format!("fenrir_logs/{}/sqlmap", safe_target_name);
        
        if let Err(e) = fs::create_dir_all(&output_dir) {
            eprintln!("Aviso: Falha ao criar diretório de log '{}': {}", output_dir, e);
        } else {
             cmd.arg("--output-dir");
             cmd.arg(&output_dir);
             println!("(Saída do Sqlmap será salva em: {})", output_dir);
        }
    }
    
    // Pega as 'flags' (ex: --version, --dbs, --tables, --dump)
    if let Some(flags) = args_map.get("flags").and_then(|v| v.as_array()) {
         for flag in flags {
            if let Some(flag_str) = flag.as_str() {
                cmd.arg(flag_str);
            }
        }
    }
    
    let spawn_result = cmd.spawn();
    
    match spawn_result {
        Ok(_) => println!("Comando SQLMap enviado pro terminal."),
        Err(e) => eprintln!("Erro ao tentar rodar SQLMap: {}", e),
    }
}
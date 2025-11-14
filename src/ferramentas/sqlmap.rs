// --- MÓDULO SQLMAP (O PADRÃO) ---
// A gente deixa o "esqueleto" pronto pro futuro.

use serde_json::Value;
use std::fs; // Pra gente poder criar a pasta de output
use tokio::process::Command;

// A IA vai chamar 'TAREFA: sqlmap'
// ... (código existente) ...
pub fn run(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
// ... (código existente) ...
            return;
        }
    };

    println!("Rodando SQLMap (Hardcoded)...");
// ... (código existente) ...
    let mut cmd = Command::new("sqlmap");
    
    // --- MUDANÇA "SÊNIOR" ---
    // A gente FORÇA o modo "batch" pra ele não ficar perguntando
    cmd.arg("--batch");
    
    // Pega o 'url' (quase obrigatório)
    let target_url = match args_map.get("url").and_then(|v| v.as_str()) {
        Some(url) => {
            cmd.arg("-u");
            cmd.arg(url);
            url // Salva o 'url' pra gente usar no nome da pasta
        }
        None => {
             // Se não tiver 'url', talvez tenha 'flags' (tipo --version)
             "" 
        }
    };
    
    // Se a gente tem um 'url', a gente define a pasta de output
    if !target_url.is_empty() {
        // Limpa o 'url' pra virar nome de pasta
        let safe_target_name = target_url
            .replace("http://", "")
            .replace("https://", "")
            .replace("/", "_");
            
        let output_dir = format!("fenrir_logs/{}/sqlmap", safe_target_name);
        
        // Cria o diretório
        if let Err(e) = fs::create_dir_all(&output_dir) {
            eprintln!("Aviso: Falha ao criar diretório de log '{}': {}", output_dir, e);
        } else {
             cmd.arg("--output-dir");
             cmd.arg(&output_dir);
             println!("(Saída do Sqlmap será salva em: {})", output_dir);
        }
    }
    // --- FIM DA MUDANÇA ---
    
    
    // Pega as 'flags' (ex: --version, --dbs, --tables, --dump)
// ... (código existente) ...
    if let Some(flags) = args_map.get("flags").and_then(|v| v.as_array()) {
         for flag in flags {
            if let Some(flag_str) = flag.as_str() {
// ... (código existente) ...
                cmd.arg(flag_str);
            }
        }
    }
    
    // ... (aqui a gente adicionaria mais lógicas 'hardcoded'
    // para --dbs, --tables, etc.) -> A IA já pode mandar em 'flags'!
    
    let spawn_result = cmd.spawn();
// ... (código existente) ...
    match spawn_result {
        Ok(_) => println!("Comando SQLMap enviado pro terminal."),
// ... (código existente) ...
    }
}
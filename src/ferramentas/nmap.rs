// --- MÓDULO NMAP (AGORA "HARDCODED") ---
// A IA só preenche, a gente FAZ.

use serde_json::Value;
use std::fs; // Pra gente poder criar a pasta de output
use tokio::process::Command;

// A função 'run' é o nosso "backend carai"
// ... (código existente) ...
pub fn run(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
// ... (código existente) ...
    };

    // 1. Pega o 'target' (obrigatório)
// ... (código existente) ...
        None => {
            eprintln!("Erro: Oráculo mandou 'nmap' mas faltou o 'target'!");
            return;
        }
    };
    
    // --- MUDANÇA "SÊNIOR" ---
    // A gente vai FORÇAR o output em XML pra usar no relatório.
    let output_dir = format!("fenrir_logs/{}", target);
    let output_xml = format!("{}/nmap_scan.xml", output_dir);
    
    // Cria o diretório de log pro alvo, se não existir
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Aviso: Falha ao criar diretório de log '{}': {}", output_dir, e);
        // Não retorna, tenta rodar mesmo assim
    }
    // --- FIM DA MUDANÇA ---


    // 2. Pega as 'flags' (opcional)
// ... (código existente) ...
            .iter()
            .map(|v| v.as_str().unwrap_or("")) // Converte cada flag
            .filter(|s| !s.is_empty()) // Remove flags vazias
            .collect::<Vec<&str>>(),
        None => vec![], // Sem flags
    };
// ... (código existente) ...
    // A IA não injeta nada aqui.
    println!("Rodando Nmap (Hardcoded)...");
    let mut cmd = Command::new("nmap"); // O COMANDO "HARDCODED"
    
    // Adiciona as flags (seguras)
    for flag in flags {
        cmd.arg(flag);
    }
    
    // --- MUDANÇA "SÊNIOR" ---
    // Adiciona nossas flags "hardcoded" de output
    cmd.arg("-oX"); // Output em XML
    cmd.arg(&output_xml); // O caminho do arquivo
    // --- FIM DA MUDANÇA ---
    
    // Adiciona o target (seguro)
// ... (código existente) ...
    cmd.arg(target);

    // 4. Roda
// ... (código existente) ...
    let spawn_result = cmd.spawn();
    
    match spawn_result {
        Ok(_) => println!("Scan Nmap enviado pro terminal. (Saída em: {})", output_xml),
        Err(e) => eprintln!("Oxe! Deu erro ao TENTAR rodar o 'nmap': {}", e),
    }
}
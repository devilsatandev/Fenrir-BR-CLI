// --- MÓDULO GOBUSTER (HARDCODED) ---
// Pra achar diretório que nem um "Semi Deus"

use serde_json::Value;
use std::fs;
use std::path::Path;
use tokio::process::Command;

// A IA vai chamar 'TAREFA: gobuster'
pub fn run(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
        Some(map) => map,
        None => {
            eprintln!("Erro: Oráculo mandou 'gobuster' mas não mandou os ARGS!");
            return;
        }
    };

    println!("Rodando Gobuster (Hardcoded)...");
    let mut cmd = Command::new("gobuster");

    // --- MUDANÇA "SÊNIOR": A gente SEMPRE usa o modo 'dir' por padrão
    cmd.arg("dir");

    // 1. Pega o 'url' (obrigatório)
    let target_url = match args_map.get("url").and_then(|v| v.as_str()) {
        Some(u) => {
            cmd.arg("-u");
            cmd.arg(u);
            u // Salva pra gente usar no log
        }
        None => {
            eprintln!("Erro: Oráculo mandou 'gobuster' mas faltou o 'url'!");
            return;
        }
    };

    // 2. Pega a 'wordlist' (opcional, com um DEFAULT "pique sênior")
    let wordlist = args_map
        .get("wordlist")
        .and_then(|v| v.as_str())
        .unwrap_or("/usr/share/wordlists/dirbuster/directory-list-2.3-medium.txt"); // DEFAULT

    // Checa se a wordlist existe ANTES de rodar
    if !Path::new(wordlist).exists() {
        eprintln!("Erro: Wordlist '{}' não encontrada, seu corno!", wordlist);
        eprintln!("A IA sugeriu essa, mas talvez você precise de outra?");
        eprintln!("(Ex: /usr/share/wordlists/rockyou.txt, /usr/share/seclists/...)");
        return;
    }
    cmd.arg("-w");
    cmd.arg(wordlist);

    // 3. Pega 'flags' adicionais (ex: -x .php,.txt)
    if let Some(flags) = args_map.get("flags").and_then(|v| v.as_array()) {
        for flag in flags {
            if let Some(flag_str) = flag.as_str() {
                cmd.arg(flag_str); // Ex: "-x .php"
            }
        }
    }

    // 4. MUDANÇA "SÊNIOR": Salvar o output
    // Limpa o 'url' pra virar nome de pasta
    let safe_target_name = target_url
        .replace("http://", "")
        .replace("https://", "")
        .replace("/", "_");
        
    let output_dir = format!("fenrir_logs/{}", safe_target_name);
    let output_file = format!("{}/gobuster_scan.log", output_dir);

    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Aviso: Falha ao criar diretório de log '{}': {}", output_dir, e);
    }
    cmd.arg("-o");
    cmd.arg(&output_file);


    // 5. Roda
    let spawn_result = cmd.spawn();

    match spawn_result {
        Ok(_) => println!("Scan Gobuster enviado pro terminal. (Saída em: {})", output_file),
        Err(e) => eprintln!("Oxe! Deu erro ao TENTAR rodar o 'gobuster': {}", e),
    }
}
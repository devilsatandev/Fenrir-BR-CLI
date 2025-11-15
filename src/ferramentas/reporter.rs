// Por enquanto, é um "esqueleto".

use serde_json::Value;
use std::fs;
use std::io::Write;
use chrono::Local; // Pra botar data no relatório

// A IA vai chamar 'TAREFA: generate_report'
pub fn run(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
        Some(map) => map,
        None => {
            eprintln!("Erro: Oráculo mandou 'generate_report' mas não mandou os ARGS!");
            return;
        }
    };

    // 1. Pega o 'target' (obrigatório)
    let target = match args_map.get("target").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            eprintln!("Erro: Oráculo mandou 'generate_report' mas faltou o 'target'!");
            return;
        }
    };
    
    println!("Gerando Relatório de Pentest (Pique Sênior) para: '{}'...", target);
    
    // --- Define os caminhos ---
    let report_dir = "fenrir_reports";
    // Limpa o 'target' pra virar nome de arquivo
    let safe_target_name = target
        .replace("http://", "")
        .replace("https://", "")
        .replace("/", "_");
        
    let report_file_path = format!("{}/{}_report.md", report_dir, safe_target_name);
    
    let nmap_xml_path = format!("fenrir_logs/{}/nmap_scan.xml", safe_target_name);
    let sqlmap_dir_path = format!("fenrir_logs/{}/sqlmap", safe_target_name);
    let gobuster_log_path = format!("fenrir_logs/{}/gobuster_scan.log", safe_target_name);


    // --- Cria o diretório de relatórios ---
    if let Err(e) = fs::create_dir_all(report_dir) {
         eprintln!("Erro ao criar diretório de relatórios: {}", e);
         return;
    }
    
    // --- O Esqueleto do Relatório ---
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut report_content = String::new();
    report_content.push_str(&format!("# Relatório de Auditoria (Fenrir) - Alvo: `{}`\n\n", target));
    report_content.push_str(&format!("*Gerado em: {}*\n\n", timestamp));
    
    // --- Seção Nmap ---
    report_content.push_str("## 1. Scan de Portas (Nmap)\n\n");
    match fs::read_to_string(&nmap_xml_path) {
        Ok(_xml_data) => { // _xml_data pra ele não encher o saco
            // AQUI, o "Semi Deus" de verdade usaria uma crate (tipo 'quick-xml' ou 'serde-xml')
            // pra *parsear* o XML e achar as portas abertas.
            //
            // O "Estagiário de 10 dólar" (KKKK) vai só dizer que o arquivo existe.
            report_content.push_str(&format!("* ✅ **Sucesso**: Arquivo de scan Nmap encontrado em `{}`.\n", nmap_xml_path));
            report_content.push_str("* **(TODO)**: Parsear o XML e listar portas/serviços abertos aqui.\n");
        }
        Err(_) => {
            report_content.push_str(&format!("* ❌ **Erro**: Nenhum arquivo de scan Nmap encontrado em `{}`.\n", nmap_xml_path));
        }
    }
    
    // --- Seção Gobuster ---
    report_content.push_str("\n## 2. Scan de Diretórios (Gobuster)\n\n");
    match fs::read_to_string(&gobuster_log_path) {
        Ok(log_data) => {
            if log_data.is_empty() {
                report_content.push_str(&format!("* ⚠️ **Aviso**: Arquivo de scan Gobuster encontrado (`{}`), mas está vazio. (Provavelmente não achou nada).\n", gobuster_log_path));
            } else {
                report_content.push_str(&format!("* ✅ **Sucesso**: Arquivo de scan Gobuster encontrado em `{}`.\n", gobuster_log_path));
                report_content.push_str("* **(TODO)**: Listar os diretórios e arquivos (HTTP 200, 301, 403) encontrados.\n");
            }
        }
        Err(_) => {
             report_content.push_str(&format!("* ❌ **Erro**: Nenhum arquivo de scan Gobuster encontrado em `{}`.\n", gobuster_log_path));
        }
    }

    // --- Seção Sqlmap ---
    report_content.push_str("\n## 3. Scan de Injeção (Sqlmap)\n\n");
    match fs::read_dir(&sqlmap_dir_path) {
        Ok(_entries) => { // _entries pra ele calar a boca
            // AQUI, o "Semi Deus" ia ler os logs e CSVs dentro da pasta.
            // O "Estagiário" vai só listar o que tem lá.
            report_content.push_str(&format!("* ✅ **Sucesso**: Pasta de output do Sqlmap encontrada em `{}`.\n", sqlmap_dir_path));
            report_content.push_str("* **(TODO)**: Listar bancos de dados e tabelas vulneráveis aqui.\n");
        }
        Err(_) => {
             report_content.push_str(&format!("* **Erro**: Nenhuma pasta de output do Sqlmap encontrada em `{}`.\n", sqlmap_dir_path));
        }
    }
    
    // --- Salva o Relatório ---
    match fs::File::create(&report_file_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(report_content.as_bytes()) {
                eprintln!("Erro ao salvar o relatório: {}", e);
            } else {
                println!("SUCESSO! Relatório salvo em: {}", report_file_path);
            }
        }
        Err(e) => {
            eprintln!("Erro ao criar o arquivo de relatório: {}", e);
        }
    }
}
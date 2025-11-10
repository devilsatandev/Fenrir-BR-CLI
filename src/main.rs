use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize}; // A gente ainda usa pro LOG, mas não pro parser
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command; // Usa o Command do Tokio
use tokio::task; // Pra rodar o input síncrono sem travar

// --- NOSSO CONTRATO INTERNO ---
// A gente ainda usa o struct, pq é organizado pra porra.
// Mas agora a gente PREENCHE ele na mão, lendo o MD.
#[derive(Serialize, Deserialize, Debug, Default)] // Adicionei Default
struct FenrirTask {
    task_type: String,
    ia_explanation: String,
    command_to_run: Option<String>,
    target_path: Option<String>,
    application: Option<String>,
}

// --- CONSTANTES ---
const TIMEOUT_SEGUNDOS: Duration = Duration::from_secs(60);
const LOG_FILE: &str = "fenrir_tasks.log";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let pb = ProgressBar::new_spinner(); // Spinner pra gente ver rodando

    if args.len() > 1 {
        // Modo "um comando e vaza"
        let consulta_completa = args[1..].join(" ");
        processar_solicitacao(&consulta_completa, &pb).await;
    } else {
        // Modo interativo
        println!("Ei, cara! Modo interativo do Fenrir.");
        println!("Manda a braba (ou 'sair' pra vazar).");
        interativo(&pb).await;
    }
}

async fn interativo(pb: &ProgressBar) {
    let stdin = io::stdin();
    let mut input_buffer = String::new();

    loop {
        input_buffer.clear();
        match stdin.read_line(&mut input_buffer) {
            Ok(0) => break, // Fim da entrada (Ctrl+D)
            Ok(_) => {
                let trimado = input_buffer.trim().to_lowercase();
                if trimado.is_empty() {
                    continue;
                }
                if trimado == "sair" || trimado == "exit" {
                    println!("Falou, parceiro! Até a próxima.");
                    break;
                }

                // Se não for "sair", é pro Oráculo!
                processar_solicitacao(&trimado, pb).await;
                println!("\nPróxima? (ou 'sair' pra vazar)");
            }
            Err(e) => {
                eprintln!("Oxe! Deu erro lendo sua entrada: {}", e);
                break;
            }
        }
    }
}

// --- O CÉREBRO DO FENRIR ---
async fn processar_solicitacao(consulta: &str, pb: &ProgressBar) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["VAI", "CORNO!", "PENSE", "DESGRAÇA!", "...", "VAI", "LOGO", "CARALHO!", "(ノ°Д°）ノ", "┻━┻", "...", "VAI", "CORNO!"])
            .template("{spinner:.bold.yellow} {msg}")
            .unwrap(),
    );
    pb.set_message("Chamando o Oráculo (Gemini)...");
    pb.enable_steady_tick(Duration::from_millis(150));

    match chamar_gemini_com_timeout(consulta).await {
        Ok(task) => {
            // Oráculo respondeu!
            pb.finish_with_message("! Oráculo respondeu!");

            // 1. Loga a tarefa ANTES de executar
            if let Err(e) = log_task(&task) {
                eprintln!("Xii, deu erro pra logar a tarefa: {}", e);
            }

            // 2. O FREIO DE MÃO (Pavê ou pa executá?)
            let acao_proposta = format!(
                "O Oráculo sugeriu: '{}' \nTipo: '{}' \nComando: '{}' \nArquivo: '{}'",
                task.ia_explanation,
                task.task_type,
                task.command_to_run.as_deref().unwrap_or("N/A"),
                task.target_path.as_deref().unwrap_or("N/A")
            );

            println!("\n--- PROPOSTA DO ORÁCULO ---");
            println!("{}", acao_proposta);
            println!("-----------------------------");

            let confirmacao = ask_for_confirmation("Executa essa porra? (s/n):").await;

            if confirmacao {
                println!("Ok, segurando o volante...");
                // 3. O Executor (As "Mãos" do Fenrir)
                match task.task_type.as_str() {
                    "execute_command" => {
                        if let Some(cmd) = task.command_to_run {
                            handle_execute_command(&cmd);
                        } else {
                            eprintln!("Erro: Oráculo mandou 'execute_command' mas não mandou o comando!");
                        }
                    }
                    "open_editor" => {
                        if let (Some(path), Some(app)) = (task.target_path, task.application) {
                            handle_open_editor(&app, &path);
                        } else {
                            eprintln!("Erro: Oráculo mandou 'open_editor' mas faltou o app ou o arquivo!");
                        }
                    }
                    "unknown" | _ => {
                        println!("O Oráculo não entendeu o que fazer. (Disse: '{}')", task.ia_explanation);
                    }
                }
            } else {
                println!("Ação cancelada. Sabonetou!");
            }
        }
        Err(e) => {
            // Deu ruim no Oráculo
            pb.finish_with_message("! DEU RUIM!");
            eprintln!("Ops! Deu ruim na comunicação com o Oráculo: {}", e);
        }
    }
}

// --- O ORÁCULO (GEMINI) ---
// Chama o 'gemini' e força ele a devolver nosso MARKDOWN.
async fn chamar_gemini_com_timeout(consulta: &str) -> Result<FenrirTask, String> {

    // --- O NOVO META-PROMPT (MARKDOWN, DESGRAÇA!) ---
    let meta_prompt = format!(
r#"
Você é um Oráculo para um CLI em Rust chamado Fenrir.
Sua ÚNICA função é traduzir a linguagem natural do usuário em uma FICHA DE TAREFA em formato Markdown.
NÃO responda com explicações. NÃO converse. APENAS A FICHA.
Use 'N/A' para campos não aplicáveis.

O formato da Ficha é:
TAREFA: [execute_command | open_editor | unknown]
EXPLICACAO: [O que você entendeu que o usuário quer, em português.]
COMANDO: [O comando shell completo. (N/A se não for 'execute_command')]
ARQUIVO: [O arquivo ou pasta alvo. (N/A se não for 'open_editor')]
APP: [O aplicativo para abrir. (N/A se não for 'open_editor')]

Exemplos:
Consulta: 'liste os arquivos da pasta atual'
Ficha:
TAREFA: execute_command
EXPLICACAO: O usuário quer listar os arquivos na pasta atual.
COMANDO: ls -l
ARQUIVO: N/A
APP: N/A'
'
Consulta: 'abre o main.rs no rustrover'
Ficha:
TAREFA: open_editor
EXPLICACAO: O usuário quer abrir o arquivo 'main.rs' no 'rustrover'.
COMANDO: N/A
ARQUIVO: main.rs
APP: rustrover

Consulta: 'apaga a porra toda'
Ficha:
TAREFA: execute_command
EXPLICACAO: O usuário quer deletar tudo recursivamente (CUIDADO!).
COMANDO: rm -rf /
ARQUIVO: N/A
APP: N/A

Consulta: 'quantos pau tem uma canoa'
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

    // 2. Define a 'future' (a "promessa" de rodar o comando)
    let cmd_future = Command::new("gemini") // O COMANDO QUE TU ME DISSE
        .arg(meta_prompt) // Passa o promptão como argumento
        .stdout(Stdio::piped()) // Pega a saida (stdout)
        .stderr(Stdio::piped()) // Pega o erro (stderr)
        .output(); // Roda e espera

    // 3. Roda com o timeout
    match tokio::time::timeout(TIMEOUT_SEGUNDOS, cmd_future).await {
        Ok(Ok(output)) => {
            // Gemini respondeu a tempo
            if output.status.success() {
                let saida_str = String::from_utf8_lossy(&output.stdout).to_string();

                // --- O NOVO PARSER (CADERNINHO DE FIADO) ---
                // Chega de 'serde_json'. Vamos ler linha por linha.

                let mut task = FenrirTask::default(); // Cria uma task vazia
                task.task_type = "unknown".to_string(); // Começa como 'unknown'

                for line in saida_str.lines() {
                    // Pega "CHAVE: VALOR" e quebra no *primeiro* ':'
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim();

                        match key {
                            "TAREFA" => task.task_type = value.to_string(),
                            "EXPLICACAO" => task.ia_explanation = value.to_string(),
                            "COMANDO" if value != "N/A" => task.command_to_run = Some(value.to_string()),
                            "ARQUIVO" if value != "N/A" => task.target_path = Some(value.to_string()),
                            "APP" if value != "N/A" => task.application = Some(value.to_string()),
                            _ => {} // Ignora linha lixo ou "N/A"
                        }
                    }
                }

                // Se depois de tudo, a explicação tá vazia, deu merda.
                if task.ia_explanation.is_empty() {
                    Err(format!("Oráculo não devolveu uma Ficha Markdown válida. \nSaída crua: '{}'", saida_str))
                } else {
                    Ok(task) // SUCESSO!
                }

            } else {
                // Gemini rodou, mas deu erro (ex: API key errada)
                let erro_str = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!(
                    "O processo 'gemini' deu erro (stderr): {}",
                    erro_str
                ))
            }
        }
        Ok(Err(e)) => {
            // Erro ao TENTAR rodar o 'gemini' (ex: não achou o comando)
            Err(format!(
                "Falha ao executar o processo 'gemini'. Tá instalado? Tá no PATH? Erro: {}",
                e
            ))
        }
        Err(_) => {
            // Timeout!
            Err("Tente novamente, tempo esgotado.".to_string())
        }
    }
}


// --- LOG E FREIO DE MÃO ---

// Salva a tarefa no 'fenrir_tasks.log'
// (Continua usando JSON pra logar, pq log JSON é bom pra 'brincar depois')
fn log_task(task: &FenrirTask) -> io::Result<()> {
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

// Pergunta 's' ou 'n'
async fn ask_for_confirmation(acao_proposta: &str) -> bool {
    print!("{}", acao_proposta); // Mostra a pergunta
    io::stdout().flush().unwrap(); // Força o 'print' a aparecer

    // Ler input do usuário é síncrono (trava a thread)
    // Então, a gente joga pra uma thread do 'tokio' não travar o runtime
    let result = task::spawn_blocking(|| {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or(0);
        input.trim().to_lowercase()
    })
        .await; // Espera a thread síncrona terminar

    match result {
        Ok(input) => input == "s" || input == "sim",
        Err(_) => false, // Se der erro na thread, cancela
    }
}


// --- AS "MÃOS" DO FENRIR ---
// Funções que REALMENTE fazem o trabalho sujo.

// Executa um comando no shell
fn handle_execute_command(comando: &str) {
    println!("Rodando: '{}'...", comando);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(comando)
            .spawn() // Usa spawn pra não travar o Fenrir
    } else {
        Command::new("sh") // Usa 'sh' (funciona em Mac e Linux)
            .arg("-c")
            .arg(comando)
            .spawn() // Usa spawn pra não travar o Fenrir
    };

    match output {
        Ok(_) => println!("Comando enviado pro terminal."),
        Err(e) => eprintln!("Oxe! Deu erro ao TENTAR rodar o comando: {}", e),
    }
}

// Abre um arquivo no editor
fn handle_open_editor(app: &str, path: &str) {
    println!("Tentando abrir '{}' no '{}'...", path, app);

    // No macOS, é melhor usar 'open -a'
    let cmd_para_rodar = if cfg!(target_os = "macos") && app == "rustrover" {
        // Comando específico pro RustRover no macOS
        format!("open -a RustRover \"{}\"", path)
    } else if cfg!(target_os = "macos") {
        // Comando genérico 'open' do macOS
        format!("open -a \"{}\" \"{}\"", app, path)
    } else {
        // Comando genérico pra Linux/Windows
        format!("{} \"{}\"", app, path)
    };

    // A gente chama o 'handle_execute_command' pra rodar o comando de abrir
    println!("(Usando o comando: '{}')", cmd_para_rodar);
    handle_execute_command(&cmd_para_rodar);
}
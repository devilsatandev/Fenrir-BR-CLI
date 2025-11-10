// --- ARQUIVOS DE MÓDULO ---
// A "cagada de junior" (tudo no main) ACABOU.
// Declaramos os módulos que o Rust vai procurar.
// (ex: 'mod oraculo' faz o Rust procurar 'src/oraculo.rs')
mod executor;
mod oraculo;
mod ferramentas;

// --- IMPORTS (use) ---
// Agora a gente chama as funções dos *nossos* módulos.


// use crate::executor::{ask_for_confirmation, handle_execute_command, handle_open_editor, log_task};
// use crate::oraculo::{chamar_gemini_com_timeout, FenrirTask};

use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::io::{self};
use std::time::Duration;

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
// O main.rs agora só "orquestra".
// Ele chama o Oráculo, depois chama o Executor.
async fn processar_solicitacao(consulta: &str, pb: &ProgressBar) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["VAI", "CORNO!", "PENSE", "DESGRAÇA!", "...", "VAI", "LOGO", "CARALHO!", "(ノ°Д°）ノ", "┻━┻", "...", "VAI", "CORNO!"])
            .template("{spinner:.bold.yellow} {msg}")
            .unwrap(),
    );
    pb.set_message("Chamando o Oráculo (Gemini)...");
    pb.enable_steady_tick(Duration::from_millis(150));

    // 1. CHAMA O ORÁCULO (que agora tá em 'src/oraculo.rs')
    match oraculo::chamar_gemini_com_timeout(consulta).await {
        Ok(task) => {
            // Oráculo respondeu!
            pb.finish_with_message("! Oráculo respondeu!");

            // 2. CHAMA O EXECUTOR (log_task)
            if let Err(e) = executor::log_task(&task) {
                eprintln!("Xii, deu erro pra logar a tarefa: {}", e);
            }

            // 3. CHAMA O EXECUTOR (Freio de Mão)
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

            let confirmacao = executor::ask_for_confirmation("Executa essa porra? (s/n):").await;

            if confirmacao {
                println!("Ok, segurando o volante...");

                // 4. CHAMA O EXECUTOR (As "Mãos")
                match task.task_type.as_str() {
                    "execute_command" => {
                        if let Some(cmd) = task.command_to_run {
                            executor::handle_execute_command(&cmd);
                        } else {
                            eprintln!("Erro: Oráculo mandou 'execute_command' mas não mandou o comando!");
                        }
                    }
                    "open_editor" => {
                        if let (Some(path), Some(app)) = (task.target_path, task.application) {
                            executor::handle_open_editor(&app, &path);
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
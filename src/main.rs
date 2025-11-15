// --- MODULE FILES ---
// We declare the modules that Rust will look for.
mod executor;
mod oraculo;
mod ferramentas;

// --- IMPORTS ---
use crate::executor::{fill_task_recursively, handle_execute_command_with_timeout, handle_open_editor, log_task};
use crate::ferramentas::gobuster as gobuster_tool;
use crate::oraculo::chamar_gemini_com_timeout;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::env;
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let pb = ProgressBar::new_spinner();

    if args.len() > 1 {
        // "One command and exit" mode
        let full_query = args[1..].join(" ");
        process_request(&full_query, &pb).await;
    } else {
        // Interactive mode
        println!("Welcome to Fenrir Interactive Mode.");
        println!("Enter your command or 'exit' to quit.");
        interactive_mode(&pb).await;
    }
}
// desgrassa da puta que me pariu nessa porra caralho buceta deppois eu termino 
async fn interactive_mode(pb: &ProgressBar) {
    let stdin = io::stdin();
    let mut input_buffer = String::new();

    loop {
        input_buffer.clear();
        match stdin.read_line(&mut input_buffer) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed_input = input_buffer.trim().to_lowercase();
                if trimmed_input.is_empty() {
                    continue;
                }
                if trimmed_input == "exit" {
                    println!("Exiting. Goodbye!");
                    break;
                }

                process_request(&trimmed_input, pb).await;
                println!("\nEnter your next command (or 'exit' to quit):");
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}
// --- MAIN ORACLE → EXECUTOR FLOW ---
async fn process_request(query: &str, pb: &ProgressBar) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["/", "-", "\\", "|", "+", "*"])
            .template("{spinner:.bold.blue} {msg}")
            .unwrap(),
    );
    pb.set_message("Calling the Oracle (Gemini)...");
    pb.enable_steady_tick(Duration::from_millis(120));

    // 1. CALL THE ORACLE
    match chamar_gemini_com_timeout(query).await {
        Ok(mut task) => {
            pb.finish_with_message("Oracle has responded!");

            // 2. LOG THE TASK
            if let Err(e) = log_task(&task) {
                eprintln!("Error logging the task: {}", e);
            }

            // 3. RECURSIVE TASK FILLING WITH CONFIRMATION LOOP
            // TENTE POR SUA CONTA E RISCO SE FUDER ESSA PORRA NA TEORIA FUNCIONAVA
            task = fill_task_recursively(task).await;

            // 4. CHECK IF TASK WAS CONFIRMED
            if !task.is_confirmed {
                println!("Task was not confirmed. Exiting.");
                return;
            }

            // 5. EXECUTE THE TASK WITH TIME-BASED SEGMENTATION
            println!("\n⏱️  Executing task with time segment: {:?}", task.time_segment);

            match task.task_type.as_str() {
                "gobuster" => {
                    // Only run gobuster when a URL/target is provided.
                    if let Some(url) = task.target_path {
                        let args = json!({ "url": url });
                        gobuster_tool::run(Some(args));
                    } else {
                        println!("No URL provided for gobuster; using normal search instead.");
                        println!("Falling back to standard handling for this request.");
                    }
                }
                
                "execute_command" => {
                    if let Some(cmd) = task.command_to_run {
                        let args = json!({ "cmd": cmd });
                        handle_execute_command_with_timeout(Some(args), task.time_segment).await;
                    } else {
                        eprintln!("Error: Oracle suggested 'execute_command' but provided no command!");
                    }
                }
                "open_editor" => {
                    if let (Some(path), Some(app)) = (task.target_path, task.application) {
                        let args = json!({ "app": app, "path": path });
                        handle_open_editor(Some(args));
                    } else {
                        eprintln!("Error: Oracle suggested 'open_editor' but is missing the application or file path!");
                    }
                }
                "unknown" | _ => {
                    println!("The Oracle could not determine a clear action. (Said: '{}')", task.ia_explanation);
                }
            }
        }
        Err(e) => {
            pb.finish_with_message("Request failed!");
            eprintln!("Error communicating with the Oracle: {}", e);
        }
    }
}
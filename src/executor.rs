// --- EXECUTOR MODULE ---
// The "Hands" of Fenrir.
// Enhanced with recursive task filling, parallel verification, and timeout handling.

use crate::oraculo::{FenrirTask, ExecutionTimeSegment};
use chrono::Local;
use serde_json::{json, Value};
use std::fs::{OpenOptions, File};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::task;

// --- CONSTANTS (Executor only) ---
const LOG_FILE: &str = "fenrir_tasks.log";

// --- RECURSIVE TASK FILLING WITH CONFIRMATION LOOP ---
pub async fn fill_task_recursively(task: FenrirTask) -> FenrirTask {
    let mut current_task = task;
    let mut iteration = 0;
    const MAX_ITERATIONS: u32 = 5;

    loop {
        iteration += 1;
        if iteration > MAX_ITERATIONS {
            eprintln!("⚠️  Max iterations reached for task filling");
            break;
        }

        println!("\n📋 Task Filling Iteration {}", iteration);
        println!("Current State: {:?}", current_task.task_type);

        // Try to fill missing fields
        if current_task.command_to_run.is_none() && current_task.target_path.is_none() {
            println!("❌ Task is incomplete. Attempting to enhance...");
            
            // Launch parallel verification tasks using tokio::spawn
            let verify_tasks = vec![
                tokio::spawn(verify_task_command(current_task.clone())),
                tokio::spawn(verify_task_paths(current_task.clone())),
            ];

            for verify_task in verify_tasks {
                match verify_task.await {
                    Ok(Some(enhanced_task)) => {
                        current_task = enhanced_task;
                        println!("✅ Task enhanced from parallel verification");
                        break;
                    }
                    _ => continue,
                }
            }
        }

        // Display current task and ask for confirmation
        let proposed_action = format!(
            "Type: {}\nExplanation: {}\nCommand: {}\nTarget: {}",
            current_task.task_type,
            current_task.ia_explanation,
            current_task.command_to_run.as_deref().unwrap_or("N/A"),
            current_task.target_path.as_deref().unwrap_or("N/A")
        );

        println!("\n📝 Proposed Task:\n{}", proposed_action);
        println!("─────────────────────────");

        print!("\n✓ Confirm this task? (y/n/edit): ");
        io::stdout().flush().unwrap();

        let response = task::spawn_blocking(|| {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap_or(0);
            input.trim().to_lowercase()
        })
        .await
        .unwrap_or_default();

        match response.as_str() {
            "y" | "yes" => {
                current_task.is_confirmed = true;
                println!("✅ Task confirmed!");
                break;
            }
            "n" | "no" => {
                println!("❌ Task rejected. Please try another request.");
                break;
            }
            "edit" => {
                println!("✏️  Entering edit mode...");
                current_task = edit_task_interactive(current_task).await;
            }
            _ => {
                println!("Invalid input. Please enter 'y', 'n', or 'edit'.");
            }
        }
    }

    current_task
}

// --- PARALLEL VERIFICATION: Check if command needs completion ---
async fn verify_task_command(task: FenrirTask) -> Option<FenrirTask> {
    if task.task_type != "execute_command" || task.command_to_run.is_some() {
        return None;
    }

    println!("🔍 Verifying command field...");
    
    // Simulate async verification
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut enhanced = task;
    if enhanced.command_to_run.is_none() {
        enhanced.command_to_run = Some("echo 'Command to be determined'".to_string());
    }

    Some(enhanced)
}

// --- PARALLEL VERIFICATION: Check if paths need completion ---
async fn verify_task_paths(task: FenrirTask) -> Option<FenrirTask> {
    if task.task_type != "open_editor" || task.target_path.is_some() {
        return None;
    }

    println!("🔍 Verifying file path field...");
    
    // Simulate async verification
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut enhanced = task;
    if enhanced.target_path.is_none() {
        enhanced.target_path = Some("./file.txt".to_string());
    }

    Some(enhanced)
}

// --- INTERACTIVE TASK EDITING ---
async fn edit_task_interactive(mut task: FenrirTask) -> FenrirTask {
    println!("\n🛠️  Interactive Edit Mode");
    println!("1. Edit Task Type");
    println!("2. Edit Explanation");
    println!("3. Edit Command");
    println!("4. Edit Target Path");
    println!("5. Edit Tags");
    println!("6. Done editing");
    
    print!("Choose option: ");
    io::stdout().flush().unwrap();

    let response = task::spawn_blocking(|| {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or(0);
        input.trim().to_string()
    })
    .await
    .unwrap_or_default();

    match response.as_str() {
        "1" => {
            print!("New task type: ");
            io::stdout().flush().unwrap();
            let new_type = task::spawn_blocking(|| {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or(0);
                input.trim().to_string()
            })
            .await
            .unwrap_or_default();
            task.task_type = new_type;
        }
        "2" => {
            print!("New explanation: ");
            io::stdout().flush().unwrap();
            let new_explanation = task::spawn_blocking(|| {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or(0);
                input.trim().to_string()
            })
            .await
            .unwrap_or_default();
            task.ia_explanation = new_explanation;
        }
        "3" => {
            print!("New command: ");
            io::stdout().flush().unwrap();
            let new_command = task::spawn_blocking(|| {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or(0);
                input.trim().to_string()
            })
            .await
            .unwrap_or_default();
            task.command_to_run = Some(new_command);
        }
        "4" => {
            print!("New target path: ");
            io::stdout().flush().unwrap();
            let new_path = task::spawn_blocking(|| {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or(0);
                input.trim().to_string()
            })
            .await
            .unwrap_or_default();
            task.target_path = Some(new_path);
        }
        "5" => {
            print!("New tags (comma separated): ");
            io::stdout().flush().unwrap();
            let tags_input = task::spawn_blocking(|| {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap_or(0);
                input.trim().to_string()
            })
            .await
            .unwrap_or_default();

            let tags_vec: Vec<String> = tags_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if tags_vec.is_empty() {
                task.tags = None;
            } else {
                task.tags = Some(tags_vec);
            }
        }
        _ => println!("Done editing."),
    }

    task
}

// --- PUBLIC FUNCTIONS ---

// Saves the task to 'fenrir_tasks.log' with time segment information
pub fn log_task(task: &FenrirTask) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_FILE)?;

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let time_segment_str = task.time_segment
        .map(|ts| format!("{:?}", ts))
        .unwrap_or_else(|| "Unclassified".to_string());
    
    let log_entry = format!(
        "\n--- [ {} ] (Segment: {}, Retries: {}) ---\n{}\n",
        timestamp,
        time_segment_str,
        task.retry_count,
        serde_json::to_string_pretty(task).unwrap_or_else(|_| "Error serializing task".to_string())
    );

    file.write_all(log_entry.as_bytes())
}
// Note: confirmation helper removed (was unused).

// Executes a command in the shell with timeout based on time segment
pub async fn handle_execute_command_with_timeout(args: Option<Value>, time_segment: Option<ExecutionTimeSegment>) {
    let command = match args.as_ref().and_then(|a| a.get("cmd")).and_then(|v| v.as_str()) {
        Some(cmd_str) => cmd_str.to_string(),
        None => {
            eprintln!("Error: Oracle suggested 'command' but did not provide the 'cmd' JSON!");
            return;
        }
    };

    let timeout = time_segment.map(|ts| ts.max_timeout()).unwrap_or(Duration::from_secs(60));
    
    println!("⏱️  Running: '{}' with timeout: {:?}...", command, timeout);

    let cmd_future = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(&command)
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
    };

    match tokio::time::timeout(timeout, cmd_future).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                println!("✅ Command completed successfully.");
            } else {
                eprintln!("⚠️  Command exited with error code");
            }
        }
        Ok(Err(e)) => eprintln!("❌ Error executing command: {}", e),
        Err(_) => eprintln!("⏰ Command timed out after {:?}", timeout),
    }
}

// Legacy function for backward compatibility
pub fn handle_execute_command(args: Option<Value>) {
    let command = match args.as_ref().and_then(|a| a.get("cmd")).and_then(|v| v.as_str()) {
        Some(cmd_str) => cmd_str,
        None => {
            eprintln!("Error: Oracle suggested 'command' but did not provide the 'cmd' JSON!");
            return;
        }
    };

    println!("Running: '{}'...", command);
    let spawn_result = if cfg!(target_os = "windows") {
        Command::new("cmd").arg("/C").arg(command).spawn()
    } else {
        Command::new("sh").arg("-c").arg(command).spawn()
    };

    match spawn_result {
        Ok(_) => println!("Command sent to the terminal."),
        Err(e) => eprintln!("Error trying to run the command: {}", e),
    }
}

// Opens a file in an editor
pub fn handle_open_editor(args: Option<Value>) {
    let args_map = match args.as_ref().and_then(|a| a.as_object()) {
        Some(map) => map,
        None => {
            eprintln!("Error: Oracle suggested 'open_editor' but did not provide ARGS!");
            return;
        }
    };

    // Extract path (required)
    let path = match args_map.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            eprintln!("Error: 'path' is required to open an editor.");
            return;
        }
    };

    // If file does not exist, ask to create
    if !Path::new(&path).exists() {
        print!("File '{}' does not exist. Create it? (y/n): ", path);
        io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let resp = input.trim().to_lowercase();
        if resp == "y" || resp == "yes" {
            match File::create(&path) {
                Ok(_) => println!("Created file: {}", path),
                Err(e) => {
                    eprintln!("Failed to create file '{}': {}", path, e);
                    return;
                }
            }
        } else {
            println!("Aborting open_editor: file not created.");
            return;
        }
    }

    // App may be missing: suggest common editors
    let mut app_str = args_map.get("app").and_then(|v| v.as_str()).map(|s| s.to_string());
    if app_str.is_none() {
        println!("No application specified to open '{}'. Choose an option:", path);
        println!("1) Visual Studio Code");
        println!("2) TextEdit (macOS)");
        println!("3) RustRover");
        println!("4) Cancel");
        print!("Choice: ");
        io::stdout().flush().ok();
        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).ok();
        match choice.trim() {
            "1" => app_str = Some("Visual Studio Code".to_string()),
            "2" => app_str = Some("TextEdit".to_string()),
            "3" => app_str = Some("RustRover".to_string()),
            _ => {
                println!("Cancelled.");
                return;
            }
        }
    }

    let app_str = app_str.unwrap();
    println!("Opening '{}' with '{}'...", path, app_str);

    let command_to_run = if cfg!(target_os = "macos") {
        format!("open -a \"{}\" \"{}\"", app_str, path)
    } else {
        format!("{} \"{}\"", app_str, path)
    };

    println!("(Using command: '{}')", command_to_run);
    let cmd_json = json!({ "cmd": command_to_run });
    handle_execute_command(Some(cmd_json));
}
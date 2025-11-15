// --- ORACLE MODULE ---
// All logic for communicating with the AI is here.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

// --- CONSTANTS (Oracle only) ---
const TIMEOUT_SECONDS: Duration = Duration::from_secs(60);

// --- ENUMS FOR TASK EXECUTION TIME SEGMENTATION ---
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTimeSegment {
    Quick,   // < 5 seconds
    Medium,  // 5-60 seconds
    Long,    // > 60 seconds
}

impl ExecutionTimeSegment {
    pub fn from_command(command: &str) -> Self {
        let lower = command.to_lowercase();
        
        // Quick commands
        if lower.starts_with("ls ") || lower.starts_with("pwd") || 
           lower.starts_with("echo ") || lower.starts_with("cat ") ||
           lower.contains("--version") || lower.contains("-h") {
            return ExecutionTimeSegment::Quick;
        }
        
        // Long running commands
        if lower.contains("nmap") || lower.contains("sqlmap") || 
           lower.contains("gobuster") || lower.contains("nikto") ||
           lower.contains("scan") || lower.contains("fuzz") {
            return ExecutionTimeSegment::Long;
        }
        
        // Default to medium
        ExecutionTimeSegment::Medium
    }
    
    pub fn max_timeout(&self) -> Duration {
        match self {
            ExecutionTimeSegment::Quick => Duration::from_secs(10),
            ExecutionTimeSegment::Medium => Duration::from_secs(60),
            ExecutionTimeSegment::Long => Duration::from_secs(300),
        }
    }
}

// --- CONTRACT ---
// (Public for 'main.rs' to use)
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FenrirTask {
    #[serde(skip)]
    pub task_type: String,
    pub ia_explanation: String,
    pub command_to_run: Option<String>,
    pub target_path: Option<String>,
    pub application: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(skip)]
    pub time_segment: Option<ExecutionTimeSegment>,
    #[serde(skip)]
    pub retry_count: u32,
    #[serde(skip)]
    pub is_confirmed: bool,
}

impl FenrirTask {
    pub fn categorize_by_time(&mut self) {
        if let Some(ref cmd) = self.command_to_run {
            self.time_segment = Some(ExecutionTimeSegment::from_command(cmd));
        }
    }
}

// --- MAIN PUBLIC FUNCTION WITH TIMEOUT HANDLING AND AI FALLBACK ---
pub async fn chamar_gemini_com_timeout(query: &str) -> Result<FenrirTask, String> {
    chamar_gemini_com_timeout_recursive(query, 0).await
}

async fn chamar_gemini_com_timeout_recursive(
    query: &str,
    retry_count: u32,
) -> Result<FenrirTask, String> {
    let meta_prompt = format!(
        r#"
You are an Oracle for a Rust CLI named Fenrir.
Your ONLY function is to translate the user's natural language into a TASK CARD in Markdown format.
DO NOT respond with explanations. DO NOT chat. ONLY THE TASK CARD.
Use "N/A" for non-applicable fields.

The Task Card format is:
TASK_TYPE: [execute_command | open_editor | unknown]
EXPLANATION: [What you understood the user wants, in English.]
COMMAND: [The full shell command. (N/A if not 'execute_command')]
FILE: [The target file or folder. (N/A if not 'open_editor')]
APP: [The application to open with. (N/A if not 'open_editor')]

--- Standard Examples ---
Query: "list the files in the current folder"
Task Card:
TASK_TYPE: execute_command
EXPLANATION: The user wants to list the files in the current directory.
COMMAND: ls -l
FILE: N/A
APP: N/A

Query: "open main.rs in rustrover"
Task Card:
TASK_TYPE: open_editor
EXPLANATION: The user wants to open the file 'main.rs' in 'rustrover'.
COMMAND: N/A
FILE: main.rs
APP: rustrover

--- Tool Examples (SecOps) ---
Query: "scan the ports of localhost"
Task Card:
TASK_TYPE: execute_command
EXPLANATION: The user wants to run a version scan (sV) with Nmap on 'localhost'.
COMMAND: nmap -sV localhost
FILE: N/A
APP: N/A

Query: "start the metasploit console"
Task Card:
TASK_TYPE: execute_command
EXPLANATION: The user wants to start the Metasploit console.
COMMAND: msfconsole
FILE: N/A
APP: N/A

NOW, THE USER'S QUERY IS:
'{query}'

GENERATE ONLY THE TASK CARD.
"#,
        query = query
    );

    let cmd_future = Command::new("gemini")
        .arg(meta_prompt)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match tokio::time::timeout(TIMEOUT_SECONDS, cmd_future).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout).to_string();
                let mut task = parse_task_card(&output_str)?;
                task.categorize_by_time();
                task.retry_count = retry_count;
                Ok(task)
            } else {
                let error_str = String::from_utf8_lossy(&output.stderr).to_string();
                handle_gemini_timeout_error(query, retry_count, &error_str).await
            }
        }
        Ok(Err(e)) => Err(format!(
            "Failed to execute the 'gemini' process. Is it installed and in the PATH? Error: {}",
            e
        )),
        Err(_) => handle_gemini_timeout_error(query, retry_count, "timeout").await,
    }
}

// --- TIMEOUT ERROR HANDLER WITH AI FALLBACK STRATEGIES (Non-recursive) ---
async fn handle_gemini_timeout_error(
    query: &str,
    retry_count: u32,
    error_type: &str,
) -> Result<FenrirTask, String> {
    eprintln!("\n⚠️  Oracle timeout detected (attempt {})", retry_count + 1);
    
    // Retry strategy (max 2 retries before fallback)
    if retry_count < 2 {
        eprintln!("🔄 Retrying with simplified query...");
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Use Box::pin for async recursion
        return Box::pin(chamar_gemini_com_timeout_recursive(query, retry_count + 1)).await;
    }

    // If retries exhausted, use fallback strategies
    eprintln!("⚡ Activating fallback strategy...");
    
    match analyze_query_fallback(query) {
        Some(mut fallback_task) => {
            fallback_task.categorize_by_time();
            fallback_task.retry_count = retry_count;
            eprintln!("✅ Fallback strategy generated task: {}", fallback_task.task_type);
            Ok(fallback_task)
        }
        None => Err(format!(
            "Request timed out after {} retries and fallback strategies could not resolve the query: {}",
            retry_count, error_type
        )),
    }
}

// --- FALLBACK STRATEGY FOR COMMON COMMANDS ---
fn analyze_query_fallback(query: &str) -> Option<FenrirTask> {
    let lower = query.to_lowercase();

    // Switch case para tarefas complexas
    let (task_type, explanation, command) = match true {
        _ if lower.contains("list") || lower.contains("ls") => {
            ("execute_command", "List files in the directory", Some("ls -lah"))
        }
        _ if lower.contains("scan") || lower.contains("nmap") => {
            ("execute_command", "Run network scan", Some("nmap -sV localhost"))
        }
        _ if lower.contains("sqlmap") || lower.contains("sql injection") => {
            ("execute_command", "Run SQLMap", Some("sqlmap --wizard"))
        }
        _ if lower.contains("gobuster") || lower.contains("brute force") => {
            ("execute_command", "Run directory brute force", Some("gobuster dir -u http://localhost"))
        }
        _ if lower.contains("ache") || lower.contains("achar") || lower.contains("encontre") || lower.contains("encontrar") || lower.contains("find") || lower.contains("locate") || lower.contains("discover") => {
            ("gobuster", "Use gobuster tool to discover directories on a target", None)
        }
        _ if (lower.contains("open") || lower.contains("edit")) && lower.contains(".rs") => {
            ("open_editor", "Open Rust file in editor", None)
        }
        _ if lower.contains("compile") || lower.contains("build") => {
            ("execute_command", "Compile Rust project", Some("cargo build"))
        }
        _ if lower.contains("run") && lower.contains("cargo") => {
            ("execute_command", "Run Rust binary", Some("cargo run"))
        }
        _ if lower.contains("help") || lower.contains("--help") => {
            ("execute_command", "Show help information", Some("--help"))
        }
        _ => ("unknown", "Could not determine task type", None),
    };

    if task_type == "unknown" {
        return None;
    }

    Some(FenrirTask {
        task_type: task_type.to_string(),
        ia_explanation: format!("[FALLBACK] {}", explanation),
        command_to_run: command.map(|s| s.to_string()),
        target_path: None,
        application: None,
        tags: None,
        time_segment: None,
        retry_count: 0,
        is_confirmed: false,
    })
}

// A more robust parser for the Task Card format
fn parse_task_card(output: &str) -> Result<FenrirTask, String> {
    let mut task = FenrirTask::default();
    task.task_type = "unknown".to_string();
    let mut fields_found = 0;

    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            if value == "N/A" {
                continue;
            }

            match key {
                "TASK_TYPE" => {
                    task.task_type = value.to_string();
                    fields_found += 1;
                }
                "EXPLANATION" => {
                    task.ia_explanation = value.to_string();
                    fields_found += 1;
                }
                "COMMAND" => {
                    task.command_to_run = Some(value.to_string());
                }
                "FILE" => {
                    task.target_path = Some(value.to_string());
                }
                "APP" => {
                    task.application = Some(value.to_string());
                }
                _ => {} // Ignore unknown keys
            }
        }
    }

    if fields_found < 2 {
        // Basic validation: we need at least TASK_TYPE and EXPLANATION
        Err(format!(
            "Oracle did not return a valid Task Card. Raw output: '{}'",
            output
        ))
    } else {
        Ok(task) // SUCCESS!
    }
}
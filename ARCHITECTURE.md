# 🏗️ Arquitetura - Fenrir CLI (v0.2.0)

## Visão Geral do Sistema

```
┌─────────────────────────────────────────────────────────────┐
│                     USER INPUT                              │
└────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │     ORACLE MODULE (oraculo.rs)  │
        │  - Gemini API Integration       │
        │  - Retry Strategy (2x)          │
        │  - Fallback Analysis            │
        │  - Time Segmentation            │
        └────────────────┬────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │   EXECUTOR MODULE (executor.rs) │
        │  - Recursive Task Filling       │
        │  - Parallel Verification        │
        │  - Interactive Confirmation     │
        │  - Dynamic Timeout Execution    │
        └────────────────┬────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │   TOOLS MODULE (ferramentas/)   │
        │  - nmap.rs                      │
        │  - sqlmap.rs                    │
        │  - gobuster.rs                  │
        │  - reporter.rs                  │
        └────────────────┬────────────────┘
                         │
        ┌────────────────▼────────────────┐
        │     SYSTEM EXECUTION            │
        │  - Shell Commands               │
        │  - File Operations              │
        │  - Editor Integration           │
        └─────────────────────────────────┘
```

---

## 🔄 Detalhes por Módulo

### 1. ORACLE MODULE (`src/oraculo.rs`)

**Responsabilidade**: Traduzir input natural em tarefas estruturadas

#### Structures
```rust
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FenrirTask {
    pub task_type: String,                      // Type de tarefa
    pub ia_explanation: String,                 // Explicação IA
    pub command_to_run: Option<String>,         // Comando shell
    pub target_path: Option<String>,            // Path de arquivo
    pub application: Option<String>,            // App para abrir
    pub time_segment: Option<ExecutionTimeSegment>, // [NEW] Categoria
    pub retry_count: u32,                       // [NEW] Retries
    pub is_confirmed: bool,                     // [NEW] Confirmação
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTimeSegment {
    Quick,   // < 5s  (10s timeout)
    Medium,  // 5-60s (60s timeout)
    Long,    // > 60s (300s timeout)
}
```

#### Functions Flow
```
chamar_gemini_com_timeout()
    ↓
chamar_gemini_com_timeout_recursive(query, retry_count)
    ├─ Chamada Gemini API
    ├─ Se sucesso → parse_task_card() + categorize_by_time()
    └─ Se erro → handle_gemini_timeout_error()
            ├─ Se retry < 2 → Retry recursivo com delay
            └─ Se retry >= 2 → analyze_query_fallback()
                └─ Switch case com padrões conhecidos
```

#### Fallback Strategy Patterns
```rust
match true {
    _ if cmd.contains("list") || cmd.contains("ls") 
        → execute_command: "ls -lah"
    
    _ if cmd.contains("scan") || cmd.contains("nmap") 
        → execute_command: "nmap -sV localhost"
    
    _ if cmd.contains("sqlmap") 
        → execute_command: "sqlmap --wizard"
    
    _ if cmd.contains("gobuster") 
        → execute_command: "gobuster dir -u http://localhost"
    
    _ if cmd.contains("open") && cmd.contains(".rs") 
        → open_editor: "main.rs"
    
    _ if cmd.contains("compile") || cmd.contains("build") 
        → execute_command: "cargo build"
    
    _ if cmd.contains("run") && cmd.contains("cargo") 
        → execute_command: "cargo run"
    
    _ else 
        → unknown
}
```

#### Time Segmentation Logic
```rust
ExecutionTimeSegment::from_command(cmd) {
    Quick markers: ls, pwd, echo, cat, --version
    Long markers: nmap, sqlmap, gobuster, nikto, scan, fuzz
    Default: Medium
}
```

---

### 2. EXECUTOR MODULE (`src/executor.rs`)

**Responsabilidade**: Executar tarefas com validação e confirmação

#### Main Flow
```
fill_task_recursively(task)
    └─ Loop (max 5 iterações)
        ├─ 1. Valida completude
        ├─ 2. Se incompleta:
        │   ├─ tokio::spawn(verify_task_command())  ← Paralelo
        │   └─ tokio::spawn(verify_task_paths())    ← Paralelo
        ├─ 3. Exibe proposta
        ├─ 4. Aguarda input:
        │   ├─ "y/yes" → Marca is_confirmed=true, sai
        │   ├─ "n/no"  → Marca is_confirmed=false, sai
        │   └─ "edit"  → edit_task_interactive()
        └─ Retorna task
```

#### Async Verification Functions
```rust
verify_task_command(task) → Option<FenrirTask>
    - Verifica se task_type == "execute_command"
    - Se command_to_run é None, tenta completar
    - Simula delay de 200ms para async
    - Retorna enhanced task

verify_task_paths(task) → Option<FenrirTask>
    - Verifica se task_type == "open_editor"
    - Se target_path é None, tenta completar
    - Simula delay de 200ms para async
    - Retorna enhanced task
```

#### Interactive Edit Menu
```
🛠️  Interactive Edit Mode
1. Edit Task Type
2. Edit Explanation
3. Edit Command
4. Edit Target Path
5. Done editing

Choose option: _
```

#### Execution Functions
```rust
log_task(task)
    - Salva em fenrir_tasks.log
    - Inclui timestamp, segment, retry_count
    - JSON formatado

handle_execute_command_with_timeout(args, time_segment)
    - Extrai comando do JSON
    - Define timeout baseado no segment
    - Executa com tokio::time::timeout()
    - Feedback: ✅ ⏰ ❌

handle_execute_command(args)
    - Legacy, sem timeout
    - Mantido para compatibilidade

handle_open_editor(args)
    - Abre arquivo em editor
    - Suporte especial para macOS
```

---

### 3. MAIN ORCHESTRATION (`src/main.rs`)

**Flow Principal**:
```
interactive_mode()
    └─ loop:
        ├─ Lê input do usuário
        ├─ Se "exit" → break
        └─ Chama process_request()

process_request(query)
    ├─ 1. Progress bar (Gemini)
    ├─ 2. Oracle.chamar_gemini_com_timeout()
    ├─ 3. Executor.log_task()
    ├─ 4. Executor.fill_task_recursively()  [NEW]
    ├─ 5. Valida is_confirmed
    ├─ 6. Match task.task_type:
    │   ├─ "execute_command"
    │   │   └─ handle_execute_command_with_timeout() [NEW]
    │   ├─ "open_editor"
    │   │   └─ handle_open_editor()
    │   └─ "unknown"
    │       └─ Mensagem de erro
    └─ Loop continua
```

---

### 4. TOOLS MODULE (`src/ferramentas/`)

**Estrutura**:
```
ferramentas/
├── mod.rs           - Module declarations
├── nmap.rs          - Nmap scanner
├── sqlmap.rs        - SQLMap tool
├── gobuster.rs      - Directory brute force
└── reporter.rs      - Report generation
```

**Padrão Comum**:
```rust
pub fn run(args: Option<Value>) {
    // Extrai argumentos do JSON
    let args_map = args.as_ref()
        .and_then(|a| a.as_object())?;
    
    // Extrai campos necessários
    let target = args_map.get("target").as_str()?;
    let flags = args_map.get("flags").as_array()?;
    
    // Build command
    let mut cmd = Command::new("tool_name");
    cmd.arg(target);
    for flag in flags { cmd.arg(flag); }
    
    // Execute
    cmd.spawn()?;
}
```

---

## 📊 Data Flow Diagram

```
┌──────────────────┐
│  User Input      │
│ "scan localhost" │
└────────┬─────────┘
         │
         ▼
    ┌─────────────────────────────────────┐
    │ ORACLE - Gemini API                 │
    │ Request: "scan localhost"           │
    │ Response: Task Card (with retry)    │
    └────────┬────────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────┐
    │ CATEGORIZATION                      │
    │ Command: "nmap -sV localhost"       │
    │ → Category: Long (5 minutos)        │
    └────────┬────────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────┐
    │ RECURSIVE FILLING                   │
    │ ├─ Verify command field (tokio)    │
    │ ├─ Display proposal                 │
    │ ├─ Ask confirmation (y/n/edit)     │
    │ └─ Interactive edit (if needed)    │
    └────────┬────────────────────────────┘
             │
             ▼ (if confirmed)
    ┌─────────────────────────────────────┐
    │ EXECUTOR - With Dynamic Timeout     │
    │ Command: "nmap -sV localhost"       │
    │ Timeout: 300 seconds (Long)         │
    │ Status: ✅ Completed                │
    └────────┬────────────────────────────┘
             │
             ▼
    ┌─────────────────────────────────────┐
    │ LOGGING                             │
    │ fenrir_tasks.log                    │
    │ [Timestamp] Segment: Long, Retries: 0
    │ {JSON task data}                    │
    └─────────────────────────────────────┘
```

---

## 🔒 Error Handling Strategy

```
Gemini API Call
    ├─ Success
    │   └─ Parse and continue
    ├─ Timeout (attempt 1)
    │   ├─ Wait 500ms
    │   └─ Retry (attempt 2)
    ├─ Timeout (attempt 2)
    │   ├─ Wait 500ms
    │   └─ Retry (attempt 3)
    └─ Timeout (attempt 3+)
        ├─ Analyze query pattern
        └─ Use fallback strategy
            ├─ Success: use fallback task
            └─ Fail: return error
```

---

## 🎯 Performance Considerations

### Async Patterns
- ✅ `tokio::spawn()` - Parallel verification
- ✅ `tokio::time::timeout()` - Dynamic timeouts
- ✅ `task::spawn_blocking()` - Console I/O

### Optimization Points
1. **Parallel Verification** - 2 tasks rodando simultaneamente
2. **Dynamic Timeouts** - Não aguarda mais que necessário
3. **Early Exit** - Confirma e executa sem loops extras
4. **Fallback Strategy** - Sem chamadas extras à API se falhar

---

## 🧪 Testing Scenarios

| Scenario | Expected | Status |
|----------|----------|--------|
| Gemini timeout → Fallback | "nmap" pattern detected | ✅ |
| Task incomplete | Parallel verification | ✅ |
| User confirms | Executa com timeout | ✅ |
| User edits | Modo interativo | ✅ |
| Long command | 300s timeout | ✅ |
| Quick command | 10s timeout | ✅ |

---

## 🚀 Future Enhancements

1. **Caching** - Cache de padrões reconhecidos
2. **Telemetry** - Metrics de execução
3. **Undo/Redo** - Desfazer última ação
4. **Batch Mode** - Executar múltiplas tarefas
5. **Custom Timeouts** - User-defined timeouts
6. **Task History** - UI para ver histórico
7. **Confidence Score** - AI retorna confiança da tarefa

---

## 📋 Architecture Summary

| Camada | Componente | Responsabilidade |
|--------|-----------|-----------------|
| Input | Main | Orchestração e I/O |
| Intelligence | Oracle | Parse natural → Task |
| Validation | Executor | Verify e confirm |
| Execution | Tools | Run actual commands |
| Logging | Both | Track activities |

---

**Versão**: 0.2.0  
**Data**: 15 de novembro de 2025  
**Status**: ✅ Pronto para produção

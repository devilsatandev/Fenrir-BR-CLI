# Fenrir - Funcionalidades Implementadas

Data: 15 de novembro de 2025

## 📋 Resumo das Implementações

Este documento detalha as três funcionalidades principais implementadas no Fenrir CLI.

---

## 1️⃣ Switch Case para Tarefas Complexas com Timeout e IA Fallback

### 📝 Descrição
Implementado um sistema robusto de tratamento de timeout que:
- Detecta quando uma chamada à IA (Gemini) falha por timeout
- Realiza **2 tentativas automáticas** com espera entre elas
- Se todas falhem, ativa **estratégias de fallback** baseadas em análise de padrões

### 📂 Onde foi implementado
**Arquivo**: `src/oraculo.rs`

**Funções principais**:
- `chamar_gemini_com_timeout()` - Entry point principal
- `chamar_gemini_com_timeout_recursive()` - Orquestração com retry
- `handle_gemini_timeout_error()` - Lógica de fallback
- `analyze_query_fallback()` - Switch case com padrões conhecidos

### 🎯 Lógica de Fallback (Switch Case)
```rust
match true {
    _ if lower.contains("list") || lower.contains("ls") => "ls -lah",
    _ if lower.contains("scan") || lower.contains("nmap") => "nmap -sV localhost",
    _ if lower.contains("sqlmap") => "sqlmap --wizard",
    _ if lower.contains("gobuster") => "gobuster dir -u http://localhost",
    _ if lower.contains("open") && lower.contains(".rs") => "open_editor",
    _ if lower.contains("compile") || lower.contains("build") => "cargo build",
    _ if lower.contains("run") && lower.contains("cargo") => "cargo run",
    _ => "unknown",
}
```

### ✅ Benefícios
- Resilência contra falhas de rede ou timeout da IA
- Recuperação automática sem intervenção do usuário
- Fallback para comandos conhecidos
- Informações de debug com 📝 emojis para melhor visibilidade

---

## 2️⃣ Preenchimento Recursivo de Task com Confirmação Iterativa

### 📝 Descrição
Implementado um sistema **recursivo e interativo** que:
- Valida se a tarefa está completa (campos obrigatórios preenchidos)
- Se incompleta, testa **alternativas em paralelo** usando `tokio::spawn`
- Oferece confirmação iterativa com opções: Confirmar | Rejeitar | Editar
- Permite edição interativa de cada campo da tarefa
- Máximo de 5 iterações para evitar loops infinitos

### 📂 Onde foi implementado
**Arquivo**: `src/executor.rs`

**Funções principais**:
- `fill_task_recursively()` - Loop principal recursivo
- `verify_task_command()` - Verifica campo de comando (async paralelo)
- `verify_task_paths()` - Verifica campo de caminho (async paralelo)
- `edit_task_interactive()` - Edição interativa de campos

### 🔄 Fluxo
```
1. Recebe task do Oracle
2. Loop iterativo (máx 5 iterações):
   a. Valida completude
   b. Se incompleta → tokio::spawn 2 verificações em paralelo
   c. Exibe proposta formatada
   d. Aguarda confirmação do usuário:
      - "y" → Confirma e sai
      - "n" → Rejeita e sai
      - "edit" → Entra em modo edição
3. Retorna task confirmada ou rejeitada
```

### ✨ Modo Edição Interativa
O usuário pode editar:
1. Task Type (execute_command, open_editor, unknown)
2. Explicação (descrição da ação)
3. Comando (shell command)
4. Caminho de arquivo (target path)

### ✅ Benefícios
- Confirmação explícita antes de executar ações
- Correção de erros da IA antes da execução
- Parallelização com tokio para melhor performance
- UX clara com opções de edição

---

## 3️⃣ Segmentação de Tarefas por Tempo de Execução

### 📝 Descrição
Implementado um sistema de **categorização automática** que:
- Classifica tarefas em 3 segmentos: Quick, Medium, Long
- Define timeouts apropriados para cada categoria
- Aplica timeout dinâmico na execução do comando

### 📂 Onde foi implementado
**Arquivo**: `src/oraculo.rs`

**Tipos e Funções**:
- `enum ExecutionTimeSegment` - Três categorias (Quick/Medium/Long)
- `ExecutionTimeSegment::from_command()` - Análise do comando
- `ExecutionTimeSegment::max_timeout()` - Timeout por categoria

### ⏱️ Categorias e Timeouts

| Categoria | Duração | Exemplos |
|-----------|---------|----------|
| **Quick** | < 5s | `ls`, `pwd`, `echo`, `--version` |
| **Medium** | 5-60s | Operações padrão, build pequeno |
| **Long** | > 60s | `nmap`, `sqlmap`, `gobuster`, `nikto` |

### 📂 Onde é aplicado
**Arquivo**: `src/executor.rs`

**Função**:
- `handle_execute_command_with_timeout()` - Executa com timeout dinâmico

**Uso em main.rs**:
```rust
handle_execute_command_with_timeout(Some(args), task.time_segment).await;
```

### 🎯 Lógica de Classificação
```rust
pub fn from_command(command: &str) -> Self {
    if comando contém: ls, pwd, echo, cat, --version
        → Quick (10 segundos)
    
    if comando contém: nmap, sqlmap, gobuster, nikto, scan, fuzz
        → Long (5 minutos)
    
    else
        → Medium (60 segundos)
}
```

### ✅ Benefícios
- Evita timeouts prematuros para tarefas longas
- Cancela rapidamente tarefas que travam
- Logging com informação de segmento de tempo
- Melhor experiência do usuário com feedback visual

---

## 🔧 Estrutura da FenrirTask Atualizada

```rust
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FenrirTask {
    pub task_type: String,              // execute_command, open_editor, unknown
    pub ia_explanation: String,         // Explicação da IA
    pub command_to_run: Option<String>, // Comando a executar
    pub target_path: Option<String>,    // Caminho de arquivo
    pub application: Option<String>,    // Aplicação para abrir
    pub time_segment: Option<ExecutionTimeSegment>,  // [NEW]
    pub retry_count: u32,               // [NEW] Contador de retries
    pub is_confirmed: bool,             // [NEW] Confirmado pelo usuário
}
```

---

## 📊 Fluxo Completo de Execução

```
┌─────────────────────────────────────────┐
│ 1. Usuário digita comando               │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│ 2. ORACLE (Gemini)                      │
│    - Com retry automático (2x)          │
│    - Com fallback strategy              │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│ 3. CATEGORIZAÇÃO DE TEMPO               │
│    - Quick, Medium, ou Long             │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│ 4. PREENCHIMENTO RECURSIVO              │
│    - Validação (até 5 iterações)        │
│    - Verificações paralelas (tokio)     │
│    - Edição interativa                  │
│    - Confirmação do usuário             │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│ 5. EXECUTOR COM TIMEOUT DINÂMICO        │
│    - Timeout baseado no segmento        │
│    - Logging com informações de tempo   │
│    - Feedback visual (✅ ⏰ ❌)         │
└─────────────────────────────────────────┘
```

---

## 🚀 Como Testar

### Teste 1: Timeout e Fallback
```bash
cargo run "scan network for vulnerabilities"
```
Se Gemini timeout, ativa fallback automaticamente.

### Teste 2: Preenchimento Recursivo
```bash
cargo run "build the rust project"
```
Pede confirmação com opção de edição antes de executar.

### Teste 3: Timeout Dinâmico
```bash
cargo run "run nmap scan on localhost"
```
Aplica timeout de 5 minutos (categoria Long).

---

## 📝 Logs

Os logs são salvos em `fenrir_tasks.log` com informações:
```
--- [ 2025-11-15 14:30:00 ] (Segment: Long, Retries: 0) ---
{
  "task_type": "execute_command",
  "ia_explanation": "The user wants to run a version scan...",
  "command_to_run": "nmap -sV localhost",
  ...
}
```

---

## ✅ Conclusão

Todas as 3 funcionalidades foram implementadas e compilam sem erros:
- ✅ Switch case para timeout com fallback IA
- ✅ Preenchimento recursivo com verificações paralelas
- ✅ Segmentação de tarefas por tempo de execução

O código está pronto para testes em ambiente real.

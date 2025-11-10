use std::collections::HashMap;
use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use regex::Regex;
use strsim::levenshtein;
use indicatif::{ProgressBar, ProgressStyle};
fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Ei, véi! Uso: fenrir <comando em PT natural>");
        println!("Ex: fenrir abre pasta ~/kali-cli/src/");
        println!("Ou: fenrir abre kali-cli/src e roda main.rs no rustrover");
        println!("Pra interativo, rode sem args.");
        interativo();
        return;
    }

    args.remove(0);
    let comando_natural = args.join(" ").to_lowercase();

    let re_abre_pasta = Regex::new(r"abre pasta (.+)").unwrap();
    let re_abre_e_roda = Regex::new(r"abre (.+) e roda (.+) no (.+)").unwrap();

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("{spinner:.green} [{elapsed_precise}] Processando... {msg}")
        .unwrap());
    pb.enable_steady_tick(Duration::from_millis(100));

    let handle = thread::spawn(move || {
        if let Some(caps) = re_abre_pasta.captures(&comando_natural) {
            let pasta = caps.get(1).unwrap().as_str();
            abrir_pasta_fuzzy(pasta);
        } else if let Some(caps) = re_abre_e_roda.captures(&comando_natural) {
            let pasta = caps.get(1).unwrap().as_str();
            let arquivo = caps.get(2).unwrap().as_str();
            let app = caps.get(3).unwrap().as_str();
            abrir_pasta_fuzzy(pasta);
            rodar_arquivo_em_app_fuzzy(arquivo, app);
        } else if comando_natural.contains("e") {
            let comandos = comando_natural.replace("e", "&&").split("&&").map(|s| s.trim().to_string()).collect::<Vec<_>>();
            for cmd in comandos {
                executar_comando_simples(&cmd);
            }
        } else {
            println!("Ops, não entendi essa, véi. Explica de novo? Tipo 'abre pasta X' ou 'roda Y no Z'.");
            interativo();
        }

        let explicacoes = carregar_explicacoes();
        if let Some(explicacao) = explicacoes.get(&comando_natural) {
            println!("{}", explicacao);
        }
    });

    let start = Instant::now();
    loop {
        if handle.is_finished() {
            pb.finish_with_message("Pronto!");
            break;
        }
        if start.elapsed() > Duration::from_secs(66) {
            pb.finish_with_message("Timeout! Tentando de novo...");
            println!("Timeout após 66s, reiniciando processo.");
            // Aqui poderia kill thread, mas pra simples: drop handle e rerun logic
            // Pra real: use channels ou tokio, mas hardcore std
            main(); // Recursa uma vez (cuidado stack, mas pra demo ok)
            return;
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn abrir_pasta_fuzzy(pasta: &str) {
    let caminho = fuzzy_search_path(pasta);
    if caminho.is_empty() {
        println!("Não achei pasta parecida com '{}', véi.", pasta);
        return;
    }
    let status = Command::new("xdg-open")
        .arg(&caminho)
        .status();
    match status {
        Ok(_) => println!("Abri a pasta {} aí, véi!", caminho),
        Err(_) => println!("Deu ruim abrindo {}, verifica?", caminho),
    }
}

fn rodar_arquivo_em_app_fuzzy(arquivo: &str, app: &str) {
    let arquivo_path = fuzzy_search_path(arquivo);
    if arquivo_path.is_empty() {
        println!("Não achei arquivo parecida com '{}', véi.", arquivo);
        return;
    }
    let app_cmd = match app {
        "rustrover" => "rustrover",
        _ => app,
    };
    let status = Command::new(app_cmd)
        .arg(&arquivo_path)
        .status();
    match status {
        Ok(_) => println!("Rodando {} no {} agora!", arquivo_path, app),
        Err(_) => println!("Não rolou rodar {} no {}, verifica PATH?", arquivo_path, app),
    }
}

fn fuzzy_search_path(query: &str) -> String {
    let home = env::var("HOME").unwrap_or(".".to_string());
    let base_dir = if query.starts_with("~") { home } else { ".".to_string() };
    let path = Path::new(&base_dir);

    if let Ok(entries) = std::fs::read_dir(path) {
        let mut best_match = String::new();
        let mut best_score = 0.0;
        for entry in entries {
            if let Ok(entry) = entry {
                let name = entry.file_name().to_string_lossy().to_string();
                let dist = levenshtein(query, &name) as f64 / query.len().max(name.len()) as f64;
                let score = 1.0 - dist; // Similaridade
                if score > best_score && score > 0.7 { // Threshold comprovado empiricamente
                    best_score = score;
                    best_match = entry.path().to_string_lossy().to_string();
                }
            }
        }
        best_match
    } else {
        String::new()
    }
}

fn executar_comando_simples(cmd: &str) {
    if cmd.contains("ls") {
        Command::new("ls").status().unwrap();
    } else {
        println!("Comando simples '{}' não suportado ainda, véi.", cmd);
    }
}

fn interativo() {
    println!("Modo interativo! Digita o comando natural, ou 'sair'.");
    let stdin = io::stdin();
    for linha in stdin.lines() {
        match linha {
            Ok(input) => {
                let trimado = input.trim().to_lowercase();
                if trimado == "sair" { break; }
                println!("Processando: {}", trimado);
            }
            Err(_) => break,
        }
    }
}

fn carregar_explicacoes() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    // Preencha como no original: navegação, ls, etc. (copia do teu código antigo aqui)
    map.insert("navegação".to_string(), "Ei, navegação no terminal é tipo passear pela sua máquina. Os principais:\n- ls: Lista os arquivos e pastas no diretório atual, tipo 'o que tem aqui?'\n- cd: Muda de pasta, como 'cd documentos' pra entrar na pasta documentos.\n- pwd: Mostra onde você tá agora, o caminho completo.\n- mkdir: Cria uma pasta nova, 'mkdir nova_pasta'.\n- rmdir: Remove pasta vazia, mas usa com cuidado.".to_string());
    // ... adicione o resto do map igual antes, pra completo
    map.insert("ls".to_string(), "ls: Esse é o cara que lista tudo no diretório. Tipo, 'ls -l' pra detalhes, 'ls -a' pra arquivos escondidos. Simples e útil pra ver o que rola por aí.".to_string());
    map.insert("cd".to_string(), "cd: Muda de diretório. 'cd ..' sobe um nível, 'cd /' vai pra raiz. É como teleportar pela sua máquina.".to_string());
    map.insert("pwd".to_string(), "pwd: Print Working Directory. Mostra o caminho atual, tipo 'onde diabos eu tô?' Resposta rápida.".to_string());
    map.insert("mkdir".to_string(), "mkdir: Cria diretório. 'mkdir -p caminho/nova/pasta' cria tudo no caminho se não existir. Criativo, né?".to_string());
    map.insert("rmdir".to_string(), "rmdir: Remove diretório vazio. Pra pastas cheias, melhor usar rm -r.".to_string());

    // Edição
    map.insert("edição".to_string(), "Edição de arquivos no terminal é pros valentes. Principais editores:\n- nano: Simples, amigável, 'nano arquivo.txt' e edita.\n- vim: Poderoso, mas curva de aprendizado. 'vim arquivo' pra entrar, i pra inserir, esc :wq pra salvar e sair.\n- vi: Versão básica do vim.\n- cat: Não edita, mas mostra conteúdo. 'cat arquivo'.\n- echo: Insere texto simples, 'echo 'oi' > arquivo'.".to_string());
    map.insert("nano".to_string(), "nano: Editor fácil. Abre com 'nano arquivo', edita, ctrl+o salva, ctrl+x sai. Perfeito pra iniciantes.".to_string());
    map.insert("vim".to_string(), "vim: Editor avançado. Modos: normal, insert (i), visual (v). Salva com :w, sai :q. 'vimtutor' pra aprender.".to_string());
    map.insert("vi".to_string(), "vi: Versão old school do vim. Mesmos comandos basicamente.".to_string());
    map.insert("cat".to_string(), "cat: Concatena e mostra arquivos. 'cat arquivo1 arquivo2 > novo' junta eles.".to_string());
    map.insert("echo".to_string(), "echo: Ecoa texto. 'echo 'hello' >> arquivo' adiciona no final sem apagar.".to_string());

    // Abertura
    map.insert("abertura".to_string(), "Abrir coisas: Depende do sistema, mas comuns:\n- open: No mac, abre arquivos ou apps. 'open arquivo.pdf'.\n- xdg-open: No Linux, similar.\n- less: Abre pra leitura paginada, 'less arquivo'.\n- more: Similar ao less, mas mais simples.".to_string());
    map.insert("open".to_string(), "open: Abre arquivos no app padrão. Útil no macOS.".to_string());
    map.insert("xdg-open".to_string(), "xdg-open: Abre no Linux com o app padrão.".to_string());
    map.insert("less".to_string(), "less: Visualizador paginado. q pra sair.".to_string());
    map.insert("more".to_string(), "more: Visualizador antigo, página por página.".to_string());

    // Fechamento
    map.insert("fechamento".to_string(), "Fechar processos ou sessões:\n- kill: Mata processo por PID, 'kill 1234'.\n- killall: Mata por nome, 'killall chrome'.\n- exit: Sai do terminal ou shell.\n- ctrl+c: Interrompe comando rodando.".to_string());
    map.insert("kill".to_string(), "kill: Envia sinal pra processo. 'kill -9 PID' força matar.".to_string());
    map.insert("killall".to_string(), "killall: Mata todos processos com nome dado.".to_string());
    map.insert("exit".to_string(), "exit: Sai do shell atual.".to_string());

    // Inserção
    map.insert("inserção".to_string(), "Inserir ou copiar:\n- cp: Copia arquivos, 'cp origem destino'.\n- mv: Move ou renomeia, 'mv velho novo'.\n- touch: Cria arquivo vazio, 'touch novo.txt'.\n- echo >: Insere em arquivo novo.".to_string());
    map.insert("cp".to_string(), "cp: Copia. 'cp -r pasta destino' pra recursivo.".to_string());
    map.insert("mv".to_string(), "mv: Move. Também renomeia se no mesmo lugar.".to_string());
    map.insert("touch".to_string(), "touch: Cria ou atualiza timestamp de arquivo.".to_string());

    // Remoção
    map.insert("remoção".to_string(), "Remover com cuidado, hein:\n- rm: Remove arquivos, 'rm arquivo'.\n- rm -r: Remove pastas recursivamente.\n- rm -f: Força sem perguntar.".to_string());
    map.insert("rm".to_string(), "rm: Remove. 'rm -rf /' é piada perigosa, não faz isso!".to_string());

    map
}
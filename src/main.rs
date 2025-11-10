
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Ei, cara! Uso: {} <comando ou categoria>", args[0]);
        println!("Categorias disponíveis: navegação, edição, abertura, fechamento, inserção, remoção");
        println!("Ou digite um comando específico como 'ls' pra uma explicação rapidinha.");
        println!("Pra interativo, rode sem argumentos e vai perguntando.");
        interativo();
        return;
    }

    let consulta = args[1].to_lowercase();
    let explicacoes = carregar_explicacoes();

    if let Some(explicacao) = explicacoes.get(&consulta) {
        println!("{}", explicacao);
    } else if consulta == "todas" {
        for (chave, valor) in explicacoes.iter() {
            println!("=== {} ===\n{}\n", chave, valor);
        }
    } else {
        println!("Ops, não achei isso não. Tenta uma categoria ou comando válido, véi.");
        interativo();
    }
}

fn interativo() {
    println!("Modo interativo ativado! Digita o comando ou categoria, ou 'sair' pra vazar.");
    let stdin = io::stdin();
    let explicacoes = carregar_explicacoes();

    for linha in stdin.lines() {
        match linha {
            Ok(input) => {
                let trimado = input.trim().to_lowercase();
                if trimado == "sair" {
                    println!("Falou, parceiro! Até a próxima.");
                    break;
                }
                if let Someuse std::collections::HashMap;
use std::env;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Ei, cara! Uso: {} <comando ou categoria>", args[0]);
        println!("Categorias disponíveis: navegação, edição, abertura, fechamento, inserção, remoção");
        println!("Ou digite um comando específico como 'ls' pra uma explicação rapidinha.");
        println!("Pra interativo, rode sem argumentos e vai perguntando.");
        interativo();
        return;
    }

    let consulta = args[1].to_lowercase();
    let explicacoes = carregar_explicacoes();

    if let Some(explicacao) = explicacoes.get(&consulta) {
        println!("{}", explicacao);
    } else if consulta =(explicacao) = explicacoes.get(&trimado) {
                    println!("{}", explicacao);
                } else {
                    println!("Não rolou. Tenta de novo, ou digita 'todas' pra ver tudo.");
                }
            }
            Err(_) => break,
        }
    }
}

fn carregar_explicacoes() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    // Navegação
    map.insert("navegação".to_string(), "Ei, navegação no terminal é tipo passear pela sua máquina. Os principais:\n- ls: Lista os arquivos e pastas no diretório atual, tipo 'o que tem aqui?'\n- cd: Muda de pasta, como 'cd documentos' pra entrar na pasta documentos.\n- pwd: Mostra onde você tá agora, o caminho completo.\n- mkdir: Cria uma pasta nova, 'mkdir nova_pasta'.\n- rmdir: Remove pasta vazia, mas usa com cuidado.".to_string());
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

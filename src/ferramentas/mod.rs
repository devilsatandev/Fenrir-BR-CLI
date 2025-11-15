// --- MÓDULO PAI (ferramentas/mod.rs) ---
// O "Arsenal" do Fenrir.
// Quando o 'main.rs' fala 'mod ferramentas', ele lê esse
// arquivo. E esse arquivo diz quem mais tá no time.

// Declara os módulos "filho" (os arquivos .rs na pasta)
pub mod nmap;
pub mod sqlmap;
pub mod reporter; // A "ARMA" DO TECH LEAD
pub mod gobuster; // A NOVA ARMA
// pub mod metasploit; // (Exemplo futuro)
// pub mod hydra; // (Exemplo futuro)
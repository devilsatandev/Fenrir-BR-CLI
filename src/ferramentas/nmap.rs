// --- MÓDULO NMAP ---

// Por enquanto, o Oráculo chama o 'nmap' como um 'execute_command'.
//
// No futuro "Pique Claudão Nível 2", a gente podia fazer
// uma função em Rust puro aqui, tipo:
//
// use std::process::Command;
//
// pub struct NmapScan {
//     pub target: String,
//     pub scan_type: String,
// }
//
// pub fn run_scan(scan: NmapScan) -> Result<String, String> {
//     let output = Command::new("nmap")
//         .arg(scan.scan_type)
//         .arg(scan.target)
//         .output();
//
//     match output {
//         Ok(out) => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
//         Err(e) => Err(e.to_string()),
//     }
// }
//
// ...e o Oráculo aprenderia a preencher esse 'struct'.
// Mas isso é "100 dólar a consultoria", não "10". KKKKKK 


// sai de narnia aquele jumento vai fazer tudo errado
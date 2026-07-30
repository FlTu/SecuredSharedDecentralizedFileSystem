//! CLI SyFi (docs/015-roadmap.md Phase 2) — valide le Vault Engine en
//! conditions reelles, sans interface graphique. Parsing d'arguments
//! manuel volontairement simple (pas de `clap`, pour eviter d'ajouter une
//! dependance non essentielle a ce stade).

use std::env;
use std::path::PathBuf;
use vault::{Vault, ROOT_NODE_ID};

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  syfi create <coffre> <passphrase>");
    eprintln!("  syfi import <coffre> <passphrase> <fichier_source>");
    eprintln!("  syfi ls <coffre> <passphrase>");
    eprintln!("  syfi export <coffre> <passphrase> <node_id_hex> <destination>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "create" if args.len() == 4 => cmd_create(&args[2], &args[3]),
        "import" if args.len() == 5 => cmd_import(&args[2], &args[3], &args[4]),
        "ls" if args.len() == 4 => cmd_ls(&args[2], &args[3]),
        "export" if args.len() == 6 => cmd_export(&args[2], &args[3], &args[4], &args[5]),
        _ => {
            print_usage();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Erreur: {e}");
        std::process::exit(1);
    }
}

fn cmd_create(path: &str, passphrase: &str) -> Result<(), Box<dyn std::error::Error>> {
    Vault::create(PathBuf::from(path), passphrase.as_bytes())?;
    println!("Coffre cree: {path}");
    Ok(())
}

fn cmd_import(path: &str, passphrase: &str, source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut v = Vault::open(PathBuf::from(path), passphrase.as_bytes())?;
    let name = PathBuf::from(source)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| source.to_string());
    let node = v.import_file(source, ROOT_NODE_ID, &name)?;
    println!("Importe: {name} -> node {}", hex::encode(node_bytes(&node)));
    Ok(())
}

fn cmd_ls(path: &str, passphrase: &str) -> Result<(), Box<dyn std::error::Error>> {
    let v = Vault::open(PathBuf::from(path), passphrase.as_bytes())?;
    for entry in v.list_directory(ROOT_NODE_ID) {
        println!(
            "{:8} {:>10}  {}",
            match entry.entry_type {
                manifest::EntryType::File => "fichier",
                manifest::EntryType::Directory => "dossier",
            },
            entry.size,
            entry.name
        );
    }
    Ok(())
}

fn cmd_export(path: &str, passphrase: &str, node_hex: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    let v = Vault::open(PathBuf::from(path), passphrase.as_bytes())?;
    let bytes = hex::decode(node_hex)?;
    if bytes.len() != 16 {
        return Err("identifiant de noeud invalide (attendu 16 octets en hexadecimal)".into());
    }
    let mut id = [0u8; 16];
    id.copy_from_slice(&bytes);
    v.export_file(common::NodeId(id), dest)?;
    println!("Exporte vers {dest}");
    Ok(())
}

fn node_bytes(id: &common::NodeId) -> [u8; 16] {
    id.0
}

/// Encodage/decodage hexadecimal minimal, pour eviter une dependance externe
/// juste pour cette conversion d'affichage.
mod hex {
    pub fn encode(bytes: [u8; 16]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("longueur hexadecimale impaire".into());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

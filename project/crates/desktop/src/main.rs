//! Interface desktop minimale (docs/015-roadmap.md Phase 9).
//!
//! Explorateur virtuel : ouvre un coffre (chemin + passphrase), affiche le
//! contenu de la racine, permet d'exporter un fichier selectionne. Pas de
//! montage FUSE/WinFsp reel a ce stade (explorateur interne d'abord,
//! conformement a la decision prise dans 015-roadmap.md).
//!
//! NON COMPILE NI EXECUTE DANS LE SANDBOX qui a servi a construire le reste
//! du squelette : l'arbre de dependances d'eframe (winit, wgpu, wayland...)
//! depasse ce que le Rust 1.75 (installe via apt, sans rustup) de ce
//! sandbox peut construire, et le sandbox n'a de toute facon pas de serveur
//! d'affichage. A valider en premier sur ta machine avec `cargo run -p
//! desktop` (rustup, Rust recent) — remets d'abord "crates/desktop" dans
//! les membres du workspace racine.

use eframe::egui;
use manifest::EntryType;
use vault::{Vault, ROOT_NODE_ID};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "SyFi — explorateur (squelette)",
        options,
        Box::new(|_cc| Box::new(SyfiApp::default())),
    )
}

#[derive(Default)]
struct SyfiApp {
    vault_path: String,
    passphrase: String,
    vault: Option<Vault>,
    status: String,
    selected: Option<[u8; 16]>,
    export_dest: String,
}

impl eframe::App for SyfiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SyFi — explorateur de coffre (squelette)");

            ui.horizontal(|ui| {
                ui.label("Coffre :");
                ui.text_edit_singleline(&mut self.vault_path);
            });
            ui.horizontal(|ui| {
                ui.label("Passphrase :");
                ui.add(egui::TextEdit::singleline(&mut self.passphrase).password(true));
            });

            if ui.button("Ouvrir").clicked() {
                match Vault::open(&self.vault_path, self.passphrase.as_bytes()) {
                    Ok(v) => {
                        self.vault = Some(v);
                        self.status = "Coffre ouvert.".to_string();
                    }
                    Err(e) => {
                        self.vault = None;
                        self.status = format!("Erreur a l'ouverture : {e}");
                    }
                }
            }

            ui.separator();

            if let Some(vault) = &self.vault {
                ui.label("Contenu de la racine :");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for entry in vault.list_directory(ROOT_NODE_ID) {
                        let icon = match entry.entry_type {
                            EntryType::File => "📄",
                            EntryType::Directory => "📁",
                        };
                        let label = format!("{icon} {} ({} octets)", entry.name, entry.size);
                        if ui.selectable_label(self.selected == Some(entry.node_id), label).clicked() {
                            self.selected = Some(entry.node_id);
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Exporter vers :");
                    ui.text_edit_singleline(&mut self.export_dest);
                    if ui.button("Exporter le fichier selectionne").clicked() {
                        if let Some(id) = self.selected {
                            match vault.export_file(common::NodeId(id), &self.export_dest) {
                                Ok(()) => self.status = "Export reussi.".to_string(),
                                Err(e) => self.status = format!("Erreur a l'export : {e}"),
                            }
                        } else {
                            self.status = "Selectionne d'abord un fichier.".to_string();
                        }
                    }
                });
            }

            ui.separator();
            ui.label(&self.status);
        });
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use std::fs;
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Unified Game Launcher",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Default)]
struct MyApp {
    picked_path: Option<String>,
    subfolders_with_exes: Vec<String>,
}

impl MyApp {
    fn update_subfolders_with_exes(&mut self) {
        if let Some(ref path) = self.picked_path {
            self.subfolders_with_exes.clear();

            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        if contains_exe(&entry_path) {
                            if let Some(path_str) = entry_path.to_str() {
                                self.subfolders_with_exes.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn contains_exe(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("exe") {
                return true;
            }
        }
    }

    false
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Set path for Steam");

            if ui.button("Open folder…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.picked_path = Some(path.display().to_string());
                    self.update_subfolders_with_exes();
                }
            }

            if let Some(ref picked_path) = self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Steam Path:");
                    ui.monospace(picked_path);
                });

                ui.heading("Subfolders with Executables");
                for folder in &self.subfolders_with_exes {
                    ui.label(folder);
                }
            }
        });
    }
}

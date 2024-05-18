#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::egui;
use egui::CursorIcon;
use log::{debug, info};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct MyApp {
    config: Config,
    games: Vec<Game>,
    search_string: String,
}

#[derive(Default, Debug)]
struct Game {
    appid: i64,
    size: i64,
    name: String,
    path: String,
}

#[derive(Default)]
struct Config {
    steam_path: String,
}

impl Config {
    fn load() -> Self {
        let mut config_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("config.txt")
            .unwrap();

        let mut config = String::new();
        config_file.read_to_string(&mut config).unwrap();

        println!("{}", config);
        Config { steam_path: config }
    }
}

impl MyApp {
    fn new(config: Config) -> Self {
        let mut app = MyApp {
            config,
            ..Default::default()
        };
        app.find_installed_games();
        app
    }

    fn find_installed_games(&mut self) {
        let steam_path = Path::new(&self.config.steam_path);

        if steam_path.is_dir() {
            if let Ok(entries) = fs::read_dir(steam_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        info!("calling read_file_to_game_struct with path:{:?}", path);
                        let game = self.read_file_to_game_struct(path);
                        self.games.push(game);
                    }
                }
            }
        }
    }

    fn read_file_to_game_struct(&mut self, path: PathBuf) -> Game {
        debug!("{:?}", path);
        let contents = fs::read_to_string(path).expect("unable to read from app manifest file");

        let lines: Vec<&str> = contents.trim_matches('"').split("\n").collect();
        let mut game = Game::default();

        for line in lines {
            let k_v: Vec<&str> = line.split_whitespace().collect();

            if k_v.len() >= 2 {
                let key = k_v[0].trim_matches('"');
                let value = k_v[1..].join(" ").trim_matches('"').to_string();

                match key {
                    "appid" => game.appid = value.parse().unwrap(),
                    "SizeOnDisk" => game.size = value.parse().unwrap(),
                    "name" => game.name = value.to_string(),
                    "installdir" => {
                        game.path = format!(
                            r"{}\{}\{}",
                            self.config.steam_path,
                            "common",
                            value.to_string()
                        )
                    }
                    _ => {}
                }
            }
        }
        debug!("{:?}", game);
        game
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(200.0)
            .width_range(200.0..=250.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Installed Games");
                    ui.separator();
                    let _search_field = ui.add(
                        egui::TextEdit::singleline(&mut self.search_string)
                            .hint_text("Find By Name"),
                    );
                    ui.separator();
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                        |ui| {
                            for game in &self.games {
                                debug!("{:?}", &game.path);
                                let game_label = ui.label(&game.name);
                                if game_label.hovered() {
                                    ctx.set_cursor_icon(CursorIcon::PointingHand);
                                }
                                if game_label.clicked() {
                                    println!("{}", &game.name);
                                }
                                ui.add_space(5.0);
                            }
                        },
                    )
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Set path for Steam");

            let open_folder = ui.button("Open Folder");

            if open_folder.hovered() {
                ctx.set_cursor_icon(CursorIcon::PointingHand);
            }

            if open_folder.clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    println!("{:?}", path);
                    // need to save the path to config and to the config file
                }
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1300.0, 700.0])
            .with_drag_and_drop(false),
        ..Default::default()
    };

    let config = Config::load();

    info!("starting app");
    eframe::run_native(
        "Unified Game Launcher",
        options,
        Box::new(|_cc| Box::new(MyApp::new(config))),
    )
}

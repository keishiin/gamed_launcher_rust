#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)]

use eframe::egui;
use egui::CursorIcon;
use log::{debug, info};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Default)]
struct MyApp {
    config: Config,
    games: Vec<Game>,
    search_string: String,
    game_selected: Game,
    current_page: String,
    settings_page_flag: bool, // this is temp maybe i can find some better way to do this
}

#[derive(Default, Debug, Clone)]
struct Game {
    appid: i64,
    size: f64,
    name: String,
    path: String,
    icon: String,
    _logo: String, // need to draw on top of header
    header: String,
}

#[derive(Default, Clone)]
struct Config {
    steam_path: String,
    steam_game_cache_path: String,
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
        let parts: Vec<&str> = config.trim().split("\r\n").collect();
        if parts.len() >= 2 {
            Config {
                steam_path: parts[0].to_string(),
                steam_game_cache_path: parts[1].to_string(),
            }
        } else {
            Config {
                steam_path: String::new(),
                steam_game_cache_path: String::new(),
            }
        }
    }

    fn save(&self) {
        let contents = format!("{}\r\n{}", self.steam_path, self.steam_game_cache_path);
        println!("{}", contents);
        fs::write("config.txt", contents).expect("unable to save to file");
    }
}

impl MyApp {
    fn new(config: Config) -> Self {
        let mut app = MyApp {
            config,
            settings_page_flag: true,
            current_page: "Settings".to_string(),
            ..Default::default()
        };

        if !app.config.steam_path.is_empty() {
            app.find_installed_games();
        }

        if !app.games.is_empty() {
            app.game_selected = app.games[0].clone();
        }
        app
    }

    fn find_installed_games(&mut self) {
        if !self.games.is_empty() {
            self.games.clear();
        }

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
                    "SizeOnDisk" => {
                        let size: f64 = value.parse().unwrap();
                        let gb = size / 1024f64.powi(3);
                        game.size = (gb * 10.0).round() / 10.0;
                    }
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
                let icon = format!(
                    r"{}/{}_icon.jpg",
                    self.config.steam_game_cache_path, game.appid
                );
                let header = format!(
                    r"{}/{}_library_hero.jpg",
                    self.config.steam_game_cache_path, game.appid
                );
                let logo = format!(
                    r"{}/{}_logo.jpg",
                    self.config.steam_game_cache_path, game.appid
                );
                game.icon = icon.to_string();
                game.header = header.to_string();
                game._logo = logo.to_string();
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
                            let games = self.games.clone();
                            for game in games {
                                debug!("{:?}", &game.path);
                                let game_label = ui.label(&game.name);
                                if game_label.hovered() {
                                    ctx.set_cursor_icon(CursorIcon::PointingHand);
                                }
                                if game_label.clicked() {
                                    debug!("{}", &game.name);
                                    self.game_selected = game;
                                }
                                ui.add_space(5.0);
                            }
                        },
                    )
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // settings page <-> game page toggle?
            let page_toggle = ui.button(self.current_page.as_str());

            if page_toggle.hovered() {
                ctx.set_cursor_icon(CursorIcon::PointingHand);
            }

            if page_toggle.clicked() {
                if self.current_page == "Settings" {
                    self.current_page = "Main Page".to_string();
                    self.settings_page_flag = false;
                } else {
                    self.current_page = "Settings".to_string();
                    self.settings_page_flag = true;
                }
            }
            ui.separator();

            ui.add_space(20.0);
            if self.settings_page_flag {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.label(egui::RichText::new("Settings").size(45.0));
                    ui.add_space(25.0);

                    ui.label(egui::RichText::new("Steam Path for Instlled Games").size(15.0));
                    ui.add_space(5.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_string)
                                .hint_text(&self.config.steam_path)
                                .desired_width(850.0)
                                .interactive(false),
                        );
                        let open_folder = ui.button(egui::RichText::new("Open Finder").size(15.0));
                        if open_folder.hovered() {
                            ctx.set_cursor_icon(CursorIcon::PointingHand);
                        }

                        if open_folder.clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.config.steam_path = path.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.add_space(25.0);
                    ui.label(egui::RichText::new("Steam Cache Path for Game Images").size(15.0));
                    ui.add_space(5.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_string)
                                .hint_text(&self.config.steam_game_cache_path)
                                .desired_width(850.0)
                                .interactive(false),
                        );
                        let open_folder = ui.button(egui::RichText::new("Open Finder").size(15.0));
                        if open_folder.hovered() {
                            ctx.set_cursor_icon(CursorIcon::PointingHand);
                        }

                        if open_folder.clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.config.steam_game_cache_path =
                                    path.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.add_space(25.0);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        let save = ui.button(egui::RichText::new("Save").size(35.0));

                        if save.hovered() {
                            ctx.set_cursor_icon(CursorIcon::PointingHand);
                        }

                        if save.clicked() {
                            self.config.save();
                            self.find_installed_games();
                        }
                    });
                });
            } else {
                ui.add(
                    egui::Image::new(format!("file://{}", self.game_selected.header.clone()))
                        .max_height(ui.available_height() / 1.5)
                        .max_width(ui.available_width()),
                );

                ui.label(egui::RichText::new(&self.game_selected.name).size(25.0));
                ui.add_space(5.0);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    let play_button = ui.button(egui::RichText::new("Play").size(25.0));
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("Game Size: ").size(25.0));
                    ui.label(egui::RichText::new(&self.game_selected.size.to_string()).size(25.0));
                    ui.label(egui::RichText::new("GB").size(25.0));

                    if play_button.hovered() {
                        ctx.set_cursor_icon(CursorIcon::PointingHand);
                    }

                    if play_button.clicked() {
                        let url = format!("steam://run/{}", &self.game_selected.appid);
                        let status = Command::new("cmd")
                            .args(&["/C", "start", &url])
                            .status()
                            .expect("failed to run game");

                        if status.success() {
                            println!(
                                "Playing Game: {}, appid: {}",
                                &self.game_selected.name, &self.game_selected.appid
                            );
                        } else {
                            eprintln!("Failed to launch game.");
                        }
                    }
                });
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
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MyApp::new(config))
        }),
    )
}

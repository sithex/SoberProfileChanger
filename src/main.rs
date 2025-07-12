use eframe::egui;
use egui::{Color32, Vec2, Pos2, Rect, Rounding, Stroke, FontId, Align2, TextureHandle, ColorImage, TextureOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::env;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 360.0])
            .with_resizable(true)
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "Sober - Who's Playing?",
        options,
        Box::new(|cc| Box::new(SoberApp::new(cc))),
    )
}

#[derive(Clone)]
struct Profile {
    name: String,
    cookie_file: String,
    display_name: String,
    emoji: String,
    image: Option<TextureHandle>,
}

struct SoberApp {
    profiles: Vec<Profile>,
    selected_profile: Option<usize>,
    sober_logo: Option<TextureHandle>,
    error_message: Option<String>,
    cookie_directory: PathBuf,
    show_directory_dialog: bool,
    temp_directory_input: String,
}

impl SoberApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        
        // Load Sober logo if available
        let sober_logo = Self::load_image_from_path(ctx, "Sober_logo.png");
        
        // Load saved directory or use default
        let cookie_directory = Self::load_saved_directory();
        
        let mut app = Self {
            profiles: Vec::new(),
            selected_profile: None,
            sober_logo,
            error_message: None,
            cookie_directory: cookie_directory.clone(),
            show_directory_dialog: false,
            temp_directory_input: cookie_directory.to_string_lossy().to_string(),
        };
        
        // Auto-curate profiles from cookie files
        app.load_profiles(ctx);
        
        app
    }
    
    fn load_saved_directory() -> PathBuf {
        let config_path = Self::get_config_file_path();
        
        if let Ok(saved_dir) = fs::read_to_string(&config_path) {
            let saved_dir = saved_dir.trim();
            if !saved_dir.is_empty() {
                let expanded_path = Self::expand_path(saved_dir);
                if expanded_path.exists() {
                    return expanded_path;
                }
            }
        }
        
        // Default directory
        Self::expand_path("~/.var/app/org.vinegarhq.Sober/data/sober/")
    }
    
    fn get_config_file_path() -> PathBuf {
        let mut config_path = dirs::config_dir().unwrap_or_else(|| {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
        config_path.push("sober-cookie-manager");
        fs::create_dir_all(&config_path).ok();
        config_path.push("directory.txt");
        config_path
    }
    
    fn expand_path(path: &str) -> PathBuf {
        if path.starts_with("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                return home_dir.join(&path[2..]);
            }
        }
        PathBuf::from(path)
    }
    
    fn save_directory(&self) {
        let config_path = Self::get_config_file_path();
        if let Err(e) = fs::write(&config_path, self.cookie_directory.to_string_lossy().as_ref()) {
            eprintln!("Failed to save directory preference: {}", e);
        }
    }
    
    fn load_profiles(&mut self, ctx: &egui::Context) {
        self.profiles.clear();
        self.error_message = None;
        
        // Scan cookie directory for cookies_* files
        match fs::read_dir(&self.cookie_directory) {
            Ok(entries) => {
                let mut cookie_files = Vec::new();
                
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            if file_name.starts_with("cookies_") && (file_name.ends_with(".txt") || !file_name.contains('.')) {
                                // Extract profile name from filename
                                let profile_name = if file_name.ends_with(".txt") {
                                    file_name
                                        .strip_prefix("cookies_")
                                        .and_then(|s| s.strip_suffix(".txt"))
                                        .unwrap_or("Unknown")
                                        .to_string()
                                } else {
                                    file_name
                                        .strip_prefix("cookies_")
                                        .unwrap_or("Unknown")
                                        .to_string()
                                };
                                
                                cookie_files.push((profile_name, file_name.to_string()));
                            }
                        }
                    }
                }
                
                // Sort alphabetically
                cookie_files.sort_by(|a, b| a.0.cmp(&b.0));
                
                // Create profiles
                for (i, (profile_name, cookie_file)) in cookie_files.into_iter().enumerate() {
                    let display_name = Self::format_profile_name(&profile_name);
                    let emoji = Self::get_profile_emoji(i);
                    
                    // Try to load profile-specific image from cookie directory
                    let image_path = self.cookie_directory.join(format!("{}.png", profile_name.to_lowercase()));
                    let image = Self::load_image_from_path(ctx, image_path.to_str().unwrap_or(""));
                    
                    let profile = Profile {
                        name: profile_name.clone(),
                        cookie_file,
                        display_name,
                        emoji,
                        image,
                    };
                    
                    self.profiles.push(profile);
                }
                
                if self.profiles.is_empty() {
                    self.error_message = Some(format!(
                        "No cookie files found in {}. Looking for files named 'cookies_*.txt' or 'cookies_*'.",
                        self.cookie_directory.display()
                    ));
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Error scanning directory {}: {}", self.cookie_directory.display(), e));
            }
        }
    }
    
    fn format_profile_name(name: &str) -> String {
        // Convert snake_case or kebab-case to Title Case
        name.replace('_', " ")
            .replace('-', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars.as_str().to_lowercase().chars()).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn get_profile_emoji(index: usize) -> String {
        let emojis = ["🦆", "🐱", "🐶", "🐸", "🐨", "🦊", "🐰", "🐼", "🦁", "🐯"];
        emojis.get(index % emojis.len()).unwrap_or(&"👤").to_string()
    }
    
    fn load_image_from_path(ctx: &egui::Context, path: &str) -> Option<TextureHandle> {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_flat_samples();
                
                let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                Some(ctx.load_texture(path, color_image, TextureOptions::default()))
            }
            Err(_) => {
                println!("Could not load image: {}", path);
                None
            }
        }
    }
    
    fn copy_cookie_file(&mut self, profile_index: usize) {
        if let Some(profile) = self.profiles.get(profile_index) {
            let source_path = self.cookie_directory.join(&profile.cookie_file);
            let target_path = self.cookie_directory.join("cookies");
            
            match fs::copy(&source_path, &target_path) {
                Ok(_) => {
                    println!("Successfully copied {} to {}", source_path.display(), target_path.display());
                    self.error_message = Some(format!("✅ Switched to {} profile", profile.display_name));
                }
                Err(e) => {
                    let error_msg = format!("Failed to copy {}: {}", source_path.display(), e);
                    println!("{}", error_msg);
                    self.error_message = Some(error_msg);
                }
            }
        }
    }
    
    fn apply_directory_change(&mut self, ctx: &egui::Context) {
        let new_path = Self::expand_path(&self.temp_directory_input);
        
        if new_path.exists() && new_path.is_dir() {
            self.cookie_directory = new_path;
            self.save_directory();
            self.load_profiles(ctx);
            self.selected_profile = None;
            self.show_directory_dialog = false;
            self.error_message = Some("✅ Directory changed successfully".to_string());
        } else {
            self.error_message = Some("❌ Directory does not exist or is not a directory".to_string());
        }
    }
    
    fn draw_custom_title_bar(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title_bar")
            .exact_height(40.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(32, 47, 64)).inner_margin(0.0))
            .show(ctx, |ui| {
                let title_bar_rect = ui.max_rect();
                let title_bar_response = ui.interact(title_bar_rect, egui::Id::new("title_bar"), egui::Sense::click());
                
                if title_bar_response.is_pointer_button_down_on() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        
                        // Sober logo
                        if let Some(logo) = &self.sober_logo {
                            ui.add(egui::Image::from_texture(logo).fit_to_exact_size(Vec2::new(24.0, 24.0)));
                        } else {
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 12.0, Color32::WHITE);
                            ui.painter().circle_filled(rect.center(), 8.0, Color32::from_rgb(32, 47, 64));
                        }
                        
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("Sober")
                                .font(FontId::proportional(16.0))
                                .color(Color32::WHITE)
                                .strong()
                        );
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(8.0);
                            
                            // Refresh button
                            let refresh_button_size = Vec2::new(32.0, 28.0);
                            let (refresh_rect, refresh_response) = ui.allocate_exact_size(refresh_button_size, egui::Sense::click());
                            
                            let refresh_bg_color = if refresh_response.hovered() {
                                Color32::from_rgb(70, 120, 180)
                            } else {
                                Color32::TRANSPARENT
                            };
                            
                            ui.painter().rect_filled(refresh_rect, Rounding::same(0.0), refresh_bg_color);
                            ui.painter().text(
                                refresh_rect.center(),
                                Align2::CENTER_CENTER,
                                "↻",
                                FontId::proportional(16.0),
                                Color32::WHITE,
                            );
                            
                            if refresh_response.clicked() {
                                self.load_profiles(ctx);
                            }
                            
                            // Close button
                            let close_button_size = Vec2::new(32.0, 28.0);
                            let (close_rect, close_response) = ui.allocate_exact_size(close_button_size, egui::Sense::click());
                            
                            let close_bg_color = if close_response.hovered() {
                                Color32::from_rgb(196, 43, 28)
                            } else {
                                Color32::TRANSPARENT
                            };
                            
                            ui.painter().rect_filled(close_rect, Rounding::same(0.0), close_bg_color);
                            ui.painter().text(
                                close_rect.center(),
                                Align2::CENTER_CENTER,
                                "✕",
                                FontId::proportional(14.0),
                                Color32::WHITE,
                            );
                            
                            if close_response.clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
                });
            });
    }
    
    fn draw_profile_avatar(
        &self,
        ui: &mut egui::Ui,
        profile: &Profile,
        is_selected: bool,
        size: f32,
    ) -> egui::Response {
        let avatar_size = Vec2::new(size, size);
        let (rect, response) = ui.allocate_exact_size(avatar_size, egui::Sense::click());
        
        let bg_color = if is_selected {
            Color32::from_rgb(70, 120, 180)
        } else if response.hovered() {
            Color32::from_rgb(60, 80, 110)
        } else {
            Color32::from_rgb(45, 62, 80)
        };
        
        let border_color = if is_selected {
            Color32::from_rgb(100, 150, 220)
        } else {
            Color32::from_rgb(70, 90, 120)
        };
        
        // Draw background
        ui.painter().rect_filled(rect, Rounding::same(8.0), bg_color);
        
        // Draw border
        ui.painter().rect_stroke(rect, Rounding::same(8.0), Stroke::new(2.0, border_color));
        
        // Draw image or emoji
        if let Some(texture) = &profile.image {
            let image_rect = Rect::from_center_size(rect.center(), Vec2::new(size * 0.8, size * 0.8));
            ui.painter().image(
                texture.id(),
                image_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                &profile.emoji,
                FontId::proportional(size * 0.4),
                Color32::WHITE,
            );
        }
        
        // Draw name below
        let name_rect = Rect::from_center_size(
            Pos2::new(rect.center().x, rect.bottom() + 12.0),
            Vec2::new(size + 20.0, 20.0),
        );
        
        ui.painter().text(
            name_rect.center(),
            Align2::CENTER_CENTER,
            &profile.display_name,
            FontId::proportional(10.0),
            Color32::LIGHT_GRAY,
        );
        
        response
    }
}

impl eframe::App for SoberApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        visuals.widgets.active.bg_stroke = Stroke::NONE;
        visuals.widgets.open.bg_stroke = Stroke::NONE;
        visuals.panel_fill = Color32::from_rgb(32, 47, 64);
        ctx.set_visuals(visuals);
        
        self.draw_custom_title_bar(ctx, frame);
        
        let bg_color = Color32::from_rgb(32, 47, 64);
        
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(bg_color).inner_margin(0.0))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.add_space(25.0);
                
                // Sober logo
                ui.vertical_centered(|ui| {
                    if let Some(logo) = &self.sober_logo {
                        ui.add(egui::Image::from_texture(logo).fit_to_exact_size(Vec2::new(32.0, 32.0)));
                    } else {
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(32.0, 32.0), egui::Sense::hover());
                        ui.painter().circle_filled(rect.center(), 16.0, Color32::WHITE);
                        ui.painter().circle_filled(rect.center(), 12.0, Color32::from_rgb(32, 47, 64));
                    }
                });
                
                ui.add_space(20.0);
                
                // Title
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Who's playing?")
                            .font(FontId::proportional(24.0))
                            .color(Color32::WHITE)
                    );
                });
                
                ui.add_space(20.0);
                
                // Error message display
                if let Some(error) = &self.error_message.clone() {
                    ui.vertical_centered(|ui| {
                        let color = if error.starts_with("✅") {
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::LIGHT_RED
                        };
                        
                        ui.label(
                            egui::RichText::new(error)
                                .font(FontId::proportional(12.0))
                                .color(color)
                        );
                    });
                    ui.add_space(10.0);
                }
                
                // Profile selection
                if self.profiles.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("No profiles found")
                                .font(FontId::proportional(16.0))
                                .color(Color32::GRAY)
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("Create 'cookies_*.txt' files to auto-generate profiles")
                                .font(FontId::proportional(12.0))
                                .color(Color32::DARK_GRAY)
                        );
                    });
                } else {
                    // Dynamic profile layout
                    ui.vertical_centered(|ui| {
                        let profiles_per_row = 3;
                        let avatar_size = 80.0;
                        let spacing = 20.0;
                        
                        // Clone profiles to avoid borrowing issues
                        let profiles_clone = self.profiles.clone();
                        
                        for chunk in profiles_clone.chunks(profiles_per_row) {
                            ui.horizontal(|ui| {
                                let row_width = chunk.len() as f32 * avatar_size + (chunk.len() - 1) as f32 * spacing;
                                let available_width = ui.available_width();
                                let start_offset = (available_width - row_width) / 2.0;
                                ui.add_space(start_offset);
                                
                                for (i, profile) in chunk.iter().enumerate() {
                                    let global_index = self.profiles.iter().position(|p| p.name == profile.name).unwrap();
                                    let is_selected = self.selected_profile == Some(global_index);
                                    
                                    let response = self.draw_profile_avatar(ui, profile, is_selected, avatar_size);
                                    
                                    if response.clicked() {
                                        if self.selected_profile == Some(global_index) {
                                            self.selected_profile = None;
                                        } else {
                                            self.selected_profile = Some(global_index);
                                            self.copy_cookie_file(global_index);
                                        }
                                    }
                                    
                                    if i < chunk.len() - 1 {
                                        ui.add_space(spacing);
                                    }
                                }
                            });
                            ui.add_space(15.0);
                        }
                    });
                }
                
                ui.add_space(20.0);
                
                // Directory selection button
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Cookie Directory:")
                                .font(FontId::proportional(12.0))
                                .color(Color32::LIGHT_GRAY)
                        );
                        
                        if ui.button("📁 Change Directory").clicked() {
                            self.show_directory_dialog = true;
                            self.temp_directory_input = self.cookie_directory.to_string_lossy().to_string();
                        }
                    });
                    
                    // Show current directory
                    ui.label(
                        egui::RichText::new(format!("📂 {}", self.cookie_directory.display()))
                            .font(FontId::proportional(10.0))
                            .color(Color32::DARK_GRAY)
                    );
                });
                
                // Directory dialog
                if self.show_directory_dialog {
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        ui.group(|ui| {
                            ui.set_min_width(400.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("Enter Cookie Directory Path:")
                                        .font(FontId::proportional(12.0))
                                        .color(Color32::WHITE)
                                );
                                
                                ui.add_space(5.0);
                                
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.temp_directory_input)
                                        .desired_width(380.0)
                                        .hint_text("e.g., ~/.var/app/org.vinegarhq.Sober/data/sober/")
                                );
                                
                                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    self.apply_directory_change(ctx);
                                }
                                
                                ui.add_space(10.0);
                                
                                ui.horizontal(|ui| {
                                    if ui.button("✅ Apply").clicked() {
                                        self.apply_directory_change(ctx);
                                    }
                                    
                                    if ui.button("❌ Cancel").clicked() {
                                        self.show_directory_dialog = false;
                                    }
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("🔄 Reset to Default").clicked() {
                                            self.temp_directory_input = "~/.var/app/org.vinegarhq.Sober/data/sober/".to_string();
                                        }
                                    });
                                });
                            });
                        });
                    });
                }
                
                ui.add_space(15.0);
                
                // Status text
                ui.vertical_centered(|ui| {
                    match self.selected_profile {
                        Some(index) => {
                            if let Some(profile) = self.profiles.get(index) {
                                ui.label(
                                    egui::RichText::new(format!("{} profile active", profile.display_name))
                                        .font(FontId::proportional(14.0))
                                        .color(Color32::LIGHT_BLUE)
                                );
                            }
                        }
                        None => {
                            ui.label(
                                egui::RichText::new("Select a profile to switch cookies")
                                    .font(FontId::proportional(14.0))
                                    .color(Color32::GRAY)
                            );
                        }
                    }
                });
            });
    }
}

// Updated Cargo.toml dependencies:
/*
[dependencies]
eframe = "0.27"
egui = "0.27"
image = "0.24"
dirs = "5.0"
*/

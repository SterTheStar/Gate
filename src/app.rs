use eframe::egui;
use crate::core::disk::{DiskInfo, get_disks};
use crate::core::scanner::{Scanner, FileNode, ScanMessage, ScanProgress};
use crate::ui::{disk_select, tree};
use std::path::PathBuf;
use humansize::{format_size, DECIMAL};

pub struct GateApp {
    disks: Vec<DiskInfo>,
    selected_disk_mount: Option<String>,
    scanner: Option<Scanner>,
    root_node: Option<FileNode>,
    is_scanning: bool,
    scan_progress: ScanProgress,
    tree_view: tree::TreeView,
    error_message: Option<String>,
    zoom_factor: f32,
    show_disk_modal: bool,
    show_settings_modal: bool,
    ui_scale: f32,
    dark_mode: bool,
}

impl GateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        Self {
            disks: get_disks(),
            selected_disk_mount: None,
            scanner: None,
            root_node: None,
            is_scanning: false,
            scan_progress: ScanProgress::default(),
            tree_view: tree::TreeView::new(),
            error_message: None,
            zoom_factor: 1.0,
            show_disk_modal: false,
            show_settings_modal: false,
            ui_scale: 1.35,
            dark_mode: true,
        }
    }

    fn start_scan(&mut self) {
        if let Some(mount) = &self.selected_disk_mount {
            self.is_scanning = true;
            self.root_node = None;
            self.error_message = None;
            self.scan_progress = ScanProgress::default();
            self.scanner = Some(Scanner::new(PathBuf::from(mount)));
        }
    }
    
    fn delete_item(&mut self, path: PathBuf) {
        match trash::delete(&path) {
            Ok(_) => {
                self.start_scan();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to delete: {}", e));
            }
        }
    }

    fn go_home(&mut self) {
        self.selected_disk_mount = None;
        self.root_node = None;
        self.is_scanning = false;
        self.scanner = None;
        self.disks = get_disks();
    }

    fn settings_modal(&mut self, ctx: &egui::Context) {
        let mut is_open = self.show_settings_modal;
        let mut close_requested = false;
        
        egui::Window::new("Settings")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(egui::RichText::new("Appearance").strong());
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        if ui.selectable_label(self.dark_mode, "Dark").clicked() {
                            self.dark_mode = true;
                            ctx.set_visuals(egui::Visuals::dark());
                        }
                        if ui.selectable_label(!self.dark_mode, "Light").clicked() {
                            self.dark_mode = false;
                            ctx.set_visuals(egui::Visuals::light());
                        }
                    });
                    
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("UI Scale:");
                        ui.add(egui::Slider::new(&mut self.ui_scale, 0.8..=2.0).step_by(0.05));
                    });
                });
                
                ui.add_space(12.0);
                
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(egui::RichText::new("Analysis").strong());
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Default Zoom:");
                        ui.add(egui::Slider::new(&mut self.zoom_factor, 0.5..=2.0));
                    });
                });
                
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }
                });
            });
            
        self.show_settings_modal = is_open && !close_requested;
    }
}

impl eframe::App for GateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(self.ui_scale);
        
        // Handle scanner messages
        let mut scan_finished = false;
        if let Some(scanner) = &self.scanner {
            while let Some(msg) = scanner.try_recv() {
                match msg {
                    ScanMessage::Progress(p) => {
                        self.scan_progress = p;
                    }
                    ScanMessage::Completed(node) => {
                        self.root_node = Some(node);
                        self.is_scanning = false;
                        scan_finished = true;
                    }
                    ScanMessage::Error(e) => {
                        self.error_message = Some(e);
                        self.is_scanning = false;
                        scan_finished = true;
                    }
                }
            }
            ctx.request_repaint();
        }
        if scan_finished {
            self.scanner = None;
        }

        if self.selected_disk_mount.is_none() {
            // HOME PAGE
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25);
                    
                    ui.label(egui::RichText::new("TheGate").size(64.0).strong().color(egui::Color32::from_rgb(240, 240, 245)));
                    ui.label(egui::RichText::new("Advanced Storage Intelligence").size(18.0).weak());
                    
                    ui.add_space(60.0);
                    
                    ui.horizontal(|ui| {
                        let btn_width = 160.0;
                        let btn_height = 48.0;
                        let total_width = (btn_width * 2.0) + 20.0;
                        ui.add_space((ui.available_width() - total_width) / 2.0);
                        
                        let btn_disk = egui::Button::new(egui::RichText::new("🖴 Select Disk").size(16.0))
                            .min_size(egui::vec2(btn_width, btn_height))
                            .rounding(egui::Rounding::same(10.0));
                        
                        if ui.add(btn_disk).clicked() {
                            self.show_disk_modal = true;
                        }
                        
                        ui.add_space(20.0);
                        
                        let btn_settings = egui::Button::new(egui::RichText::new("⚙ Settings").size(16.0))
                            .min_size(egui::vec2(btn_width, btn_height))
                            .rounding(egui::Rounding::same(10.0));
                            
                        if ui.add(btn_settings).clicked() {
                            self.show_settings_modal = true;
                        }
                    });
                });
            });
            
            // Modals
            if self.show_disk_modal {
                if disk_select::disk_modal_ui(ctx, &self.disks, &mut self.selected_disk_mount, &mut self.show_disk_modal) {
                    self.start_scan();
                }
            }
            
            if self.show_settings_modal {
                self.settings_modal(ctx);
            }
            
        } else {
            // APP FOOTER
            egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    if self.is_scanning {
                        ui.spinner();
                        ui.label(format!("Scanning: {} files", self.scan_progress.files_scanned));
                    } else if let Some(root) = &self.root_node {
                        ui.label(format!("Total Files: {}", self.scan_progress.files_scanned));
                        ui.separator();
                        ui.label(format!("Total Size: {}", format_size(root.size, DECIMAL)));
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        ui.add(egui::Slider::new(&mut self.zoom_factor, 0.5..=2.0).text("Tree Zoom").show_value(true));
                        ui.separator();
                        if let Some(path) = &self.tree_view.selected_path {
                            ui.label(egui::RichText::new(path.to_string_lossy()).small().weak());
                        }
                    });
                });
            });

            // MAIN APP VIEW
            egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    
                    // Back button
                    let back_btn = egui::Button::new(egui::RichText::new("⏴ Back").size(13.0))
                        .min_size(egui::vec2(0.0, 26.0));
                    if ui.add(back_btn).clicked() {
                        self.go_home();
                    }
                    
                    ui.separator();
                    
                    if let Some(mount) = &self.selected_disk_mount {
                        if let Some(disk) = self.disks.iter().find(|d| &d.mount_point == mount) {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&disk.name).strong());
                                ui.label(egui::RichText::new(format!("({})", disk.mount_point)).weak().small());
                            });
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        
                        // Settings button
                        let settings_btn = egui::Button::new(egui::RichText::new("⚙").size(14.0))
                            .min_size(egui::vec2(30.0, 26.0));
                        if ui.add(settings_btn).clicked() {
                            self.show_settings_modal = true;
                        }

                        ui.add_space(4.0);

                        // Rescan button
                        let rescan_btn = egui::Button::new(egui::RichText::new("🔄 Rescan").size(13.0))
                            .min_size(egui::vec2(0.0, 26.0));
                        if ui.add(rescan_btn).clicked() {
                            self.start_scan();
                        }
                        
                        ui.separator();
                        egui::Frame::none()
                            .fill(egui::Color32::from_black_alpha(100))
                            .rounding(egui::Rounding::same(6.0))
                            .stroke(ui.visuals().widgets.inactive.bg_stroke)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("🔍").size(14.0));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.tree_view.search_query)
                                            .hint_text("Search...")
                                            .frame(false)
                                            .desired_width(150.0)
                                    );
                                });
                            });
                    });
                });
                ui.add_space(6.0);
            });

            if self.show_settings_modal {
                self.settings_modal(ctx);
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                if self.is_scanning {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(60.0);
                            ui.heading(egui::RichText::new("Analyzing Storage").size(36.0).strong());
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Please wait while we map your disk usage...").weak());
                            ui.add_space(40.0);
                            
                            ui.group(|ui| {
                                ui.set_width(450.0);
                                ui.add_space(15.0);
                                
                                let elapsed = self.scan_progress.start_time.elapsed().as_secs_f32().max(0.001);
                                let files_per_sec = self.scan_progress.files_scanned as f32 / elapsed;
                                let bytes_per_sec = self.scan_progress.bytes_scanned as f32 / elapsed;

                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new("Files Scanned").strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(10.0);
                                        ui.label(egui::RichText::new(format!("{} ({:.0} f/s)", self.scan_progress.files_scanned, files_per_sec)).monospace());
                                    });
                                });
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new("Total Size Found").strong());
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(10.0);
                                        ui.label(egui::RichText::new(format!("{} ({}/s)", format_size(self.scan_progress.bytes_scanned, DECIMAL), format_size(bytes_per_sec as u64, DECIMAL))).monospace());
                                    });
                                });
                                ui.add_space(15.0);
                            });

                            ui.add_space(40.0);
                            ui.label(egui::RichText::new("Current Activity").weak().small());
                            ui.add(egui::Label::new(egui::RichText::new(&self.scan_progress.current_path).small().weak()).truncate());
                            
                            ui.add_space(30.0);
                            
                            // Calculate real progress based on used space
                            let used_space = self.disks.iter()
                                .find(|d| Some(&d.mount_point) == self.selected_disk_mount.as_ref())
                                .map(|d| d.total_space - d.available_space)
                                .unwrap_or(1);
                            
                            let progress_ratio = (self.scan_progress.bytes_scanned as f32 / used_space as f32).min(0.99);
                            
                            let pb = egui::ProgressBar::new(progress_ratio)
                                .animate(true)
                                .rounding(egui::Rounding::same(6.0))
                                .text(format!("{:.1}%", progress_ratio * 100.0));
                            
                            ui.add_sized([450.0, 24.0], pb);
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Mapping directory structure and file sizes...").small().weak());
                        });
                    });
                } else if let Some(root) = &mut self.root_node {
                    let mut tree_action = None;
                    
                    egui::ScrollArea::vertical()
                        .id_source("main_tree_scroll")
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.set_max_width(ui.available_width());
                            if let Some(action) = self.tree_view.ui_zoomed(ui, root, self.zoom_factor) {
                                tree_action = Some(action);
                            }
                        });

                    if let Some(action) = tree_action {
                        match action {
                            tree::TreeAction::Delete(path) => self.delete_item(path),
                            tree::TreeAction::Open(path) => {
                                let _ = open::that(path);
                            }
                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("No data. Select a disk to start.");
                    });
                }
            });
        }

        let mut close_error = false;
        if let Some(err) = &self.error_message {
            egui::Window::new("Error").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, err);
                if ui.button("Close").clicked() {
                    close_error = true;
                }
            });
        }
        if close_error {
            self.error_message = None;
        }
    }
}


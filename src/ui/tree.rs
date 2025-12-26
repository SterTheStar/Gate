use egui::{Ui, Color32, Rounding, Vec2, Align, Layout, FontId};
use crate::core::scanner::FileNode;
use humansize::{format_size, DECIMAL};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TreeAction {
    Delete(PathBuf),
    Open(PathBuf),
}

pub struct TreeView {
    pub selected_path: Option<PathBuf>,
    pub search_query: String,
}

impl TreeView {
    pub fn new() -> Self {
        Self { 
            selected_path: None,
            search_query: String::new(),
        }
    }

    pub fn ui_zoomed(&mut self, ui: &mut Ui, node: &mut FileNode, zoom: f32) -> Option<TreeAction> {
        let total_size = node.size;
        
        // Apply zoom to the UI style for this scope
        let mut style = ui.style_mut().clone();
        style.text_styles.iter_mut().for_each(|(_, font_id)| {
            font_id.size *= zoom;
        });
        style.spacing.item_spacing *= zoom;
        style.spacing.indent *= zoom;
        style.spacing.interact_size *= zoom;
        
        ui.scope(|ui| {
            ui.set_style(style);
            self.recursive_tree(ui, node, total_size, zoom)
        }).inner
    }

    fn recursive_tree(&mut self, ui: &mut Ui, node: &mut FileNode, parent_size: u64, zoom: f32) -> Option<TreeAction> {
        // Filter by search query
        if !self.search_query.is_empty() && !node.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
            if node.is_dir {
                let mut has_matching_child = false;
                for child in &node.children {
                    if self.matches_search(child) {
                        has_matching_child = true;
                        break;
                    }
                }
                if !has_matching_child {
                    return None;
                }
            } else {
                return None;
            }
        }

        let size_text = format_size(node.size, DECIMAL);
        let percentage = if parent_size > 0 {
            (node.size as f32 / parent_size as f32) * 100.0
        } else {
            0.0
        };

        let mut action = None;
        let is_selected = self.selected_path.as_ref() == Some(&node.path);
        
        if node.is_dir {
            let id = ui.make_persistent_id(&node.path);
            let header = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
            
            header.show_header(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    self.draw_percentage_bar(ui, percentage, zoom);
                    ui.add_space(5.0 * zoom);
                    
                    let response = ui.selectable_label(is_selected, &node.name);
                    self.handle_response(ui, &response, node, &mut action);
                    
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(10.0 * zoom);
                        ui.label(egui::RichText::new(size_text).monospace().weak());
                    });
                });
            }).body(|ui| {
                for child in &mut node.children {
                    if let Some(act) = self.recursive_tree(ui, child, node.size, zoom) {
                        action = Some(act);
                    }
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.add_space(20.0 * zoom); // Indent files to align with folder icons
                self.draw_percentage_bar(ui, percentage, zoom);
                ui.add_space(5.0 * zoom);
                
                let response = ui.selectable_label(is_selected, &node.name);
                self.handle_response(ui, &response, node, &mut action);
                
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(10.0 * zoom);
                    ui.label(egui::RichText::new(size_text).monospace().weak());
                });
            });
        }
        
        action
    }

    fn draw_percentage_bar(&self, ui: &mut Ui, percentage: f32, zoom: f32) {
        let (rect, _) = ui.allocate_at_least(Vec2::new(45.0 * zoom, 16.0 * zoom), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, Rounding::same(4.0 * zoom), Color32::from_gray(40));
        let fill_width = (rect.width() * (percentage / 100.0)).max(1.0);
        painter.rect_filled(
            egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, rect.height())),
            Rounding::same(4.0 * zoom),
            self.get_percentage_color(percentage)
        );
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{:.1}%", percentage),
            FontId::monospace(9.0 * zoom),
            Color32::WHITE
        );
    }

    fn matches_search(&self, node: &FileNode) -> bool {
        if node.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
            return true;
        }
        for child in &node.children {
            if self.matches_search(child) {
                return true;
            }
        }
        false
    }

    fn handle_response(&mut self, _ui: &mut Ui, response: &egui::Response, node: &FileNode, action: &mut Option<TreeAction>) {
        response.context_menu(|ui| {
            if ui.button("Open").clicked() {
                *action = Some(TreeAction::Open(node.path.clone()));
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                *action = Some(TreeAction::Delete(node.path.clone()));
                ui.close_menu();
            }
        });
        if response.clicked() {
            self.selected_path = Some(node.path.clone());
        }
    }

    fn get_percentage_color(&self, p: f32) -> Color32 {
        if p > 50.0 {
            Color32::from_rgb(255, 100, 100)
        } else if p > 20.0 {
            Color32::from_rgb(255, 200, 100)
        } else {
            Color32::from_rgb(100, 200, 100)
        }
    }
}

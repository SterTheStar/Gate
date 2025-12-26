use egui::{Ui, Vec2, Color32, Rounding, Stroke, Align, Layout, RichText};
use crate::core::disk::DiskInfo;
use humansize::{format_size, DECIMAL};

pub fn disk_modal_ui(ctx: &egui::Context, disks: &[DiskInfo], selected_disk: &mut Option<String>, is_open: &mut bool) -> bool {
    let mut selection_changed = false;
    let mut open = *is_open;
    let mut selection_made = false;
    
    egui::Window::new("Select Storage Device")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([480.0, 420.0])
        .show(ctx, |ui| {
            ui.add_space(10.0);
            
            egui::ScrollArea::vertical()
                .id_source("disk_modal_scroll")
                .max_height(360.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for disk in disks {
                            if disk_row_minimal(ui, disk) {
                                *selected_disk = Some(disk.mount_point.clone());
                                selection_changed = true;
                                selection_made = true;
                            }
                            ui.add_space(4.0);
                        }
                    });
                });
            
            ui.add_space(10.0);
        });
    
    *is_open = open && !selection_made;
    selection_changed
}

fn disk_row_minimal(ui: &mut Ui, disk: &DiskInfo) -> bool {
    let width = ui.available_width();
    let height = 48.0;
    
    let (rect, response) = ui.allocate_at_least(Vec2::new(width, height), egui::Sense::click());
    let is_hovered = response.hovered();
    
    // Background
    let bg_color = if is_hovered {
        Color32::from_gray(35)
    } else {
        Color32::from_gray(25)
    };
    
    ui.painter().rect_filled(rect, Rounding::same(6.0), bg_color);
    
    // Accent bar on hover
    if is_hovered {
        let accent_rect = egui::Rect::from_min_size(
            rect.min,
            Vec2::new(3.0, height)
        );
        ui.painter().rect_filled(accent_rect, Rounding::same(1.5), Color32::from_rgb(100, 150, 255));
    }

    // Content
    ui.allocate_ui_at_rect(rect.shrink2(Vec2::new(16.0, 0.0)), |ui| {
        ui.horizontal_centered(|ui| {
            // Left: Name and Path in one line
            let name = if disk.name.is_empty() { "Local Disk" } else { &disk.name };
            ui.label(RichText::new(name).strong().size(14.0).color(Color32::from_rgb(230, 235, 245)));
            ui.add_space(4.0);
            ui.label(RichText::new(format!("({})", disk.mount_point)).small().color(Color32::from_rgb(110, 115, 125)));
            
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Right: Storage Info in one line
                ui.label(
                    RichText::new(format!(" / {}", format_size(disk.total_space, DECIMAL)))
                        .small()
                        .color(Color32::from_rgb(100, 105, 115))
                );
                ui.label(
                    RichText::new(format_size(disk.available_space, DECIMAL))
                        .strong()
                        .size(14.0)
                        .color(Color32::from_rgb(180, 185, 195))
                );
                ui.label(RichText::new("Free:").small().color(Color32::from_rgb(100, 105, 115)));
            });
        });
    });

    if is_hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    response.clicked()
}

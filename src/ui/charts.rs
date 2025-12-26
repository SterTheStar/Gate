use egui::{Ui, Color32, Stroke, Vec2, Shape, Rounding};
use crate::core::scanner::FileNode;
use humansize::{format_size, DECIMAL};
use std::f32::consts::TAU;

struct ChartEntry {
    name: String,
    size: u64,
    percentage: f32,
    color: Color32,
}

pub fn show_charts(ui: &mut Ui, node: &FileNode) {
    ui.vertical(|ui| {
        ui.add_space(10.0);
        ui.heading(egui::RichText::new("Disk Usage Distribution").strong().size(20.0));
        ui.add_space(20.0);

        let total_size = node.size as f64;
        if total_size == 0.0 || node.children.is_empty() {
            ui.label("No data to display");
            return;
        }

        ui.horizontal_top(|ui| {
            // Left side: Pie Chart
            let chart_size = 300.0;
            let (rect, response) = ui.allocate_at_least(Vec2::splat(chart_size), egui::Sense::hover());
            
            let mut children: Vec<_> = node.children.iter().collect();
            children.sort_by(|a, b| b.size.cmp(&a.size));

            let top_n = 8;
            let mut entries: Vec<ChartEntry> = Vec::new();
            let colors = [
                Color32::from_rgb(100, 149, 237), // Cornflower Blue
                Color32::from_rgb(255, 127, 80),  // Coral
                Color32::from_rgb(60, 179, 113),  // Medium Sea Green
                Color32::from_rgb(255, 215, 0),    // Gold
                Color32::from_rgb(138, 43, 226),  // Blue Violet
                Color32::from_rgb(255, 105, 180), // Hot Pink
                Color32::from_rgb(0, 206, 209),   // Dark Turquoise
                Color32::from_rgb(210, 105, 30),  // Chocolate
            ];

            let mut accounted_size = 0u64;
            for (i, child) in children.iter().take(top_n).enumerate() {
                entries.push(ChartEntry {
                    name: child.name.clone(),
                    size: child.size,
                    percentage: child.size as f32 / total_size as f32,
                    color: colors[i % colors.len()],
                });
                accounted_size += child.size;
            }

            let other_size = node.size.saturating_sub(accounted_size);
            if other_size > 0 {
                entries.push(ChartEntry {
                    name: "Others".to_string(),
                    size: other_size,
                    percentage: other_size as f32 / total_size as f32,
                    color: Color32::from_gray(100),
                });
            }

            // Draw Pie Chart
            let painter = ui.painter_at(rect);
            let center = rect.center();
            let radius = rect.width() / 2.5;
            let mut start_angle = -TAU / 4.0;

            for (i, entry) in entries.iter().enumerate() {
                let sweep = entry.percentage * TAU;
                if sweep > 0.001 {
                    let mut points = vec![center];
                    let n_points = (sweep * 50.0).max(10.0) as i32;
                    for step in 0..=n_points {
                        let angle = start_angle + sweep * (step as f32 / n_points as f32);
                        points.push(center + Vec2::new(angle.cos(), angle.sin()) * radius);
                    }
                    
                    let mut fill_color = entry.color;
                    if response.hover_pos().map_or(false, |pos| {
                        let to_pos = pos - center;
                        let dist = to_pos.length();
                        let angle = to_pos.y.atan2(to_pos.x);
                        let mut normalized_angle = angle;
                        while normalized_angle < start_angle { normalized_angle += TAU; }
                        while normalized_angle > start_angle + sweep { normalized_angle -= TAU; }
                        dist <= radius && normalized_angle >= start_angle && normalized_angle <= start_angle + sweep
                    }) {
                        fill_color = Color32::from_rgb(
                            (fill_color.r() as u16 + 30).min(255) as u8,
                            (fill_color.g() as u16 + 30).min(255) as u8,
                            (fill_color.b() as u16 + 30).min(255) as u8,
                        );
                        
                        egui::show_tooltip_at_pointer(ui.ctx(), ui.layer_id(), response.id.with(i), |ui: &mut egui::Ui| {
                            ui.label(format!("{}: {} ({:.1}%)", entry.name, format_size(entry.size, DECIMAL), entry.percentage * 100.0));
                        });
                    }

                    painter.add(Shape::convex_polygon(points, fill_color, Stroke::new(1.0, Color32::from_gray(30))));
                    start_angle += sweep;
                }
            }

            ui.add_space(20.0);

            // Right side: Legend
            ui.vertical(|ui| {
                ui.add_space(20.0);
                for entry in &entries {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_at_least(Vec2::splat(12.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, Rounding::same(2.0), entry.color);
                        ui.label(format!("{}: {:.1}%", entry.name, entry.percentage * 100.0));
                    });
                    ui.add_space(4.0);
                }
            });
        });
    });
}

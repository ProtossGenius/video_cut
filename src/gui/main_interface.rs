use egui::{Ui, Color32, Rect, pos2, vec2, Stroke};
use crate::project::ProjectState;

pub fn show(ui: &mut Ui, state: &mut ProjectState) {
    egui::Panel::bottom("status_bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Status Bar | Mode: NORMAL");
        });
    });

    egui::Panel::left("media_pool").resizable(true).min_size(200.0).show(ui, |ui| {
        ui.heading("Media Pool");
        ui.label("导入的文件列表");
    });

    egui::Panel::bottom("timeline").resizable(true).min_size(300.0).show(ui, |ui| {
        ui.heading("Timeline");
        
        egui::ScrollArea::both().id_salt("timeline_scroll").show(ui, |ui| {
            let track_height = 80.0;
            let timeline_width = 3000.0; // 假定一个虚拟宽度
            
            let track_count = state.timeline.tracks.len().max(1); // 至少留点空间
            let (rect, _response) = ui.allocate_exact_size(
                vec2(timeline_width, track_height * track_count as f32), 
                egui::Sense::click_and_drag()
            );
            
            let painter = ui.painter_at(rect);
            
            for (i, track) in state.timeline.tracks.iter().enumerate() {
                let y_offset = rect.min.y + (i as f32) * track_height;
                let track_rect = Rect::from_min_size(pos2(rect.min.x, y_offset), vec2(timeline_width, track_height));
                
                // 背景
                let bg_color = if i % 2 == 0 { Color32::from_gray(35) } else { Color32::from_gray(45) };
                painter.rect_filled(track_rect, 0.0, bg_color);
                
                // 分割线
                painter.line_segment([track_rect.min, pos2(track_rect.max.x, track_rect.min.y)], Stroke::new(1.0, Color32::from_gray(70)));
                
                // 轨道名称
                painter.text(
                    pos2(track_rect.min.x + 10.0, track_rect.min.y + 5.0),
                    egui::Align2::LEFT_TOP,
                    &track.name,
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
                
                // 绘制剪辑块
                for clip in &track.clips {
                    // 比例：1 pixel = 10000 us (10 ms) -> 100 pixels / s
                    let us_per_pixel = 10000.0;
                    
                    let x_start = track_rect.min.x + (clip.timeline_start.0 as f32 / us_per_pixel);
                    let width = (clip.duration().0 as f32 / us_per_pixel).max(2.0);
                    
                    let clip_rect = Rect::from_min_size(
                        pos2(x_start, track_rect.min.y + 25.0),
                        vec2(width, track_height - 30.0)
                    );
                    
                    // 剪辑块背景和边框
                    painter.rect_filled(clip_rect, 4.0, Color32::from_rgb(60, 120, 180));
                    painter.rect_stroke(clip_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(100, 180, 255)), egui::StrokeKind::Inside);
                    
                    // 剪辑名称
                    // 增加裁剪区，防止文字画出 Clip 外部
                    let mut clip_text_rect = clip_rect;
                    clip_text_rect.min.x += 4.0;
                    clip_text_rect.min.y += 4.0;
                    
                    painter.with_clip_rect(clip_rect).text(
                        clip_text_rect.min,
                        egui::Align2::LEFT_TOP,
                        &clip.name,
                        egui::FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
            }
        });
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.heading("Viewport");
        ui.label("视频预览区域");
    });
}

use eframe::egui;
use waveview_model::viewer::{ViewerCommand, ViewerState};
use waveview_model::DisplayedItemId;

pub fn render(
    ui: &mut egui::Ui,
    viewer: &ViewerState,
    canvas_rect: egui::Rect,
    interaction_rect: egui::Rect,
    visible_rows: std::ops::Range<usize>,
    row_height: f32,
    commands: &mut Vec<ViewerCommand>,
) -> (egui::Response, Option<DisplayedItemId>) {
    let time_viewport = viewer.viewport();
    let view_start = time_viewport.start() as f32;
    let view_end = time_viewport.end() as f32;
    let pixels_per_tick = canvas_rect.width() / time_viewport.span() as f32;
    let signals = viewer
        .displayed_items()
        .iter()
        .filter_map(|item| {
            item.signal_id()
                .and_then(|id| viewer.waveform().signal(id))
                .map(|signal| (signal, item))
        })
        .collect::<Vec<_>>();

    ui.painter().rect_filled(
        canvas_rect,
        egui::CornerRadius::ZERO,
        ui.visuals().extreme_bg_color,
    );

    ui.skip_ahead_auto_ids(visible_rows.start);
    let response = ui.interact(
        interaction_rect,
        egui::Id::new("wave_canvas_interaction"),
        egui::Sense::click_and_drag(),
    );
    let hover_pos = response.hover_pos();
    let hover_fraction =
        hover_pos.map(|position| (position.x - canvas_rect.left()) / canvas_rect.width().max(1.0));

    ui.vertical(|ui| {
        for (signal, item) in signals
            .iter()
            .take(visible_rows.end)
            .skip(visible_rows.start)
        {
            let mut wave = crate::wave::Wave::new(
                item.alias().unwrap_or_else(|| signal.name()),
                view_start..=view_end,
                signal.signal(),
                item.value_format(),
                item.color(),
            );
            wave.height = row_height;
            wave.ui(ui);
        }
    });

    if ui.rect_contains_pointer(egui::Rect::EVERYTHING) {
        let zoom = ui.input(|input| input.zoom_delta());
        if zoom != 1.0 {
            if let Some(fraction) = hover_fraction {
                commands.push(ViewerCommand::ZoomTime {
                    anchor: time_viewport.start() + f64::from(fraction) * time_viewport.span(),
                    factor: f64::from(zoom),
                });
            }
        }

        let scroll_x = ui.input(|input| input.smooth_scroll_delta.x);
        if scroll_x != 0.0 {
            commands.push(ViewerCommand::PanTime(f64::from(
                -scroll_x / pixels_per_tick,
            )));
        }
    }

    let measurement_color = egui::Color32::from_rgb(0xd2, 0x99, 0x1d);
    let mut active_measurement_start = viewer.cursor_state().measurement_start();
    if let Some(position) = response.hover_pos() {
        let time = view_start + (position.x - canvas_rect.left()) / pixels_per_tick;
        let rounded_time = time.round();
        let hover_time = rounded_time.max(0.0) as u64;

        if response.drag_started() {
            active_measurement_start = Some(hover_time);
            commands.push(ViewerCommand::BeginMeasurement(hover_time));
        } else if response.dragged() {
            commands.push(ViewerCommand::UpdateMeasurement(hover_time));
        } else if response.clicked() {
            commands.push(ViewerCommand::SetCursor(hover_time));
        }

        let hover_x = canvas_rect.left() + (rounded_time - view_start) * pixels_per_tick;
        ui.painter().line_segment(
            [
                egui::pos2(hover_x, interaction_rect.top()),
                egui::pos2(hover_x, interaction_rect.bottom()),
            ],
            egui::Stroke::new(2.0, measurement_color),
        );

        if let Some(start_time) = active_measurement_start {
            let start_x = canvas_rect.left() + (start_time as f32 - view_start) * pixels_per_tick;
            ui.painter().line_segment(
                [
                    egui::pos2(start_x, interaction_rect.top()),
                    egui::pos2(start_x, interaction_rect.bottom()),
                ],
                egui::Stroke::new(2.0, measurement_color),
            );
            ui.painter().rect_filled(
                egui::Rect::from_two_pos(
                    egui::pos2(start_x, interaction_rect.top()),
                    egui::pos2(hover_x, interaction_rect.bottom()),
                ),
                egui::CornerRadius::ZERO,
                measurement_color.linear_multiply(0.1),
            );
        }
    }

    for position in viewer.marks().values() {
        let mark_x = canvas_rect.left() + (position.time() as f32 - view_start) * pixels_per_tick;
        if canvas_rect.x_range().contains(mark_x) {
            ui.painter().line_segment(
                [
                    egui::pos2(mark_x, interaction_rect.top()),
                    egui::pos2(mark_x, interaction_rect.bottom()),
                ],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_GREEN.linear_multiply(0.65)),
            );
        }
    }

    if let Some(cursor_time) = viewer.cursor() {
        let cursor_x = canvas_rect.left() + (cursor_time as f32 - view_start) * pixels_per_tick;
        if canvas_rect.x_range().contains(cursor_x) {
            ui.painter().line_segment(
                [
                    egui::pos2(cursor_x, interaction_rect.top()),
                    egui::pos2(cursor_x, interaction_rect.bottom()),
                ],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
            );
        }
    }

    if response.drag_stopped() {
        commands.push(ViewerCommand::EndMeasurement);
    }

    let context_target = response
        .secondary_clicked()
        .then(|| {
            let position = response.interact_pointer_pos()?;
            let local_row = ((position.y - canvas_rect.top())
                / (row_height + ui.spacing().item_spacing.y))
                .floor()
                .max(0.0) as usize;
            signals
                .get(visible_rows.start + local_row)
                .map(|(_, item)| item.id())
        })
        .flatten();

    (response, context_target)
}

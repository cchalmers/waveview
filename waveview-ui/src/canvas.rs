use eframe::egui;
use waveview_model::viewer::{DisplayedItem, ViewerCommand, ViewerState, VisualSelectionKind};
use waveview_model::waveform::WaveformSignal;
use waveview_model::DisplayedItemId;

type DisplayedSignal<'a> = (&'a WaveformSignal, &'a DisplayedItem);

struct MarkGeometry {
    canvas_rect: egui::Rect,
    visible_rows: std::ops::Range<usize>,
    row_height: f32,
    view_start: f32,
    pixels_per_tick: f32,
}

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
    let mark_geometry = MarkGeometry {
        canvas_rect,
        visible_rows: visible_rows.clone(),
        row_height,
        view_start,
        pixels_per_tick,
    };

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

    paint_visual_selection(ui, viewer, &signals, &mark_geometry);

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
    let show_mouse_time_cursor = response.dragged()
        || ui.ctx().data(|data| {
            data.get_temp::<bool>(egui::Id::new("waveview_mouse_time_cursor_visible"))
                .unwrap_or(true)
        });
    if let Some(position) = response.hover_pos() {
        let time = view_start + (position.x - canvas_rect.left()) / pixels_per_tick;
        let rounded_time = time.round();
        let hover_time = rounded_time.max(0.0) as u64;
        let local_row = ((position.y - canvas_rect.top())
            / (row_height + ui.spacing().item_spacing.y))
            .floor()
            .max(0.0) as usize;
        let hovered_item = signals
            .get(visible_rows.start + local_row)
            .map(|(_, item)| item.id());

        if response.drag_started() {
            if let Some(id) = hovered_item {
                commands.push(ViewerCommand::SetFocusedItem(id));
            }
            commands.push(ViewerCommand::BeginMeasurement(hover_time));
        } else if response.dragged() {
            commands.push(ViewerCommand::UpdateMeasurement(hover_time));
        } else if response.clicked() {
            commands.push(ViewerCommand::ClearVisualSelection);
            if let Some(id) = hovered_item {
                commands.push(ViewerCommand::SetFocusedItem(id));
            }
            commands.push(ViewerCommand::SetCursor(hover_time));
        }

        if show_mouse_time_cursor {
            let hover_x = canvas_rect.left() + (rounded_time - view_start) * pixels_per_tick;
            ui.painter().line_segment(
                [
                    egui::pos2(hover_x, interaction_rect.top()),
                    egui::pos2(hover_x, interaction_rect.bottom()),
                ],
                egui::Stroke::new(2.0, measurement_color),
            );
        }
    }

    if viewer.marks_visible() {
        paint_marks(ui, viewer, interaction_rect, &mark_geometry);
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

    if viewer.marks_visible() {
        paint_mark_badges(ui, viewer, &signals, &mark_geometry);
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

fn paint_visual_selection(
    ui: &egui::Ui,
    viewer: &ViewerState,
    signals: &[DisplayedSignal<'_>],
    geometry: &MarkGeometry,
) {
    let Some(selection) = viewer.visual_selection() else {
        return;
    };
    let Some(anchor_row) = signals
        .iter()
        .position(|(_, item)| item.id() == selection.anchor().item())
    else {
        return;
    };
    let active_row = signals
        .iter()
        .position(|(_, item)| item.id() == selection.active().item())
        .unwrap_or(anchor_row);
    let rows = match selection.kind() {
        VisualSelectionKind::Time => anchor_row..=anchor_row,
        VisualSelectionKind::Lines | VisualSelectionKind::Block => {
            anchor_row.min(active_row)..=anchor_row.max(active_row)
        }
    };
    let row_span = geometry.row_height + ui.spacing().item_spacing.y;
    let time_range = selection.time_range();
    let (left, right) = match selection.kind() {
        VisualSelectionKind::Lines => (geometry.canvas_rect.left(), geometry.canvas_rect.right()),
        VisualSelectionKind::Time | VisualSelectionKind::Block => {
            let left = geometry.canvas_rect.left()
                + (*time_range.start() as f32 - geometry.view_start) * geometry.pixels_per_tick;
            let right = geometry.canvas_rect.left()
                + (*time_range.end() as f32 - geometry.view_start) * geometry.pixels_per_tick;
            (left, right)
        }
    };
    for row in rows.filter(|row| geometry.visible_rows.contains(row)) {
        let local_row = row - geometry.visible_rows.start;
        let top = geometry.canvas_rect.top() + local_row as f32 * row_span;
        let rect = egui::Rect::from_min_max(
            egui::pos2(left, top),
            egui::pos2(right.max(left + 2.0), top + geometry.row_height),
        )
        .intersect(geometry.canvas_rect);
        let selection_color = egui::Color32::from_rgb(0xd2, 0x99, 0x1d);
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::ZERO,
            selection_color.linear_multiply(0.12),
        );
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::ZERO,
            egui::Stroke::new(1.0, selection_color),
            egui::StrokeKind::Inside,
        );
    }
}

fn paint_marks(
    ui: &egui::Ui,
    viewer: &ViewerState,
    interaction_rect: egui::Rect,
    geometry: &MarkGeometry,
) {
    for position in viewer.marks().values() {
        let mark_x = geometry.canvas_rect.left()
            + (position.time() as f32 - geometry.view_start) * geometry.pixels_per_tick;
        if geometry.canvas_rect.x_range().contains(mark_x) {
            ui.painter().line_segment(
                [
                    egui::pos2(mark_x, interaction_rect.top()),
                    egui::pos2(mark_x, interaction_rect.bottom()),
                ],
                egui::Stroke::new(1.0, egui::Color32::LIGHT_GREEN.linear_multiply(0.65)),
            );
        }
    }
}

fn paint_mark_badges(
    ui: &egui::Ui,
    viewer: &ViewerState,
    signals: &[DisplayedSignal<'_>],
    geometry: &MarkGeometry,
) {
    for (index, (&name, position)) in viewer.marks().iter().enumerate() {
        let mark_x = geometry.canvas_rect.left()
            + (position.time() as f32 - geometry.view_start) * geometry.pixels_per_tick;
        let Some(row) = signals
            .iter()
            .position(|(_, item)| item.id() == position.item())
            .filter(|row| geometry.visible_rows.contains(row))
        else {
            continue;
        };
        if geometry.canvas_rect.x_range().contains(mark_x) {
            let total = viewer
                .marks()
                .values()
                .filter(|candidate| {
                    candidate.time() == position.time() && candidate.item() == position.item()
                })
                .count();
            let ordinal = viewer
                .marks()
                .values()
                .take(index)
                .filter(|candidate| {
                    candidate.time() == position.time() && candidate.item() == position.item()
                })
                .count();
            let offset = (ordinal as f32 - (total.saturating_sub(1)) as f32 * 0.5) * 15.0;
            let local_row = row - geometry.visible_rows.start;
            let row_span = geometry.row_height + ui.spacing().item_spacing.y;
            let center = egui::pos2(
                (mark_x + offset).clamp(
                    geometry.canvas_rect.left() + 7.0,
                    geometry.canvas_rect.right() - 7.0,
                ),
                geometry.canvas_rect.top()
                    + local_row as f32 * row_span
                    + geometry.row_height * 0.5,
            );
            paint_mark_badge(ui, center, name);
        }
    }
}

fn paint_mark_badge(ui: &egui::Ui, center: egui::Pos2, name: char) {
    let badge = egui::Rect::from_center_size(center, egui::Vec2::splat(14.0));
    ui.painter().rect_filled(
        badge,
        2.0,
        ui.visuals().extreme_bg_color.linear_multiply(0.92),
    );
    ui.painter().rect_stroke(
        badge,
        2.0,
        egui::Stroke::new(1.0, egui::Color32::LIGHT_GREEN),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::monospace(10.0),
        egui::Color32::LIGHT_GREEN,
    );
}

use eframe::egui;
use waveview_model::viewer::ViewerCommand;

pub fn height() -> f32 {
    timeline::DEFAULT_HEIGHT
}

pub fn render(
    ui: &mut egui::Ui,
    capture_end: u64,
    view_start: u64,
    view_end: u64,
    cursor: Option<u64>,
    measurement_start: Option<u64>,
    commands: &mut Vec<ViewerCommand>,
) {
    let full = timeline::TimeRange::new(0, capture_end.max(1));
    let visible = timeline::TimeRange::new(view_start, view_end).clamp_to(full);
    let selection = measurement_start
        .zip(cursor)
        .map(|(start, end)| timeline::TimeRange::new(start, end));
    let response = timeline::Timeline::new(full, visible, &|time| time.to_string())
        .cursor(cursor)
        .selection(selection)
        .show(ui);

    commands.extend(response.actions.into_iter().map(|action| match action {
        timeline::Action::Pan(delta) => ViewerCommand::PanTime(delta),
        timeline::Action::Zoom { anchor, factor } => ViewerCommand::ZoomTime {
            anchor: anchor as f64,
            factor,
        },
        timeline::Action::Fit => ViewerCommand::FitTime,
        timeline::Action::SetCursor(time) => ViewerCommand::SetCursor(time),
        timeline::Action::BeginSelection(time) => ViewerCommand::BeginMeasurement(time),
        timeline::Action::UpdateSelection(time) => ViewerCommand::UpdateMeasurement(time),
        timeline::Action::EndSelection => ViewerCommand::EndMeasurement,
    }));
}

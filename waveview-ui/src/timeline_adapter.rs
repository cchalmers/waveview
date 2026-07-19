use eframe::egui;
use waveview_model::ui_types::TimelinePresentation;
use waveview_model::viewer::ViewerCommand;

pub fn height() -> f32 {
    timeline::DEFAULT_HEIGHT
}

pub fn render(
    ui: &mut egui::Ui,
    presentation: TimelinePresentation,
    marks: &[u64],
    commands: &mut Vec<ViewerCommand>,
) {
    let full = timeline::TimeRange::new(0, presentation.capture_end.max(1));
    let visible =
        timeline::TimeRange::new(presentation.view_start, presentation.view_end).clamp_to(full);
    let selection = presentation
        .measurement_start
        .zip(presentation.cursor)
        .map(|(start, end)| timeline::TimeRange::new(start, end));
    let response = timeline::Timeline::new(full, visible, &|time| time.to_string())
        .cursor(presentation.cursor)
        .selection(selection)
        .marks(marks)
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

use eframe::egui;
use waveview_model::ui_types::{MarkPresentation, TimelinePresentation};
use waveview_model::viewer::ViewerCommand;

pub fn height() -> f32 {
    timeline::DEFAULT_HEIGHT
}

pub fn render(
    ui: &mut egui::Ui,
    presentation: TimelinePresentation,
    marks: &[MarkPresentation],
    commands: &mut Vec<ViewerCommand>,
) {
    let full = timeline::TimeRange::new(0, presentation.capture_end.max(1));
    let visible =
        timeline::TimeRange::new(presentation.view_start, presentation.view_end).clamp_to(full);
    let selection = presentation
        .measurement_start
        .zip(presentation.cursor)
        .map(|(start, end)| timeline::TimeRange::new(start, end));
    let markers = marks
        .iter()
        .map(|mark| timeline::Marker {
            time: mark.time,
            label: mark.name,
        })
        .collect::<Vec<_>>();
    let response = timeline::Timeline::new(full, visible, &|time| time.to_string())
        .cursor(presentation.cursor)
        .selection(selection)
        .marks(&markers)
        .show(ui);

    for action in response.actions {
        match action {
            timeline::Action::Pan(delta) => commands.push(ViewerCommand::PanTime(delta)),
            timeline::Action::Zoom { anchor, factor } => {
                commands.push(ViewerCommand::ZoomTime {
                    anchor: anchor as f64,
                    factor,
                });
            }
            timeline::Action::Fit => commands.push(ViewerCommand::FitTime),
            timeline::Action::SetCursor(time) => {
                commands.push(ViewerCommand::ClearVisualSelection);
                commands.push(ViewerCommand::SetCursor(time));
            }
            timeline::Action::BeginSelection(time) => {
                commands.push(ViewerCommand::BeginMeasurement(time));
            }
            timeline::Action::UpdateSelection(time) => {
                commands.push(ViewerCommand::UpdateMeasurement(time));
            }
            timeline::Action::EndSelection => commands.push(ViewerCommand::EndMeasurement),
        }
    }
}

use eframe::egui;
use timeline::{Action, Marker, TimeRange, Timeline};

struct Demo {
    visible: TimeRange,
    cursor: Option<u64>,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            visible: TimeRange::new(0, 1_000),
            cursor: Some(250),
        }
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(root_ui, |ui| {
            let full = TimeRange::new(0, 10_000);
            let response = Timeline::new(full, self.visible, &|time| time.to_string())
                .cursor(self.cursor)
                .marks(&[
                    Marker {
                        time: 100,
                        label: 'a',
                    },
                    Marker {
                        time: 500,
                        label: 'b',
                    },
                    Marker {
                        time: 900,
                        label: 'c',
                    },
                ])
                .show(ui);
            for action in response.actions {
                match action {
                    Action::SetCursor(time) => self.cursor = Some(time),
                    Action::Fit => self.visible = full,
                    _ => {}
                }
            }
            ui.label("Click to set the cursor; double-click to fit.");
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "timeline demo",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::<Demo>::default())),
    )
}

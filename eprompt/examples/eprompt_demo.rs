use eframe::egui;
use eprompt::{Action, OutputEntry, OutputKind, Prompt};

struct Demo {
    input: String,
    output: Vec<OutputEntry>,
    history: Vec<String>,
    history_index: Option<usize>,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: vec![
                OutputEntry::new("eprompt", OutputKind::Success),
                OutputEntry::new(
                    "A reusable egui REPL surface. Try 'help', 'clear', or any expression.",
                    OutputKind::Muted,
                ),
            ],
            history: Vec::new(),
            history_index: None,
        }
    }
}

impl eframe::App for Demo {
    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(root_ui, |ui| {
            let response = Prompt::new(&mut self.input)
                .prompt("waveview ❯")
                .output(&self.output)
                .status("ready")
                .show(ui);
            for action in response.actions {
                match action {
                    Action::Submit(command) if !command.trim().is_empty() => {
                        self.output.push(OutputEntry::new(
                            format!("waveview ❯ {command}"),
                            OutputKind::Command,
                        ));
                        self.history.push(command.clone());
                        self.history_index = None;
                        match command.trim() {
                            "clear" => self.output.clear(),
                            "help" => self.output.push(OutputEntry::new(
                                "help   show this message\nclear  clear the transcript\n↑/↓    browse history",
                                OutputKind::Normal,
                            )),
                            command if command.starts_with("error ") => self.output.push(
                                OutputEntry::new(&command[6..], OutputKind::Error),
                            ),
                            command => self.output.push(OutputEntry::new(
                                format!("=> {command}"),
                                OutputKind::Normal,
                            )),
                        }
                        self.input.clear();
                    }
                    Action::HistoryPrevious => {
                        if !self.history.is_empty() {
                            let index = self
                                .history_index
                                .unwrap_or(self.history.len())
                                .saturating_sub(1);
                            self.history_index = Some(index);
                            self.input.clone_from(&self.history[index]);
                        }
                    }
                    Action::HistoryNext => {
                        if let Some(index) = self.history_index {
                            let next = index + 1;
                            if next < self.history.len() {
                                self.history_index = Some(next);
                                self.input.clone_from(&self.history[next]);
                            } else {
                                self.history_index = None;
                                self.input.clear();
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "eprompt demo",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::<Demo>::default())),
    )
}

#![warn(clippy::all, rust_2018_idioms)]

use egui::{Color32, Frame, Key, Margin, RichText, Stroke, TextEdit, Ui};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputKind {
    #[default]
    Normal,
    Command,
    Success,
    Error,
    Muted,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutputEntry {
    pub text: String,
    pub kind: OutputKind,
}

impl OutputEntry {
    pub fn new(text: impl Into<String>, kind: OutputKind) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Changed(String),
    Submit(String),
    Complete,
    HistoryPrevious,
    HistoryNext,
    Cancel,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PromptResponse {
    pub actions: Vec<Action>,
    pub has_focus: bool,
}

/// A backend-independent prompt surface. Input, history, completion, and output remain owned by
/// the caller; this widget only renders them and reports user intent.
pub struct Prompt<'a> {
    input: &'a mut String,
    prompt: &'a str,
    output: &'a [OutputEntry],
    completions: &'a [String],
    status: Option<&'a str>,
    desired_rows: usize,
    fill_available: bool,
}

impl<'a> Prompt<'a> {
    pub fn new(input: &'a mut String) -> Self {
        Self {
            input,
            prompt: ">",
            output: &[],
            completions: &[],
            status: None,
            desired_rows: 8,
            fill_available: true,
        }
    }

    pub fn prompt(mut self, prompt: &'a str) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn output(mut self, output: &'a [OutputEntry]) -> Self {
        self.output = output;
        self
    }

    pub fn completions(mut self, completions: &'a [String]) -> Self {
        self.completions = completions;
        self
    }

    pub fn status(mut self, status: &'a str) -> Self {
        self.status = Some(status);
        self
    }

    pub fn desired_rows(mut self, desired_rows: usize) -> Self {
        self.desired_rows = desired_rows;
        self
    }

    pub fn fill_available(mut self, fill_available: bool) -> Self {
        self.fill_available = fill_available;
        self
    }

    pub fn show(self, ui: &mut Ui) -> PromptResponse {
        let Self {
            input,
            prompt,
            output,
            completions,
            status,
            desired_rows,
            fill_available,
        } = self;
        let mut actions = Vec::new();

        let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
        let height = if fill_available {
            ui.available_height()
        } else {
            row_height * (desired_rows as f32 + 3.0)
        };
        let spacing = ui.spacing().item_spacing.y;
        let status_height = status.map_or(0.0, |_| row_height + spacing);
        let prompt_height = ui.spacing().interact_size.y + 10.0 + status_height;
        let completion_height = if completions.is_empty() {
            0.0
        } else {
            row_height + spacing
        };
        let minimum_height = prompt_height + completion_height;
        let (full_rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height.max(minimum_height)),
            egui::Sense::hover(),
        );
        let prompt_top = full_rect.bottom() - prompt_height;
        let completion_top = prompt_top - completion_height;
        let prompt_rect = egui::Rect::from_min_max(
            egui::pos2(full_rect.left(), prompt_top),
            full_rect.right_bottom(),
        );
        let completion_rect = egui::Rect::from_min_max(
            egui::pos2(full_rect.left(), completion_top),
            egui::pos2(full_rect.right(), prompt_top),
        );
        let transcript_rect = egui::Rect::from_min_max(
            full_rect.left_top(),
            egui::pos2(full_rect.right(), completion_top),
        );

        let before = input.clone();
        let response = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .max_rect(prompt_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_width(prompt_rect.width());
                    Frame::new()
                        .fill(ui.visuals().extreme_bg_color)
                        .stroke(Stroke::new(
                            1.0,
                            ui.visuals().widgets.inactive.bg_stroke.color,
                        ))
                        .corner_radius(4.0)
                        .inner_margin(Margin::symmetric(8, 5))
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                if let Some(status) = status {
                                    ui.horizontal(|ui| {
                                        ui.small(RichText::new(status).monospace().weak());
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.small(
                                                    RichText::new(
                                                        "Enter submit · ↑↓ history · Tab complete",
                                                    )
                                                    .weak(),
                                                );
                                            },
                                        );
                                    });
                                }
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(prompt)
                                            .monospace()
                                            .strong()
                                            .color(Color32::LIGHT_BLUE),
                                    );
                                    ui.add(
                                        TextEdit::singleline(input)
                                            .font(egui::TextStyle::Monospace)
                                            .frame(Frame::NONE)
                                            .desired_width(f32::INFINITY),
                                    )
                                })
                                .inner
                            })
                            .inner
                        })
                        .inner
                },
            )
            .inner;

        if !completions.is_empty() {
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(completion_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for completion in completions {
                            ui.weak(RichText::new(completion).monospace());
                        }
                    });
                },
            );
        }

        if transcript_rect.height() > 0.0 {
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(transcript_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("eprompt_transcript")
                        .stick_to_bottom(true)
                        .max_height(transcript_rect.height())
                        .min_scrolled_height(transcript_rect.height())
                        .show(ui, |ui| {
                            ui.set_min_size(transcript_rect.size());
                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                                for entry in output.iter().rev() {
                                    render_output_entry(ui, entry);
                                }
                            });
                        });
                },
            );
        }

        if *input != before {
            actions.push(Action::Changed(input.clone()));
        }
        let submitted = (response.has_focus() || response.lost_focus())
            && ui.input(|input_state| input_state.key_pressed(Key::Enter));
        if submitted {
            actions.push(Action::Submit(input.clone()));
            response.request_focus();
        }
        if response.has_focus() {
            ui.input(|input_state| {
                if input_state.key_pressed(Key::Tab) {
                    actions.push(Action::Complete);
                }
                if input_state.key_pressed(Key::ArrowUp) {
                    actions.push(Action::HistoryPrevious);
                }
                if input_state.key_pressed(Key::ArrowDown) {
                    actions.push(Action::HistoryNext);
                }
                if input_state.key_pressed(Key::Escape) {
                    actions.push(Action::Cancel);
                }
            });
        }

        PromptResponse {
            actions,
            has_focus: response.has_focus() || submitted,
        }
    }
}

pub fn show_panel(title: &str, ui: &mut Ui, prompt: Prompt<'_>) -> PromptResponse {
    ui.heading(title);
    ui.separator();
    prompt.show(ui)
}

/// Paint one transcript entry without owning scroll or prompt state.
pub fn render_output_entry(ui: &mut Ui, entry: &OutputEntry) {
    ui.label(
        RichText::new(&entry.text)
            .monospace()
            .color(output_color(ui, entry.kind)),
    );
}

/// Paint the label beside a caller-owned text editor.
pub fn render_prompt_label(ui: &mut Ui, prompt: &str) {
    ui.label(
        RichText::new(prompt)
            .monospace()
            .strong()
            .color(Color32::LIGHT_BLUE),
    );
}

fn output_color(ui: &Ui, kind: OutputKind) -> Color32 {
    match kind {
        OutputKind::Normal => ui.visuals().text_color(),
        OutputKind::Command => Color32::LIGHT_BLUE,
        OutputKind::Success => Color32::LIGHT_GREEN,
        OutputKind::Error => ui.visuals().error_fg_color,
        OutputKind::Muted => ui.visuals().weak_text_color(),
    }
}

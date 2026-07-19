use eframe::egui;
use waveview_model::ui_types::MenuAction;

pub fn render_file(ui: &mut egui::Ui, native: bool) -> Option<MenuAction> {
    let mut action = None;
    if ui.button("Open File…").clicked() {
        action = Some(MenuAction::OpenFile);
    }
    if ui.button("Open URL…").clicked() {
        action = Some(MenuAction::OpenUrl);
    }
    if ui.button("Connect live…").clicked() {
        action = Some(MenuAction::ConnectLive);
    }
    if ui.button("Reset").clicked() {
        action = Some(MenuAction::Reset);
    }
    if native && ui.button("Quit").clicked() {
        action = Some(MenuAction::Quit);
    }
    close_after_action(ui, action)
}

pub fn render_edit(ui: &mut egui::Ui, has_focused_item: bool) -> Option<MenuAction> {
    let mut action = None;
    if ui.button("Undo display change").clicked() {
        action = Some(MenuAction::UndoDisplayChange);
    }
    if ui.button("Redo display change").clicked() {
        action = Some(MenuAction::RedoDisplayChange);
    }
    if has_focused_item && ui.button("Remove selected signal").clicked() {
        action = Some(MenuAction::RemoveFocusedItem);
    }
    close_after_action(ui, action)
}

pub fn render_view(
    ui: &mut egui::Ui,
    signal_browser_open: bool,
    info_open: bool,
    samples_open: bool,
    row_height: &mut f32,
) -> Option<MenuAction> {
    let mut action = None;
    let mut signals_selected = signal_browser_open;
    if ui
        .checkbox(&mut signals_selected, "Signal browser")
        .clicked()
    {
        action = Some(MenuAction::ToggleSignalBrowser);
    }
    ui.separator();
    if ui.button("Fit time").clicked() {
        action = Some(MenuAction::FitTime);
    }
    ui.separator();
    ui.add(egui::Slider::new(row_height, 25.0..=128.0).text("height"));

    if !info_open && ui.button("Show info").clicked() {
        action = Some(MenuAction::ShowInfo);
    } else if info_open && ui.button("Hide info").clicked() {
        action = Some(MenuAction::HideInspector);
    }
    if !samples_open && ui.button("Show samples").clicked() {
        action = Some(MenuAction::ShowSamples);
    } else if samples_open && ui.button("Hide samples").clicked() {
        action = Some(MenuAction::HideInspector);
    }
    close_after_action(ui, action)
}

pub fn render_help(ui: &mut egui::Ui) -> Option<MenuAction> {
    let action = ui
        .button("Keyboard shortcuts")
        .clicked()
        .then_some(MenuAction::ShowKeyHelp);
    close_after_action(ui, action)
}

fn close_after_action(ui: &mut egui::Ui, action: Option<MenuAction>) -> Option<MenuAction> {
    if action.is_some() {
        ui.close();
    }
    action
}

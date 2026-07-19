use std::cell::RefCell;
use std::rc::Rc;

use molt::{ContextID, Interp, MoltResult, Value};
use waveview_model::ui_types::{PromptOutput, PromptOutputKind};
use waveview_model::viewer::{DisplayColor, ValueFormat, ViewerCommand};

const MAX_OUTPUT_ENTRIES: usize = 2_000;

pub struct PromptRuntime {
    interp: Interp,
    puts_output: Rc<RefCell<Vec<PromptOutput>>>,
    viewer_commands: Rc<RefCell<Vec<ViewerCommand>>>,
    input: String,
    output: Vec<PromptOutput>,
    open: bool,
    focus_requested: bool,
}

impl Default for PromptRuntime {
    fn default() -> Self {
        let puts_output = Rc::new(RefCell::new(Vec::new()));
        let mut interp = Interp::new();
        interp.add_command("help", command_help);
        let puts_context = interp.save_context(puts_output.clone());
        interp.add_context_command("puts", capture_puts, puts_context);
        let viewer_commands = Rc::new(RefCell::new(Vec::new()));
        let viewer_context = interp.save_context(viewer_commands.clone());
        interp.add_context_command("zoom", command_zoom, viewer_context);
        interp.add_context_command("cursor", command_cursor, viewer_context);
        interp.add_context_command("signal", command_signal, viewer_context);
        interp.add_context_command("search", command_search, viewer_context);
        interp.add_context_command("undo", command_undo, viewer_context);
        interp.add_context_command("redo", command_redo, viewer_context);

        Self {
            interp,
            puts_output,
            viewer_commands,
            input: String::new(),
            output: Vec::new(),
            open: false,
            focus_requested: false,
        }
    }
}

fn command_help(_interp: &mut Interp, _id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 1, 1, "")?;
    molt::molt_ok!(
        "Waveview commands:\n\
         zoom fit\n\
         cursor set <ticks> | cursor clear\n\
         signal focus next|previous ?count?\n\
         signal format binary|hex|unsigned|signed|ascii\n\
         signal alias <name> | signal unalias\n\
         signal color default|red|orange|yellow|green|cyan|blue|purple|gray\n\
         search <regex>\n\
         undo | redo\n\
         Standard Tcl commands are also available."
    )
}

impl PromptRuntime {
    pub fn open(&mut self) {
        self.open = true;
        self.focus_requested = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.focus_requested = false;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn take_focus_request(&mut self) -> bool {
        std::mem::take(&mut self.focus_requested)
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut String {
        &mut self.input
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn output(&self) -> &[PromptOutput] {
        &self.output
    }

    pub fn submit(&mut self) -> Vec<ViewerCommand> {
        let script = self.input.trim().to_owned();
        if script.is_empty() {
            return Vec::new();
        }

        self.viewer_commands.borrow_mut().clear();
        self.output
            .push(PromptOutput::new(script.clone(), PromptOutputKind::Command));
        let result = self.interp.eval(&script);
        self.flush_puts();
        match result {
            Ok(value) if !value.as_str().is_empty() => self.output.push(PromptOutput::new(
                value.to_string(),
                PromptOutputKind::Result,
            )),
            Ok(_) => {}
            Err(exception) => self.output.push(PromptOutput::new(
                exception.value().to_string(),
                PromptOutputKind::Error,
            )),
        }
        self.input.clear();
        self.trim_output();
        self.focus_requested = true;
        self.viewer_commands.borrow_mut().drain(..).collect()
    }

    fn flush_puts(&mut self) {
        self.output.extend(self.puts_output.borrow_mut().drain(..));
    }

    fn trim_output(&mut self) {
        let excess = self.output.len().saturating_sub(MAX_OUTPUT_ENTRIES);
        if excess != 0 {
            self.output.drain(..excess);
        }
    }
}

fn command_zoom(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 2, 2, "fit")?;
    match argv[1].as_str() {
        "fit" => push_viewer_command(interp, id, ViewerCommand::FitTime),
        subcommand => molt::molt_err!("unknown zoom subcommand \"{}\": expected fit", subcommand),
    }
}

fn command_cursor(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 2, 3, "set time|clear")?;
    match argv[1].as_str() {
        "set" => {
            molt::check_args(2, argv, 3, 3, "time")?;
            let time = argv[2]
                .as_str()
                .parse::<u64>()
                .map_err(|_| molt::Exception::molt_err("cursor time must be an integer".into()))?;
            let commands: &mut Rc<RefCell<Vec<ViewerCommand>>> = interp.context(id);
            commands.borrow_mut().extend([
                ViewerCommand::SetCursor(time),
                ViewerCommand::RevealTime(time),
            ]);
            molt::molt_ok!()
        }
        "clear" => {
            molt::check_args(2, argv, 2, 2, "")?;
            push_viewer_command(interp, id, ViewerCommand::ClearCursor)
        }
        subcommand => molt::molt_err!(
            "unknown cursor subcommand \"{}\": expected set or clear",
            subcommand
        ),
    }
}

fn command_signal(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 2, 4, "focus|format|alias|unalias|color ...")?;
    match argv[1].as_str() {
        "focus" => command_signal_focus(interp, id, argv),
        "format" => {
            molt::check_args(2, argv, 3, 3, "binary|hex|unsigned|signed|ascii")?;
            let Some(format) = ValueFormat::from_name(argv[2].as_str()) else {
                return molt::molt_err!(
                    "unknown signal format \"{}\": expected binary, hex, unsigned, signed, or ascii",
                    argv[2]
                );
            };
            push_viewer_command(interp, id, ViewerCommand::SetFocusedValueFormat(format))
        }
        "alias" => {
            molt::check_args(2, argv, 3, 3, "name")?;
            push_viewer_command(
                interp,
                id,
                ViewerCommand::SetFocusedAlias(Some(argv[2].as_str().to_owned())),
            )
        }
        "unalias" => {
            molt::check_args(2, argv, 2, 2, "")?;
            push_viewer_command(interp, id, ViewerCommand::SetFocusedAlias(None))
        }
        "color" => {
            molt::check_args(
                2,
                argv,
                3,
                3,
                "default|red|orange|yellow|green|cyan|blue|purple|gray",
            )?;
            let Some(color) = DisplayColor::from_name(argv[2].as_str()) else {
                return molt::molt_err!(
                    "unknown signal color \"{}\": expected default, red, orange, yellow, green, cyan, blue, purple, or gray",
                    argv[2]
                );
            };
            push_viewer_command(interp, id, ViewerCommand::SetFocusedColor(color))
        }
        subcommand => molt::molt_err!(
            "unknown signal subcommand \"{}\": expected focus, format, alias, unalias, or color",
            subcommand
        ),
    }
}

fn command_signal_focus(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(2, argv, 3, 4, "next|previous ?count?")?;
    let direction = match argv[2].as_str() {
        "next" => 1_isize,
        "previous" | "prev" => -1_isize,
        direction => {
            return molt::molt_err!(
                "unknown focus direction \"{}\": expected next or previous",
                direction
            );
        }
    };
    let count = if let Some(value) = argv.get(3) {
        value
            .as_str()
            .parse::<isize>()
            .ok()
            .filter(|count| *count > 0)
            .ok_or_else(|| molt::Exception::molt_err("count must be a positive integer".into()))?
    } else {
        1
    };
    push_viewer_command(
        interp,
        id,
        ViewerCommand::FocusDisplayedRelative(direction.saturating_mul(count)),
    )
}

fn command_search(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 2, 2, "pattern")?;
    push_viewer_command(interp, id, ViewerCommand::SetSearch(argv[1].to_string()))
}

fn command_undo(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 1, 1, "")?;
    push_viewer_command(interp, id, ViewerCommand::UndoDisplayChange)
}

fn command_redo(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 1, 1, "")?;
    push_viewer_command(interp, id, ViewerCommand::RedoDisplayChange)
}

fn push_viewer_command(interp: &mut Interp, id: ContextID, command: ViewerCommand) -> MoltResult {
    let commands: &mut Rc<RefCell<Vec<ViewerCommand>>> = interp.context(id);
    commands.borrow_mut().push(command);
    molt::molt_ok!()
}

fn capture_puts(interp: &mut Interp, id: ContextID, argv: &[Value]) -> MoltResult {
    molt::check_args(1, argv, 2, 2, "string")?;
    let output: &mut Rc<RefCell<Vec<PromptOutput>>> = interp.context(id);
    output.borrow_mut().push(PromptOutput::new(
        argv[1].to_string(),
        PromptOutputKind::Normal,
    ));
    molt::molt_ok!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreter_persists_values_and_captures_puts_and_errors() {
        let mut prompt = PromptRuntime::default();
        prompt.set_input("set answer 42".to_owned());
        assert!(prompt.submit().is_empty());
        prompt.set_input("puts $answer".to_owned());
        assert!(prompt.submit().is_empty());
        prompt.set_input("error broken".to_owned());
        assert!(prompt.submit().is_empty());

        assert!(prompt.output().iter().any(|entry| {
            entry.kind == PromptOutputKind::Command && entry.text == "set answer 42"
        }));
        assert!(prompt.output().iter().any(|entry| entry.text == "42"));
        assert!(prompt
            .output()
            .iter()
            .any(|entry| entry.kind == PromptOutputKind::Error && entry.text == "broken"));
    }

    #[test]
    fn viewer_commands_are_queued_without_borrowing_viewer_state() {
        let mut prompt = PromptRuntime::default();
        prompt.set_input(
            "zoom fit; cursor set 42; signal focus previous 2; signal format signed; signal alias {program counter}; signal color cyan; search {clock.*}; undo"
                .to_owned(),
        );

        assert_eq!(
            prompt.submit(),
            vec![
                ViewerCommand::FitTime,
                ViewerCommand::SetCursor(42),
                ViewerCommand::RevealTime(42),
                ViewerCommand::FocusDisplayedRelative(-2),
                ViewerCommand::SetFocusedValueFormat(ValueFormat::Signed),
                ViewerCommand::SetFocusedAlias(Some("program counter".to_owned())),
                ViewerCommand::SetFocusedColor(DisplayColor::Cyan),
                ViewerCommand::SetSearch("clock.*".to_owned()),
                ViewerCommand::UndoDisplayChange,
            ]
        );
    }

    #[test]
    fn help_lists_the_waveview_command_vocabulary() {
        let mut prompt = PromptRuntime::default();
        prompt.set_input("help".to_owned());
        assert!(prompt.submit().is_empty());
        assert!(prompt
            .output()
            .iter()
            .any(|entry| entry.text.contains("signal alias <name>")));
    }
}

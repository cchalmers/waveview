use std::cell::RefCell;
use std::rc::Rc;

use molt::{ContextID, Interp, MoltResult, Value};
use waveview_model::ui_types::{PromptOutput, PromptOutputKind};

const MAX_OUTPUT_ENTRIES: usize = 2_000;

pub struct PromptRuntime {
    interp: Interp,
    puts_output: Rc<RefCell<Vec<PromptOutput>>>,
    input: String,
    output: Vec<PromptOutput>,
    open: bool,
    focus_requested: bool,
}

impl Default for PromptRuntime {
    fn default() -> Self {
        let puts_output = Rc::new(RefCell::new(Vec::new()));
        let mut interp = Interp::new();
        let puts_context = interp.save_context(puts_output.clone());
        interp.add_context_command("puts", capture_puts, puts_context);

        Self {
            interp,
            puts_output,
            input: String::new(),
            output: vec![PromptOutput::new(
                "Molt command console",
                PromptOutputKind::Muted,
            )],
            open: false,
            focus_requested: false,
        }
    }
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

    pub fn submit(&mut self) {
        let script = self.input.trim().to_owned();
        if script.is_empty() {
            return;
        }

        self.output.push(PromptOutput::new(
            format!(": {script}"),
            PromptOutputKind::Command,
        ));
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
        prompt.submit();
        prompt.set_input("puts $answer".to_owned());
        prompt.submit();
        prompt.set_input("error broken".to_owned());
        prompt.submit();

        assert!(prompt.output().iter().any(|entry| entry.text == "42"));
        assert!(prompt
            .output()
            .iter()
            .any(|entry| entry.kind == PromptOutputKind::Error && entry.text == "broken"));
    }
}

use crate::vcd;
use eframe::egui;

const MAX_BUFFER_BYTES: usize = 512 * 1024 * 1024;
type ParsedVcd = (Vec<(vcd::ScopedVar, vcd::Signal)>, u64);

pub struct LiveVcd {
    pub open: bool,
    url: String,
    status: String,
    bytes: Vec<u8>,
    sender: Option<ewebsock::WsSender>,
    receiver: Option<ewebsock::WsReceiver>,
}

impl Default for LiveVcd {
    fn default() -> Self {
        Self {
            open: false,
            url: "ws://127.0.0.1:9123/ws".to_owned(),
            status: "disconnected".to_owned(),
            bytes: Vec::new(),
            sender: None,
            receiver: None,
        }
    }
}

impl LiveVcd {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        if !self.open {
            return None;
        }
        let mut requested_url = None;
        let mut is_open = self.open;
        egui::Window::new("Live VCD")
            .open(&mut is_open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("WebSocket URL");
                ui.text_edit_singleline(&mut self.url);
                ui.horizontal(|ui| {
                    if self.receiver.is_none() {
                        if ui.button("Connect").clicked() {
                            requested_url = Some(self.url.clone());
                        }
                    } else if ui.button("Disconnect").clicked() {
                        self.disconnect("disconnected");
                    }
                    ui.label(&self.status);
                });
                ui.small(format!("buffered: {} KiB", self.bytes.len() / 1024));
            });
        self.open = is_open;
        requested_url
    }

    pub fn connect(&mut self, url: String, ctx: &egui::Context) {
        self.url = url;
        let repaint = ctx.clone();
        match ewebsock::connect_with_wakeup(
            self.url.clone(),
            ewebsock::Options::default(),
            move || repaint.request_repaint(),
        ) {
            Ok((sender, receiver)) => {
                self.sender = Some(sender);
                self.receiver = Some(receiver);
                self.status = "connecting".to_owned();
            }
            Err(error) => self.status = format!("connection failed: {error}"),
        }
    }

    pub fn poll(&mut self) -> Option<Result<ParsedVcd, String>> {
        let mut newest_parse = None;
        while let Some(event) = self
            .receiver
            .as_ref()
            .and_then(ewebsock::WsReceiver::try_recv)
        {
            match event {
                ewebsock::WsEvent::Opened => self.status = "connected".to_owned(),
                ewebsock::WsEvent::Closed => self.disconnect("closed by server"),
                ewebsock::WsEvent::Error(error) => {
                    self.disconnect(&format!("error: {error}"));
                    return Some(Err(format!("live VCD connection failed: {error}")));
                }
                ewebsock::WsEvent::Message(ewebsock::WsMessage::Text(message)) => {
                    if message.contains(r#""type":"reset""#) {
                        self.bytes.clear();
                        self.status = "receiving snapshot".to_owned();
                    } else if message.contains(r#""type":"resync""#) {
                        self.disconnect("server requested reconnect");
                        return Some(Err(
                            "live VCD client fell behind; reconnect to resync".to_owned()
                        ));
                    }
                }
                ewebsock::WsEvent::Message(ewebsock::WsMessage::Binary(chunk)) => {
                    if self.bytes.len().saturating_add(chunk.len()) > MAX_BUFFER_BYTES {
                        self.disconnect("capture exceeded 512 MiB limit");
                        return Some(Err("live VCD exceeded the 512 MiB safety limit".to_owned()));
                    }
                    self.bytes.extend_from_slice(&chunk);
                    let mut cursor = std::io::Cursor::new(&self.bytes);
                    if let Ok(parsed) = vcd::read_clocked_vcd(&mut cursor) {
                        self.status = "live".to_owned();
                        newest_parse = Some(Ok(parsed));
                    }
                }
                ewebsock::WsEvent::Message(_) => {}
            }
        }
        newest_parse
    }

    fn disconnect(&mut self, status: &str) {
        self.sender = None;
        self.receiver = None;
        self.status = status.to_owned();
    }
}

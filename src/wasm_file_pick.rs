pub struct OpenFileElement {
    input: web_sys::HtmlInputElement,
    // onchange: Option<wasm_bindgen::closure::Closure<>>,
    // files: Rc<RefCell<Option<FileList>>>,
}

impl OpenFileElement {
    pub fn new() -> OpenFileElement {
        let window = web_sys::window().expect("Window not found");
        let document = window.document().expect("Document not found");
        let input_el = document.create_element("input").unwrap();
        let input: web_sys::HtmlInputElement = wasm_bindgen::JsCast::dyn_into(input_el).unwrap();

        input.set_id("waveview-input-element");
        input.set_type("file");
        OpenFileElement { input }
    }

    pub fn on_change(&mut self, mut f: impl FnMut(web_sys::FileList) + 'static) {
        let input = self.input.clone();
        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
            if let Some(files) = input.files() {
                f(files);
            }
        });
        use wasm_bindgen::JsCast;
        self.input.set_onchange(Some(closure.as_ref().unchecked_ref()));
        // self.input.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        // self.onchange = Some(closure);
    }

    /// This approach seems to work on Chrome and Firefox but not Safari. It is possible to trigger
    /// it from a .click() but it seems to need it on another click handler directly? A .click() on
    /// the canvas element can trigger it so maybe something like
    ///   https://stackoverflow.com/questions/17384845/trigger-a-mouse-click-upon-an-input-type-button-from-canvas
    /// (or any click on the canvas when the button is highlighted?)
    pub fn select_file(&self) {
        self.input.click();
    }
}

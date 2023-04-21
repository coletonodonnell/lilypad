mod block_editor;
mod file_picker;
mod parse;
mod theme;

use druid::widget::{Button, Either, Flex};
use druid::{
    AppDelegate, AppLauncher, Data, FileDialogOptions, Lens, PlatformError, Widget, WidgetExt,
    WindowDesc,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use block_editor::EditorModel;

#[derive(Clone, Data)]
pub struct AppModel {
    #[data(eq)]
    pub dir: Option<PathBuf>,
    pub file: Option<String>,

    pub source: Arc<Mutex<String>>,
    #[data(eq)]
    pub diagnostics: Vec<block_editor::diagnostics::Diagnostic>,
    #[data(eq)]
    pub diagnostic_selection: Option<u64>,
}

struct EditorLens;

impl Lens<AppModel, EditorModel> for EditorLens {
    fn with<V, F: FnOnce(&EditorModel) -> V>(&self, data: &AppModel, f: F) -> V {
        f(&EditorModel {
            source: data.source.clone(),
            diagnostics: data.diagnostics.clone(),
            diagnostic_selection: data.diagnostic_selection,
        })
    }

    fn with_mut<V, F: FnOnce(&mut EditorModel) -> V>(&self, data: &mut AppModel, f: F) -> V {
        let mut editor_model = EditorModel {
            source: data.source.clone(),
            diagnostics: data.diagnostics.clone(),
            diagnostic_selection: data.diagnostic_selection,
        };
        let val = f(&mut editor_model);
        data.source = editor_model.source;
        data.diagnostics = editor_model.diagnostics;
        data.diagnostic_selection = editor_model.diagnostic_selection;
        val
    }
}

fn main() -> Result<(), PlatformError> {
    let data = AppModel {
        dir: None,
        file: None,
        source: Arc::new(Mutex::new(String::new())),
        diagnostics: vec![],
        diagnostic_selection: None,
    };
    // launch
    let main_window = WindowDesc::new(app_widget()).title("Lilypad Editor");
    AppLauncher::with_window(main_window)
        .delegate(LilypadAppDelegate {})
        .launch(data)
}

fn app_widget() -> impl Widget<AppModel> {
    let editor = block_editor::widget().lens(EditorLens).expand();

    let dir_picker = Button::new("Choose directory").on_click(|ctx, _data, _env| {
        let options = FileDialogOptions::new().select_directories();
        ctx.submit_command(druid::commands::SHOW_OPEN_PANEL.with(options))
    });

    Either::new(
        |data, _env| data.dir.is_some(),
        Flex::row()
            .with_child(file_picker::widget())
            .with_flex_child(editor, 1.0)
            .must_fill_main_axis(true),
        dir_picker,
    )
}

struct LilypadAppDelegate;

impl AppDelegate<AppModel> for LilypadAppDelegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppModel,
        _env: &druid::Env,
    ) -> druid::Handled {
        if let Some(dir) = cmd.get(druid::commands::OPEN_FILE) {
            data.dir = Some(dir.path.clone());
        }
        druid::Handled::No
    }
}

// temp shim
pub(crate) mod vscode {
    use druid::Selector;

    use crate::block_editor::{
        diagnostics::{Diagnostic, VSCodeCommand},
        text_range::TextEdit,
    };

    pub const SET_TEXT_SELECTOR: Selector<String> = Selector::new("set_text");
    pub const APPLY_EDIT_SELECTOR: Selector<TextEdit> = Selector::new("apply_edit");
    pub const COPY_SELECTOR: Selector<()> = Selector::new("copy");
    pub const CUT_SELECTOR: Selector<()> = Selector::new("cut");
    pub const PASTE_SELECTOR: Selector<String> = Selector::new("paste");
    pub const DIAGNOSTICS_SELECTOR: Selector<Vec<Diagnostic>> = Selector::new("diagnostics");
    pub const QUICK_FIX_SELECTOR: Selector<Vec<VSCodeCommand>> = Selector::new("quick_fix");

    // pub fn started() {}
    pub fn edited(_: &str, _: usize, _: usize, _: usize, _: usize) {}
    pub fn set_clipboard(_: String) {}
    pub fn request_quick_fixes(_: usize, _: usize) {}
    pub fn execute_command(_: String, _: wasm_bindgen::JsValue) {}
}

pub(crate) use println as console_log;

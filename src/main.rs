mod app;
mod document;

use app::TextEditorApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Rust Text Editor",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(TextEditorApp::new(cc)) as Box<dyn eframe::App>)
        }),
    )
}

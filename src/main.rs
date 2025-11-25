// Hide the console on Windows
#![windows_subsystem = "windows"]

use native_dialog::MessageLevel;

mod config;
mod flag;
mod interface;
mod parse;
mod pdf;
mod print;
mod table_data;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{}", info.to_string());

        native_dialog::DialogBuilder::message()
            .set_text(format!("Error: {}", info.to_string()))
            .set_title("Error in lifpdf")
            .set_level(MessageLevel::Error)
            .alert()
            .show()
            .unwrap();
    }));

    let main_window = MainWindow::new()?;

    interface::interface_main_window(&main_window)?;

    // main_window.window().on

    main_window.run()
}

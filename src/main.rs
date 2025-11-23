// Hide the console on Windows
#![windows_subsystem = "windows"]

use native_dialog::MessageLevel;

mod config;
mod interface;
mod parse;
mod pdf;
mod table_data;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{}", info.to_string());

        native_dialog::DialogBuilder::message()
            .set_text(format!(
                "Error: {}",
                info.payload_as_str().unwrap().to_string()
            ))
            .set_title("Error in lifpdf")
            .set_level(MessageLevel::Error)
            .alert()
            .show()
            .unwrap();
    }));

    let main_window = MainWindow::new()?;

    interface::interface_main_window(&main_window)?;

    main_window.run()
}

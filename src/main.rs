// Hide the console on Windows
#![windows_subsystem = "windows"]

mod config;
mod interface;
mod parse;
mod pdf;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    interface::interface_main_window(&main_window)?;

    main_window.run()
}

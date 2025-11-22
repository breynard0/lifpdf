mod interface;
mod parse;
mod pdf;
mod config;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    interface::interface_main_window(&main_window)?;

    main_window.run()
}

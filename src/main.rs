// Hide the console on Windows
#![windows_subsystem = "windows"]

mod config;
mod interface;
mod parse;
mod pdf;
mod table_data;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // println!("{}", SkaterTime {
    //     minutes: 0,
    //     seconds: 28,
    //     subsecond: 0.540,
    // }.to_string());
    let race = parse::RaceEvent::parse_lif(
        include_str!("../test_data/testfile.lif").to_string(),
        "Testfile".to_string(),
    )?;
    pdf::gen_timesheet_pdf(race).unwrap();

    // let main_window = MainWindow::new()?;
    //
    // interface::interface_main_window(&main_window)?;
    //
    // main_window.run()
    Ok(())
}

// use crate::ui::AppState;

use crate::parse::RaceEvent;
use slint::SharedString;

mod parse;
mod pdf;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.set_race_event_set(true);
    main_window.set_input_event(
        RaceEvent::parse_lif(
            include_str!("../test_data/testfile.lif").to_string(),
            "testfile".to_string(),
        )?
        .into(),
    );
    main_window.run()
}

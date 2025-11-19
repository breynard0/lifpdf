// use crate::ui::AppState;

use crate::parse::{RaceEvent, cmp_slint_skater_time};
use slint::{Model, SharedString, StandardListViewItem, VecModel};
use std::rc::Rc;

mod parse;
mod pdf;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    main_window.set_race_event_set(true);
    let event: SlintRaceEvent = RaceEvent::parse_lif(
        include_str!("../test_data/testfile.lif").to_string(),
        "testfile".to_string(),
    )?
    .into();
    main_window.set_event(event.event);

    let table = event.competitors.clone();

    let main_window_weak = main_window.as_weak();
    
    main_window.on_table_changed(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            let sort_idx = main_window.get_table_sort_index();
            let sort_ascending = main_window.get_table_sort_ascending();
            let row_data: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> =
                Rc::new(VecModel::default());
            for r in table
                .clone()
                .sort_by(move |x, o| {
                    let cmp = match sort_idx {
                        1 => x.skater_id.cmp(&o.skater_id),
                        2 => x.lane.cmp(&o.lane),
                        3 => x.first_name.cmp(&o.first_name),
                        4 => x.last_name.cmp(&o.last_name),
                        5 => x.club.cmp(&o.club),
                        6 => cmp_slint_skater_time(&x.time, &o.time),
                        // 0 and everything else, sort by place
                        _ => x.place.cmp(&o.place),
                    };
                    if sort_ascending { cmp } else { cmp.reverse() }
                })
                .iter()
            {
                let items = Rc::new(VecModel::default());

                items.push(slint::format!("{}", r.place).into());
                items.push(slint::format!("{}", r.skater_id).into());
                items.push(slint::format!("{}", r.lane).into());
                items.push(slint::format!("{}", r.first_name).into());
                items.push(slint::format!("{}", r.last_name).into());
                items.push(slint::format!("{}", r.club).into());
                items.push(slint::format!("{}", r.time.to_string()).into());

                row_data.push(items.into());
            }

            main_window.set_table_data(row_data.into());
        }
    });

    main_window.run()
}

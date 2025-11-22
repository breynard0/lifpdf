use crate::config::{load_config, save_config};
use crate::parse::{RaceEvent, cmp_slint_skater_time};
use crate::{MainWindow, SettingsData, SlintCompetitorRow, SlintRaceEvent};
use slint::{
    ComponentHandle, Model, ModelExt, ModelRc, SharedString, StandardListViewItem, VecModel,
};
use std::rc::Rc;

pub fn interface_main_window(main_window: &MainWindow) -> Result<(), slint::PlatformError> {
    main_window.set_race_event_set(true);
    let event: SlintRaceEvent = RaceEvent::parse_lif(
        include_str!("../test_data/testfile.lif").to_string(),
        "testfile".to_string(),
    )?
    .into();
    main_window.set_event(event.event);

    // If config file exists, load it. Otherwise, do nothing
    match load_config() {
        Some(x) => {
            main_window.set_settings_data(x);
        }
        None => {}
    }

    {
        let main_window_weak = main_window.as_weak();
        main_window.on_table_changed(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                main_window
                    .set_table_data(gen_sorted_table(&main_window, &event.competitors).into());
            }
        });
    }

    // Settings menu stuff
    fn push_settings_data(
        paths: &mut Vec<String>,
        settings_data: &mut SettingsData,
        main_window: MainWindow,
    ) {
        let new_search_paths: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(
            paths
                .iter()
                .map(|x| SharedString::from(x))
                .collect::<Vec<_>>(),
        ));
        // Convert it to a ModelRc.
        let new_new_search_paths = ModelRc::from(new_search_paths.clone());
        settings_data.search_paths = new_new_search_paths;
        main_window.set_settings_data(settings_data.clone());
        save_config(settings_data.clone());
    }
    {
        let main_window_weak = main_window.as_weak();
        main_window.on_settings_add_path(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                let settings_data = &mut main_window.get_settings_data();
                let paths: &mut Vec<String> = &mut settings_data
                    .search_paths
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                paths.push(String::new());

                push_settings_data(paths, settings_data, main_window);
            }
        });
    }
    {
        let main_window_weak = main_window.as_weak();
        main_window.on_settings_remove_path(move |i| {
            if let Some(main_window) = main_window_weak.upgrade() {
                let settings_data = &mut main_window.get_settings_data();
                let paths: &mut Vec<String> = &mut settings_data
                    .search_paths
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                if i >= 0 && i < paths.len() as i32 {
                    paths.remove(i as usize);
                } else if i == 0 && paths.len() == 1 {
                    *paths = vec![];
                }
                push_settings_data(paths, settings_data, main_window);
            }
        });
    }
    {
        let main_window_weak = main_window.as_weak();
        main_window.on_settings_edit_path(move |i, s| {
            if let Some(main_window) = main_window_weak.upgrade() {
                let settings_data = &mut main_window.get_settings_data();
                let paths: &mut Vec<String> = &mut settings_data
                    .search_paths
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                if i >= 0 && i < paths.len() as i32 {
                    paths[i as usize] = s.to_string();
                }
                push_settings_data(paths, settings_data, main_window);
            }
        });
    }
    Ok(())
}

fn gen_sorted_table(
    main_window: &MainWindow,
    table: &ModelRc<SlintCompetitorRow>,
) -> Rc<VecModel<ModelRc<StandardListViewItem>>> {
    let sort_idx = main_window.get_table_sort_index();
    let sort_ascending = main_window.get_table_sort_ascending();
    let row_data: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
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
    row_data
}

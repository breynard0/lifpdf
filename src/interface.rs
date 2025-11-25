use crate::config::{load_config, save_config};
use crate::parse::{RaceEvent, cmp_slint_skater_time};
use crate::pdf::{gen_timesheet_pdf, pdf_to_image};
use crate::table_data::gen_table_row;
use crate::{MainWindow, SettingsData, SlintCompetitorRow, SlintRaceEvent};
use native_dialog::MessageLevel;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use slint::{
    ComponentHandle, Model, ModelExt, ModelRc, SharedString, StandardListViewItem, VecModel, Weak,
};
use std::cell::RefCell;
use std::fs::read_dir;
use std::io::Read;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn watcher_fn(res: notify::Result<notify::Event>, main_window_weak: Weak<MainWindow>) {
    match res {
        Ok(event) => match event.kind {
            // Reload paths for any event but access
            EventKind::Access(_) => {}
            _ => {
                slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        let settings = load_config().unwrap();
                        let search_paths = settings
                            .search_paths
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>();
                        let mut files = vec![];
                        for search_path in search_paths {
                            for file in match read_dir(&search_path) {
                                Ok(x) => x.collect(),
                                Err(_) => vec![],
                            } {
                                if let Ok(file) = file {
                                    let name = file.file_name().to_string_lossy().to_string();
                                    if name
                                        .chars()
                                        .rev()
                                        .take(4)
                                        .collect::<String>()
                                        .to_lowercase()
                                        == "fil."
                                    {
                                        if let Ok(metadata) = file.metadata() {
                                            let time = match metadata.modified() {
                                                Ok(t) => t,
                                                Err(_) => metadata.created().unwrap_or_else(|_| {
                                                    std::time::SystemTime::UNIX_EPOCH
                                                }),
                                            };
                                            files.push((name, time))
                                        }
                                    }
                                }
                            }
                        }
                        files.sort_by(|x, y| x.1.cmp(&y.1).reverse());

                        let filter = &main_window.get_lif_file_filter().to_string();

                        let lif_files = files
                            .iter()
                            .filter(|x| x.0.contains(filter))
                            .map(|x| SharedString::from(&x.0))
                            .collect::<Vec<_>>();
                        main_window.set_lif_files(ModelRc::new(VecModel::from(lif_files)));
                    };
                })
                .expect("Failed to get Slint context in watcher closure");
            }
        },
        Err(e) => println!("Watch Error: {:?}", e),
    }
}

pub fn interface_main_window(main_window: &MainWindow) -> Result<(), slint::PlatformError> {
    main_window.set_race_event_set(false);

    // If config file exists, load it. Otherwise, do nothing
    match load_config() {
        Some(x) => {
            main_window.set_settings_data(x);
        }
        None => {}
    }

    // Watch paths for changes
    let main_window_weak = main_window.as_weak();
    let watcher = Arc::new(Mutex::new(
        notify::recommended_watcher(move |e| {
            let main_window_weak_clone = main_window_weak.clone();
            watcher_fn(e, main_window_weak_clone);
        })
        .expect("Could not watch directories"),
    ));
    set_watcher(watcher.clone(), &main_window);

    // Run it once to populate path
    // If it's stupid and it works, it's not stupid
    watcher_fn(
        Ok(notify::Event::new(EventKind::Any)),
        main_window.as_weak(),
    );

    // Run when filter changes
    {
        let main_window_weak = main_window.as_weak();
        main_window.on_filter_changed(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                watcher_fn(
                    Ok(notify::Event::new(EventKind::Any)),
                    main_window.as_weak(),
                );
            }
        })
    }

    {
        let main_window_weak = main_window.as_weak();
        main_window.on_settings_close_button_clicked(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                set_watcher(watcher.clone(), &main_window);
            }
        })
    }

    // Load settings data
    {
        let main_window_weak = main_window.as_weak();
        main_window.on_settings_button_clicked(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                let config = load_config().unwrap();
                main_window.set_settings_data(config);
            }
        })
    }

    // General settings updates
    main_window.on_general_settings_update(move |settings_data| {
        save_config(settings_data);
    });

    let cur_path = Rc::new(RefCell::new(None));
    {
        let main_window_weak = main_window.as_weak();
        let cur_path_clone = cur_path.clone();
        main_window.on_table_changed(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                let selected_lif = main_window.get_selected_lif_file();
                let lif_files = main_window
                    .get_lif_files()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>();
                if selected_lif >= 0 && lif_files.len() > 0 {
                    // I don't think there will be any situation where it is needed to disambiguate
                    // between identically name files between directories, so I'm implementing it
                    // like this. Can change later if necessary
                    let event_name = lif_files.get(selected_lif as usize).unwrap();
                    let mut event_path = None;

                    for search_path in load_config().unwrap().search_paths.iter() {
                        let search_path = search_path.to_string();
                        for file in read_dir(search_path).expect("Failed to read directory") {
                            if let Ok(file) = file {
                                if file.file_name().to_string_lossy().to_string() == *event_name {
                                    event_path = Some(file.path());
                                    break;
                                }
                            }
                        }
                    }

                    match event_path {
                        Some(e) => {
                            let mut f = std::fs::File::open(&e).expect("Failed to open file");
                            let mut buffer = Vec::new();
                            f.read_to_end(&mut buffer)
                                .expect("Failed to read bytes of file");

                            let mut file_contents = String::new();
                            for byte in buffer {
                                file_contents.push(byte as char);
                            }

                            match RaceEvent::parse_lif(file_contents, event_name.clone()) {
                                Ok(event) => {
                                    let event: SlintRaceEvent = event.into();
                                    main_window.set_event(event.event);
                                    main_window.set_table_data(
                                        gen_sorted_table(&main_window, &event.competitors).into(),
                                    );
                                    main_window.set_race_event_set(true);
                                    let mut cur_path = cur_path_clone.borrow_mut();
                                    *cur_path = Some(e.to_string_lossy().to_string());
                                }
                                Err(e) => {
                                    let _ = native_dialog::DialogBuilder::message()
                                        .set_level(MessageLevel::Error)
                                        .set_title("Error parsing file")
                                        .set_text(format!("Failed to parse {}, {}", event_name, e))
                                        .alert()
                                        .show();
                                }
                            };
                        }
                        None => {
                            println!("No file called {} found", event_name);
                        }
                    }
                }
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
                let settings_data = &mut load_config().unwrap();
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
                let settings_data = &mut load_config().unwrap();
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
                let settings_data = &mut load_config().unwrap();
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

    // PDF generation and display
    let pub_pdf_bitmap = Rc::new(RefCell::new(None));
    {
        let main_window_weak = main_window.as_weak();
        let cur_path_clone = cur_path.clone();
        let pub_pdf_clone = pub_pdf_bitmap.clone();
        main_window.on_genpdf_button_clicked(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                let path = cur_path_clone.borrow();
                if let Some(path) = path.as_ref() {
                    let mut f = std::fs::File::open(path).expect("Failed to open file");
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer)
                        .expect("Failed to read bytes of file");

                    let mut file_contents = String::new();
                    for byte in buffer {
                        file_contents.push(byte as char);
                    }

                    match RaceEvent::parse_lif(file_contents, path.clone()) {
                        Ok(event) => {
                            let mut pdf =
                                gen_timesheet_pdf(event.clone()).expect("PDF generation failed");

                            let (images, width, height) = pdf_to_image(&mut pdf).unwrap();
                            let mut slint_imgs = vec![];
                            for image in &images {
                                let img_buf = slint::Image::from_rgba8(
                                    slint::SharedPixelBuffer::clone_from_slice(
                                        &image, width, height,
                                    ),
                                );
                                slint_imgs.push(img_buf);
                            }

                            main_window.set_pdf_images(ModelRc::new(VecModel::from(slint_imgs)));
                            main_window.set_tab_index(1);

                            if load_config().unwrap().pdf_output_enabled {
                                if !std::fs::exists(load_config().unwrap().pdf_output_path).unwrap()
                                {
                                    let _ = std::fs::create_dir_all(
                                        load_config().unwrap().pdf_output_path,
                                    );
                                }

                                pdf.save(format!(
                                    "{}/{}.pdf",
                                    load_config().unwrap().pdf_output_path,
                                    event.event.event_code
                                ))
                                .expect("Error writing PDF to disk");
                            }

                            let mut pub_pdf = pub_pdf_clone.borrow_mut();
                            *pub_pdf = Some((images, width, height));
                        }
                        Err(e) => {
                            println!("Failed to parse {}, {}", path, e);
                        }
                    };
                }
            }
        });
    }

    // Printing
    {
        let pub_pdf_clone = pub_pdf_bitmap.clone();
        main_window.on_print_button_clicked(move || {
            let pdf = pub_pdf_clone.borrow();
            if let Some(doc) = pdf.as_ref() {
                crate::print::print_document(&doc.0, doc.1, doc.2);
            }
        });
    }

    Ok(())
}

fn set_watcher(watcher_cln: Arc<Mutex<RecommendedWatcher>>, main_window: &MainWindow) {
    let mut watcher = watcher_cln.lock().unwrap();
    // Get paths
    let settings_data = &mut load_config().unwrap();
    let paths: &mut Vec<String> = &mut settings_data
        .search_paths
        .iter()
        .map(|x| x.to_string())
        .collect();

    // Reset watcher
    let main_window_weak = main_window.as_weak();
    *watcher = notify::recommended_watcher(move |e| {
        let main_window_weak_clone = main_window_weak.clone();
        watcher_fn(e, main_window_weak_clone);
    })
    .expect("Could not watch directories");
    // Add paths
    for path in paths {
        watcher
            .watch(path.as_ref(), RecursiveMode::NonRecursive)
            .expect("Failed to watch path");
    }
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

        let table_row = gen_table_row(r.into());

        for entry in table_row {
            items.push(SharedString::from(entry).into());
        }

        row_data.push(items.into());
    }
    row_data
}

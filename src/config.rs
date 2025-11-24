use crate::SettingsData;
use slint::{Model, ModelRc, SharedString, VecModel};

#[derive(serde::Serialize, serde::Deserialize)]
struct SettingsDataAnalog {
    paths: Vec<String>,
    pdf_output_enabled: bool,
    pdf_output_path: String,
}

fn get_path() -> String {
    #[cfg(target_os = "windows")]
    let slash = '\\';
    #[cfg(not(target_os = "windows"))]
    let slash = '/';
    format!(
        "{}{}lifpdf.json",
        dirs::config_dir().unwrap().to_str().unwrap(),
        slash
    )
}

pub fn save_config(data: SettingsData) {
    let config = SettingsDataAnalog {
        paths: data.search_paths.iter().map(|x| x.to_string()).collect(),
        pdf_output_enabled: data.pdf_output_enabled,
        pdf_output_path: data.pdf_output_path.into(),
    };

    let json = serde_json::to_string(&config)
        .expect("Failed to parse config. This is an internal error and should be reported.");

    let path = get_path();
    std::fs::write(&path, json).expect(&format!("Error writing to {}", path));
}

pub fn load_config() -> Option<SettingsData> {
    let path = get_path();

    let exists = std::fs::exists(&path).unwrap_or_else(|_| false);

    if exists {
        let analog: SettingsDataAnalog = serde_json::from_str(
            &std::fs::read_to_string(path).expect("Failed to read config file"),
        )
            .unwrap();

        Some(SettingsData {
            pdf_output_enabled: analog.pdf_output_enabled,
            pdf_output_path: analog.pdf_output_path.into(),
            search_paths: ModelRc::new(VecModel::from(
                analog
                    .paths
                    .iter()
                    .map(|x| SharedString::from(x))
                    .collect::<Vec<_>>(),
            )),
        })
    } else {
        Some(SettingsData::default())
    }
}

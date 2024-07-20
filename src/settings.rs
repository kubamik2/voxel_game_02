use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub borderless: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resolution: (1000, 1000),
            fullscreen: false,
            borderless: false
        }
    }
}

impl Settings {
    pub fn from_file<T: Into<std::path::PathBuf>>(path: T) -> anyhow::Result<Self> {
        let settings_file = std::fs::File::open(path.into())?;
        let reader = std::io::BufReader::new(settings_file);
        let settings: Self = serde_json::from_reader(reader)?;   
        Ok(settings)
    }
}
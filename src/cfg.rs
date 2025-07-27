use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub filename_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("./libraries"),
            filename_format: "python04rel64rspyo3py{py_version}".to_string(),
        }
    }
}

pub fn parse_cfg() -> Result<Config, std::io::Error> {
    let content = std::fs::read_to_string("server.cfg")?;

    let mut cfg = Config::default();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once(' ').unwrap();
        let key_trim_lower = key.trim().to_lowercase();
        let key = key_trim_lower.as_str();
        match key {
            "python_plugins_dir" => cfg.dir = PathBuf::from(value.trim()),
            "python_filename_format" => cfg.filename_format = value.trim().to_string(),
            _ => {}
        }
    }

    Ok(cfg)
}

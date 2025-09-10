use std::fs::File;
use std::io::read_to_string;

use anyhow::Context;
use camino::Utf8Path as Path;
use camino::Utf8PathBuf as PathBuf;
use serde::Deserialize;
use serde::Serialize;

const DATA_DIR: &str = "data";
const DAYS_TO_GHOST: u16 = 90;

#[derive(Serialize, Deserialize)]
pub struct PlyConfig {
    pub data_dir: PathBuf,
    pub days_to_ghost: u16,
    pub cycle: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PartialPlyConfig {
    pub data_dir: Option<PathBuf>,
    pub days_to_ghost: Option<u16>,
    pub cycle: Option<String>,
}

fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .map(|dir| {
            PathBuf::from_path_buf(dir.join(".config/ply/ply.toml"))
                .expect("config path should be UTF-8")
        })
        .expect("home directory should exist")
}

fn tilde_expand(dir: &Path) -> PathBuf {
    Path::new(&shellexpand::tilde(dir.as_str())).to_path_buf()
}

impl From<PartialPlyConfig> for PlyConfig {
    fn from(config: PartialPlyConfig) -> PlyConfig {
        let data_dir = tilde_expand(&config.data_dir.unwrap_or(Path::new(DATA_DIR).to_path_buf()));

        PlyConfig {
            data_dir,
            days_to_ghost: config.days_to_ghost.unwrap_or(DAYS_TO_GHOST),
            cycle: config.cycle,
        }
    }
}

pub fn config() -> PlyConfig {
    let config_path = default_config_path();

    // TODO: logging for failures in these subops, fern?
    let config: PartialPlyConfig = File::open(config_path)
        .context("failed to open config")
        .and_then(|file| read_to_string(file).context("failed to read config"))
        .and_then(|content| toml::from_str(&content).context("failed to parse config into TOML"))
        .unwrap();

    config.into()
}

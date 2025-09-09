use camino::Utf8Path as Path;
use camino::Utf8PathBuf as PathBuf;

const DATA_DIR: &str = "./data";
const DAYS_TO_GHOST: u16 = 90;

pub struct PlyConfig {
    pub data_dir: PathBuf,
    pub days_to_ghost: u16,
}

pub fn default_config() -> PlyConfig {
    PlyConfig {
        data_dir: Path::new(DATA_DIR).to_path_buf(),
        days_to_ghost: DAYS_TO_GHOST,
    }
}

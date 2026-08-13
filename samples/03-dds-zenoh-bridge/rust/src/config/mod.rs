pub mod load_yaml;

use std::path::PathBuf;

/// Sample root (`samples/03-dds-zenoh-bridge/`), one level above `rust/`.
pub fn sample_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Cargo manifest must live in rust/")
        .to_path_buf()
}

pub fn configs_dir() -> PathBuf {
    sample_root().join("configs")
}

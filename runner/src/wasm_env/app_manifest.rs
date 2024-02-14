use serde::Deserialize;
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct AppManifest {
    pub path: PathBuf,
    pub app_name: String,
    pub app_bin_path: PathBuf,
    pub refresh_period: Option<Duration>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestSchema {
    name: String,
    bin: String,
    refresh_period_ms: Option<u32>,
}

impl AppManifest {
    pub fn open(manifest_dir: impl AsRef<Path>) -> io::Result<Self> {
        let mut manifest_filepath = manifest_dir.as_ref().to_path_buf();
        manifest_filepath.push("manifest.json");
        let mut manifest_file = std::fs::File::open(&manifest_filepath).map_err(|err| {
            tracing::error!("Failed to open manifest file: {err}");
            err
        })?;
        let mut manifest_contents = String::new();
        manifest_file.read_to_string(&mut manifest_contents)?;

        if let Ok(manifest) = serde_json::from_str::<ManifestSchema>(&mut manifest_contents) {
            if manifest.bin.contains('/') || manifest.bin.contains('\\') {
                tracing::error!("Invalid binary filename: {}", &manifest.bin);
                return Err(io::ErrorKind::InvalidData.into());
            }

            let mut bin_path = manifest_dir.as_ref().to_path_buf();
            bin_path.push(manifest.bin);

            Ok(AppManifest {
                path: manifest_filepath,
                app_name: manifest.name,
                app_bin_path: bin_path,
                refresh_period: manifest
                    .refresh_period_ms
                    .map(|duration| Duration::from_millis(duration.into())),
            })
        } else {
            tracing::error!(
                "Failed to parse manifest at path: {}",
                manifest_filepath.display()
            );
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

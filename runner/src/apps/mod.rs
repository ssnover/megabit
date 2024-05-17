pub use manifest::AppManifest;
use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub mod manifest;

#[derive(Debug, Clone)]
pub struct Library {
    data_dir: PathBuf,
    apps: Arc<Mutex<Vec<AppManifest>>>,
}

impl Library {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let apps = load_from_path(&path)?;

        let library = Self {
            data_dir: path.as_ref().to_path_buf(),
            apps: Arc::new(Mutex::new(apps)),
        };

        Ok(library)
    }

    fn get(&self, checksum: &str) -> Option<(usize, AppManifest)> {
        let apps = self.apps.lock().unwrap();
        apps.iter()
            .enumerate()
            .find(|(_idx, app)| app.md5sum.as_str() == checksum)
            .map(|(idx, app)| (idx, app.clone()))
    }

    pub fn get_app(&self, checksum: &str) -> Option<AppManifest> {
        self.get(checksum).map(|(_idx, app)| app)
    }

    pub fn get_first(&self) -> Option<AppManifest> {
        let apps = self.apps.lock().unwrap();
        apps.get(0).cloned()
    }

    pub fn get_next(&self, current_checksum: &str) -> Option<AppManifest> {
        let apps = self.apps.lock().unwrap();
        if let Some((idx, _)) = self.get(current_checksum) {
            let next_idx = if idx == apps.len() - 1 { 0 } else { idx + 1 };
            Some(apps[next_idx].clone())
        } else {
            tracing::warn!("Attempted to get app manifest for nonexistent app with checksum {current_checksum}");
            None
        }
    }

    pub fn get_prev(&self, current_checksum: &str) -> Option<AppManifest> {
        let apps = self.apps.lock().unwrap();
        if let Some((idx, _)) = self.get(current_checksum) {
            let next_idx = if idx == 0 { apps.len() - 1 } else { idx - 1 };
            Some(apps[next_idx].clone())
        } else {
            tracing::warn!("Attempted to get app manifest for nonexistent app with checksum {current_checksum}");
            None
        }
    }
}

fn load_from_path(path: impl AsRef<Path>) -> io::Result<Vec<AppManifest>> {
    let apps = std::fs::read_dir(&path)?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(app) = AppManifest::open(&path) {
                    tracing::info!("Found app {} at path: {}", &app.app_name, path.display());
                    Ok(Some(app))
                } else {
                    tracing::warn!("Unable to build app from bundle at {}", path.display());
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
        .filter_map(|app: anyhow::Result<Option<AppManifest>>| match app {
            Ok(app) => app,
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    tracing::info!("Found {} apps", apps.len());

    Ok(apps)
}

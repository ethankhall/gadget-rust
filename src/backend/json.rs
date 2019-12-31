use super::{models::RedirectModel, RowChange};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub struct JsonBackend {
    storage: Arc<RwLock<Vec<RedirectModel>>>,
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct JsonFile {
    redirects: Vec<RedirectModel>,
}

impl JsonBackend {
    pub fn new<S: ToString>(path: S) -> Self {
        let file_path = PathBuf::from(&path.to_string());
        if !file_path.exists() {
            warn!("{:?} does not exist, creating new file.", file_path);
            let backend = JsonBackend {
                path: file_path,
                storage: Arc::default(),
            };
            backend.save();
            return backend;
        } else {
            let json_file: JsonFile =
                match serde_json::from_str(&std::fs::read_to_string(&file_path).unwrap()) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Unable to read JSON file: {}", e);
                        panic!();
                    }
                };

            let backend = JsonBackend {
                path: file_path,
                storage: Arc::new(RwLock::new(json_file.redirects)),
            };
            return backend;
        }
    }

    fn save(&self) {
        match self.storage.read() {
            Ok(vec) => {
                let data = JsonFile {
                    redirects: vec.iter().map(|x| x.clone()).collect(),
                };
                match std::fs::write(
                    self.path.clone(),
                    serde_json::to_string_pretty(&data).unwrap(),
                ) {
                    Ok(_) => info!("Updated JSON file"),
                    Err(e) => error!("Unable to update JSON file because {}", e),
                }
            }
            Err(e) => error!("Unable to read storage contents because {}", e),
        }
    }
}

impl super::Backend for JsonBackend {
    fn get_redirect(&self, redirect_ref: &str) -> RowChange<RedirectModel> {
        return match self.storage.read() {
            Ok(vec) => {
                if let Some(dest) = vec.iter().find(|redirect| {
                    redirect.public_ref == redirect_ref || redirect.alias == redirect_ref
                }) {
                    RowChange::Value(dest.clone())
                } else {
                    RowChange::NotFound
                }
            }
            Err(e) => RowChange::Err(format!("{}", e)),
        };
    }

    fn create_redirect(&self, new_alias: &str, new_destination: &str) -> RowChange<RedirectModel> {
        let result = match self.storage.write() {
            Ok(mut vec) => {
                if let Some(_) = vec.iter().find(|redirect| redirect.alias == new_alias) {
                    return RowChange::Err("Alias already exists".to_string());
                }

                let id = vec.iter().map(|x| x.redirect_id).max().unwrap_or(0) + 1;

                let model = RedirectModel::new(id, new_alias, new_destination);
                vec.push(model.clone());

                RowChange::Value(model)
            }
            Err(e) => RowChange::Err(format!("{}", e)),
        };

        self.save();
        result
    }

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str) -> RowChange<usize> {
        let mut result = RowChange::NotFound;
        match self.storage.write() {
            Ok(mut vec) => {
                for i in 0..vec.len() {
                    if vec[i].public_ref == redirect_ref || vec[i].alias == redirect_ref {
                        vec[i].set_destination(new_dest);
                        result = RowChange::Value(1);
                        break;
                    }
                }
            }
            Err(e) => result = RowChange::Err(format!("{}", e)),
        }

        self.save();
        result
    }

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize> {
        let mut result = RowChange::NotFound;
        match self.storage.write() {
            Ok(mut vec) => {
                for i in 0..vec.len() {
                    if vec[i].public_ref == redirect_ref || vec[i].alias == redirect_ref {
                        vec.remove(i);
                        result = RowChange::Value(1);
                        break;
                    }
                }
            }
            Err(e) => result = RowChange::Err(format!("{}", e)),
        }

        self.save();
        result
    }

    fn get_all(&self, page: u64, limit: usize) -> Result<Vec<RedirectModel>, String> {
        let begin: usize = limit * page as usize;
        let end: usize = begin + limit;
        match self.storage.read() {
            Ok(v) => Ok(v
                .get((begin)..(end))
                .map(|x| x.clone())
                .unwrap_or_default()
                .to_vec()),
            Err(e) => Err(e.to_string()),
        }
    }
}

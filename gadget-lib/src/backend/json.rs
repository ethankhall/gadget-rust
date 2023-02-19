use std::path::PathBuf;

use crate::backend::prelude::*;
use crate::prelude::LibResult;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

pub struct JsonBackend {
    in_memory: InMemoryBackend,
    file_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct JsonFile {
    redirects: Vec<RedirectModel>,
}

impl JsonFile {
    fn save(&self, path: &PathBuf) -> LibResult<()> {
        let contents = serde_json::to_string_pretty(&self).unwrap();
        Ok(std::fs::write(path, contents)?)
    }
}

impl JsonBackend {
    pub fn new(file_path: PathBuf) -> LibResult<Self> {
        let backend = if !file_path.exists() {
            warn!("{:?} does not exist, creating new file.", file_path);
            let backend = InMemoryBackend::new(Default::default());
            let redirects = backend.get_internal_model()?;

            let json_file = JsonFile { redirects };
            json_file.save(&file_path)?;
            backend
        } else {
            let json_file: JsonFile =
                match serde_json::from_str(&std::fs::read_to_string(&file_path)?) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Unable to read JSON file: {}", e);
                        panic!();
                    }
                };
            InMemoryBackend::new(json_file.redirects)
        };

        Ok(JsonBackend {
            in_memory: backend,
            file_path,
        })
    }

    fn save(&self) -> LibResult<()> {
        let redirects = self.in_memory.get_internal_model().unwrap();

        let json_file = JsonFile { redirects };
        json_file.save(&self.file_path)?;
        Ok(())
    }
}

impl<'a> Backend<'a> for JsonBackend {
    fn get_redirect(&self, redirect_ref: &str) -> LibResult<Option<RedirectModel>> {
        self.in_memory.get_redirect(redirect_ref)
    }

    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
        username: &str,
    ) -> LibResult<RedirectModel> {
        let result = self
            .in_memory
            .create_redirect(new_alias, new_destination, username);
        self.save()?;
        result
    }

    fn update_redirect(
        &self,
        redirect_ref: &str,
        new_dest: &str,
        username: &str,
    ) -> LibResult<RedirectModel> {
        let result = self
            .in_memory
            .update_redirect(redirect_ref, new_dest, username);
        self.save()?;
        result
    }

    fn delete_redirect(&self, redirect_ref: &str) -> LibResult<usize> {
        let result = self.in_memory.delete_redirect(redirect_ref);
        self.save()?;
        result
    }

    fn get_all(&self, page: u64, limit: usize) -> LibResult<Vec<RedirectModel>> {
        self.in_memory.get_all(page, limit)
    }
}

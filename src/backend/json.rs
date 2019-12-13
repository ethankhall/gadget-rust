use std::{path::Path, sync::{Arc, RwLock}};

struct JsonBackend {
    storage: Arc<RwLock<Vec<RedirectModel>>>,
    path: Box<Path>
}

impl super::Backend for JsonBackend {
    fn get_redirect(&self, redirect_ref: &str) -> RowChange<RedirectModel> {
        match storage.read() {
            Ok(vec) => {
                if let Some(dest) = vec.iter().find(|redirect| redirect.matches(path)) {
                    return dest.get_destination(&url);
                }
        
                if let Some(dest) = vec.iter().find(|redirect| redirect.matches(path)) {
                    return dest.get_destination(&url);
                }
            }
        }
    }

    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
    ) -> RowChange<RedirectModel> {
        
    }

    fn update_redirect(&self, redirect_ref: &str, new_dest: &str) -> RowChange<usize> {

    }

    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize> {

    }

    fn get_all(&self, page: i64, limit: i64) -> Result<Vec<RedirectModel>, String> {
        match self.storage.read() {
            Ok(v) => Ok(v.get((limit * page)..((page + 1) * limit)).map(|x| x.clone()).collect()),
            Err(e) => Err(e)
        }
    }
}
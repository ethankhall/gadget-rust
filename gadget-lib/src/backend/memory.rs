use crate::backend::prelude::*;
use std::sync::{Arc, RwLock};
use crate::prelude::{LibResult, GadgetLibError};


pub struct InMemoryBackend {
    storage: Arc<RwLock<Vec<RedirectModel>>>,
}

impl InMemoryBackend {
    pub fn new(redirects: Vec<RedirectModel>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(redirects.clone())),
        }
    }

    pub fn get_internal_model(&self) -> LibResult<Vec<RedirectModel>> {
        Ok(self.storage.read()?.clone())
    }
}

impl<'a> super::Backend<'a> for InMemoryBackend {
    #[tracing::instrument(skip(self))]
    fn get_redirect(&self, redirect_ref: &str) -> LibResult<Option<RedirectModel>> {
        let vec = self.storage.read()?;
        if let Some(dest) = vec
            .iter()
            .find(|redirect| redirect.public_ref == redirect_ref || redirect.alias == redirect_ref)
        {
            Ok(Some(dest.clone()))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(self))]
    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
        username: &str,
    ) -> LibResult<RedirectModel> {
        let mut vec = self.storage.write()?;
        if vec.iter().any(|redirect| redirect.alias == new_alias) {
            return Err(GadgetLibError::RedirectExists("Alias already exists".to_string()));
        }

        let id = vec.iter().map(|x| x.redirect_id).max().unwrap_or(0) + 1;

        let model = RedirectModel::new(id, new_alias, new_destination, Some(username.to_string()));
        vec.push(model.clone());

        Ok(model)
    }

    #[tracing::instrument(skip(self))]
    fn update_redirect(&self, redirect_ref: &str, new_dest: &str, username: &str) -> LibResult<RedirectModel> {
        let mut vec = self.storage.write()?;
        for i in 0..vec.len() {
            if vec[i].public_ref == redirect_ref || vec[i].alias == redirect_ref {
                vec[i].set_destination(new_dest);
                vec[i].update_username(Some(username));
                return Ok(vec[i].clone());
            }
        }
        Err(GadgetLibError::RedirectDoesNotExists(redirect_ref.to_string()))
    }

    #[tracing::instrument(skip(self))]
    fn delete_redirect(&self, redirect_ref: &str) -> LibResult<usize> {
        let mut vec = self.storage.write()?;
        for i in 0..vec.len() {
            if vec[i].public_ref == redirect_ref || vec[i].alias == redirect_ref {
                vec.remove(i);
                return Ok(1);
            }
        }
        Err(GadgetLibError::RedirectDoesNotExists(redirect_ref.to_string()))
    }

    #[tracing::instrument(skip(self))]
    fn get_all(&self, page: u64, limit: usize) -> LibResult<Vec<RedirectModel>> {
        let begin: usize = limit * page as usize;
        let end: usize = begin + limit;
        let v = self.storage.read()?;
        let end = std::cmp::min(end, v.len());
        let data = v.get((begin)..(end)).unwrap_or_default().to_vec();
        Ok(data)
    }
}

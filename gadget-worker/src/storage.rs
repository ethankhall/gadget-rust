use crate::Result;
use gadget_lib::prelude::*;
use serde::{Deserialize, Serialize};
use worker::kv::KvStore;

pub struct WorkerStore {
    store: KvStore,
    backend: InMemoryBackend,
}

#[derive(Serialize, Deserialize)]
struct KVStorageModel {
    redirects: Vec<RedirectModel>,
}

impl WorkerStore {
    pub async fn new(env: &worker::Env) -> Result<Self> {
        let kv = env.kv("gadget")?;
        let data: KVStorageModel = match kv.get("default").json().await? {
            Some(value) => value,
            None => KVStorageModel {
                redirects: Default::default(),
            },
        };

        let in_mem = InMemoryBackend::new(data.redirects);

        Ok(WorkerStore {
            backend: in_mem,
            store: kv,
        })
    }

    async fn save(&self) -> Result<()> {
        let redirects = self.backend.get_internal_model().unwrap();
        let stroage = KVStorageModel { redirects };

        self.store.put("default", stroage)?.execute().await?;

        Ok(())
    }
}

impl WorkerStore {
    pub async fn get_redirect(&self, redirect_ref: &str) -> Result<Option<RedirectModel>> {
        Ok(self.backend.get_redirect(redirect_ref)?)
    }

    pub async fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
        username: &str,
    ) -> Result<RedirectModel> {
        let result = self
            .backend
            .create_redirect(new_alias, new_destination, username);
        self.save().await?;
        Ok(result?)
    }

    pub async fn update_redirect(
        &self,
        redirect_ref: &str,
        new_dest: &str,
        username: &str,
    ) -> Result<RedirectModel> {
        let result = self
            .backend
            .update_redirect(redirect_ref, new_dest, username);
        self.save().await?;
        Ok(result?)
    }

    pub async fn delete_redirect(&self, redirect_ref: &str) -> Result<usize> {
        let result = self.backend.delete_redirect(redirect_ref);
        self.save().await?;
        Ok(result?)
    }

    pub async fn get_all(&self, page: u64, limit: usize) -> Result<Vec<RedirectModel>> {
        Ok(self.backend.get_all(page, limit)?)
    }
}

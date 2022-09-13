struct KVStore {
    context: worker::Context,
}

impl gadget_lib::Backend for KVStore {
    #[tracing::instrument(skip(self))]
    fn get_redirect(&self, redirect_ref: &str) -> RowChange<RedirectModel> {
        match self.storage.read() {
            Ok(vec) => {
                if let Some(dest) = vec.iter().find(|redirect| {
                    redirect.alias == redirect_ref
                }) {
                    RowChange::Value(dest.clone())
                } else {
                    RowChange::NotFound
                }
            }
            Err(e) => RowChange::Err(format!("{}", e)),
        }
    }

    #[tracing::instrument(skip(self))]
    fn create_redirect(
        &self,
        new_alias: &str,
        new_destination: &str,
        username: &str,
    ) -> RowChange<RedirectModel> {
        self.context.
        let result = match self.storage.write() {
            Ok(mut vec) => {
                if vec.iter().any(|redirect| redirect.alias == new_alias) {
                    return RowChange::Err("Alias already exists".to_string());
                }

                let model =
                    RedirectModel::new(new_alias, new_destination, Some(username.to_string()));
                vec.push(model.clone());

                RowChange::Value(model)
            }
            Err(e) => RowChange::Err(format!("{}", e)),
        };

        self.save();
        result
    }

    #[tracing::instrument(skip(self))]
    fn update_redirect(
        &self,
        redirect_ref: &str,
        new_dest: &str,
        username: &str,
    ) -> RowChange<usize> {
        let mut result = RowChange::NotFound;
        match self.storage.write() {
            Ok(mut vec) => {
                for i in 0..vec.len() {
                    if vec[i].alias == redirect_ref {
                        vec[i].set_destination(new_dest);
                        vec[i].update_username(Some(username));
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

    #[tracing::instrument(skip(self))]
    fn delete_redirect(&self, redirect_ref: &str) -> RowChange<usize> {
        let mut result = RowChange::NotFound;
        match self.storage.write() {
            Ok(mut vec) => {
                for i in 0..vec.len() {
                    if vec[i].alias == redirect_ref {
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

    #[tracing::instrument(skip(self))]
    fn get_all(&self, page: u64, limit: usize) -> RowChange<Vec<RedirectModel>> {
        let begin: usize = limit * page as usize;
        let end: usize = begin + limit;
        match self.storage.read() {
            Ok(v) => {
                let end = std::cmp::min(end, v.len());
                let data = v.get((begin)..(end)).unwrap_or_default().to_vec();
                RowChange::Value(data)
            }
            Err(e) => RowChange::Err(e.to_string()),
        }
    }
}
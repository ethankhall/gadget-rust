use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;


pub struct InternalDataStore {
    current_results: Arc<RefCell<BTreeMap<String, String>>>,
}

impl InternalDataStore {
    pub fn new() -> Self {
        InternalDataStore { current_results: Arc::new(RefCell::new(BTreeMap::new())) }
    }

    pub fn update(&self, values: BTreeMap<String, String>) {
        self.current_results.replace(values);
    }

    pub fn retrieve_lookup(&self, name: String) -> Option<String> {
        return match self.current_results.try_borrow() {
            Ok(val) => {
                return match val.get(name.as_str()) {
                    Some(result) => Some(format!("{}", result)),
                    None => None
                };
            }
            Err(_) => None
        };
    }
}

unsafe impl Sync for InternalDataStore {}

unsafe impl Send for InternalDataStore {}
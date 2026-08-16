use std::collections::HashMap;
use std::sync::RwLock;

use super::Action;

#[derive(Default)]
pub struct ActionStore {
    items: RwLock<HashMap<String, Action>>,
}

impl ActionStore {
    pub fn get(&self, id: &str) -> Option<Action> {
        self.items.read().expect("action store").get(id).cloned()
    }

    pub fn set(&self, action: Action) {
        self.items
            .write()
            .expect("action store")
            .insert(action.id().to_string(), action);
    }

    pub fn delete(&self, id: &str) {
        self.items.write().expect("action store").remove(id);
    }

    pub fn list(&self) -> Vec<Action> {
        self.items
            .read()
            .expect("action store")
            .values()
            .cloned()
            .collect()
    }

    pub fn filter(&self, pred: impl Fn(&Action) -> bool) -> Vec<Action> {
        self.list().into_iter().filter(|a| pred(a)).collect()
    }
}

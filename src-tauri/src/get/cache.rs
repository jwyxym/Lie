use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

static CACHE: LazyLock<RwLock<BTreeMap<String, String>>> =
	LazyLock::new(|| RwLock::new(BTreeMap::new()));

pub fn get(k: String) -> Option<String> {
    CACHE.read().unwrap().get(&k).cloned()
}

pub fn set(k: String, v: String) {
    CACHE.write().unwrap().insert(k, v);
}
use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

static CACHE: LazyLock<RwLock<BTreeMap<String, String>>> =
	LazyLock::new(|| RwLock::new(BTreeMap::new()));
static CACHE_BYTES: LazyLock<RwLock<BTreeMap<String, Vec<u8>>>> =
	LazyLock::new(|| RwLock::new(BTreeMap::new()));

pub fn get(k: String) -> Option<String> {
	CACHE.read().unwrap().get(&k).cloned()
}

pub fn set(k: String, v: String) {
	CACHE.write().unwrap().insert(k, v);
}

pub fn get_bytes(k: String) -> Option<Vec<u8>> {
	CACHE_BYTES.read().unwrap().get(&k).cloned()
}

pub fn set_bytes(k: String, v: Vec<u8>) {
	CACHE_BYTES.write().unwrap().insert(k, v);
}
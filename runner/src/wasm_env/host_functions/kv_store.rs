use crate::wasm_env::KvStore;

pub fn write(kv_store: &mut KvStore, key: String, data: Vec<u8>) -> Result<(), extism::Error> {
    kv_store.insert(key, data);
    Ok(())
}

pub fn read(kv_store: &KvStore, key: String) -> Result<Vec<u8>, extism::Error> {
    Ok(kv_store.get(&key).unwrap_or(&vec![]).clone())
}

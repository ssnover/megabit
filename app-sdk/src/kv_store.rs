use crate::host;

pub fn read<T: serde::de::DeserializeOwned>(
    key: impl Into<String>,
) -> Result<Option<T>, extism_pdk::Error> {
    let bytes = unsafe { host::kv_store_read(key.into())? };
    if !bytes.is_empty() {
        Ok(Some(rmp_serde::from_slice(&bytes[..])?))
    } else {
        Ok(None)
    }
}

pub fn write(
    key: impl Into<String>,
    value: impl serde::Serialize,
) -> Result<(), extism_pdk::Error> {
    let value_bytes = rmp_serde::to_vec(&value)?;
    unsafe { host::kv_store_write(key.into(), value_bytes)? };
    Ok(())
}

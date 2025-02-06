use std::{fs, path::Path};

pub trait ConverterOutput: Sized {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>>;
}

impl<T> ConverterOutput for T
where
    T: Into<String> + AsRef<Path>,
{
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let path = self.as_ref();
        let bytes = fs::read(path)?;
        Ok(bytes)
    }
}

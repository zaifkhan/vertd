use std::{fs, path::Path};

pub trait ConverterInput: Sized {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>>;
}

impl ConverterInput for String {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = fs::read(self)?;
        Ok(bytes)
    }
}

impl ConverterInput for &str {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = fs::read(self)?;
        Ok(bytes)
    }
}

impl ConverterInput for Vec<u8> {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.clone())
    }
}

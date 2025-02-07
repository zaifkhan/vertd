use super::format::ConverterFormat;

pub struct ConverterInput {
    pub format: ConverterFormat,
    pub bytes: Vec<u8>,
}

impl ConverterInput {
    pub fn new(format: ConverterFormat, bytes: Vec<u8>) -> Self {
        Self { format, bytes }
    }
}

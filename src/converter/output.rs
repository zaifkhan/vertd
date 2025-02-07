use super::format::ConverterFormat;

pub struct ConverterOutput {
    pub format: ConverterFormat,
}

impl ConverterOutput {
    pub fn new(format: ConverterFormat) -> Self {
        Self { format }
    }
}

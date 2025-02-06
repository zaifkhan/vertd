use input::ConverterInput;

mod input;
mod output;

pub struct Converter<I, O>
where
    I: ConverterInput,
    O: ConverterInput,
{
    pub input: I,
    pub output: O,
}

impl<I, O> Converter<I, O>
where
    I: ConverterInput,
    O: ConverterInput,
{
    pub fn new(input: I, output: O) -> Self {
        todo!()
    }

    pub fn convert(&self) -> anyhow::Result<O> {
        todo!()
    }
}

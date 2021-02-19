use crate::scalar_value::ScalarValue;

#[derive(Debug)]
pub enum BlockFieldType {
    Field,
    Input,
}

#[derive(Debug)]
pub enum BlockShape {
    Command,
    Reporter,
    Boolean,
    Hat,
}

#[derive(Debug)]
pub enum BlockInput<'spec> {
    Literal(ScalarValue),
    Reporter(Block<'spec>),
    Substack(usize),
}

#[derive(Debug)]
pub struct BlockSpec {
    pub name: &'static str,
    pub field_types: Box<[BlockFieldType]>,
    // used to map field names to indices. requires a linear search but still
    // faster than a hash map because blocks only have a few fields.
    pub field_names: Box<[String]>,
    pub shape: BlockShape,
}

#[derive(Debug)]
pub struct Block<'spec> {
    pub spec: &'spec BlockSpec,
    pub field_values: Box<[BlockInput<'spec>]>,
    pub next: Option<usize>,
    pub parent: Option<usize>,
}

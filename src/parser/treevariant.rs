#[derive(Debug)]
pub enum TreeVariant {
    Start,
    Name(String),
    StructDef,
    TArgsDef,
    TArgs,
    PropsDef,
    Prop,
    EnumDef,
    VariantsDef,
    PType,
    PTArgsBlock,
    PTArgs,
}
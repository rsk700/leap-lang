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
    Variant,
    PType,
    PTArgsBlock,
    PTArgs,
}
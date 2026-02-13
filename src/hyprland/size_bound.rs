use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter)]
pub enum SizeBound {
    #[default]
    Exact,
    Max,
    Min,
}

use super::private::Sealed;
pub trait Stage: Sealed {}

#[derive(Debug, Clone, Copy)]
pub struct Init;
#[derive(Debug, Clone, Copy)]
pub struct BeforePerturb;
#[derive(Debug, Clone, Copy)]
pub struct Perturbed;

use super::{BeforePerturb, Init, Perturbed, Stage};

pub trait Sealed {}
impl Sealed for Init {}
impl Sealed for BeforePerturb {}
impl Sealed for Perturbed {}
impl Stage for Init {}
impl Stage for BeforePerturb {}
impl Stage for Perturbed {}

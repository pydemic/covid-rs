pub use crate::agent::Ag;
pub use crate::epidemic::*;
pub use crate::pop_builder::PopBuilder;
pub use crate::simulation::Simulation;
pub use crate::reporter::{Report};
pub use crate::sampler::{Sampler, SimpleSampler, ContactMatrixSampler, AnySampler};

pub type Time = u32;
pub type Real = f64;
pub type Age = u8;
pub type AgeDistrib10 = [Real; 9];
pub type AgeCount10 = [u32; 9];

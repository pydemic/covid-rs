use serde::{Deserialize, Serialize};
// pub use crate::agent::Ag;
pub use crate::epidemic::*;
// pub use crate::pop_builder::PopBuilder;
// pub use crate::simulation::Simulation;
// pub use crate::reporter::{Report};
pub use crate::sampler::{AnySampler, ContactMatrixSampler, Sampler, SimpleSampler};

/// Basic representation of time. This crate usually assumes time is measured
/// in days.
pub type Time = u32;

/// Base Real type used by this crate. Uses an alias to easily change precision
/// if necessary.   
pub type Real = f64;
pub(crate) const INF: Real = Real::INFINITY;
pub(crate) const NAN: Real = Real::NAN;

/// Age of an agent.
pub type Age = u8;

/// An age distribution array in bins of 10 years.
pub type AgeDistribution10 = [Real; 9];

/// Count population in each bin of 10 years.
pub type AgeCount10 = [u32; 9];

/// Simple trait to simplify the use of age-dependent values/parameters.
pub trait ForAge<T> {
    /// Return the content of parameter for agents with the given age.
    fn for_age(&self, age: Age) -> T;
}

impl<T> ForAge<T> for T
where
    T: Copy,
{
    fn for_age(&self, age: Age) -> T {
        *self
    }
}

impl<T> ForAge<T> for [T; 9]
where
    T: Copy,
{
    fn for_age(&self, age: Age) -> T {
        self[(age / 10).max(8) as usize]
    }
}

/// A simple enumeration that may contain a scalar param or an AgeDistribution10
/// value
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgeParam {
    Scalar(Real),
    Distribution(AgeDistribution10),
}

impl Default for AgeParam {
    fn default() -> Self {
        AgeParam::Scalar(0.)
    }
}

impl ForAge<Real> for AgeParam {
    fn for_age(&self, age: Age) -> Real {
        match self {
            &AgeParam::Scalar(v) => v,
            &AgeParam::Distribution(ages) => ages.for_age(age),
        }
    }
}

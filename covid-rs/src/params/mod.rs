mod bound_params;
mod concrete_seir;
mod local_bind;
mod seir;
mod universal_seir;

pub use bound_params::*;
pub use concrete_seir::*;
pub use local_bind::*;
pub use seir::*;
pub use universal_seir::*;

use crate::prelude::{Age, Real};

pub type SEIRParamSet<T> = CachedSEIRParams<FullSEIRParams<T>, T>;
pub type SEIRUniversalParamSet = SEIRParamSet<Real>;

/// A type alias for bound age-dependent SEIR params that implements the
/// LocalBind trait.
pub type AgeDependentSEIR<T> = BoundParams<SEIRParamSet<T>, Age>;

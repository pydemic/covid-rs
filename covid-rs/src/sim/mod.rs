#[macro_use]
mod macros;

mod builder;
mod epicurve_reporter;
mod simulation;
mod epicurve_tracker;
mod manager;
mod population;
mod reporter;
mod state;
pub use builder::*;
pub use simulation::*;
pub use epicurve_reporter::*;
pub use epicurve_tracker::*;
pub use manager::*;
pub use population::*;
pub use reporter::*;
pub use state::*;

/// Type alias describing agent handles.
pub type Id = usize;

/// Iterator over ids.
pub trait Ids: Iterator<Item = Id> {}

/// Agent is just an opaque state with an Id handle. There are no trait bounds
/// and you usually should implement functionality into the state rather than
/// directly on agents.
#[derive(Debug, Copy, Clone, Default)]
pub struct Agent<S> {
    pub id: Id,
    pub state: S,
}

use rand::Rng;

use crate::prelude::{Age, EpiModel, Params};
use std::fmt::Debug;

/// Minimal bounds on agent States.
pub trait State: Send + Clone + PartialEq + Debug + Default {} // Sync?
pub trait World: Send + Clone + PartialEq + Debug + Default {} // Sync?

/// A trait for agents that have an age field. Many epidemiological models are
/// age-sensitive and it is useful to isolate this property as trait to implement
/// generic functionality associated with processing ages.
pub trait HasAge: State {
    /// Agent's age.
    fn age(&self) -> Age;

    /// Set age with given value.
    fn set_age(&mut self, value: Age) -> &mut Self;
}

/// A trait for objects that have an compartment field with a SIR value.
pub trait HasEpiModel: State {
    type Model: EpiModel;

    /// Return the epidemiological compartment.
    fn epimodel(&self) -> Self::Model;

    /// Set value of the epidemiological state.
    fn set_epimodel(&mut self, value: Self::Model) -> &mut Self;
}

/// A trait for objects that can be deterministically updated in the given
/// World.
pub trait Update<W: World>: State {
    fn update(&mut self, world: &W);
}

/// A trait for objects that can be stochastically updated in the given
/// World. Users must pass a random number generator in order to update
// this object
pub trait StochasticUpdate<W: World>: State {
    fn update_random<R: Rng>(&mut self, world: &W, rng: &mut R);
}

/////////////////////////////////////////////////////////////////////////////
// Default trait implementations
/////////////////////////////////////////////////////////////////////////////

impl State for () {}

impl<W: World> Update<W> for () {
    fn update(&mut self, _world: &W) {}
}

impl<W: World> StochasticUpdate<W> for () {
    fn update_random<R: Rng>(&mut self, _world: &W, _: &mut R) {}
}

impl<T> StochasticUpdate<Params> for T
where
    T: HasEpiModel,
    T::Model: StochasticUpdate<Params>,
{
    fn update_random<R: Rng>(&mut self, params: &Params, rng: &mut R) {
        let mut value = self.epimodel();
        value.update_random(params, rng);
        self.set_epimodel(value);
    }
}

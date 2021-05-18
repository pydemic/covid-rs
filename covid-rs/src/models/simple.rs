use getset::{CopyGetters, Getters};
use rand::Rng;

use crate::{
    epidemic::EpiModel,
    prelude::{Age, AgeDistribution10, Real, Time},
    sim::{HasAge, HasEpiModel, Population, RandomUpdate},
    utils::random_ages,
};

/// A simple agent with an age, epidemic model and vaccine model.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Getters, CopyGetters)]
pub struct SimpleAgent<M, V> {
    age: Age,
    #[getset(get = "pub")]
    epimodel: M,
    #[getset(get_copy = "pub")]
    epimodel_t: Time,
    #[getset(get = "pub")]
    vaccine: V,
    #[getset(get_copy = "pub")]
    vaccine_t: Time,
}

impl<M, V> HasAge for SimpleAgent<M, V> {
    fn age(&self) -> Age {
        return self.age;
    }

    fn set_age(&mut self, value: Age) -> &mut Self {
        self.age = value;
        return self;
    }
}

impl<M: EpiModel, V> HasEpiModel for SimpleAgent<M, V> {
    type Model = M;

    fn epimodel(&self) -> &Self::Model {
        &self.epimodel
    }

    fn epimodel_mut(&mut self) -> &mut Self::Model {
        &mut self.epimodel
    }

    fn set_epimodel(&mut self, value: Self::Model) -> &mut Self {
        self.epimodel = value;
        return self;
    }
}

impl<M, V, W> RandomUpdate<W> for SimpleAgent<M, V>
where
    Self: HasEpiModel<Model = M>,
    M: RandomUpdate<W> + EpiModel,
{
    default fn random_update<R: Rng>(&mut self, world: &W, rng: &mut R) {
        self.epimodel_random_update(world, rng);
        self.epimodel_t += 1;
        self.vaccine_t += 1;
    }
}

///////////////////////////////////////////////////////////////////////////////
// Extend population with trait
///////////////////////////////////////////////////////////////////////////////
pub trait SimpleAgentPopulationExt<M, V>: Population<State = SimpleAgent<M, V>>
where
    M: Clone,
    V: Clone,
{
    /// Set age of all agents to the given value
    fn set_ages(&mut self, value: Age) -> &mut Self {
        self.each_agent_mut(|_, ag| ag.age = value);
        return self;
    }

    /// Rewrite age of all agents according to the given age distribution.
    fn distrib_ages<R: Rng>(&mut self, distrib: AgeDistribution10, rng: &mut R) -> &mut Self {
        let ages = random_ages(self.count(), rng, distrib);
        let mut i = 0;
        self.each_agent_mut(move |_, ag| {
            ag.age = ages[i];
            i += 1;
        });

        return self;
    }

    /// Set vaccines to the given value
    fn set_vaccines(&mut self, value: V) -> &mut Self {
        self.each_agent_mut(|_, ag| ag.vaccine = value.clone());
        return self;
    }

    /// Vaccinate all individuals with the given vaccine and uniform
    /// probability
    fn vaccinate_random<R: Rng>(&mut self, value: V, prob: Real, rng: &mut R) -> &mut Self {
        self.each_agent_mut(|_, ag| {
            if rng.gen_bool(prob) {
                ag.vaccine = value.clone()
            }
        });
        return self;
    }

    /// Vaccinate all individuals older than the given age.
    fn vaccinate_elderly<R: Rng>(&mut self, value: V, prob: Real, rng: &mut R) -> &mut Self {
        self.each_agent_mut(|_, ag| {
            if rng.gen_bool(prob) {
                ag.vaccine = value.clone()
            }
        });
        return self;
    }
}

impl<P, M, V> SimpleAgentPopulationExt<M, V> for P
where
    P: Population<State = SimpleAgent<M, V>>,
    M: Clone,
    V: Clone,
{
}

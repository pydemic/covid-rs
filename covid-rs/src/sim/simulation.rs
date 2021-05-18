use crate::{
    epidemic::*,
    params::LocalBind,
    prelude::*,
    sim::{Reporter, *},
};
use getset::{Getters, Setters};
use rand::prelude::{SeedableRng, SmallRng};
use std::{cell::RefCell, fmt::Debug};

/// Simulation stores a population of agents and some objects responsible for
/// controlling the dynamics of those Agents.
#[derive(Getters)]
pub struct Simulation<W, S, PS, const N: usize> {
    #[getset(get = "pub")]
    population: Vec<S>,
    #[getset(get = "pub")]
    infections_per_agent: Vec<u16>,
    #[getset(get = "pub")]
    infections_per_iter: Vec<usize>,
    #[getset(get = "pub")]
    params: W,

    sampler: PS,
    reporter: EpicurveReporter<W, Vec<S>, { N }>,
    world_update: Vec<Box<dyn FnMut(&mut W, &Vec<S>)>>,
    population_update: Vec<Box<dyn FnMut(&W, &mut Vec<S>)>>,
    rng: RefCell<SmallRng>,
}

impl<'a, W, S, PS, const N: usize> Simulation<W, S, PS, { N }>
where
    // Self: 'a,
    PS: Sampler<Vec<S>>,
    W: LocalBind<S>,
    S: EpiModel + RandomUpdate<W::Local> + Debug,
{
    /// Create new simulation from population and sampler.
    pub fn new(params: W, population: Vec<S>, sampler: PS) -> Self {
        Simulation {
            reporter: EpicurveReporter::new(&population),
            infections_per_agent: vec![0].repeat(population.len()),
            infections_per_iter: vec![],
            population,
            params,
            sampler,
            world_update: vec![],
            population_update: vec![],
            rng: RefCell::new(SmallRng::from_entropy()),
        }
    }

    /// Return a copy of simulation ignoring local reporters and update
    /// functions
    pub fn copy(&self) -> Self
    where
        W: Clone,
        PS: Clone,
    {
        Simulation {
            population: self.population.clone(),
            infections_per_agent: self.infections_per_agent.clone(),
            infections_per_iter: self.infections_per_iter.clone(),
            params: self.params.clone(),
            sampler: self.sampler.clone(),
            reporter: self.reporter.copy(),
            world_update: vec![],
            population_update: vec![],
            rng: self.rng.clone(),
        }
    }

    /// Run simulation for the given number of steps.
    pub fn run(&mut self, n_steps: usize) {
        for n in 0..n_steps {
            // Default updates
            self.update_agents();
            self.update_pairs();

            // Arbitrary updates
            for f in self.population_update.iter_mut() {
                f(&self.params, &mut self.population);
            }
            for f in self.world_update.iter_mut() {
                f(&mut self.params, &self.population);
            }
            self.reporter.process(n, &self.params, &self.population)
        }
    }

    /// Self-update agents. Resolve the natural evolution of all agents
    fn update_agents(&mut self) {
        let rng = &mut *self.rng.borrow_mut();
        for obj in self.population.iter_mut() {
            self.params.bind_to_object(obj);
            obj.random_update(self.params.local(), rng);
        }
    }

    /// Simulate agent interactions, allowing new infections to occur.
    fn update_pairs(&mut self) {
        let rng = &mut *self.rng.borrow_mut();
        let mut cases = 0usize;

        for (i, j) in self.sampler.sample_infection_pairs(&self.population, rng) {
            if i == j {
                continue;
            }
            if let Some((src, dest)) = self.population.get_pair_mut(i, j) {
                if dest.contaminate_from(src) {
                    cases += 1;
                    self.infections_per_agent[i] += 1;
                }
            }
        }
        self.infections_per_iter.push(cases);
    }

    /// Return a sample of n agents
    pub fn sample(&self, n: usize) -> Vec<S> {
        let rng = &mut *self.rng.borrow_mut();
        let mut sample = Vec::with_capacity(n);
        for (_, ag) in self.population.randoms(n, rng) {
            sample.push(ag.clone());
        }
        return sample;
    }

    /// Render the epicurve for the current simulation
    pub fn render_epicurve_csv(&self, head: &str) -> String {
        self.reporter.render_epicurve_csv(head)
    }
}

impl<W, S, const N: usize> Simulation<W, S, SimpleSampler, { N }>
where
    W: LocalBind<S>,
    S: RandomUpdate<W::Local> + EpiModel + Debug,
{
    /// Create a new simulation from a simple sampler
    pub fn new_simple(
        params: W,
        population: Vec<S>,
        n_contacts: Real,
        prob_infection: Real,
    ) -> Self {
        let sampler = SimpleSampler::new(n_contacts, prob_infection);
        return Self::new(params, population, sampler);
    }
}

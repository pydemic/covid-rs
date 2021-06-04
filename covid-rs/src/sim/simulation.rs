use super::{
    population::{OwnsStateSlice, Population},
    state::RandomUpdate,
};
use crate::{
    epidemic::*,
    params::{EpiParamsFull, EpiParamsLocalT, FromLocalParams, LocalBind},
    prelude::*,
    trackers::{EpiTracker, Tracker},
};
use getset::{Getters, MutGetters};
use log::{debug, trace};
use rand::prelude::{SeedableRng, SmallRng};
use std::{cell::RefCell, fmt::Debug};

/// Simulation stores a population of agents and some objects responsible for
/// controlling the dynamics of those Agents.
#[derive(Getters, MutGetters)]
pub struct Simulation<W, S, PS> {
    #[getset(get = "pub", get_mut = "pub")]
    population: Vec<S>,
    #[getset(get = "pub")]
    infections_per_agent: Vec<u16>,
    #[getset(get = "pub")]
    infections_per_iter: Vec<usize>,
    #[getset(get = "pub", get_mut = "pub")]
    params: RefCell<W>,

    #[getset(get = "pub", get_mut = "pub")]
    sampler: PS,
    reporter: EpiTracker<Vec<S>>,
    world_update: Vec<Box<dyn FnMut(&mut W, &Vec<S>)>>,
    population_update: Vec<Box<dyn FnMut(&W, &mut Vec<S>)>>,
    rng: RefCell<SmallRng>,
}

impl<'a, W, S, PS> Simulation<W, S, PS>
where
    PS: PopulationSampler<Vec<S>>,
    W: LocalBind<S>,
    S: EpiModel + RandomUpdate<W::Local> + Debug,
{
    /// Create new simulation from population and sampler.
    pub fn new(params: W, population: Vec<S>, sampler: PS) -> Self {
        Simulation {
            reporter: EpiTracker::new(&population),
            infections_per_agent: vec![0].repeat(population.len()),
            infections_per_iter: vec![],
            population,
            params: RefCell::new(params),
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

    /// Run simulation for the given number of steps and return the number of
    /// new cases.
    pub fn steps(&mut self, n_steps: usize) -> usize {
        let mut cases = 0;
        for _ in 0..n_steps {
            // Default updates
            self.update_agents();
            cases += self.update_pairs();

            // Arbitrary updates
            let mut params = self.params.borrow_mut();
            for f in self.population_update.iter_mut() {
                f(&params, &mut self.population);
            }
            for f in self.world_update.iter_mut() {
                f(&mut params, &self.population);
            }
            self.reporter.track(&self.population);
        }

        return cases;
    }

    /// Like steps, but return Self, rather then the number of cases. This is
    /// useful to use builder-like APIs.
    #[inline]
    pub fn run(&mut self, n_steps: usize) -> &mut Self {
        self.steps(n_steps);
        return self;
    }

    /// Self-update agents. Resolve the natural evolution of all agents
    fn update_agents(&mut self) {
        let rng = &mut *self.rng.borrow_mut();
        let mut params = self.params.borrow_mut();
        for obj in self.population.iter_mut() {
            params.bind_to_object(obj);
            obj.random_update(params.local(), rng);
        }
    }

    /// Simulate agent interactions, allowing new infections to occur.
    fn update_pairs(&mut self) -> usize {
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
        return cases;
    }

    /// Set seed for random number generator
    pub fn seed(&mut self, seed: u64) -> &mut Self {
        self.rng.replace(SmallRng::seed_from_u64(seed));
        return self;
    }

    /// Set seed for random number generator
    pub fn seed_from(&mut self, rng: &SmallRng) -> &mut Self {
        self.rng.replace(rng.clone());
        return self;
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

    /// Population size
    pub fn count(&self) -> usize {
        self.population.len()
    }

    /// Return the tip of the epicurve
    pub fn epistate(&self, normalize: bool) -> Vec<Real> {
        let factor = self.normalization_factor(normalize);
        self.reporter
            .tip()
            .iter()
            .map(|a| *a as Real * factor)
            .collect()
    }

    /// Return curve for the n-th component of epicurve.
    ///
    /// Normalize control if results will be divided by population size.
    pub fn get_epicurve(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        self.reporter.col(n).map(|data| {
            let mut vec = Vec::with_capacity(data.len());
            let factor = self.normalization_factor(normalize);
            for x in data {
                vec.push(x as Real * factor);
            }
            return vec;
        })
    }

    /// Get epistate at a given iteration
    pub fn get_epistate(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        let factor = self.normalization_factor(normalize);
        let row = self.reporter.row(n)?;
        return Some(row.iter().map(|x| *x as Real * factor).collect());
    }

    /// Render the epicurve for the current simulation
    pub fn render_epicurve_csv(&self, head: &str) -> String {
        let mut head = head.to_string();
        let mut infections = vec![0];
        infections.extend(self.infections_per_iter.iter());
        head.push_str(",cases");
        return self
            .reporter
            .epicurves()
            .with_column(infections.iter().cloned(), true)
            .render_csv(&head, ',');
    }

    /// Used internally to normalize (or not) results
    fn normalization_factor(&self, normalize: bool) -> Real {
        if normalize {
            1.0 / self.count() as Real
        } else {
            1.0
        }
    }

    /// Initialize simulation and calibrate sampler from a curve of cases.
    ///
    /// This is a somewhat simplistic view on model calibration. We just run
    /// the simulation normally but at each step we recalibrate the sampler
    /// to produce the same number of infections as expected from the epidemic
    /// curve.
    pub fn calibrate_sampler_from_cases(&mut self, cases: &[Real]) -> &mut Self
    where
        S::Clinical: Default,
    {
        // TODO: create calibrator struct
        let alpha = 0.5;
        let min_contacts = 0.0;
        let max_contacts = 10.0;
        let min_scale = 1.0 / 1.5;
        let max_scale = 1.33;
        let e_ratio = 0.25;
        let r = 0.85;

        let mut n_iter = 0;
        let mut excess = 0.0;
        let mut acc_cases = 0.0;
        let mut acc_target = 0.0;
        let mut c_mean = self.sampler.contacts();

        for raw_target in cases {
            n_iter += 1;
            acc_target += raw_target;

            let target = (raw_target + e_ratio * excess).max(0.0);
            let estimate = self.sampler.expected_infection_pairs(&self.population);
            let grow = ((target + alpha) / (estimate + alpha)).clamp(min_scale, max_scale);

            // Calibrate contacts. Other implementations might calibrate different
            // coefficients, but we do not have any way to generalize it yet.
            let c1 = self.sampler.contacts();
            let c2 = (c1 * grow).clamp(min_contacts, max_contacts);
            c_mean = c_mean * r + c2 * (1.0 - r);
            self.sampler.set_contacts(c2);

            // Run and register the number of cases
            let n_cases = self.steps(1);
            acc_cases += n_cases as Real;
            excess = acc_target - acc_cases as Real;

            // If excess is very large (very negative), we might want to create
            // artificial infections to quickstart a infection
            if excess > 0.25 * (acc_target + alpha) {
                let n = (excess * 0.25) as usize;
                self.population
                    .contaminate_at_random(n, &mut *self.rng.borrow_mut());
                acc_cases += n as Real;
                excess = acc_target - acc_cases as Real;
            }

            trace!(target: "calibrate_sample_cases", "iter {}, coeff: {:.2} ({:.2})\n  - target: {} ({}); cases: {} (~ {:.1}); excess: {}", n_iter, c2, c_mean, raw_target, target, n_cases, estimate, excess);
        }
        self.sampler.set_contacts(c_mean);
        debug!(target: "calibrate_sample_cases", "final contacts: {}, {} iterations", c_mean, n_iter);
        return self;
    }

    /// Get epidemiological params for given agent
    ///
    /// Return Some(FullSEIRParams<f64>) if agent exists.
    pub fn get_local_epiparams(&self, i: usize) -> Option<EpiParamsFull<f64>>
    where
        S: EpiModel,
        W::Local: EpiParamsLocalT,
    {
        let ag = self.population.get(i)?;
        let mut params = self.params.borrow_mut();
        params.bind_to_object(ag);
        Some(FromLocalParams::from_local_params(params.local()))
    }
}

impl<W, S> Simulation<W, S, SimpleSampler>
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

impl<W, S, PS> OwnsStateSlice for Simulation<W, S, PS>
where
    PS: PopulationSampler<Vec<S>> + Default,
    W: LocalBind<S> + Default,
    S: EpiModel + RandomUpdate<W::Local> + Debug,
{
    type Elem = S;

    fn owned_data_from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = S>,
    {
        let mut population = vec![];
        population.extend(states);
        Self::new(Default::default(), population, Default::default())
    }

    fn as_state_slice(&self) -> &[S] {
        self.population.as_slice()
    }

    fn as_state_mut_slice(&mut self) -> &mut [S] {
        self.population.as_mut_slice()
    }
}

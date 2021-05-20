use crate::{
    epidemic::*,
    params::{FromUniversalParams, FullSEIRParams, LocalBind, UniversalSEIRParams},
    prelude::*,
    sim::{Reporter, *},
};
use getset::{Getters, MutGetters};
use rand::prelude::{SeedableRng, SmallRng};
use std::{cell::RefCell, fmt::Debug};

/// Simulation stores a population of agents and some objects responsible for
/// controlling the dynamics of those Agents.
#[derive(Getters, MutGetters)]
pub struct Simulation<W, S, PS, const N: usize> {
    #[getset(get = "pub")]
    population: Vec<S>,
    #[getset(get = "pub")]
    infections_per_agent: Vec<u16>,
    #[getset(get = "pub")]
    infections_per_iter: Vec<usize>,
    #[getset(get = "pub")]
    params: RefCell<W>,

    #[getset(get = "pub", get_mut = "pub")]
    sampler: PS,
    reporter: EpicurveReporter<W, Vec<S>, { N }>,
    world_update: Vec<Box<dyn FnMut(&mut W, &Vec<S>)>>,
    population_update: Vec<Box<dyn FnMut(&W, &mut Vec<S>)>>,
    rng: RefCell<SmallRng>,
}

impl<'a, W, S, PS, const N: usize> Simulation<W, S, PS, { N }>
where
    PS: PopulationSampler<Vec<S>>,
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

    /// Run simulation for the given number of steps.
    pub fn run(&mut self, n_steps: usize) -> &mut Self {
        for n in 0..n_steps {
            // Default updates
            self.update_agents();
            self.update_pairs();

            // Arbitrary updates
            let mut params = self.params.borrow_mut();
            for f in self.population_update.iter_mut() {
                f(&params, &mut self.population);
            }
            for f in self.world_update.iter_mut() {
                f(&mut params, &self.population);
            }
            self.reporter.process(n, &params, &self.population)
        }

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
    pub fn epistate(&self, normalize: bool) -> [Real; N] {
        let factor = self.normalization_factor(normalize);
        self.reporter.tip().map(|a| a as Real * factor)
    }

    /// Return curve for the n-th component of epicurve.
    ///
    /// Normalize control if results will be divided by population size.
    pub fn get_epicurve(&self, n: usize, normalize: bool) -> Option<Vec<Real>> {
        self.reporter.col(n).map(|data| {
            let mut vec = Vec::with_capacity(data.len());
            let factor = self.normalization_factor(normalize);
            for &x in data {
                vec.push(x as Real * factor);
            }
            return vec;
        })
    }

    /// Get epistate at a given iteration
    pub fn get_epistate(&self, n: usize, normalize: bool) -> Option<[Real; N]> {
        let factor = self.normalization_factor(normalize);
        self.reporter.row(n).map(|a| a.map(|x| x as Real * factor))
    }

    /// Render the epicurve for the current simulation
    pub fn render_epicurve_csv(&self, head: &str) -> String {
        self.reporter.render_epicurve_csv(head)
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
    ///
    ///  
    pub fn calibrate_sampler_from_cases(&mut self, cases: &[Real]) {
        // TODO: create calibrator struct
        let alpha = 0.5;
        let min_contacts = 0.0;
        let max_contacts = 10.0;
        let min_scale = 0.75;
        let max_scale = 1.25;
        let e_ratio = 0.5;
        let mut error = 0.0;

        for raw_target in cases {
            let target = (raw_target - e_ratio * error).max(0.0);
            let estimate = self.sampler.expected_infection_pairs(&self.population);
            let grow = ((target + alpha) / (estimate + alpha)).clamp(min_scale, max_scale);

            // Calibrate contacts. Other implementations might calibrate different
            // coefficients, but we do not have any way to generalize it yet.
            let c1 = self.sampler.contacts();
            let c2 = (c1 * grow).clamp(min_contacts, max_contacts);
            self.sampler.set_contacts(c2);

            // Run and register the number of cases
            self.run(1);
            let n_cases = *self.infections_per_iter().last().unwrap();
            error += n_cases as Real - raw_target;
        }
    }

    /// Get epidemiological params for given agent
    ///
    /// Return Some(FullSEIRParams<f64>) if agent exists.
    pub fn get_local_epiparams(&self, i: usize) -> Option<FullSEIRParams<f64>>
    where
        S: EpiModel,
        W::Local: UniversalSEIRParams,
    {
        let ag = self.population.get(i)?;
        let mut params = self.params.borrow_mut();
        params.bind_to_object(ag);
        Some(FromUniversalParams::from_universal_params(params.local()))
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

impl<W, S, PS, const N: usize> OwnsStateSlice for Simulation<W, S, PS, { N }>
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

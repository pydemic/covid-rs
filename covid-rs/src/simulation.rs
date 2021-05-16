use log::*;
use std::cell::RefCell;

use crate::{
    agent::Ag,
    epidemic::{Params, VariantSEICHAR},
    iter::AgentsIter,
    pop::Pop,
    prelude::*,
    sampler::Sampler,
    utils::{PointStatsAcc, Stats, StatsVec},
};
use getset::{CopyGetters, Getters, Setters};
use rand::prelude::*;
use rayon::prelude::*;

#[derive(Debug, CopyGetters, Getters, Setters)]
pub struct Simulation<S: Sampler<Pop>> {
    agents: Pop,
    sampler: S,

    #[getset(get = "pub")]
    curves: Vec<[usize; VariantSEICHAR::CARDINALITY]>,

    #[getset(get = "pub")]
    stats: SimulationStats,

    #[getset(get_copy = "pub")]
    n_iter: usize,
    pub(crate) rng: RefCell<SmallRng>,

    #[getset(get = "pub", set = "pub")]
    params_baseline: Params,

    #[getset(get = "pub", set = "pub")]
    params_voc: Params,

    #[getset(get = "pub", set = "pub")]
    parallel: bool,
}

impl<S: Sampler<Pop>> Simulation<S> {
    /// Create new simulation from list of agents and sampler object.
    /// The list of agents is usually created with a population builder object.
    /// The sampler defines the basic simulation strategy for choosing new
    /// pairs of infections.
    pub fn new(data: Vec<Ag>, sampler: S) -> Self {
        let mut new = Simulation {
            agents: Pop::new_data(data.clone()),
            sampler: sampler,
            curves: vec![],
            n_iter: 0,
            stats: SimulationStats::default(),
            rng: RefCell::new(SmallRng::from_entropy()),
            params_baseline: Params::default(),
            params_voc: Params::default(),
            parallel: false,
        };
        new.sampler.init(&mut new.agents);
        return new;
    }

    /// Borrows slice with all agents
    pub fn agents(&self) -> &[Ag] {
        self.agents.as_slice()
    }

    /// Mutably borrows slice with all agents
    pub fn agents_mut(&mut self) -> &mut [Ag] {
        self.agents.as_mut_slice()
    }

    /// Run n steps of simulation.
    pub fn run(&mut self, n: usize) {
        let mut cases;

        for _ in 0..n {
            self.update_agents();
            {
                let rng = &mut *self.rng.borrow_mut();
                cases = self
                    .agents
                    .contaminate_from_sampler(None.into(), &self.sampler, rng);
            }
            self.on_step_finish(cases);
            self.n_iter += 1;
        }
    }

    /// Compute and store simulation statistics for each step.
    fn on_step_finish(&mut self, new_infections: usize) {
        info!(
            "step [{}]: {} infections, R0 = {:.2}",
            self.n_iter,
            self.stats.infections.last(),
            self.stats.r0.last()
        );
        self.curves.push(self.curve_tip());
        self.stats.infections.add(new_infections as Real);
        self.stats.r0.add(self.agents.iter().r0());
    }

    /// Compute the tip of epicurves.
    fn curve_tip(&self) -> [usize; VariantSEICHAR::CARDINALITY] {
        let mut stat = [0; VariantSEICHAR::CARDINALITY];
        for agent in self.agents.iter() {
            stat[agent.state().index()] += 1;
        }
        return stat;
    }

    /// Return curve for the n-th component of statistics
    pub fn curve(&self, n: usize) -> Vec<Real> {
        let pop_size = self.agents.len() as Real;
        if n >= VariantSEICHAR::CARDINALITY {
            self.curves.iter().map(|_| 0.0).collect()
        } else {
            self.curves
                .iter()
                .map(|stat| (stat[n] as Real) / pop_size)
                .collect()
        }
    }

    /// Set seed for random number generator
    pub fn seed(&mut self, seed: u64) {
        self.rng.replace(SmallRng::seed_from_u64(seed));
    }
}

impl<S: Sampler<Pop>> Simulation<S> {
    fn update_agents(&mut self) {
        if self.parallel {
            self.update_agents_parallel()
        } else {
            let rng = &mut *self.rng.borrow_mut();
            self.agents
                .update(rng, &self.params_baseline, &self.params_voc)
        }
    }

    fn update_agents_parallel(&mut self) {
        let params = &self.params_baseline;
        let params_voc = &self.params_baseline;
        let global_rng = &mut self.rng.borrow_mut().clone();
        {
            // let lock = Mutex::new(&rng);
            self.agents
                .as_mut_slice()
                .par_iter_mut()
                .for_each(move |agent| {
                    let mut rng = global_rng.clone();
                    agent.update(&mut rng, params, params_voc);
                });
        }
    }
}

trait RngCell {
    fn cell(&self) -> &RefCell<SmallRng>;

    fn with_rng<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut SmallRng) -> T,
    {
        let cell = self.cell();
        let rng = &mut *cell.borrow_mut();
        f(rng)
    }
}

impl<T: Sampler<Pop>> RngCell for Simulation<T> {
    fn cell(&self) -> &RefCell<SmallRng> {
        &self.rng
    }
}

#[derive(Debug, Default, Clone)]
pub struct SimulationStats {
    pub(crate) infections: PointStatsAcc,
    pub(crate) r0: StatsVec,
}

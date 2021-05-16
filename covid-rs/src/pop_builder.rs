use crate::{
    agent::Infect,
    pop::Pop,
    prelude::*,
    sampler::*,
    sim::{EpicurveTracker, HasAge, Population, StochasticUpdate},
    utils::*,
};
use getset::{CopyGetters, Getters, Setters};
use log::*;
use rand::prelude::*;

#[derive(Debug, Getters, Setters, CopyGetters)]
pub struct PopBuilder<P: Population> {
    data: P,

    #[getset(get_copy = "pub", set = "pub")]
    prob_voc: Real,
    rng: SmallRng,
}

impl<P: Population + Clone> PopBuilder<P> {
    pub fn new(n: usize) -> Self {
        PopBuilder {
            data: P::from_states((0..n).map(|_| P::State::default())),
            prob_voc: 0.5,
            rng: SmallRng::from_entropy(),
        }
    }

    /// Helper function that creates an agent. It does not add agent to
    /// population.
    fn new_agent(&self) -> P::State {
        P::State::default()
    }

    /// Seed the population builder RNG with a predictable value
    pub fn seed(&mut self, seed: u64) -> &mut Self {
        self.rng = SmallRng::seed_from_u64(seed);
        return self;
    }

    /// Return a copy of internal agents buffer.
    pub fn agents(&self) -> Vec<P::State> {
        return self.data.to_states();
    }

    /// Assign random ages to agents from distribution
    pub fn age_counts(&mut self, counts: AgeCount10) -> &mut Self
    where
        P::State: HasAge,
    {
        let mut data = vec![];
        for (i, &n) in counts.iter().enumerate() {
            for _ in 0..n {
                let mut st = self.new_agent();
                let start = i * 10;
                st.set_age(self.rng.gen_range(start..start + 10) as Age);
                data.push(st);
            }
        }
        self.data = P::from_states(data);
        return self;
    }
}
impl PopBuilder<Pop> {
    /// Register sampler to build simulation
    pub fn build<S: Sampler<Pop>>(&self, sampler: S) -> Simulation<S> {
        let sim = Simulation::new(self.data.clone().into(), sampler);
        sim.rng.replace(self.rng.clone());
        return sim;
    }

    /// Register sampler to build simulation
    pub fn build_simple(
        &self,
        n_contacts: Real,
        prob_infection: Real,
    ) -> Simulation<SimpleSampler> {
        self.build(SimpleSampler::new(n_contacts, prob_infection))
    }

    /// Add a clone of a single agent to builder.
    pub fn push_agent(&mut self, agent: &Ag) -> &mut Self {
        self.data.push(agent);
        return self;
    }

    /// Add a copy of agents to list
    pub fn push_agents<I>(&mut self, agents: I) -> &mut Self
    where
        I: Iterator<Item = Ag>,
    {
        for agent in agents {
            self.data.push(&agent);
        }
        return self;
    }

    /// Contaminate n individuals at random with some variant
    pub fn contaminate_at_random(&mut self, n: usize, infect: Infect) -> &mut Self {
        for i in self.data.sample_agents(n, &mut self.rng) {
            let variant = Variant::random(&mut self.rng, self.prob_voc);
            self.data[i].contaminate(variant, infect);
        }
        return self;
    }

    /// Infect individuals from epicurve. It receives a mutable Stats object
    /// used to trace statistics from infection probability at each iteration.
    pub fn contaminate_from_epicurve(
        &mut self,
        epicurve: &[usize],
        sampler: &AnySampler,
        params: &Params,
        smoothness: Real,
        stats: &mut impl Stats,
    ) -> &mut Self {
        let mut builder = EpicurveBuilder::new(self.data.as_slice(), params, smoothness, sampler);
        builder.run(epicurve, stats);
        self.data = builder.data;
        return self;
    }
}

/// ## Modify population
///
/// Methods in this section modify population inplace after it is created
/// either setting a size in the constructor or by pushing agents using other
/// methods.
impl<P: Population> PopBuilder<P> {
    /// Assign random ages to agents from distribution
    pub fn age_distribution(&mut self, probs: AgeDistrib10) -> &mut Self
    where
        P::State: HasAge,
    {
        let mut ages = random_ages(self.data.count(), &mut self.rng, probs);
        self.data.each_agent_mut(|i, st: &mut P::State| {
            if let Some(age) = ages.pop() {
                st.set_age(age);
            }
        });
        return self;
    }
}

#[derive(Debug, Clone)]
struct EpicurveBuilder<P, S> {
    data: P,
    rng: SmallRng,
    params: Params,
    smoothness: Real,
    sampler: S,
    cases_pid: PID<Real>,
    ratio_pid: PID<Real>,
}

impl<P, S> EpicurveBuilder<P, S>
where
    P: Population,
    P::State: SIRLike,
    S: Sampler<P> + Clone,
{
    fn new(data: &[P::State], params: &Params, smoothness: Real, sampler: &S) -> Self {
        EpicurveBuilder {
            data: P::from_states(Vec::from(data)),
            rng: SmallRng::from_entropy(),
            params: params.clone(),
            smoothness: smoothness,
            sampler: sampler.clone(),
            cases_pid: PID::new(-0.5, -0.25, -0.5),
            ratio_pid: PID::new(-1.0, -0.25, -1.25),
        }
    }

    fn run<ST: Stats>(&mut self, epicurve: &[usize], stats: &mut ST)
    where
        P::State: StochasticUpdate<Params>,
    {
        if epicurve.len() == 0 {
            return;
        }
        let n0 = epicurve[0];
        let pop_size = self.data.count();
        let rng = &mut self.rng;
        let pop = &mut self.data;
        let mut epicurves = EpicurveTracker::<usize, 7>::new();

        // Seed population with infections
        pop.contaminate_at_random(n0, rng, |_, st| {
            st.infect();
            return true;
        });
        epicurves.update(pop, false);

        // The following loop uses a PID-like controller to keep the ratio between
        // desired cases and simulated ones equal to 1.0. It infect agents at
        // random

        let (a, b) = (self.smoothness, 1.0 - self.smoothness);
        let mut prob = self.sampler.prob_infection();
        let mut cases_acc = 0;
        let mut n_acc = 0;

        for &n in epicurve.iter() {
            n_acc += n;
            pop.update_random(&self.params, rng);
            let mut extra_cases = 0;
            let mut cases = pop.update_sampler_with(&self.sampler, rng, |src, dest| {
                let mut out = dest.contaminated_from(src)?;
                out.infect();
                return Some(out);
            });
            cases_acc += cases;

            // Add extra cases
            {
                let err = (cases as i32 - n as i32) as f64;
                let extra = self
                    .cases_pid
                    // .feedback(a * self.cases_pid.error() + b * err, 1.0);
                    .feedback(err.into(), 1.0);
                trace!("extra = {}", extra);
                if extra > 0.0 {
                    extra_cases = pop.contaminate_at_random(extra as usize, rng, |_, st| {
                        st.infect();
                        return true;
                    });
                    self.cases_pid
                        .set_acc(self.cases_pid.acc() + extra_cases as Real);
                    cases_acc += extra_cases;
                    cases += extra_cases;
                }
            }

            // Update probabilities with PID
            let ratio = ((n as f64 + self.cases_pid.error()).max(0.0) + 0.5) / (n as f64 + 0.5);
            {
                let u = self.ratio_pid.feedback(ratio.ln(), 1.0);
                let logp = self.sampler.prob_infection().ln() + u;
                let prob_i = logp.exp().min(1.0);
                self.sampler.set_prob_infection(prob_i);

                prob = a * prob + b * prob_i;
                stats.add(prob);
            }
            epicurves.update(pop, true);

            let r = cases_acc as f64 / pop_size as f64;
            println!("");
            trace!(target: "iter", "ratio={}, n={}, cases={}/{}, p={}, attack={}", ratio, n, cases, n + self.cases_pid.error() as usize, self.sampler.prob_infection(), 100.0 * r);
            trace!(target: "iter", "  - acc={}({})/{}, cases={}({})/{}, prob={}", cases_acc, n_acc, pop_size, cases, extra_cases, n, prob);
            trace!(target: "iter", "  - curves={:?}", epicurves.tip());
        }
        let expect: i32 = epicurve.iter().map(|&e| e as i32).sum();
        debug!(target: "init", "from_epicurve: prob={}, cases={} ({}), ", prob, expect, self.cases_pid.acc());
        self.sampler.set_prob_infection(prob);
    }
}

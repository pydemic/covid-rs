use crate::{
    prelude::{EpiModel, Real},
    sim::{HasAge, Population},
};
use getset::*;
use ndarray::prelude::*;
use rand::prelude::*;

/// The sampler trait defines how new pairs of infected individuals are selected
/// from population.
pub trait Sampler<P>
where
    P: Population,
{
    /// Receive a list of agents and return the vector of indexes of all new
    /// infections ocurred in a simulation step.
    ///
    /// This function is called once per step in a simulation and if the sampler
    /// requires any state  
    fn sample_infection_pairs(&self, pool: &P, rng: &mut impl Rng) -> Vec<(usize, usize)>;

    /// Update any necessary internal state from the initial list of agents.
    /// This is called everytime the sampler is registered in a simulation.
    /// The sampler may modify  
    fn init(&mut self, _pool: &mut P) {}

    /// Baseline probability of infection. Different samplers may interpret this
    /// value in slightly different ways. This is a global probability that
    /// can be used to tweak other infection probabilities for specific
    /// contacts.
    fn prob_infection(&self) -> Real;

    /// Set prob infection to value.
    fn set_prob_infection(&mut self, value: Real) -> &mut Self;
}

////////////////////////////////////////////////////////////////////////////////
// CONCRETE IMPLEMENTATIONS
////////////////////////////////////////////////////////////////////////////////

/// A NoOP sampler. This is actually just a type alias to unit.
type NoOpSampler = ();

impl<P: Population> Sampler<P> for () {
    fn prob_infection(&self) -> Real {
        0.0
    }

    fn set_prob_infection(&mut self, _value: Real) -> &mut Self {
        return self;
    }

    fn sample_infection_pairs(&self, _pool: &P, _rng: &mut impl Rng) -> Vec<(usize, usize)> {
        vec![]
    }
}

/// A simple sampling strategy that picks up a fixed number of contacts per
/// infectious individual and infect randomly in population using the given
/// probability of infection.
#[derive(Debug, Copy, Clone, PartialEq, Default, Getters, CopyGetters, Setters)]
pub struct SimpleSampler {
    #[getset(get_copy = "pub", set = "pub")]
    n_contacts: Real,
    prob_infection: Real,
}

impl SimpleSampler {
    pub fn new(n_contacts: Real, prob_infection: Real) -> Self {
        SimpleSampler {
            n_contacts,
            prob_infection,
        }
    }
}

impl<P> Sampler<P> for SimpleSampler
where
    P: Population,
    P::State: EpiModel,
{
    fn prob_infection(&self) -> Real {
        self.prob_infection
    }
    fn set_prob_infection(&mut self, value: Real) -> &mut Self {
        self.prob_infection = value;
        return self;
    }
    fn sample_infection_pairs(&self, pop: &P, rng: &mut impl Rng) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        let n = pop.count();

        pop.each_agent(&mut |i, st| {
            if st.is_contagious() {
                let mut m = round_probabilistically(self.n_contacts, rng);
                while m > 0 {
                    if rng.gen_bool(self.prob_infection) {
                        let j = rng.gen_range(0..n);
                        if i == j {
                            continue;
                        } else if let Some(v) = pop.map_agent(j, |ag| ag.is_susceptible()) {
                            if v {
                                pairs.push((i, j));
                            }
                        }
                    }
                    m -= 1;
                }
            }
        });
        return pairs;
    }
}

/// A simple sampling strategy that picks up a fixed number of contacts per
/// infectious individual and infect randomly in population using the given
/// probability of infection.
#[derive(Debug, Clone, PartialEq, Getters, CopyGetters, Setters)]
pub struct ContactMatrixSampler {
    bin_size: u8,
    bin_map: Vec<Vec<usize>>,
    
    #[getset(get = "pub", set = "pub")]
    contact_matrix: Array2<Real>,
    prob_infection: Real,
}

impl ContactMatrixSampler {
    pub fn new(bin_size: u8, contact_matrix: Array2<Real>, prob_infection: Real) -> Self {
        assert!(contact_matrix.is_square(), "Contact matrix must be square");
        ContactMatrixSampler {
            bin_size,
            contact_matrix,
            prob_infection,
            bin_map: vec![],
        }
    }

    pub fn nbins(&self) -> usize {
        self.contact_matrix.nrows()
    }

    fn bin_for_age(&self, age: u8) -> usize {
        Self::bin_for_age_static(age, self.bin_size, self.nbins())
    }
    fn bin_for_age_static(age: u8, bin_size: u8, n: usize) -> usize {
        ((age / bin_size) as usize).clamp(0, n - 1)
    }
}

impl<P> Sampler<P> for ContactMatrixSampler
where
    P: Population,
    P::State: HasAge + EpiModel,
{
    fn init(&mut self, pop: &mut P) {
        let nbins = self.nbins();
        let bin_size = self.bin_size;

        for _ in 0..(255 / self.bin_size) {
            self.bin_map.push(vec![]);
        }

        pop.each_agent(&mut |i, st| {
            let k = Self::bin_for_age_static(st.age(), bin_size, nbins);
            self.bin_map[k].push(i);
        });
    }

    fn prob_infection(&self) -> Real {
        self.prob_infection
    }

    fn set_prob_infection(&mut self, value: Real) -> &mut Self {
        self.prob_infection = value;
        return self;
    }

    fn sample_infection_pairs(&self, pop: &P, rng: &mut impl Rng) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        pop.each_agent(&mut |i, st| {
            if st.is_contagious() {
                let u = self.bin_for_age(st.age());
                for v in 0..self.nbins() {
                    let mut m = round_probabilistically(self.contact_matrix[(u, v)], rng);
                    let group = &self.bin_map[v];
                    while m > 0 {
                        if rng.gen_bool(self.prob_infection) {
                            let j = group[rng.gen_range(0..group.len())];
                            if i == j {
                                continue;
                            }
                            pairs.push((i, j));
                        }
                        m -= 1;
                    }
                }
            }
        });
        return pairs;
    }
}

fn round_probabilistically(f: Real, rng: &mut impl Rng) -> usize {
    let int = f as usize;
    if rng.gen_bool(f - (int as Real)) {
        return int + 1;
    }
    return int;
}

/// TODO: impl PythonSampler and use dyn to make this go away!
#[derive(Debug, Clone, PartialEq)]
pub enum AnySampler {
    Simple(SimpleSampler),
    ContactMatrix(ContactMatrixSampler),
}

impl<P> Sampler<P> for AnySampler
where
    P: Population,
    P::State: HasAge + EpiModel,
{
    fn sample_infection_pairs(&self, pool: &P, rng: &mut impl Rng) -> Vec<(usize, usize)> {
        match self {
            AnySampler::Simple(s) => s.sample_infection_pairs(pool, rng),
            AnySampler::ContactMatrix(s) => s.sample_infection_pairs(pool, rng),
        }
    }

    fn init(&mut self, pool: &mut P) {
        match self {
            AnySampler::Simple(s) => s.init(pool),
            AnySampler::ContactMatrix(s) => s.init(pool),
        }
    }

    fn prob_infection(&self) -> Real {
        match self {
            AnySampler::Simple(s) => Sampler::<P>::prob_infection(s),
            AnySampler::ContactMatrix(s) => Sampler::<P>::prob_infection(s),
        }
    }

    fn set_prob_infection(&mut self, value: Real) -> &mut Self {
        match *self {
            AnySampler::Simple(ref mut s) => {
                Sampler::<P>::set_prob_infection(s, value);
            }
            AnySampler::ContactMatrix(ref mut s) => {
                Sampler::<P>::set_prob_infection(s, value);
            }
        }
        return self;
    }
}

impl From<SimpleSampler> for AnySampler {
    fn from(sampler: SimpleSampler) -> AnySampler {
        AnySampler::Simple(sampler)
    }
}

impl From<ContactMatrixSampler> for AnySampler {
    fn from(sampler: ContactMatrixSampler) -> AnySampler {
        AnySampler::ContactMatrix(sampler)
    }
}

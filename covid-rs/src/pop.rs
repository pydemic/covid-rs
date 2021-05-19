use crate::{
    agent::Infect,
    epidemic::{Params, Variant, SEICHARLike},
    prelude::{Ag, Real, Sampler, SEAIRLike, SEIRLike, SIRLike},
    sim::{Agent, Id, Population},
};
use rand::{
    prelude::{SliceRandom, SmallRng},
    Rng,
};
use std::{
    collections::HashSet,
    iter::FromIterator,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pop(Vec<Ag>);

impl Pop {
    /// Force contamination of n agents randomly selecting the VoC.
    /// Use prob_voc=1.0 to contaminate with a VoC and prob_voc=0.0 to fully
    /// contaminate with the baseline.
    pub fn contaminate_at_random_alt(
        &mut self,
        n: usize,
        infect: Infect,
        prob_voc: Real,
        rng: &mut impl Rng,
    ) -> usize {
        self._contaminate_at_random_from_loop(n, infect, prob_voc, rng)
    }

    pub fn contaminate_at_random_from_sampler(
        &mut self,
        n: usize,
        infect: Infect,
        sampler: &impl Sampler<Pop>,
        prob_voc: Real,
        rng: &mut impl Rng,
    ) -> usize {
        let mut cases = self.contaminate_at_most_from_sampler(n, infect, sampler, rng);
        if cases == n {
            return cases;
        }
        cases += self.contaminate_at_random_alt(n - cases, infect, prob_voc, rng);
        return cases;
    }

    /// Execute a sampler step and contaminate all sampled pairs with the chosen
    /// infect strategy.
    pub fn contaminate_from_sampler(
        &mut self,
        infect: Infect,
        sampler: &impl Sampler<Pop>,
        rng: &mut impl Rng,
    ) -> usize {
        let mut cases = 0;
        for (i, j) in self.sample_infection_pairs(sampler, rng) {
            if self.contaminate_pair(i, j, infect) {
                cases += 1;
            }
        }
        return cases;
    }

    /// Like contaminate_from_sampler, but limit the number of new cases to n.
    pub fn contaminate_at_most_from_sampler(
        &mut self,
        n: usize,
        infect: Infect,
        sampler: &impl Sampler<Pop>,
        rng: &mut impl Rng,
    ) -> usize {
        let mut cases = 0;
        let mut pairs = self.sample_infection_pairs(sampler, rng);

        if pairs.len() < n {
            pairs.shuffle(rng);
        }

        for (i, j) in pairs {
            if cases >= n {
                return cases;
            }
            if self.contaminate_pair(i, j, infect) {
                cases += 1;
            }
        }
        return cases;
    }

    /// Indexes of susceptible individuals
    pub fn susceptible(&self) -> Vec<usize> {
        self.indexes(|a| a.is_susceptible())
    }

    pub fn indexes<F>(&self, f: F) -> Vec<usize>
    where
        F: Fn(&Ag) -> bool,
    {
        let mut vec = vec![];
        for (i, agent) in self.iter().enumerate() {
            if f(agent) {
                vec.push(i);
            }
        }
        return vec;
    }

    /// Return sample of infection pairs
    pub fn sample_infection_pairs(
        &self,
        sampler: &impl Sampler<Pop>,
        rng: &mut impl Rng,
    ) -> Vec<(usize, usize)> {
        return sampler.sample_infection_pairs(self, rng);
    }

    
    /// Return random agent from group as mutable reference.
    pub fn gen_agent(&self, rng: &mut impl Rng) -> &Ag {
        let vec = self.as_slice();
        &vec[rng.gen_range(0..vec.len())]
    }

    /// Return random agent from group as mutable reference.
    pub fn gen_agent_mut(&mut self, rng: &mut impl Rng) -> &mut Ag {
        let vec = self.as_mut_slice();
        &mut vec[rng.gen_range(0..vec.len())]
    }
}

impl Pop {
    fn _contaminate_at_random_from_list(
        &mut self,
        n: usize,
        infect: Infect,
        prob_voc: Real,
        rng: &mut impl Rng,
    ) -> usize {
        let mut pop = self.susceptible();
        (pop.len() > n).then(|| pop.shuffle(rng));

        let size = pop.len().min(n);
        let data = self.as_mut_slice();
        for i in 0..size {
            let mut agent = data[pop[i]];
            agent.contaminate(Variant::random(rng, prob_voc), infect);
        }
        return size;
    }

    fn _contaminate_at_random_from_loop(
        &mut self,
        n: usize,
        infect: Infect,
        prob_voc: Real,
        rng: &mut impl Rng,
    ) -> usize {
        let mut cases = 0;
        let mut tries = 0;

        while cases < n {
            let variant = Variant::random(rng, prob_voc);
            let agent = self.gen_agent_mut(rng);
            if agent.contaminate(variant, infect) {
                cases += 1;
            }

            tries += 1;
            if tries >= 3 * n && tries > 15 {
                let extra = self._contaminate_at_random_from_list(n - cases, infect, prob_voc, rng);
                return cases + extra;
            }
        }
        return n;
    }
}

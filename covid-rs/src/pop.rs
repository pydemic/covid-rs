use crate::{
    agent::Infect,
    epidemic::{Params, Variant, SEICHAR},
    prelude::{Ag, Real, Sampler, SEAIR, SEIR, SIR},
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
    pub fn new() -> Self {
        Pop(Vec::new())
    }
    pub fn new_data(data: Vec<Ag>) -> Self {
        Pop(data)
    }

    pub fn as_slice(&self) -> &[Ag] {
        &self.0
    }
    pub fn as_vec(&self) -> &Vec<Ag> {
        &self.0
    }
    pub fn as_mut_slice(&mut self) -> &mut [Ag] {
        &mut self.0
    }
    pub fn as_mut_vec(&mut self) -> &mut Vec<Ag> {
        &mut self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, a: &Ag) {
        self.0.push(*a);
    }

    pub fn get(&self, i: usize) -> Option<&Ag> {
        self.0.get(i)
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut Ag> {
        self.0.get_mut(i)
    }

    /// Iterator over agents.
    pub fn iter(&self) -> impl Iterator<Item = &Ag> {
        self.as_slice().iter()
    }

    /// Mutable iterator over agents.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Ag> {
        self.as_mut_slice().iter_mut()
    }

    /** RANDOM GENERATION AND SIMULATION **************************************/

    /// Update all agents in group, considering only self iterations.
    pub fn update(&mut self, rng: &mut SmallRng, params_baseline: &Params, params_voc: &Params) {
        for agent in self.iter_mut() {
            agent.update(rng, params_baseline, params_voc);
        }
    }

    /// Infect susceptible agent with variant and return true if infection occurs.
    pub fn contaminate_agent(&mut self, i: usize, variant: Variant, infect: Infect) -> bool {
        if let Some(agent) = self.get_mut(i) {
            return agent.contaminate(variant, infect);
        }
        return false;
    }

    /// Infect j with variant from i, by index. Return true if infection occurs.
    pub fn contaminate_pair(&mut self, i: usize, j: usize, infect: Infect) -> bool {
        if i != j {
            if let Some(agent) = self.get(i) {
                if let Some(variant) = agent.active_variant() {
                    if self.contaminate_agent(j, variant, infect) {
                        self[i].register_secondary_infection();
                        return true;
                    }
                }
            }
        }
        return false;
    }

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

    /// Sample n random agents. Return the corresponding indexes.
    pub fn sample_agents(&self, n: usize, rng: &mut impl Rng) -> Vec<usize> {
        let m = self.len();
        if n >= m {
            return (0..m).collect();
        }
        let mut out = HashSet::new();
        while out.len() < n {
            out.insert(rng.gen_range(0..m));
        }
        return out.iter().map(|i| *i).collect();
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

impl IntoIterator for Pop {
    type Item = Ag;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<Ag> for Pop {
    fn from_iter<I: IntoIterator<Item = Ag>>(iter: I) -> Pop {
        return Pop::new_data(iter.into_iter().collect());
    }
}

impl Into<Vec<Ag>> for Pop {
    fn into(self) -> Vec<Ag> {
        self.0
    }
}

impl<'a> Into<Pop> for &'a [Ag] {
    fn into(self) -> Pop {
        Pop(self.into())
    }
}

impl Index<usize> for Pop {
    type Output = Ag;

    fn index(&self, i: usize) -> &Ag {
        &self.0[i]
    }
}

impl IndexMut<usize> for Pop {
    fn index_mut(&mut self, i: usize) -> &mut Ag {
        &mut self.0[i]
    }
}

impl Population for Pop {
    type State = Ag;

    fn from_states<I>(states: I) -> Self
    where
        I: IntoIterator<Item = Self::State>,
    {
        FromIterator::from_iter(states)
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn get_agent(&self, id: crate::sim::Id) -> Option<Agent<Self::State>> {
        self.get(id).map(|&state| Agent { id, state })
    }

    fn set_agent(&mut self, id: Id, state: &Self::State) -> &mut Self {
        self[id] = *state;
        return self;
    }

    fn map_agent_mut<B>(&mut self, id: Id, f: impl FnOnce(&mut Self::State) -> B) -> Option<B> {
        self.get_mut(id).map(f)
    }

    fn map_agent<B>(&self, id: Id, f: impl FnOnce(&Self::State) -> B) -> Option<B> {
        self.get(id).map(f)
    }

    fn each_agent<F>(&self, f: &mut F)
    where
        F: FnMut(Id, &Self::State),
    {
        for (id, st) in self.iter().enumerate() {
            f(id, st)
        }
    }

    fn each_agent_mut(&mut self, f: impl FnMut(Id, &mut Self::State)) {
        let mut g = f;
        for (id, st) in self.iter_mut().enumerate() {
            g(id, st)
        }
    }

    fn random<R: Rng>(&self, rng: &mut R) -> (Id, Self::State) {
        let id = rng.gen_range(0..self.len());
        return (id, self[id]);
    }
}

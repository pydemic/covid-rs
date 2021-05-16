use crate::{agent::Ag, prelude::{Age, Real, SIRLike}};
use std::iter::{Filter, Iterator, Map};

pub trait AgentsIter<'a>
where
    Self: Sized + Iterator<Item = &'a Ag>,
{
    fn r0(self) -> Real {
        let mut total = 0;
        let mut acc = 0;
        for agent in self {
            match (agent.state(), agent.secondary_infections()) {
                (st, 0) => {
                    if st.is_recovered() {
                        total += 1;
                    }
                }
                (_, n) => {
                    total += 1;
                    acc += n;
                }
            }
        }
        return (acc as Real) / (total as Real);
    }

    fn ages(self) -> Map<Self, &'a dyn Fn(&'a Ag) -> Age> {
        self.map(&|a: &'a Ag| a.age())
    }

    fn indexes<F>(self, f: F) -> Vec<usize>
    where
        F: Fn(&Ag) -> bool,
    {
        let mut vec = vec![];
        for (i, agent) in self.enumerate() {
            if f(agent) {
                vec.push(i);
            }
        }
        return vec;
    }

    fn iter_indexes<F>(self, pred: F) -> Filter<Self, F>
    where
        F: Fn(&&Ag) -> bool,
    {
        self.filter(pred)
    }

    fn iter_susceptible(self) -> Filter<Self, &'a dyn Fn(&&Ag) -> bool> {
        self.iter_indexes(&|a| a.is_susceptible())
    }
}

impl<'a, I: Iterator<Item = &'a Ag>> AgentsIter<'a> for I {}

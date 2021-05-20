use crate::prelude::{Real, INF, NAN};
use getset::*;
use serde::{Deserialize, Serialize};

/// A trait for some data structure that holds statistics about an scalar
/// variable.
///
/// Scalar is easily applicable to a vector of Reals, but also admit other
/// representations that avoid storing the entire dataset in memory if that is
/// not necessary.
pub trait Sampling {
    fn add(&mut self, x: Real);
    fn add_many<I>(&mut self, xs: I)
    where
        I: IntoIterator<Item = Real>,
    {
        for x in xs {
            self.add(x);
        }
    }
    fn sample_size(&self) -> usize;
    fn total(&self) -> Real;
    fn min(&self) -> Real;
    fn max(&self) -> Real;
    fn var(&self) -> Real;
    fn skew(&self) -> Real;
    fn kurt(&self) -> Real;
    fn std(&self) -> Real {
        self.var().sqrt()
    }
    fn mean(&self) -> Real {
        self.total() / self.sample_size() as Real
    }
    fn last_sample(&self) -> Real;
    fn stats(&self) -> Stats {
        Stats {
            mean: self.mean(),
            std: self.std(),
            skew: self.skew(),
            kurt: self.kurt(),
            min: self.min(),
            max: self.max(),
            size: self.sample_size(),
        }
    }
}

impl Sampling for Vec<Real> {
    fn add(&mut self, x: Real) {
        self.push(x);
    }
    fn total(&self) -> Real {
        return self.iter().fold(0.0, |acc, x| acc + x);
    }
    fn var(&self) -> Real {
        let (n, m1, m2) = self
            .iter()
            .fold((0, 0., 0.), |acc, x| (acc.0 + 1, acc.1 + x, acc.2 + x * x));
        return (m2 / n as Real) - sqr(m1 / n as Real);
    }
    fn skew(&self) -> Real {
        Accumulator::from_data(self.iter().cloned()).skew()
    }
    fn kurt(&self) -> Real {
        Accumulator::from_data(self.iter().cloned()).kurt()
    }
    fn sample_size(&self) -> usize {
        return self.len();
    }
    fn min(&self) -> Real {
        return self.iter().fold(INF, |acc, x| acc.min(*x));
    }
    fn max(&self) -> Real {
        return self.iter().fold(-INF, |acc, x| acc.max(*x));
    }
    fn stats(&self) -> Stats {
        Accumulator::from_data(self.iter().cloned()).stats()
    }
    fn last_sample(&self) -> Real {
        match self.last() {
            Some(x) => *x,
            _ => NAN,
        }
    }
}

/// A simple accumulator of point statistics.
///
/// It stores the latest value computed from number of samples, moments
/// minimum and maximum values and use it to compute descriptive statistics such
/// as the mean, variance, skewness, etc.
#[derive(Debug, Copy, Clone, PartialEq, CopyGetters, Getters)]
pub struct Accumulator {
    n: usize,
    m1: Real,
    m2: Real,
    m3: Real,
    m4: Real,
    min: Real,
    max: Real,
    last: Real,
}

impl Accumulator {
    /// Create new empty Point Stats accumulator
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed iterator into point stats accumulator
    pub fn from_data(iter: impl IntoIterator<Item = Real>) -> Self {
        let mut acc = Accumulator::new();
        acc.add_many(iter);
        return acc;
    }

    /// Merge two accumulators
    pub fn merge(&self, other: &Self) -> Self {
        Accumulator {
            n: self.n + other.n,
            m1: self.m1 + other.m1,
            m2: self.m2 + other.m2,
            m3: self.m3 + other.m3,
            m4: self.m4 + other.m4,
            min: self.min + other.min,
            max: self.max + other.max,
            last: self.last + other.last,
        }
    }
}

impl Sampling for Accumulator {
    fn add(&mut self, x: Real) {
        self.n += 1;
        self.m1 += x;
        self.m2 += x * x;
        self.m3 += x * x * x;
        self.m4 += x * x * x * x;
        self.min = Real::min(x, self.min);
        self.max = Real::max(x, self.max);
        self.last = x;
    }

    fn mean(&self) -> Real {
        self.m1 / self.n as Real
    }

    fn total(&self) -> Real {
        return self.m1;
    }

    fn var(&self) -> Real {
        let m = self.mean();
        return self.m2 / self.n as Real - m * m;
    }

    fn skew(&self) -> Real {
        let m = self.mean();
        let s = self.std();
        return (self.m3 / self.n as Real - 3.0 * m * s * s - m * m * m) / (s * s * s);
    }

    fn kurt(&self) -> Real {
        let n = self.n as Real;
        let (m, b, c, d) = (self.m1 / n, self.m2 / n, self.m3 / n, self.m4 / n);
        let m2 = m * m;
        let m4 = m2 * m2;
        return (d - 4. * m * c + 6. * m2 * b - 3. * m4) / ((b - m2) * (b - m2));
    }
    fn min(&self) -> Real {
        self.min
    }
    fn max(&self) -> Real {
        self.max
    }
    fn sample_size(&self) -> usize {
        self.n
    }
    fn last_sample(&self) -> Real {
        return self.last;
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        Accumulator {
            n: 0,
            m1: 0.,
            m2: 0.,
            m3: 0.,
            m4: 0.,
            min: INF,
            max: -INF,
            last: NAN,
        }
    }
}

#[inline]
pub fn sqr(x: Real) -> Real {
    x * x
}

/// A simple struct that stores basic values of descriptive statistics about a
/// sample or distribution.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize, Getters, CopyGetters, Setters)]
#[getset(get_copy = "pub", set = "pub")]
pub struct Stats {
    mean: Real,
    std: Real,
    skew: Real,
    kurt: Real,
    min: Real,
    max: Real,
    size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn simple_stats() {
        let mut acc = Accumulator::new();
        acc.add(0.);
        acc.add_many(vec![1., 2., 3., 4.]);
        let st = acc.stats();
        assert_eq!(st.size, 5);
        assert_approx_eq!(st.mean, 2.0, 0.001);
        assert_approx_eq!(st.std, 1.4142, 0.001);
        assert_approx_eq!(st.skew, 0.0, 0.001);
        assert_approx_eq!(st.kurt, 3.40, 0.001);
    }
}

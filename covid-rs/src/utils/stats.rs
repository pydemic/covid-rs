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
    /// Add a single observation to sample
    fn add(&mut self, x: Real) {
        self.add_count(x, 1);
    }

    /// Add many observations to sample. This can be more efficient than adding
    /// one by one.
    fn add_many<I>(&mut self, xs: I)
    where
        I: IntoIterator<Item = Real>,
    {
        for x in xs {
            self.add(x);
        }
    }

    /// Add many observations to sample from pairs of (data, number of occurrences).
    /// This can be more efficient than adding a single observation
    fn add_counts<I>(&mut self, xs: I)
    where
        I: IntoIterator<Item = (Real, usize)>,
    {
        for (x, n) in xs {
            self.add_count(x, n);
        }
    }

    /// Add n observations of x
    fn add_count(&mut self, x: Real, n: usize);

    /// Return the sample size
    fn sample_size(&self) -> usize;

    /// Return the sum of all observations
    fn total(&self) -> Real;

    /// Return the minimum value
    fn min(&self) -> Real;

    /// Return the maximum value
    fn max(&self) -> Real;

    /// Return the variance
    fn var(&self) -> Real;

    /// Return the skewness (or normalized third moment)
    fn skew(&self) -> Real;

    /// Return the kurtosis (or normalized fourth moment)
    fn kurt(&self) -> Real;

    /// Return the standard deviation
    fn std(&self) -> Real {
        self.var().sqrt()
    }

    /// Return the mean
    fn mean(&self) -> Real {
        self.total() / self.sample_size() as Real
    }

    /// Return the last sample from distribution.
    fn last_sample(&self) -> Real;

    /// Return a simple stats struct holding all statistics as fields.
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
    fn add_count(&mut self, x: Real, n: usize) {
        for _ in 0..n {
            self.push(x);
        }
    }
    fn total(&self) -> Real {
        self.iter().sum()
    }
    fn var(&self) -> Real {
        let (n, m1, m2) = self
            .iter()
            .fold((0, 0., 0.), |acc, x| (acc.0 + 1, acc.1 + x, acc.2 + x * x));
        return (m2 / n as Real) - sqr(m1 / n as Real);
    }
    fn skew(&self) -> Real {
        Accumulator::from_data(self).skew()
    }
    fn kurt(&self) -> Real {
        Accumulator::from_data(self).kurt()
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
        Accumulator::from_data(self).stats()
    }
    fn last_sample(&self) -> Real {
        match self.last() {
            Some(x) => *x,
            _ => NAN,
        }
    }
}

impl Sampling for Vec<(Real, usize)> {
    fn add_count(&mut self, x: Real, n: usize) {
        if let Some(pair) = self.last_mut() {
            if pair.0 == x {
                pair.1 += n;
                return;
            }
        }
        self.push((x, n));
    }
    fn total(&self) -> Real {
        self.iter().map(|(x, n)| x * (*n as Real)).sum()
    }
    fn var(&self) -> Real {
        let (mut m0, mut m1, mut m2) = (0.0, 0.0, 0.0);
        for (x, n) in self {
            let n_ = *n as Real;
            m0 += n_;
            m1 += n_ * x;
            m2 += n_ * x * x;
        }
        return (m2 / m0) - sqr(m1 / m0);
    }
    fn skew(&self) -> Real {
        Accumulator::from_counts(self).skew()
    }
    fn kurt(&self) -> Real {
        Accumulator::from_counts(self).kurt()
    }
    fn sample_size(&self) -> usize {
        self.iter().map(|(_, n)| *n).sum()
    }
    fn min(&self) -> Real {
        self.iter().fold(INF, |acc, (x, _)| acc.min(*x))
    }
    fn max(&self) -> Real {
        self.iter().fold(-INF, |acc, (x, _)| acc.max(*x))
    }
    fn stats(&self) -> Stats {
        Accumulator::from_counts(self).stats()
    }
    fn last_sample(&self) -> Real {
        match self.last() {
            Some((x, _)) => *x,
            _ => NAN,
        }
    }
}

// impl Sampling for HashMap<Real, usize> {
//     fn add_count(&mut self, x: Real, n: usize) {
//         if let Some(m) = self.get_mut(x) {
//             m += n;
//         } else {
//             self.insert(x, n);
//         }
//     }
//     fn sample_size(&self) -> usize {
//         self.iter().map(|(_, n)| *n).sum()
//     }
//     fn total(&self) -> Real {
//         self.iter().map(|(x, _)| *x).sum()
//     }
//     fn min(&self) -> Real {
//         self.iter().fold(INF, |acc, (x, _)| acc.min(*x))
//     }
//     fn max(&self) -> Real {
//         self.iter().fold(-INF, |acc, (x, _)| acc.max(*x))
//     }
//     fn var(&self) -> Real {
//         Accumulator::from_counts(self).var()
//     }
//     fn skew(&self) -> Real {
//         Accumulator::from_counts(self).skew()
//     }
//     fn kurt(&self) -> Real {
//         Accumulator::from_counts(self).kurt()
//     }
//     fn last_sample(&self) -> Real {
//         NAN
//     }
// }

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
    pub fn from_data<'a>(iter: impl IntoIterator<Item = &'a Real>) -> Self {
        let mut acc = Accumulator::new();
        for &x in iter {
            acc.add(x);
        }
        return acc;
    }

    /// Feed iterator of pairs of (x, n_counts) into point stats accumulator
    pub fn from_counts<'a>(iter: impl IntoIterator<Item = &'a (Real, usize)>) -> Self {
        let mut acc = Accumulator::new();
        for (x, n) in iter {
            acc.add_count(*x, *n);
        }
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
    fn add_count(&mut self, x: Real, n: usize) {
        let m = n as Real;
        self.n += n;
        self.m1 += m * x;
        self.m2 += m * x * x;
        self.m3 += m * x * x * x;
        self.m4 += m * x * x * x * x;
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
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Stats {
    pub mean: Real,
    pub std: Real,
    pub skew: Real,
    pub kurt: Real,
    pub min: Real,
    pub max: Real,
    pub size: usize,
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

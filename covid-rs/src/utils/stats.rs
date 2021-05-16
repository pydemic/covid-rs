use crate::prelude::*;
use crate::utils::{INF, NAN};
use getset::*;
use serde::{Deserialize, Serialize};

pub trait Stats {
    fn add(&mut self, x: Real);
    fn add_many<I>(&mut self, xs: I)
    where
        I: IntoIterator<Item = Real>,
    {
        for x in xs {
            self.add(x);
        }
    }
    fn size(&self) -> usize;
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
        self.total() / self.size() as Real
    }
    fn last(&self) -> Real;
    fn stats(&self) -> PointStats {
        PointStats {
            mean: self.mean(),
            std: self.std(),
            skew: self.skew(),
            kurt: self.kurt(),
            min: self.min(),
            max: self.max(),
            size: self.size(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatsVec {
    data: Vec<Real>,
}

impl StatsVec {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn stats_acc(&self) -> PointStatsAcc {
        let mut acc = PointStatsAcc::new();
        acc.add_many(self.data.iter().map(|x| *x));
        return acc;
    }
}

impl Stats for StatsVec {
    fn add(&mut self, x: Real) {
        self.data.push(x);
    }
    fn total(&self) -> Real {
        return self.data.iter().fold(0.0, |acc, x| acc + x);
    }
    fn var(&self) -> Real {
        let (n, m1, m2) = self
            .data
            .iter()
            .fold((0, 0., 0.), |acc, x| (acc.0 + 1, acc.1 + x, acc.2 + x * x));
        return (m2 / n as Real) - sqr(m1 / n as Real);
    }
    fn skew(&self) -> Real {
        self.stats_acc().skew()
    }
    fn kurt(&self) -> Real {
        self.stats_acc().kurt()
    }
    fn size(&self) -> usize {
        return self.data.len();
    }
    fn min(&self) -> Real {
        return self.data.iter().fold(INF, |acc, x| acc.min(*x));
    }
    fn max(&self) -> Real {
        return self.data.iter().fold(-INF, |acc, x| acc.max(*x));
    }
    fn stats(&self) -> PointStats {
        let acc = self.stats_acc();
        PointStats {
            mean: acc.mean(),
            std: acc.std(),
            skew: acc.skew(),
            kurt: acc.kurt(),
            min: acc.min(),
            max: acc.max(),
            size: acc.n,
        }
    }
    fn last(&self) -> Real {
        match self.data.last() {
            Some(x) => *x,
            _ => NAN,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, CopyGetters, Getters)]
pub struct PointStatsAcc {
    n: usize,
    m1: Real,
    m2: Real,
    m3: Real,
    m4: Real,
    min: Real,
    max: Real,
    last: Real,
}

impl PointStatsAcc {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Stats for PointStatsAcc {
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
    fn size(&self) -> usize {
        self.n
    }
    fn last(&self) -> Real {
        return self.last;
    }
}

impl Default for PointStatsAcc {
    fn default() -> Self {
        PointStatsAcc {
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

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct PointStats {
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
        let mut acc = PointStatsAcc::new();
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

///! Functions in this module provide basic reporting and outputs about
///! simulation results
use crate::{
    epidemic::{StateStats, VariantSEICHAR},
    pop::Pop,
    prelude::{Params, Real, Sampler},
    simulation::Simulation,
    utils::{PointStats, PointStatsAcc, Stats},
};
use serde::{Deserialize, Serialize};

pub struct Report<'a, S: Sampler<Pop>>(&'a Simulation<S>);

impl<'a, S: Sampler<Pop>> Report<'a, S> {
    pub fn new(sim: &'a Simulation<S>) -> Self {
        Report(sim)
    }
    /*
    /// Print the state of simulation in command line
    pub fn describe(&self) {
        let res = toml::to_string(&self.results()).unwrap();
        println!("{}", res);
    }

    pub fn results(&self) -> SimulationResult {
        SimulationResult {
            stats: StatsResults {
                infections: self.0.stats().infections.stats(),
                r0: self.0.stats().r0.stats(),
                population_abs: self.curve_stats(1.0),
            },
            params: *self.0.params_baseline(),
        }
    }

    fn curve_stats(&self, scale: Real) -> StateStats<PopStat> {
        let mut stats = StateStats {
            susceptible: PointStatsAcc::new(),
            exposed: PointStatsAcc::new(),
            infectious: PointStatsAcc::new(),
            critical: PointStatsAcc::new(),
            severe: PointStatsAcc::new(),
            asymptomatic: PointStatsAcc::new(),
            recovered: PointStatsAcc::new(),
            dead: PointStatsAcc::new(),
        };
        for row in self.0.curves().iter() {
            stats.susceptible.add(row[0] as Real * scale);
            stats.exposed.add(row[1] as Real * scale);
            stats.infectious.add(row[2] as Real * scale);
            stats.critical.add(row[3] as Real * scale);
            stats.severe.add(row[4] as Real * scale);
            stats.asymptomatic.add(row[5] as Real * scale);
            stats.recovered.add(row[6] as Real * scale);
            stats.dead.add(row[7] as Real * scale);
        }
        return stats.map(|acc| PopStat {
            min: acc.min(),
            max: acc.max(),
            current: acc.last(),
        });
    }
    */
    pub fn epicurve_csv(&self) -> String {
        let mut data = VariantSEICHAR::csv_header();
        for &ln in self.0.curves().iter() {
            data.push('\n');
            for (i, x) in ln.iter().enumerate() {
                if i != 0 {
                    data.push(',');
                }
                data.push_str(&x.to_string());
            }
        }
        return data;
    }
}

/*
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct PopStat {
    min: Real,
    max: Real,
    current: Real,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct StatsResults {
    pub population_abs: StateStats<PopStat>,
    pub infections: PointStats,
    pub r0: PointStats,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct SimulationResult {
    pub params: Params,
    pub stats: StatsResults,
}
*/

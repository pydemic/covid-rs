use std::fs;

use covid::{prelude::*, sampler::SimpleSampler, utils::*};
use csv::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Clone, Copy)]
#[serde(default)]
pub struct ParamSet {
    baseline: Params,
    voc: Params,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Epicurve {
    data: Vec<usize>,
    smoothness: Real,
    // scaling: Real,
    // tol: Real,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pop_size: usize,
    initial_infections: usize,
    n_contacts: Real,
    prob_infection: Real,
    num_iter: usize,
    verbose: bool,
    params: ParamSet,
    pop_distrib: Option<AgeDistrib10>,
    pop_counts: Option<AgeCount10>,
    epicurve: Option<Epicurve>,
}

impl Config {
    pub fn write_data(&self, data: String, name: &str) {
        fs::write(name, data).unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            pop_size: 1_000,
            initial_infections: 10,
            n_contacts: 4.5,
            prob_infection: 0.1,
            num_iter: 30,
            verbose: true,
            params: ParamSet::default(),
            pop_distrib: Some([1.0; 9]),
            pop_counts: None,
            epicurve: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TableRow {
    age: Age,
    cfr: Real,
    ifr: Real,
    severe: Real,
    critical: Real,
}

pub fn read_params_table(path: &str, params: &mut Params) -> Result<()> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut prob_severe: AgeDistrib10 = [0.0; 9];
    let mut prob_asymptomatic: AgeDistrib10 = [0.0; 9];
    let mut prob_critical: AgeDistrib10 = [0.0; 9];
    let mut prob_death: AgeDistrib10 = [0.0; 9];

    for (i, res) in reader.deserialize().enumerate() {
        let row: TableRow = res?;
        prob_asymptomatic[i] = 1.0 - row.ifr / row.cfr;
        prob_severe[i] = row.severe;
        prob_critical[i] = row.critical / row.severe;
        prob_death[i] = row.cfr / row.critical;
    }

    params.set_prob_asymptomatic_distrib(prob_asymptomatic);
    params.set_prob_severe_distrib(prob_severe);
    params.set_prob_critical_distrib(prob_critical);
    params.set_prob_death_distrib(prob_death);
    return Ok(());
}

pub fn main() {
    use simple_logger::SimpleLogger;
    SimpleLogger::new().init().unwrap();

    let cfg_data = fs::read_to_string("conf.toml").unwrap();
    let mut cfg: Config = toml::from_str(&cfg_data).unwrap();

    match read_params_table("baseline-params.csv", &mut cfg.params.baseline) {
        Ok(_) => {
            println!("Using distributions from baseline-params.csv");
        }
        _ => {}
    }
    match read_params_table("voc-params.csv", &mut cfg.params.voc) {
        Ok(_) => {
            println!("Using distributions from voc-params.csv");
        }
        _ => {}
    }

    if cfg.verbose {
        println!("{:#?}", cfg);
    }
    simple_simulation(cfg);
}

pub fn simple_simulation(cfg: Config) {
    let mut sampler: AnySampler = SimpleSampler::new(cfg.n_contacts, cfg.prob_infection).into();
    let mut builder;

    // Should we construct from distribution or pop_counts?
    if let Some(ns) = cfg.pop_counts {
        builder = PopBuilder::new(0);
        builder.age_counts(ns);
    } else if let Some(pop_distrib) = cfg.pop_distrib {
        builder = PopBuilder::new(cfg.pop_size);
        builder.age_distribution(pop_distrib);
    } else {
        builder = PopBuilder::new(cfg.pop_size);
        builder.age_distribution([1.0; 9]);
    }

    // Should we infect from epicurve?
    if let Some(epi) = cfg.epicurve.clone() {
        let mut stats = StatsVec::new();
        builder.contaminate_from_epicurve(
            &epi.data,
            &mut sampler,
            &cfg.params.baseline,
            epi.smoothness,
            &mut stats,
        );
        println!("{:?}", stats.stats());
    } else {
        builder.contaminate_at_random(cfg.initial_infections, true.into());
    }

    let mut sim = builder.build(sampler);

    // Configure simulation
    sim.set_params_baseline(cfg.params.baseline);
    sim.set_params_voc(cfg.params.voc);
    sim.run(cfg.num_iter);

    // Write output
    let reporter = Report::new(&sim);
    cfg.write_data(reporter.epicurve_csv(), "epicurve.csv");
    // cfg.write_data(
    //     toml::to_string_pretty(&reporter.results()).unwrap(),
    //     "results.toml",
    // );
    // sim.describe();
}

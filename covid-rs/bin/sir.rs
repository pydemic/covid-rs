use covid::{models::*, params::AgeDependentSEIR, prelude::*, sim::*};

pub fn main() {
    // type T = models::SEICHAR<()>;
    // type T = models::SEAIR<()>;
    type T = SeirAgent<bool>;
    use simple_logger::SimpleLogger;
    SimpleLogger::new().init().unwrap();

    let mut pop: Vec<T> = new_population(10000);
    pop.set_ages(80);

    let params = AgeDependentSEIR::<AgeParam>::default();

    // Infect elements
    pop.set_agents(&[
        (0, &T::new_infectious()),
        (1, &T::new_infectious()),
        (2, &T::new_infectious()),
        (3, &T::new_infectious()),
        (4, &T::new_infectious()),
        (5, &T::new_infectious()),
        (6, &T::new_infectious()),
        (7, &T::new_infectious()),
        (8, &T::new_infectious()),
        (9, &T::new_infectious()),
    ]);

    let mut sim: Simulation<_, _, _, { T::CARDINALITY }> =
        Simulation::new_simple(params, pop, 4.5, 0.095);

    sim.run(180);
    // println!("{:#?}", pop);
    // println!("{:#?}", params);
    println!("{}", sim.render_epicurve_csv(T::CSV_HEADER));
    println!("params: {:#?}", params);
    // println!("pop: {:#?}", sim.sample(10))
}

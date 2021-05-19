use covid::{
    models::*,
    params::{FromUniversalParams, FullSEIRParams, LocalBind, VaccineDependentSEIR},
    prelude::*,
    sim::*,
    utils::default_rng,
};

pub fn main() {
    // type T = models::SEICHAR<()>;
    // type T = models::SEAIR<()>;
    type T = SeirAgent<bool>;
    use simple_logger::SimpleLogger;
    SimpleLogger::new().init().unwrap();

    let mut pop: Vec<T> = new_population(10000);
    let mut rng_ = default_rng();
    let rng = &mut rng_;

    pop.set_ages(50)
        .vaccinate_random(true, 0.9, rng)
        .contaminate_at_random(20, rng);

    let mut params: VaccineDependentSEIR<AgeParam> = Default::default();
    // let mut params = AgeDependentSEIR::<AgeParam>::default();
    let mut sim: Simulation<_, _, _, { T::CARDINALITY }> =
        Simulation::new_simple(params.clone(), pop, 4.5, 0.095);

    sim.seed_from(rng);
    sim.run(30)
        // .vaccinate_random(true, 0.25, rng)
        .run(30)
        // .vaccinate_random(true, 0.25, rng)
        .run(30)
        .run(30);

    // println!("{:#?}", pop);
    // println!("{:#?}", params);
    println!("{}", sim.render_epicurve_csv(T::CSV_HEADER));
    LocalBind::<T>::bind(&mut params, (20, true));
    println!(
        "params: {:#?}",
        FullSEIRParams::<Real>::from_universal_params(&params)
    );
    LocalBind::<T>::bind(&mut params, (20, false));
    println!(
        "params: {:#?}",
        FullSEIRParams::<Real>::from_universal_params(&params)
    );
    // println!("pop: {:#?}", sim.sample(10))
}

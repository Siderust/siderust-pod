//! Debug test: find optimal epsilon for Richardson convergence
// Debug test: find optimal epsilon for Richardson convergence
use siderust::astro::dynamics::forces::{
    CompositeForce, DragForce, ExponentialAtmosphere, TwoBody,
};
use siderust::astro::dynamics::{DynamicsContextBuilder, OrbitState, Position, Velocity};
use siderust::coordinates::frames::GCRS;
use siderust::qtty::{AreaToMass, DragCoefficient, Second};
use siderust::time::JulianDate;
use siderust_pod_dynamics::{param_partials_central_diff, Rk4Integrator};
use std::sync::Arc;

const GM_EARTH: f64 = 398_600.441_8;
const R_LEO: f64 = 6_871.0;

fn leo_state() -> OrbitState {
    OrbitState::new_at_jd(
        JulianDate::new(2_451_545.0),
        Position::<GCRS>::new(R_LEO, 0.0, 0.0),
        Velocity::<GCRS>::new(0.0, (GM_EARTH / R_LEO).sqrt(), 0.0),
    )
}

fn two_body_drag(cd: f64) -> CompositeForce {
    let area_to_mass = AreaToMass::new(0.01);
    CompositeForce::empty()
        .push(Box::new(TwoBody::earth()))
        .push(Box::new(DragForce::new(
            DragCoefficient::new(cd),
            area_to_mass,
        )))
}

fn leo_ctx() -> siderust::astro::dynamics::DynamicsContext {
    let atm = Arc::new(ExponentialAtmosphere::LEO_500KM);
    DynamicsContextBuilder::new().with_atmosphere(atm).build()
}

#[test]
fn find_optimal_epsilon() {
    let integrator = Rk4Integrator {
        step: Second::new(10.0),
    };
    let state = leo_state();
    let ctx = leo_ctx();
    let dt = Second::new(120.0);
    let cd_nominal = 2.2_f64;

    for eps in [1e-2, 1e-3, 1e-4, 1e-5, 1e-6, 1e-7] {
        let coarse = param_partials_central_diff(
            &integrator,
            state,
            dt,
            &ctx,
            cd_nominal,
            eps,
            two_body_drag,
        )
        .unwrap();
        let refined = param_partials_central_diff(
            &integrator,
            state,
            dt,
            &ctx,
            cd_nominal,
            eps / 2.0,
            two_body_drag,
        )
        .unwrap();
        let rel = coarse.max_relative_difference(&refined);
        eprintln!("eps={eps:.0e}: max_rel_diff={rel:.3e}");
    }
}

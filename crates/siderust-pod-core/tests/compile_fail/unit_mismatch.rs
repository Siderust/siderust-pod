// Subtracting positions whose length unit differs must not compile.

use siderust_pod_core::state::Position;
use affn::frames::GCRS;

fn main() {
    let a: Position<GCRS, qtty::unit::Kilometer> = Position::new(1.0, 0.0, 0.0);
    let b: Position<GCRS, qtty::unit::Meter> = Position::new(1.0, 0.0, 0.0);
    let _ = a - b;
}

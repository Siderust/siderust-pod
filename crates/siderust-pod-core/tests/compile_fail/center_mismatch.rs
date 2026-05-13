// Subtracting positions whose reference *center* differs must not compile.

use affn::cartesian::Position;
use affn::frames::ICRS;
use siderust::coordinates::centers::{Geocentric, Heliocentric};

fn main() {
    let a: Position<Geocentric, ICRS, qtty::unit::Kilometer> = Position::new(1.0, 0.0, 0.0);
    let b: Position<Heliocentric, ICRS, qtty::unit::Kilometer> = Position::new(1.0, 0.0, 0.0);
    let _ = a - b;
}

// Subtracting positions in different reference frames must not compile.

use siderust_pod_core::state::Position;
use affn::frames::{GCRS, ICRS};

fn main() {
    let a: Position<GCRS> = Position::new(1.0, 0.0, 0.0);
    let b: Position<ICRS> = Position::new(1.0, 0.0, 0.0);
    let _ = a - b;
}

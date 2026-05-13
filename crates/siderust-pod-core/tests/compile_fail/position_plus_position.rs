// `Position + Position` is forbidden by `affn`: a position is an affine
// point, and adding two points has no geometric meaning.

use siderust_pod_core::state::Position;

fn main() {
    let a: Position = Position::new(1.0, 2.0, 3.0);
    let b: Position = Position::new(4.0, 5.0, 6.0);
    let _ = a + b;
}

// Subtracting `Time` instants on different time scales must not compile —
// the operation is only defined within a single `CoordinateScale`.

use tempoch::{J2000Seconds, Time, TAI, TT};
use qtty::Second;

fn _diff(a: Time<TT>, b: Time<TAI>) -> qtty::Second {
    a - b
}

fn main() {
    let tt: Time<TT> = J2000Seconds::<TT>::try_new(Second::new(0.0)).unwrap().to_time();
    let tai: Time<TAI> = J2000Seconds::<TAI>::try_new(Second::new(0.0)).unwrap().to_time();
    let _ = _diff(tt, tai);
}

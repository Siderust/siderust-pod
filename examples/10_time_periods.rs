// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Vallés Puig, Ramon

//! # Example: time wrappers and periods
//!
//! ## Scientific scope
//!
//! This example introduces the typed time wrappers used by `siderust` and
//! `tempoch`, including Julian dates, Modified Julian dates, and typed
//! interval arithmetic. The scientific focus is time-scale representation
//! rather than celestial mechanics.
//!
//! It uses the current UTC instant to show round-trips and period
//! construction, so the printed values are runtime-dependent but the
//! demonstrated API pattern is stable.
//!
//! ## Technical scope
//!
//! The executable converts between chrono values and typed astronomical
//! times, constructs one-day and six-hour windows, and prints the results.
//! It is intentionally small and contains no reusable library surface.
//!
//! No ephemeris evaluation or observation modelling occurs here.
//!
//! ## References
//!
//! - Meeus, J. (1998). Astronomical Algorithms (2nd ed.). Willmann-Bell.
//! - IERS Conventions Centre. (2010). IERS Conventions (2010). Verlag des
//!   Bundesamts fur Kartographie und Geodasie.
use chrono::{Duration, Utc};
use siderust::qtty::Days;
use siderust::time::{Interval, JulianDate, ModifiedJulianDate, UTC};

fn main() {
    let now_utc = Utc::now();
    let jd = JulianDate::from_chrono(now_utc);
    let mjd: ModifiedJulianDate = jd.into();

    println!("UTC now : {}", now_utc.to_rfc3339());
    println!("JD (TT) : {}", jd);
    println!("MJD(TT) : {}", mjd);
    println!(
        "Back UTC: {}",
        jd.to_chrono()
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "N/A".into())
    );

    let tomorrow = jd + Days::new(1.0);
    let window = Interval::<JulianDate>::new(jd, tomorrow);
    println!("1-day window length: {}", window.end - window.start);

    let utc_time = tempoch::Time::<UTC>::from_chrono(now_utc);
    let utc_window = Interval::<tempoch::Time<UTC>>::new(
        utc_time,
        tempoch::Time::<UTC>::from_chrono(now_utc + Duration::hours(6)),
    );
    println!("UTC scale window: {}", utc_window.end - utc_window.start);
}

//! Migrated from `tests/service.test.ts`
//! Origin language: typescript

use weather_api::service::{celsius_to_fahrenheit, get_forecast, get_weather};

#[test]
fn returns_london_catalog_entry() {
    let w = get_weather("london");
    assert_eq!(w.city, "London");
    assert_eq!(w.temperature_c, 12.0);
}

#[test]
fn converts_celsius_to_fahrenheit() {
    assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
    assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
}

#[test]
fn builds_a_three_day_forecast() {
    let f = get_forecast("paris");
    assert_eq!(f.days.len(), 3);
    assert_eq!(f.city, "Paris");
}

//! Migrated from `src/service.ts` (semantic service lowering)
use crate::types::*;
use serde::{Deserialize, Serialize};

fn cities_lookup(key: &str) -> Option<Weather> {
    match key {
        "london" => Some(Weather {
            city: "London".to_string(),
            temperature_c: 12.0 as f64,
            conditions: "cloudy".to_string(),
        }),
        "paris" => Some(Weather {
            city: "Paris".to_string(),
            temperature_c: 18.0 as f64,
            conditions: "sunny".to_string(),
        }),
        "tokyo" => Some(Weather {
            city: "Tokyo".to_string(),
            temperature_c: 22.0 as f64,
            conditions: "rain".to_string(),
        }),
        _ => None,
    }
}

pub fn get_weather(city: &str) -> Weather {
    let key = city.to_lowercase();
    if let Some(found) = cities_lookup(&key) {
        return found;
    }
    Weather {
        city: city.to_string(),
        temperature_c: 15.0,
        conditions: "unknown".to_string(),
    }
}

pub fn get_forecast(city: &str) -> Forecast {
    let base = get_weather(city);
    let days = vec![
        base.clone(),
        Weather {
            city: base.city.clone(),
            temperature_c: base.temperature_c + 1.0,
            conditions: base.conditions.clone(),
        },
        Weather {
            city: base.city.clone(),
            temperature_c: base.temperature_c - 1.0,
            conditions: base.conditions.clone(),
        },
    ];
    Forecast {
        city: base.city,
        days,
    }
}

pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0) / 5.0 + 32.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn london_catalog() {
        let w = get_weather("london");
        assert_eq!(w.city, "London");
        assert_eq!(w.temperature_c, 12.0);
    }

    #[test]
    fn c_to_f() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
    }

    #[test]
    fn forecast_len() {
        let f = get_forecast("paris");
        assert_eq!(f.days.len(), 3);
        assert_eq!(f.city, "Paris");
    }
}

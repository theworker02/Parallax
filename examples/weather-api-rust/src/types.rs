//! Migrated from `src/types.ts`
//! Origin language: typescript

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Weather {
    pub city: String,
    #[serde(rename = "temperatureC")]
    pub temperature_c: f64,
    pub conditions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Forecast {
    pub city: String,
    pub days: Vec<Weather>,
}

import type { Forecast, Weather } from "./types.ts";

const CITIES: Record<string, Weather> = {
  london: { city: "London", temperatureC: 12, conditions: "cloudy" },
  paris: { city: "Paris", temperatureC: 18, conditions: "sunny" },
  tokyo: { city: "Tokyo", temperatureC: 22, conditions: "rain" },
};

/** Look up current weather for a city (deterministic catalog). */
export function getWeather(city: string): Weather {
  const key = city.toLowerCase();
  const found = CITIES[key];
  if (!found) {
    return { city, temperatureC: 15, conditions: "unknown" };
  }
  return found;
}

/** Build a trivial 3-day forecast from the current observation. */
export function getForecast(city: string): Forecast {
  const base = getWeather(city);
  const days = [
    base,
    {
      city: base.city,
      temperatureC: base.temperatureC + 1,
      conditions: base.conditions,
    },
    {
      city: base.city,
      temperatureC: base.temperatureC - 1,
      conditions: base.conditions,
    },
  ];
  return { city: base.city, days };
}

/** Convert Celsius to Fahrenheit. */
export function celsiusToFahrenheit(celsius: number): number {
  return (celsius * 9) / 5 + 32;
}



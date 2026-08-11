/** Weather observation for a city. */
export interface Weather {
  city: string;
  temperatureC: number;
  conditions: string;
}

/** Multi-day forecast. */
export interface Forecast {
  city: string;
  days: Weather[];
}

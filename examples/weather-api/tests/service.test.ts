import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  celsiusToFahrenheit,
  getForecast,
  getWeather,
} from "../src/service.ts";

describe("weather service", () => {
  it("returns london catalog entry", () => {
    const w = getWeather("london");
    assert.equal(w.city, "London");
    assert.equal(w.temperatureC, 12);
  });

  it("converts celsius to fahrenheit", () => {
    assert.equal(celsiusToFahrenheit(0), 32);
    assert.equal(celsiusToFahrenheit(100), 212);
  });

  it("builds a three-day forecast", () => {
    const f = getForecast("paris");
    assert.equal(f.days.length, 3);
    assert.equal(f.city, "Paris");
  });
});

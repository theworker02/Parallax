import type { Request, Response } from "express";
import { celsiusToFahrenheit, getForecast, getWeather } from "./service.ts";

/** GET /weather/:city */
export function getWeatherHandler(req: Request, res: Response): void {
  const city = String(req.params.city || "london");
  const weather = getWeather(city);
  res.json({
    ...weather,
    temperatureF: celsiusToFahrenheit(weather.temperatureC),
  });
}

/** GET /forecast/:city */
export function getForecastHandler(req: Request, res: Response): void {
  const city = String(req.params.city || "london");
  const forecast = getForecast(city);
  res.json(forecast);
}

/** GET /health */
export function healthHandler(_req: Request, res: Response): void {
  res.json({ ok: true });
}

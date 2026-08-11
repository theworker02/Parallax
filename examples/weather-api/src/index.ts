import express from "express";
import {
  getForecastHandler,
  getWeatherHandler,
  healthHandler,
} from "./routes.ts";

const app = express();
const port = Number(process.env.PORT || 3000);

app.get("/health", healthHandler);
app.get("/weather/:city", getWeatherHandler);
app.get("/forecast/:city", getForecastHandler);

if (process.env.NODE_ENV !== "test") {
  app.listen(port, () => {
    console.log(`weather-api listening on ${port}`);
  });
}

export default app;

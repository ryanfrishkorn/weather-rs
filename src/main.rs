// use std::collections::HashMap;
use reqwest::header::USER_AGENT;
use serde::{Deserialize};
use tokio;

#[derive(Debug, Deserialize)]
struct Forecast {
    properties: Properties,
    // periods: Vec<Period>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Properties {
    updated: String,
    forecastGenerator: String,
    generatedAt: String,
    updateTime: String,
    validTimes: String,
    periods: Vec<Period>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Period {
    number: u8,
    name: String,
    startTime: String,
    endTime: String,
    isDaytime: bool,
    temperature: u8,
    temperatureTrend: Option<String>,
    windSpeed: String,
    windDirection: String,
    shortForecast: String,
    detailedForecast: String,
}

#[tokio::main]
async fn main() {
    // println!("Weather!");
    // Locate grid data by lat/lon
    // let url = "https://api.weather.gov/points/36.1744,-115.2721";

    // Specify grid point - this gives raw numerical data
    // let url = "https://api.weather.gov/gridpoints/VEF/117,98";

    // Forecast
    let url = "https://api.weather.gov/gridpoints/VEF/117,98/forecast";

    let client = reqwest::Client::new();
    let response = match client.get(url)
        .header(USER_AGENT, "rust-implementation/console")
        .send()
        .await {
        Ok(r) => r,
        Err(e) => panic!("error waiting for response: {}", e),
    };

    // println!("response: {:?}", response);
    let content = match response.text().await {
        Ok(t) => t,
        Err(e) => panic!("error getting body: {}", e),
    };

    println!("{}", content);
    let forecast: Forecast = serde_json::from_str(content.as_str()).expect("could not parse json content");
    println!("{:?}", forecast);
}

// NWS Request:
//
// 36.17446 N, 115.27203 W
// https://api.weather.gov/points/36.1744,-115.2721

// Response Data:
//
// "properties": {
//   "updated": "2022-12-06T20:12:59+00:00",
//   "units": "us",
//   "forecastGenerator": "BaselineForecastGenerator",
//   "generatedAt": "2022-12-06T22:46:16+00:00",
//   "updateTime": "2022-12-06T20:12:59+00:00",
//   "validTimes": "2022-12-06T14:00:00+00:00/P8DT6H",
//   "elevation": {
//     "unitCode": "wmoUnit:m",
//     "value": 779.9832
//   },
//   "periods": [
//     {
//       "number": 1,
//       "name": "This Afternoon",
//       "startTime": "2022-12-06T14:00:00-08:00",
//       "endTime": "2022-12-06T18:00:00-08:00",
//       "isDaytime": true,
//       "temperature": 56,
//       "temperatureUnit": "F",
//       "temperatureTrend": "falling",
//       "windSpeed": "3 mph",
//       "windDirection": "ESE",
//       "icon": "https://api.weather.gov/icons/land/day/bkn?size=medium",
//       "shortForecast": "Partly Sunny",
//       "detailedForecast": "Partly sunny. High near 56, with temperatures falling to around 53 in the afternoon. East southeast wind around 3 mph."
//     },
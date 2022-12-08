use reqwest::header::USER_AGENT;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
struct Forecast {
    properties: ForecastProperties,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
struct ForecastProperties {
    updated: String,
    forecastGenerator: String,
    generatedAt: String,
    updateTime: String,
    validTimes: String,
    periods: Vec<ForecastPeriod>,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
struct ForecastPeriod {
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

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
struct Observation {
    properties: ObservationProperties,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
struct ObservationProperties {
    temperature: ObservationValue,
    windDirection: ObservationValue,
    windSpeed: ObservationValue,
    windGust: ObservationValue,
    barometricPressure: ObservationValue,
    relativeHumidity: ObservationValue,
    windChill: ObservationValue,
    heatIndex: ObservationValue,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
struct ObservationValue {
    unitCode: String,
    value: Option<f64>,
}

#[tokio::main]
async fn main() {
    // Locate grid data by lat/lon
    // let latitude: f32 = 36.1744;
    // let longitude: f32 = -115.2721;
    // let grid_url = format!("https://api.weather.gov/points/{},{}", latitude, longitude);

    // Specify grid point - this gives raw numerical data
    // let forecast_raw_url = "https://api.weather.gov/gridpoints/VEF/117,98";

    // Latest station observation
    let observation_url = "https://api.weather.gov/stations/KVGT/observations/latest";
    let observation_data = make_request(observation_url).await;

    let observation: Observation = serde_json::from_str(observation_data.as_str()).expect("could not parse observation json data");
    print!("Current Conditions:\n");

    match observation.properties.temperature.value {
        Some(v) => print!("    Temperature: {:?}\n", v),
        None => (),
    }
    match observation.properties.heatIndex.value {
        Some(v) => print!("    Heat Index: {:?}\n", v),
        None => (),
    }
    match observation.properties.windChill.value {
        Some(v) => print!("    Wind Chill: {:?}\n", v),
        None => (),
    }
    match observation.properties.windSpeed.value {
        Some(v) => print!("    Wind Speed: {:?}\n", v),
        None => (),
    }
    match observation.properties.windGust.value {
        Some(v) => print!("    Wind Gusts: {:?}\n", v),
        None => (),
    }
    match observation.properties.barometricPressure.value {
        Some(v) => print!("    Barometer: {:?}\n", v),
        None  => (),
    }
    print!("\n");

    // Forecast
    let forecast_url = "https://api.weather.gov/gridpoints/VEF/117,98/forecast";
    let forecast_data = make_request(forecast_url).await;

    print!("Forecast:\n");
    // println!("{}", forecast_data);
    let forecast: Forecast = serde_json::from_str(forecast_data.as_str()).expect("could not parse forecast json data");
    // println!("{:?}", forecast);
    let num_periods = 2;
    for (i, period) in forecast.properties.periods.iter().enumerate() {
        if i >= num_periods { break; }
        print!("    {}: {}\n", period.name, period.detailedForecast);
    }
}

async fn make_request(url: &str) -> String {
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
    content
}

// NWS Request:
//
// https://api.weather.gov/gridpoints/VEF/117,98/forecast

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
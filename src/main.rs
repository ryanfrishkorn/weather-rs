use std::io;
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

    /*
    let measurements: Vec<f64> = vec![ 12.0, 33.6, 45.0, 45.1, 89.8, 126.3, 172.3, 223.6, 331.0, 353.0 ];
    for m in measurements {
        let degrees = match degrees_to_direction(m) {
            Ok(s) => s,
            Err(e) => panic!("error obtaining direction: {}", e),
        };
        println!("direction: {:?} is {:?}", m, degrees);
    }
    */

    // Latest station observation
    let observation_url = "https://api.weather.gov/stations/KVGT/observations/latest";
    let observation_data = make_request(observation_url).await;

    let observation: Observation = serde_json::from_str(observation_data.as_str()).expect("could not parse observation json data");
    print!("Current Conditions:\n");

    match observation.properties.temperature.value {
        Some(v) => print!("    Temperature: {:.2?} \u{00B0}F / {:.2?} \u{00B0}C\n", celsius_to_fahrenheit(v), v),
        None => ()
    }
    match observation.properties.relativeHumidity.value {
        Some(v) => print!("    Humidity: {:.2?}%\n", v),
        None => ()
    }
    match observation.properties.heatIndex.value {
        Some(v) => print!("    Heat Index: {:.2?} \u{00B0}F / {:.2?} \u{00B0}C\n", celsius_to_fahrenheit(v), v),
        None => (),
    }
    match observation.properties.windChill.value {
        Some(v) => print!("    Wind Chill: {:.2?} \u{00B0}F / {:.2?} \u{00B0}C\n", celsius_to_fahrenheit(v), v),
        None => (),
    }
    match observation.properties.windDirection.value {
        Some(v) => print!("    Wind Direction: {:.0?}\u{00B0} {}\n", v, degrees_to_direction(v).unwrap()),
        None  => (),
    }
    match observation.properties.windSpeed.value {
        Some(v) => print!("    Wind Speed: {:.2?} mi/h / {:.2?} km/h\n", kilometers_to_miles(v), v),
        None => (),
    }
    match observation.properties.windGust.value {
        Some(v) => print!("    Wind Gusts: {:?} km/h\n", v),
        None => (),
    }
    match observation.properties.barometricPressure.value {
        Some(v) => print!("    Barometer: {:.0?} mbar\n", pascals_to_millibars(v)),
        None  => (),
    }
    print!("\n");

    // Forecast
    let forecast_url = "https://api.weather.gov/gridpoints/VEF/117,98/forecast";
    let forecast_data = make_request(forecast_url).await;

    print!("Forecast:\n");
    let forecast: Forecast = serde_json::from_str(forecast_data.as_str()).expect("could not parse forecast json data");
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

    let content = match response.text().await {
        Ok(t) => t,
        Err(e) => panic!("error getting body: {}", e),
    };
    content
}

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    let ratio: f64 = 9.0 / 5.0;
    let fahrenheit: f64 = ( celsius * ratio ) + 32.0;
    fahrenheit
}

fn kilometers_to_miles(kilometers: f64) -> f64 {
    if kilometers == 0.0 {
        return 0.0;
    }
    let mile_in_meters: f64 = 1609.344;
    let meters = kilometers * 1000.0;
    meters / mile_in_meters
}

fn pascals_to_millibars(pascals: f64) -> f64 {
    let millibars = pascals / 100.0;
    millibars.trunc()
}

fn degrees_to_direction(direction: f64) -> Result<String, io::Error> {
    let unit_quarter = 22.5;

    // like slicing a clock
    if direction <= 0.0 + unit_quarter {
        return Ok("North".to_string());
    }
    if direction <= 45.0 + unit_quarter {
        return Ok("Northeast".to_string());
    }
    if direction <= 90.0 + unit_quarter {
        return Ok("East".to_string());
    }
    if direction <= 135.0 + unit_quarter {
        return Ok("Southeast".to_string());
    }
    if direction <= 180.0 + unit_quarter {
        return Ok("South".to_string());
    }
    if direction <= 225.0 + unit_quarter {
        return Ok("Southwest".to_string());
    }
    if direction <=  270.0 + unit_quarter {
        return Ok("West".to_string());
    }
    if direction <= 315.0 + unit_quarter {
        return Ok("Northwest".to_string());
    }
    if direction <= 360.0 {
        return Ok("North".to_string());
    }

    Err(io::Error::new(io::ErrorKind::InvalidData, "could not discern direction"))
}

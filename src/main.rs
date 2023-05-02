use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::{include_str, io};

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
    let zip_code = "89145".to_string();

    let zip_search_result = zip_lookup(&zip_code);
    match zip_search_result {
        Ok(v) => println!("result: {:?}", v),
        Err(e) => {
            println!("result: {:?}", e);
            std::process::exit(1);
        }
    }

    // Locate grid data by lat/lon
    // let latitude: f32 = 36.1744;
    // let longitude: f32 = -115.2721;
    // let grid_url = format!("https://api.weather.gov/points/{},{}", latitude, longitude);

    // Specify grid point - this gives raw numerical data
    // let forecast_raw_url = "https://api.weather.gov/gridpoints/VEF/117,98";

    // Latest station observation
    let observation_url = "https://api.weather.gov/stations/KVGT/observations/latest"; /* NLV Airport */
    // let observation_url = "https://api.weather.gov/stations/KLSV/observations/latest"; /* Nellis AFB */
    // let observation_url = "https://api.weather.gov/stations/RRKN2/observations/latest"; /* Red Rock */
    // let observation_url = "https://api.weather.gov/stations/CMP10/observations/latest"; /* Henderson */
    let observation_data = match make_request(observation_url).await {
        Ok(d) => d,
        Err(e) => panic!("error requesting observation data: {}", e),
    };
    // println!("{:?}", observation_data);

    let observation: Observation = serde_json::from_str(observation_data.as_str())
        .expect("could not parse observation json data");
    println!("Current Conditions");

    if let Some(v) = observation.properties.temperature.value {
        println!(
            "    Temperature: {:.2?} \u{00B0}F / {:.2?} \u{00B0}C",
            celsius_to_fahrenheit(v),
            v
        );
    }
    if let Some(v) = observation.properties.relativeHumidity.value {
        println!("    Humidity: {:.2?}%", v);
    }
    if let Some(v) = observation.properties.heatIndex.value {
        println!(
            "    Heat Index: {:.2?} \u{00B0}F / {:.2?} \u{00B0}C",
            celsius_to_fahrenheit(v),
            v
        );
    }
    if let Some(v) = observation.properties.windChill.value {
        println!(
            "    Wind Chill: {:.2?} \u{00B0}F / {:.2?} \u{00B0}C",
            celsius_to_fahrenheit(v),
            v
        );
    }
    if let Some(v) = observation.properties.windSpeed.value {
        println!(
            "    Wind Speed: {:.2?} mi/h / {:.2?} km/h",
            kilometers_to_miles(v),
            v
        );
    }
    if let Some(v) = observation.properties.windDirection.value {
        println!(
            "    Wind Direction: {:.0?}\u{00B0} {}",
            v,
            degrees_to_direction(v).unwrap()
        );
    }
    if let Some(v) = observation.properties.windGust.value {
        println!("    Wind Gusts: {:?} km/h", v);
    }
    if let Some(v) = observation.properties.barometricPressure.value {
        println!("    Barometer: {:.0?} mbar", pascals_to_millibars(v));
    }
    println!();

    // Forecast
    let forecast_url = "https://api.weather.gov/gridpoints/VEF/117,98/forecast";
    let forecast_data = match make_request(forecast_url).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    println!("Forecast");
    let forecast: Forecast =
        serde_json::from_str(forecast_data.as_str()).expect("could not parse forecast json data");
    let num_periods = 2;
    for (i, period) in forecast.properties.periods.iter().enumerate() {
        if i >= num_periods {
            break;
        }
        println!("    {}: {}", period.name, period.detailedForecast);
    }
}

async fn make_request(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = match client
        .get(url)
        .header(USER_AGENT, "rust-implementation/console")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    let content = match response.text().await {
        Ok(t) => t,
        Err(e) => return Err(e),
    };
    Ok(content)
}

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    let ratio: f64 = 9.0 / 5.0;
    let fahrenheit: f64 = (celsius * ratio) + 32.0;
    fahrenheit
}

fn degrees_to_direction(direction: f64) -> Result<String, io::Error> {
    // Human readable directions to be referenced by index
    let phrase: Vec<&str> = vec![
        "North",
        "North Northeast",
        "Northeast",
        "East Northeast",
        "East",
        "East Southeast",
        "Southeast",
        "South Southeast",
        "South",
        "South Southwest",
        "Southwest",
        "West Southwest",
        "West",
        "West Northwest",
        "Northwest",
        "North Northwest",
    ];

    // This is the smallest slice of directional resolution
    const NOTCH_SIZE: f64 = 22.5;

    let mut index = 0;
    let mut notch = 0.0;

    while index <= phrase.len() && direction <= 360.0 {
        // specify first element (North) on final iteration
        if index == phrase.len() {
            return Ok(phrase[0].to_string());
        }
        if direction <= notch + (NOTCH_SIZE / 2.0) {
            return Ok(phrase[index].to_string());
        }
        notch += NOTCH_SIZE;
        index += 1;
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("could not discern direction from value {:?}", direction),
    ))
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

fn zip_lookup(zip: &str) -> Result<(f64, f64, &str, &str), &'static str> {
    // verify 5-digit code
    if zip.len() != 5 {
        return Err("zip code must be five digits");
    }

    // include zip data
    let zip_data: Vec<&str> = include_str!("zip_data.txt").split('\n').collect();

    for line in zip_data.into_iter() {
        if line.starts_with(zip) {
            // split by comma and return values
            let line_split: Vec<&str> = line.split(',').collect();
            let (city, state, lat, lon) = (
                line_split[1],
                line_split[2],
                line_split[3].parse::<f64>().unwrap(),
                line_split[4].parse::<f64>().unwrap(),
            );

            return Ok((lat, lon, city, state));
        }
    }
    Err("error")
}

use std::io;
use reqwest::header::USER_AGENT;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Forecast {
    pub properties: ForecastProperties,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
pub struct ForecastProperties {
    // updated: String,
    forecastGenerator: String,
    generatedAt: String,
    updateTime: String,
    validTimes: String,
    pub periods: Vec<ForecastPeriod>,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
pub struct ForecastPeriod {
    number: u8,
    pub name: String,
    startTime: String,
    endTime: String,
    isDaytime: bool,
    temperature: u8,
    temperatureTrend: Option<String>,
    windSpeed: String,
    windDirection: String,
    shortForecast: String,
    pub detailedForecast: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct Observation {
    pub properties: ObservationProperties,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct ObservationProperties {
    pub temperature: ObservationValue,
    pub windDirection: ObservationValue,
    pub windSpeed: ObservationValue,
    pub windGust: ObservationValue,
    pub barometricPressure: ObservationValue,
    pub relativeHumidity: ObservationValue,
    pub windChill: ObservationValue,
    pub heatIndex: ObservationValue,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
pub struct ObservationValue {
    pub unitCode: String,
    pub value: Option<f64>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct Points {
    pub properties: PointsProperties,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PointsProperties {
    pub forecast: String,
    pub observationStations: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct ObservationStationsGroup {
    observationStations: Vec<String>,
}

pub fn kilometers_to_miles(kilometers: f64) -> f64 {
    if kilometers == 0.0 {
        return 0.0;
    }
    let mile_in_meters: f64 = 1609.344;
    let meters = kilometers * 1000.0;
    meters / mile_in_meters
}

pub fn pascals_to_millibars(pascals: f64) -> f64 {
    let millibars = pascals / 100.0;
    millibars.trunc()
}

pub fn zip_lookup(zip: &str) -> Result<(f64, f64, &str, &str), &str> {
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

pub async fn station_lookup(stations_url: &str) -> Option<String> {
    let stations_data: ObservationStationsGroup = match make_request(stations_url).await {
        Ok(v) => serde_json::from_str(v.as_str()).expect("could not parse points data"),
        Err(e) => panic!("error requesting observation data: {}", e),
    };

    if stations_data.observationStations.is_empty() {
        return None;
    }
    // example station: https://api.weather.gov/stations/KVGT/observations/latest
    Some(format!(
        "{}{}",
        stations_data.observationStations[0].to_owned(),
        "/observations/latest"
    ))
}

pub async fn make_request(url: &str) -> Result<String, reqwest::Error> {
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

pub fn degrees_to_direction(direction: f64) -> Result<String, io::Error> {
    // Human-readable directions to be referenced by index
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

pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    let ratio: f64 = 9.0 / 5.0;
    let fahrenheit: f64 = (celsius * ratio) + 32.0;
    fahrenheit
}

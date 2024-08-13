mod weather_data;

use reqwest::header::USER_AGENT;
use std::error::Error;
use thiserror::Error as ThisError;
use weather_data::ObservationStationsGroup;

#[derive(Debug, ThisError)]
pub enum ProgramError {
    #[error("General program error")]
    General(String),
}

/// Converts kilometer units to miles.
pub fn kilometers_to_miles(kilometers: f64) -> f64 {
    if kilometers == 0.0 {
        return 0.0;
    }
    let mile_in_meters: f64 = 1609.344;
    let meters = kilometers * 1000.0;
    meters / mile_in_meters
}

/// Converts pascal units to millibars.
pub fn pascals_to_millibars(pascals: f64) -> f64 {
    let millibars = pascals / 100.0;
    millibars.trunc()
}

/// Looks up `zip` from zip code data and returns a tuple containing (lat, lon, city, state).
pub fn zip_lookup(zip: &str) -> Result<(f64, f64, &str, &str), Box<dyn Error>> {
    // verify 5-digit code
    if zip.len() != 5 {
        return Err(Box::new(ProgramError::General("zip code must be five digits".to_string())));
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
    Err(Box::new(ProgramError::General(format!("zip code {} could not be located", zip))))
}

/// Makes a request to NWS to determine the proper endpoint to query for observation data.
pub async fn station_lookup(stations_url: &str) -> Result<String, Box<dyn Error>> {
    let stations_data: ObservationStationsGroup = match make_request(stations_url).await {
        Ok(v) => serde_json::from_str(v.as_str()).expect("could not parse points data"),
        Err(e) => return Err(Box::new(ProgramError::General(format!("error requesting observation data: {}", e)))),
    };

    if stations_data.observationStations.is_empty() {
        return Err(Box::new(ProgramError::General("observationStations field is empty.".to_string())));
    }
    // example station: https://api.weather.gov/stations/KVGT/observations/latest
    Ok(format!(
        "{}{}",
        stations_data.observationStations[0].to_owned(),
        "/observations/latest"
    ))
}

/// Make a request to `url` and return full body text.
pub async fn make_request(url: &str) -> Result<String, Box<dyn Error>> {
    let timeout = 3000; // milliseconds
    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let client = reqwest::Client::builder().connect_timeout(std::time::Duration::from_millis(timeout)).user_agent(user_agent).build()?;
    let response = match client
        .get(url)
        .header(USER_AGENT, "rust-implementation/console")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return Err(Box::new(e)),
    };

    let content = match response.text().await {
        Ok(t) => t,
        Err(e) => return Err(Box::new(e)),
    };
    Ok(content)
}

/// Returns a human-readable direction from the provided degrees of direction.
pub fn degrees_to_direction(direction: f64) -> Result<String, Box<dyn Error>> {
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
    Err(Box::new(ProgramError::General(format!("Could not discern direction from value {:?}", direction))))
}

/// Converts celsius units to fahrenheit.
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    let ratio: f64 = 9.0 / 5.0;
    let fahrenheit: f64 = (celsius * ratio) + 32.0;
    fahrenheit
}

#[test]
fn check_zip_lookup() -> Result<(), Box<dyn Error>> {
    const ZIPS: &[&str] = &["89145", "44256"];
    const EXPECTED: &[(f64, f64, &str, &str)] = &[
        (36.167731, -115.26791, "Las Vegas", "NV"),
        (41.139849, -81.85646, "Medina", "OH"),
    ];

    for (i, zip) in ZIPS.iter().enumerate() {
        let result = zip_lookup(zip)?;
        assert_eq!(EXPECTED[i], result);
    }

    Ok(())
}

#[test]
fn check_celsius_to_fahrenheit() -> Result<(), Box<dyn Error>> {
    const DATA: &[(f64, f64)] = &[
        (45.0, 113.0),
        (32.0, 89.6),
        (0.0, 32.0),
        (-32.0, -25.6),
    ];

    for (given, expected) in DATA {
        let result = celsius_to_fahrenheit(*given);
        // eprintln!("given: {} result: {}", given, result);
        assert_eq!(result, *expected);
    }
    Ok(())
}

#[test]
// pub fn degrees_to_direction(direction: f64) -> Result<String, Box<dyn Error>> {
fn check_degrees_to_direction() -> Result<(), Box<dyn Error>> {
    const DATA: &[(f64, &str)] = &[
        (0.0, "North"), // exact
        (11.0, "North"),
        (12.0, "North Northeast"),
        (33.0, "North Northeast"),
        (34.0, "Northeast"),
        (45.0, "Northeast"), // exact
        (56.0, "Northeast"),
        (57.0, "East Northeast"),
        (78.0, "East Northeast"),
        (79.0, "East"),
        (90.0, "East"), // exact
        (101.0, "East"),
        (102.0, "East Southeast"),
        (123.0, "East Southeast"),
        (124.0, "Southeast"),
        (135.0, "Southeast"), // exact
        (146.0, "Southeast"),
        (147.0, "South Southeast"),
        (168.0, "South Southeast"),
        (169.0, "South"),
        (180.0, "South"), // exact
        (191.0, "South"),
        (192.0, "South Southwest"),
        (213.0, "South Southwest"),
        (214.0, "Southwest"),
        (225.0, "Southwest"), // exact
        (236.0, "Southwest"),
        (237.0, "West Southwest"),
        (258.0, "West Southwest"),
        (259.0, "West"),
        (270.0, "West"), // exact
        (281.0, "West"),
        (282.0, "West Northwest"),
        (303.0, "West Northwest"),
        (304.0, "Northwest"),
        (315.0, "Northwest"), // exact
        (326.0, "Northwest"),
        (327.0, "North Northwest"),
        (348.0, "North Northwest"),
        (349.0, "North"),
        (360.0, "North"), // exact
    ];

    for (degrees, direction) in DATA {
        let result = degrees_to_direction(*degrees)?;
        eprintln!("degrees: {:3} direction: {:15} result: {}", degrees, direction, result);
        assert_eq!(&result, *direction);
    }
    Ok(())
}

#[test]
// pub fn kilometers_to_miles(kilometers: f64) -> f64 {
fn check_kilometers_to_miles() -> Result<(), Box<dyn Error>> {
    const DATA: &[(f64, f64)] = &[
        (100.0, 62.13712),
        (60.0, 37.28227),
        (30.0, 18.64114),
    ];

    for (km, m) in DATA {
        let result = kilometers_to_miles(*km);
        // eprintln!("km: {} m: {} result: {}", km, m, result);
        assert_eq!(result.trunc(), m.trunc());
    }
    Ok(())
}

#[test]
// pub fn pascals_to_millibars(pascals: f64) -> f64 {
fn check_pascals_to_millibars() -> Result<(), Box<dyn Error>> {
    const DATA: &[(f64, f64)] = &[
        (800.0, 8.0),
        (300.0, 3.0),
        (200.0, 2.0),
    ];

    for (pascals, expected) in DATA {
        let result = pascals_to_millibars(*pascals);
        assert_eq!(result.trunc(), expected.trunc());
    }
    Ok(())
}

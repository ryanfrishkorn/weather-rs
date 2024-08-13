use std::env;
use std::error::Error;
use weather_rs::{
    celsius_to_fahrenheit, degrees_to_direction, kilometers_to_miles, make_request,
    pascals_to_millibars, station_lookup, zip_lookup,
};
use weather_rs::{Forecast, Observation, Points};

/// Display program usage summary and exit.
fn usage(code: i32) {
    let program_name = env::args()
        .next()
        .expect("could not determine program name");

    let usage_string = format!("usage: {} [options]", program_name);
    let options = [
        usage_string.as_str(),
        "--help         Show this help information",
        " -z <ZIP_CODE> Use zip code for location",
        " -p <NUM>      Show number of forecast periods",
    ];
    eprintln!("{}", options.join("\n        "));
    std::process::exit(code);
}

#[tokio::main]
async fn main() -> Result <(), Box<dyn Error>> {
    let mut zip_code = env::var("WEATHER_RS_ZIP").unwrap_or("89145".to_string()); // Vegas default
    let mut forecast_periods: usize = 3;

    for arg in env::args() {
        if arg == "--help" {
            usage(1);
        }
    }

    // args take precedence over env
    if let Some(code) = check_single("z", env::args()) {
        zip_code = code;
    }
    if let Some(n) = check_single("p", env::args()) {
        forecast_periods = n
            .parse::<usize>()
            .expect("error parsing number of forecast periods");
    }

    let zip_search_result = match zip_lookup(&zip_code) {
        Ok(v) => v,
        Err(e) => {
            println!("error: {:?}", e);
            std::process::exit(1);
        }
    };

    // Locate grid data by lat/lon
    // let latitude: f32 = 36.1744;
    // let longitude: f32 = -115.2721;
    let (latitude, longitude, _city, _state) = zip_search_result;
    let grid_url = format!("https://api.weather.gov/points/{},{}", latitude, longitude);

    let points: Points = match make_request(&grid_url).await {
        Ok(v) => serde_json::from_str(v.as_str()).expect("could not parse points data"),
        Err(e) => panic!("error making points request: {}", e),
    };
    let forecast_url = points.properties.forecast;

    // Obtain available stations
    let observation_url = station_lookup(&points.properties.observationStations).await?;

    // Latest station observation
    let observation_data = make_request(&observation_url).await?;
    // println!("{:?}", observation_data);

    let observation: Observation = serde_json::from_str(observation_data.as_str())
        .expect("could not parse observation json data");

    println!("Current Conditions {:?}", zip_search_result);
    print_conditions(&observation);

    // Forecast
    // sample forecast: https://api.weather.gov/gridpoints/VEF/117,98/forecast
    let forecast_data = match make_request(&forecast_url).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let forecast: Forecast =
        serde_json::from_str(forecast_data.as_str()).expect("could not parse forecast json data");
    println!("Forecast ({})", forecast_url);
    print_forecast(&forecast, forecast_periods);

    Ok(())
}

/// Print the forecast.
fn print_forecast(forecast: &Forecast, periods: usize) {
    for (i, period) in forecast.properties.periods.iter().enumerate() {
        if i >= periods {
            break;
        }
        println!("    {}: {}", period.name, period.detailedForecast);
    }
}

/// Print the conditions.
fn print_conditions(observation: &Observation) {
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
}

/// Check for the next value of an expected argument `needle`.
fn check_single(needle: &str, args: env::Args) -> Option<String> {
    let mut capture_next = false;
    for a in args {
        if capture_next {
            return Some(a);
        }
        if a == format!("{}{}", "-", needle) {
            capture_next = true;
        }
    }
    None
}

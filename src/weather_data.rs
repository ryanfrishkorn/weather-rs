use serde::Deserialize;

#[allow(non_snake_case, dead_code)]
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

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
pub struct Observation {
    pub properties: ObservationProperties,
}

#[allow(non_snake_case, dead_code)]
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

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
pub struct Points {
    pub properties: PointsProperties,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
pub struct PointsProperties {
    pub forecast: String,
    pub observationStations: String,
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Deserialize)]
pub struct ObservationStationsGroup {
    pub observationStations: Vec<String>,
}

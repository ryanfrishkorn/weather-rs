# weather-rs
### CLI for weather forecast data
All data is retrieved from the National Weather Service free API endpoints.

## Build / Install

### build
Run from project root.
```
cargo build --profile release
```
This will produce a binary in the `target/release/` directory.

### install
This will install the binary to the cargo bin dir
```
cargo install --path .
```

## Usage
There are very minimal options currently. You can specify a zip code with `-z`

```
$ weather-rs -z 89145
Current Conditions (36.167731, -115.26791, "Las Vegas", "NV")
    Temperature: 107.06 °F / 41.70 °C
    Humidity: 9.58%
    Heat Index: 100.99 °F / 38.33 °C
    Wind Speed: 10.29 mi/h / 16.56 km/h
    Wind Direction: 160° South Southeast
    Barometer: 1010 mbar

Forecast (https://api.weather.gov/gridpoints/VEF/118,99/forecast)
    This Afternoon: Sunny, with a high near 107. South southwest wind around 13 mph, with gusts as high as 18 mph.
    Tonight: Mostly clear. Low around 84, with temperatures rising to around 86 overnight. West wind 6 to 13 mph, with gusts as high as 18 mph.
    Friday: Sunny, with a high near 110. East northeast wind 5 to 9 mph.
```

The zipcode can also be specified by setting `WEATHER_RS_ZIP` in your shell startup file. For bash, this means
`.bash_profile` or `.bashrc`
```
export WEATHER_RS_ZIP=89145
```
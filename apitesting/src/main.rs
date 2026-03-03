use reqwest::{Client};
use serde::Deserialize;
use std::fs;
use serde_query::{DeserializeQuery, Query};

#[derive(DeserializeQuery)]
struct Data {
    #[query(".properties.timeseries.[].data.instant.details.air_pressure_at_sea_level")]
    pressure: Vec<f64>,
    #[query(".properties.timeseries.[].data.instant.details.relative_humidity")]
    humidity: Vec<f64>,
    #[query(".properties.timeseries.[].data.instant.details.air_temperature")]
    temperature: Vec<f64>,
}

#[derive(Deserialize, Debug)]
struct Config {
    location: Location,
}

#[derive(Deserialize, Debug)]
struct Location {
    lat: String,
    long: String,
    alt: String,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string("location_data.toml")?;
    let config: Config = toml::from_str(&config_str)?;

    let lat = config.location.lat;
    let long = config.location.long;
    let alt = config.location.alt;
    let last_update = "Tue, 16 Jun 2020 12:11:59 GMT";

    let client = Client::builder()
        .user_agent("MyTestApp/0.1")
        .build()?;

    let response = client
        .get("https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=".to_string() + &lat + "&lon=" + &long +"&altitude=" + &alt)
        .header("If-Modified-Since", last_update)
        .send()
        .await?;

    println!("Status: {}", response.status());

    //TODO: Fix status and decode results.
    match response.status() {
    reqwest::StatusCode::OK => println!("OK!"),
    reqwest::StatusCode::FORBIDDEN => println!("Sorry no access"),
    reqwest::StatusCode::NON_AUTHORITATIVE_INFORMATION => println!("Data Warning"),
    reqwest::StatusCode::UNPROCESSABLE_ENTITY => println!("No Data for location"),
    _ => println!("Unexpected status"),
}
    let output = response.text().await?;
    let data: Data = serde_json::from_str::<Query<Data>>(&output)?.into();
  
    println!("Air pressure: {:} hPa", data.pressure[0]);
    println!("Air Humidity: {:} %RH", data.humidity[0]);
    println!("Air Temperature: {:} C", data.temperature[0]);

    println!("Air density is {:} kg/cm3", calculate_air_density(data.temperature[0], data.humidity[0], data.pressure[0]));
    Ok(())

}

fn calculate_air_density(temperature:f64, relative_humidity:f64, pressure:f64) -> f64{
    let celcius_to_kelvin:f64 = 273.15;
    let spesific_gas_const_dry = 287.058;  // J/(kg·K) dry air
    let saturation_air_pressure:f64;
    let pressure_in_pascal = pressure*100_f64;

    if temperature > 0.0_f64 {
        saturation_air_pressure =  0.61078_f64 * f64::exp(17.27_f64*temperature / (temperature + 237.3_f64));
    } else {
        saturation_air_pressure =  0.61078_f64 * f64::exp(21.875_f64*temperature / (temperature + 265.5_f64));
    }

    println!("{:}",saturation_air_pressure);
    let actual_vapour_pressure = ((relative_humidity / 100_f64) * saturation_air_pressure) * 1000_f64;
    println!("{:}",actual_vapour_pressure);

    let spesific_gas_const_moist = spesific_gas_const_dry * (1_f64-0.387_f64 * (actual_vapour_pressure / pressure_in_pascal));
    println!("{:}",spesific_gas_const_moist);
    return pressure_in_pascal/(spesific_gas_const_moist*(temperature+celcius_to_kelvin)); //in kg/m3

}
use reqwest::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {


    let lat = "51.5";
    let long = "0";
    let alt = "0";
    let last_update = "Tue, 16 Jun 2020 12:11:59 GMT";

    let relative_humidity:f32 = 0.0; //in %
    let air_pressure:f32 = 0.0; //In Pa
    let air_temperature:f32 = 0.0;//In C

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
    match response.status().as_str() {
        "200 OK"=> println!("Body: {}", response.text().await?),
        "403 Forbidden"=>println!("Sorry no access"),
        "203 Non-Authoritative Information"=>println!("Data Warning"),
        "422 Unprocessable Entity"=>println!("No Data for location"),
        _=> println!("Unexpected status"),
    }
    
    println!("Air density is {:}", calculate_air_density(air_temperature, relative_humidity, air_pressure));
    Ok(())

}

fn calculate_air_density(temperature:f32, relative_humidity:f32, pressure:f32) -> f32{
    let CELCIUS_TO_KELVIN:f32 = 273.15;
    let sat_air_press:f32;

    if temperature > 0.0_f32 {
        sat_air_press =  0.61078_f32.powf(17.27_f32*temperature / (temperature + 237.3_f32));
    } else {
        sat_air_press =  0.61078_f32.powf(21.875_f32*temperature / (temperature + 265.5_f32));
    }

    let act_sat_press = ((relative_humidity / 100_f32) * sat_air_press) * 1000_f32;
    let spes_gas_const_dry = 287.05;
    let spes_gas_const_moist = spes_gas_const_dry * (1_f32-0.387_f32 * (act_sat_press / pressure));

    return pressure/(spes_gas_const_moist*(temperature+CELCIUS_TO_KELVIN)); //in kg/m3

}
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Report {
    pub city: String,
    pub temperature: f32,
    pub condition: &'static str,
    pub high: f32,
    pub low: f32,
}

#[derive(Deserialize)]
struct Geocoding {
    results: Option<Vec<Place>>,
}

#[derive(Deserialize)]
struct Place {
    name: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct Forecast {
    current: Current,
    daily: Daily,
}

#[derive(Deserialize)]
struct Current {
    temperature_2m: f32,
    weather_code: u8,
}

#[derive(Deserialize)]
struct Daily {
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
}

fn describe(code: u8) -> &'static str {
    match code {
        0 => "Clear",
        1 | 2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51..=57 => "Drizzle",
        61..=67 | 80..=82 => "Rain",
        71..=77 | 85 | 86 => "Snow",
        95..=99 => "Thunderstorm",
        _ => "Unknown",
    }
}

async fn request(city: String) -> Result<Report, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let geocoding: Geocoding = client
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query(&[
            ("name", city.as_str()),
            ("count", "1"),
            ("language", "en"),
            ("format", "json"),
        ])
        .send()
        .await?
        .json()
        .await?;

    let place = geocoding
        .results
        .and_then(|results| results.into_iter().next())
        .ok_or_else(|| format!("No city named \"{city}\" found"))?;

    let forecast: Forecast = client
        .get("https://api.open-meteo.com/v1/forecast")
        .query(&[
            ("latitude", place.latitude.to_string()),
            ("longitude", place.longitude.to_string()),
            ("current", "temperature_2m,weather_code".to_string()),
            ("daily", "temperature_2m_max,temperature_2m_min".to_string()),
            ("timezone", "auto".to_string()),
            ("forecast_days", "1".to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;

    Ok(Report {
        city: place.name,
        temperature: forecast.current.temperature_2m,
        condition: describe(forecast.current.weather_code),
        high: forecast.daily.temperature_2m_max.first().copied().ok_or("no forecast")?,
        low: forecast.daily.temperature_2m_min.first().copied().ok_or("no forecast")?,
    })
}

pub async fn fetch(city: String) -> Result<Report, String> {
    request(city).await.map_err(|error| error.to_string())
}

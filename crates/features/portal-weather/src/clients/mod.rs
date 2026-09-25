mod forecast;
mod open_meteo;

#[cfg(test)]
mod tests;

pub use forecast::Forecast;
pub use open_meteo::OpenMeteo;

use std::env;

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub user: String,
    pub db_name: String,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub resend_api_key: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let _ = dotenvy::dotenv();

        let user = env::var("POSTGRES_USER")
            .map_err(|_| "POSTGRES_USER missing from environment".to_string())?;

        let db_name = env::var("POSTGRES_DB")
            .map_err(|_| "POSTGRES_DB missing from environment".to_string())?;

        let url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL missing from environment".to_string())?;

        let resend_api_key = env::var("RESEND_API_KEY")
            .map_err(|_| "RESEND_API_KEY missing from environment".to_string())?;

        Ok(Self {
            database: DatabaseConfig { user, db_name, url },
            resend_api_key,
        })
    }
}

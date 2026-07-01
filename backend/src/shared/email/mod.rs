#[cfg(test)]
mod mock;

#[cfg(test)]
pub use mock::MockEmailService;

mod resend;
pub use resend::ResendEmailService;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmailError {
    #[error("Configuration mismatch")]
    InvalidConfiguration(String),
    #[error("Email provider could not handle request")]
    ProviderError(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct OutboundEmail {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub html_body: String,
}

#[async_trait::async_trait]
pub trait EmailService: Send + Sync {
    async fn send(&self, message: OutboundEmail) -> Result<(), EmailError>;
}

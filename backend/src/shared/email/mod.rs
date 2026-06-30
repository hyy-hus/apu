mod mock;
mod resend;

// pub use resend::ResendEmailService;
// pub use mock::MockEmailService;

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

pub trait EmailService: Send + Sync {
    async fn send(&self, message: OutboundEmail) -> Result<(), EmailError>;
}

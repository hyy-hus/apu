use resend_rs::Resend;

use crate::shared::email::{EmailError, EmailService, OutboundEmail};

pub struct ResendEmailService {
    client: Resend,
}

impl ResendEmailService {
    fn new(api_key: &str) -> Self {
        Self {
            client: Resend::new(api_key),
        }
    }
}

impl EmailService for ResendEmailService {
    async fn send(&self, _message: OutboundEmail) -> Result<(), EmailError> {
        Err(EmailError::InvalidConfiguration(
            "Intented to fail".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Result<(ResendEmailService, OutboundEmail), EmailError> {
        let _ = dotenvy::dotenv();

        let api_key = std::env::var("RESEND_API_KEY").map_err(|_| {
            EmailError::InvalidConfiguration("RESEND_API_KEY is not set in environment".to_string())
        })?;

        let service = ResendEmailService::new(&api_key);

        let message = OutboundEmail {
            from: "Test Sender <test.sender@test.com>".to_string(),
            to: vec!["delivered@resend.dev".to_string()],
            subject: "Test message".to_string(),
            html_body: "Test".to_string(),
        };

        Ok((service, message))
    }

    #[tokio::test]
    async fn sending_works() -> Result<(), EmailError> {
        let (service, message) = setup()?;

        service.send(message).await?;
        Ok(())
    }
}

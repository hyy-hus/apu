use resend_rs::{Resend, types::CreateEmailBaseOptions};

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
    async fn send(&self, message: OutboundEmail) -> Result<(), EmailError> {
        let recipients: Vec<&str> = message.to.iter().map(|s| s.as_str()).collect();

        self.client
            .emails
            .send(
                CreateEmailBaseOptions::new(&message.from, recipients, &message.subject)
                    .with_html(&message.html_body),
            )
            .await
            .map_err(|e| EmailError::ProviderError(e.to_string()))?;

        Ok(())
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
            from: "Test Sender <onboarding@resend.dev>".to_string(),
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

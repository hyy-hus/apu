#[cfg(test)]
use crate::shared::email::{EmailError, EmailService, OutboundEmail};
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use tracing::info;

#[cfg(test)]
pub struct MockEmailService {
    queue: Mutex<Vec<OutboundEmail>>,
}

#[cfg(test)]
impl MockEmailService {
    fn new() -> Self {
        Self {
            queue: Mutex::new(vec![]),
        }
    }
}

#[cfg(test)]
impl EmailService for MockEmailService {
    async fn send(&self, message: OutboundEmail) -> Result<(), EmailError> {
        info!("Sending message: {:?}", message);

        let mut guard = self.queue.lock().unwrap();
        guard.push(message);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (MockEmailService, OutboundEmail) {
        let service = MockEmailService::new();
        let message = OutboundEmail {
            from: "Test Sender <test.sender@test.com>".to_string(),
            to: vec!["test.receiver@test.com".to_string()],
            subject: "Test message".to_string(),
            html_body: "Test".to_string(),
        };

        (service, message)
    }

    #[tokio::test]
    async fn sending_works() -> Result<(), EmailError> {
        let (service, message) = setup();

        service.send(message).await?;
        Ok(())
    }

    #[tokio::test]
    async fn message_in_queue() -> Result<(), EmailError> {
        let (service, message) = setup();

        service.send(message.clone()).await?;
        assert_eq!(service.queue.lock().unwrap().first(), Some(&message));
        Ok(())
    }
}

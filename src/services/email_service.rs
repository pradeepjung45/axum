use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use rust_decimal::Decimal;

#[derive(Clone)]
pub struct EmailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl EmailService {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_user: String,
        smtp_password: String,
        smtp_from: String,
    ) -> Self {
let creds = Credentials::new(smtp_user, smtp_password);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
            .unwrap()
            .port(smtp_port)
            .credentials(creds)
            .build();

        Self {
            mailer,
            from: smtp_from,
        }
    }

    pub async fn send_transfer_success(&self, to: &str, amount: Decimal) {
        let subject = "MyFintechApp: Transfer Successful";
        let body = format!(
            "Transfer Successful!\n\nYou have successfully sent ${}.",
            amount
        );

        let email = Message::builder()
            .from(self.from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .unwrap();

        match self.mailer.send(email).await {
            Ok(_) => println!("✅ Email sent successfully to {}", to),
            Err(e) => eprintln!("❌ Failed to send email: {:?}", e),
        }
    }
}

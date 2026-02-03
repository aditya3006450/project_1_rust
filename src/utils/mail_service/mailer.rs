use lettre::AsyncSmtpTransport;
use lettre::AsyncTransport;
use lettre::Message;
use lettre::Tokio1Executor;
use lettre::message::MultiPart;
use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials;

use crate::utils::mail_service::mail_data::MailData;
use crate::utils::tera_service::tera_renderer::TeraRenderer;

pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl Mailer {
    pub fn new() -> Self {
        let smtp_user = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME Missing");
        let smtp_password = std::env::var("SMTP_PASSWORD").expect("SMTP_USERNAME Missing");
        let sender_mail = std::env::var("SENDER_MAIL").expect("SENDER_MAIL Missing");

        let creds = Credentials::new(smtp_user, smtp_password);
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .expect("SMTP relay invalid")
            .credentials(creds)
            .build();

        Self {
            transport,
            from: sender_mail,
        }
    }

    pub async fn send(
        &self,
        renderer: &TeraRenderer,
        mail: MailData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let html = if let Some(raw) = mail.raw_html {
            raw
        } else if let Some(tpl) = mail.template {
            renderer.render(&tpl, mail.context).unwrap()
        } else {
            String::from("No content")
        };

        let builder = Message::builder()
            .from(self.from.parse()?)
            .to(mail.to.parse()?);
        let email = builder
            .subject(mail.subject)
            .multipart(MultiPart::alternative().singlepart(SinglePart::html(html)))?;

        self.transport.send(email).await?;
        Ok(())
    }
}

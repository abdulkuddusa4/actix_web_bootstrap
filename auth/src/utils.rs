use lettre::{AsyncSmtpTransport, Message, Tokio1Executor, AsyncTransport};
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use rand::distr::Alphanumeric;

use rand::{ Rng, RngExt};
// use rand::distributions::Alphanumeric;
pub fn generate_random_string(length: usize) -> String {
    let rng = rand::rng();
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub async fn send_mail(
    mail_cred: Credentials,
    subject: &str,
    receipient: crate::common::Email,
    message:&str,
    is_html: bool
)
-> Result<(), String>
{
    let email = Message::builder()
        .from(Mailbox::new(Some("NoBody".to_owned()), "nobody@domain.tld".parse().unwrap()))
        .to(Mailbox::new(
            None, 
            receipient.as_ref().parse().unwrap()
        ))
        .subject(subject)
        .header(if is_html {ContentType::TEXT_HTML} else {ContentType::TEXT_PLAIN})
        .body(message.to_owned())
        .unwrap();

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .unwrap()
        .credentials(mail_cred.clone())
        .build();

    match mailer.send(email).await {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => return Err(e.to_string()),
    }
    Ok(())
}


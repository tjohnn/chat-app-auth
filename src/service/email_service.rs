use lettre::{smtp::authentication::Credentials, Transport, SmtpClient};
use lettre_email::Email;
use lettre_email::Mailbox;

pub fn send_otp_email(otp: &str, email: &str, name: &str)
    -> Result<(), String> { 
    let subject = "Chat App OTP";
    let body = &format!("<h3>Your chat app otp is {}.</h3> <p>OTP expires in 5 minutes</p>", otp);
    
    send_email(subject, body, (email, name))
}

fn send_email(subject: &str, body: &str, to: (&str, &str)) -> Result<(), String> {
    let email = Email::builder()
        .from(Mailbox::new_with_name("Chat App".to_owned(), "tjohndeveloper@gmail.com".to_owned()))
        .to(to)
        .subject(subject)
        .html(body)
        .build()
        .unwrap();

    // get username and password from .env
    let username = match std::env::var("SMTP_GMAIL_USERNAME") {
        Ok(username) => username,
        Err(e) => {
            return Err(e.to_string())
        }
    };
    let password = match std::env::var("SMTP_GMAIL_PASSWORD") {
        Ok(username) => username,
        Err(e) => {
            return Err(e.to_string())
        }
    };

    let creds = Credentials::new(username, password);

    // Open a remote connection to gmail
    let mut mailer = SmtpClient::new_simple("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .transport();

    // Send the email
    if let Err(e) = mailer.send(email.into()) {
        Err(e.to_string())
    } else {
        Ok(())
    }
}
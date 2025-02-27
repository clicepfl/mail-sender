use std::collections::HashMap;
use std::fs;

use dotenv::dotenv;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, routes};
use serde::Deserialize;

#[derive(Deserialize)]
struct EmailRequest {
    template_name: String,
    ics_name: Option<String>,
    email_address: String,
    subject: String,
    parameters: HashMap<String, String>,
}

#[post("/send?<secret>", format = "json", data = "<data>")]
fn send(secret: String, data: Json<EmailRequest>) -> Status {
    // Load environment variables
    dotenv().ok();
    let email_username = match std::env::var("EMAIL_USERNAME") {
        Ok(email_username) => email_username,
        Err(e) => {
            eprintln!("Error reading EMAIL_USERNAME: {:#?}", e);
            return Status::InternalServerError;
        }
    };
    let email_password = match std::env::var("EMAIL_PASSWORD") {
        Ok(email_password) => email_password,
        Err(e) => {
            eprintln!("Error reading EMAIL_PASSWORD: {:#?}", e);
            return Status::InternalServerError;
        }
    };
    let email_server = match std::env::var("EMAIL_SERVER") {
        Ok(email_server) => email_server,
        Err(e) => {
            eprintln!("Error reading EMAIL_SERVER: {:#?}", e);
            return Status::InternalServerError;
        }
    };
    let email_from = match std::env::var("EMAIL_FROM") {
        Ok(email_from) => email_from,
        Err(e) => {
            eprintln!("Error reading EMAIL_FROM: {:#?}", e);
            return Status::InternalServerError;
        }
    };
    let expected_secret = match std::env::var("SECRET") {
        Ok(expected_secret) => expected_secret,
        Err(e) => {
            eprintln!("Error reading SECRET: {:#?}", e);
            return Status::InternalServerError;
        }
    };

    // Check secret
    if secret != expected_secret {
        return Status::Unauthorized;
    }

    // Read template file
    let template_name = &data.template_name;
    let template_file = match fs::read_to_string(format!("templates/{template_name}.liquid")) {
        Ok(template_file) => template_file,
        Err(e) => {
            eprintln!("Error reading template file: {:#?}", e);
            return Status::InternalServerError;
        }
    };

    // Render template
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(&template_file)
        .unwrap();
    let body = template.render(&data.parameters).unwrap();

    // Create email message
    let mut multipart = MultiPart::alternative().singlepart(SinglePart::html(body.to_string()));

    // Attach ICS file
    if let Some(ics_name) = &data.ics_name { match fs::read(format!("ics/{ics_name}.ics")) {
        Ok(ics) => {
            multipart = multipart.singlepart(
                Attachment::new(format!("{}.ics", ics_name))
                    .body(ics, ContentType::parse("text/calendar").unwrap()),
            );
        }
        Err(e) => {
            eprintln!("Error reading ICS file: {:#?}", e);
            return Status::InternalServerError;
        }
    } };

    // Create email
    let email = Message::builder()
        .from(email_from.parse().unwrap())
        .to(data.email_address.parse().unwrap())
        .subject(&data.subject)
        .multipart(multipart)
        .unwrap();

    // Create credentials
    let creds = Credentials::new(email_username, email_password);

    // Open a remote connection to mail
    let mailer = SmtpTransport::starttls_relay(&email_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => Status::Ok,
        Err(e) => {
            eprintln!("Error sending email: {:#?}", e);
            Status::InternalServerError
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "
    USAGE

      POST /send

          accepts a JSON object with the following keys:

            - template_name: the name of the template to use
            - ics_name: the name of the ICS file to attach (optional)
            - email_address: the email address to send the email to
            - subject: the subject of the email
            - parameters: a map of parameters to pass to the template

          as well as a secret key in the Authorization header
    "
}

#[rocket::main]
async fn main() {
    rocket::build()
        .mount("/mail-sender", routes![index, send])
        .launch()
        .await
        .unwrap();
}

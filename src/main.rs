use dotenv::dotenv;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use liquid::Object;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, routes};
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use svg2pdf::usvg::fontdb;
use svg2pdf::{usvg, ConversionOptions, PageOptions};

#[derive(Deserialize)]
struct EmailRequest {
    template_name: String,
    ics_name: Option<String>,
    email_address: String,
    subject: String,
    parameters: Object,
    qrbill_params: Option<serde_json::Value>,
}

#[post("/send?<secret>", format = "json", data = "<data>")]
async fn send(secret: String, data: Json<EmailRequest>) -> Status {
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
    if let Some(ics_name) = &data.ics_name {
        match fs::read(format!("ics/{ics_name}.ics")) {
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
        }
    };

    // Attach QR bill file
    if let Some(qrbill_params) = &data.qrbill_params {
        let client = reqwest::Client::new();
        let qrbill_response = match client
            .post("https://clic.epfl.ch/qrbill-generator/")
            .json(&qrbill_params)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error calling QR bill API: {:#?}", e);
                return Status::InternalServerError;
            }
        };

        let svg_data = match qrbill_response.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Error reading QR bill response: {:#?}", e);
                return Status::InternalServerError;
            }
        };

        let mut font_db = fontdb::Database::new();
        font_db
            .load_font_file("./fonts/LiberationSans-Regular.ttf")
            .unwrap();
        let font_db_arc = Arc::new(font_db);

        let opt = usvg::Options {
            font_family: "Liberation Sans".to_string(),
            fontdb: font_db_arc,
            ..Default::default()
        };

        let rtree = match usvg::Tree::from_data(&svg_data, &opt) {
            Ok(tree) => tree,
            Err(e) => {
                eprintln!("Error parsing SVG: {:#?}", e);
                return Status::InternalServerError;
            }
        };

        let pdf_data =
            match svg2pdf::to_pdf(&rtree, ConversionOptions::default(), PageOptions::default()) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error converting SVG to PDF: {:#?}", e);
                    return Status::InternalServerError;
                }
            };

        // Use the pdf_data vector directly for the attachment
        multipart = multipart.singlepart(
            Attachment::new("qrbill.pdf".to_string())
                .body(pdf_data, ContentType::parse("application/pdf").unwrap()),
        );
    };

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
            - email_address: the email address to send the email to
            - subject: the subject of the email
            - parameters: a map of parameters to pass to the template
            - ics_name: the name of the ICS file to attach (optional)
            - qrbill_params: a JSON object with the parameters for the QR bill (optional)

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

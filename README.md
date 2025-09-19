# Mail Sender API

This repository is a small Rust backend used to send emails. The emails use [Liquid](https://shopify.github.io/liquid/) templates and may contain [ICS](https://icalendar.org/) attachments for calendar invitations, as well as QR Bills made with our [QR Bill Generator API](https://github.com/clicepfl/qrbill-generator/)

## Usage

Available at http://clic.epfl.ch/mail-sender for sending emails from it.clic@epfl.ch (requires password access).

Can be run locally for other uses, see below.

### Templates

Liquid templates must be placed in the `templates` folder and ICS files must be placed in the `ics` folder.

### API

#### POST /mail-sender/send

This endpoint sends an email based on a specified template and attachments.

##### Authentication 

Requires a secret key to be passed as a URL query parameter (e.g., `/mail-sender/send?secret=<secret_key>`).

##### Request Body

JSON object with the following field:

| Field | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `template_name` | `string` | Yes | The name of the Liquid template file to use (without `.liquid`) |
| `email_address` | `string` | Yes | The recipient's email address |
| `subject` | `string` | Yes | The subject of the email |
| `parameters` | `object` | Yes | A key-value map for rendering the template |
| `ics_name` | `string` | No | The name of the ICS file to attach (without `.ics`) |
| `qrbill_params` | `object` | No | Parameters for generating and attaching a QR bill, see [QR Bill Generator](https://github.com/clicepfl/qrbill-generator/) |

You can find an example request body in `example.json`

## Local Use

Clone the repository, set the approriate environment variables in a `.env` file and add your ics files or liquid templates to the corresponding folders (`ics` and `templates`).

The required environment variables are shown in `example.env`:

- `EMAIL_USERNAME` the username to connect to the SMTP server (e.g. `it.clic`)
- `EMAIL_PASSWORD` the password to connect to the SMTP server
- `EMAIL_SERVER` the SMTP server (e.g. `mail.epfl.ch`)
- `EMAIL_FROM` the sender of the email in the typical format (e.g. `CLIC <it.clic@epfl.ch>`)
- `SECRET` the secret required to make requests

Run `cargo run` to start the backend.

Use the following command to make requests to the backend (if run locally):

```bash
curl http://127.0.0.1:8000/mail-sender/send\?secret\=<example_secret> -X POST -H 'Content-Type: application/json' -d '@<body.json>'
```

Where `<example_secret>` should be the same secret as stored in the `SECRET` environment variable, and `<body.json>` is a file containing the request body (an example request body is given in `example.json`).

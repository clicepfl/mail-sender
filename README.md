# Mail Sender

This repository is a small Rust backend used to send emails. The emails use [Liquid](https://shopify.github.io/liquid/) templates and may contain [ICS](https://icalendar.org/) attachments for calendar invitations.

## Templates

Liquid templates must be placed in the `templates` folder and ICS files must be placed in the `ics` folder.

## Usage

The required environment variables are shown in `example.env`:

- `EMAIL_USERNAME` the username to connect to the SMTP server (e.g. `it.clic`)
- `EMAIL_PASSWORD` the password to connect to the SMTP server
- `EMAIL_SERVER` the SMTP server (e.g. `mail.epfl.ch`)
- `EMAIL_FROM` the sender of the email in the typical format (e.g. `CLIC <it.clic@epfl.ch>`)
- `SECRET` the secret required to make requests

Run `cargo run` to start the backend.

Use the following command to make requests to the backend (if run locally):

```bash
curl http://127.0.0.1:8000/send\?secret\=<example_secret> -X POST -H 'Content-Type: application/json' -d '@<body.json>'
```

Where `<example_secret>` should be the same secret as stored in the `SECRET` environment variable, and `<body.json>` is a file containing the request body (an example request body is given in `example.json`).

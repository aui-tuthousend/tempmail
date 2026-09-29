use anyhow::{Context, Result};
use lettre::message::header::{ContentDisposition, ContentTransferEncoding, ContentType};
use lettre::message::{Mailbox, MessageBuilder, MultiPart, SinglePart};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use shared::config::SmtpRelayConfig;
use tracing::info;

use crate::repository::{OutboxAttachmentRow, OutboxMessageRecord};
use crate::storage::ObjectStorage;

enum BodyContent {
    Single(SinglePart),
    Multi(MultiPart),
}

fn parse_addresses(json: &serde_json::Value) -> Result<Vec<Mailbox>> {
    let arr = json.as_array().context("addresses json is not an array")?;
    let mut boxes = Vec::new();

    for item in arr {
        let (address, name) = match item.as_str() {
            Some(address) => (address, None),
            None => {
                let address = item["address"]
                    .as_str()
                    .context("address field is missing")?;
                (address, item["name"].as_str())
            }
        };

        let mbox = match name {
            Some(name) if !name.is_empty() => format!("{name} <{address}>")
                .parse()
                .context("failed to parse mailbox with name")?,
            _ => address.parse().context("failed to parse mailbox")?,
        };
        boxes.push(mbox);
    }

    Ok(boxes)
}

fn build_body_content(text_body: Option<&str>, html_body: Option<&str>) -> BodyContent {
    let has_text = text_body.is_some_and(|b| !b.is_empty());
    let has_html = html_body.is_some_and(|b| !b.is_empty());

    match (has_text, has_html) {
        (true, true) => {
            let text_part = SinglePart::builder()
                .header(ContentType::TEXT_PLAIN)
                .body(String::from(text_body.unwrap_or("")));
            let html_part = SinglePart::builder()
                .header(ContentType::TEXT_HTML)
                .body(String::from(html_body.unwrap_or("")));
            BodyContent::Multi(
                MultiPart::alternative()
                    .singlepart(text_part)
                    .singlepart(html_part),
            )
        }
        (true, false) => BodyContent::Single(
            SinglePart::builder()
                .header(ContentType::TEXT_PLAIN)
                .body(String::from(text_body.unwrap_or(""))),
        ),
        (false, true) => BodyContent::Single(
            SinglePart::builder()
                .header(ContentType::TEXT_HTML)
                .body(String::from(html_body.unwrap_or(""))),
        ),
        (false, false) => BodyContent::Single(SinglePart::builder().body(String::new())),
    }
}

pub async fn send_outbox_message(
    record: &OutboxMessageRecord,
    from_address: &str,
    attachments: &[OutboxAttachmentRow],
    object_storage: &ObjectStorage,
    smtp: &SmtpRelayConfig,
) -> Result<()> {
    let mailer = build_mailer(smtp)?;

    info!(
        smtp_from_name = %smtp.from_name,
        from_address = %from_address,
        "building email message"
    );

    let from: Mailbox = format!("{} <{}>", smtp.from_name, from_address)
        .parse()
        .context("failed to parse from mailbox")?;

    let to_addresses = parse_addresses(&record.to_addresses)?;
    let cc_addresses = parse_optional_addresses(record.cc_addresses.as_ref())?;
    let bcc_addresses = parse_optional_addresses(record.bcc_addresses.as_ref())?;

    let mut msg_builder = MessageBuilder::new()
        .from(from.clone())
        .subject(record.subject.as_deref().unwrap_or("(no subject)"));

    for to in &to_addresses {
        msg_builder = msg_builder.to(to.clone());
    }
    for cc in &cc_addresses {
        msg_builder = msg_builder.cc(cc.clone());
    }
    for bcc in &bcc_addresses {
        msg_builder = msg_builder.bcc(bcc.clone());
    }
    if let Some(ref in_reply_to) = record.in_reply_to {
        msg_builder = msg_builder.in_reply_to(in_reply_to.clone());
    }

    let body = build_body_content(record.text_body.as_deref(), record.html_body.as_deref());

    if attachments.is_empty() {
        let msg = match body {
            BodyContent::Single(part) => msg_builder.singlepart(part)?,
            BodyContent::Multi(multi) => msg_builder.multipart(multi)?,
        };
        mailer.send(msg).await?;
    } else {
        // Build mixed multipart: body (single or alternative) + attachments
        let mut mixed = match body {
            BodyContent::Single(part) => MultiPart::mixed().singlepart(part),
            BodyContent::Multi(multi) => MultiPart::mixed().multipart(multi),
        };

        for attachment_row in attachments {
            let file_bytes = object_storage
                .get_attachment_bytes(&attachment_row.storage_key)
                .await?
                .unwrap_or_default();

            let content_type = attachment_row
                .content_type
                .as_deref()
                .and_then(|ct| ContentType::parse(ct).ok())
                .unwrap_or(ContentType::parse("application/octet-stream").unwrap());

            let attachment_part = SinglePart::builder()
                .header(content_type)
                .header(ContentDisposition::attachment(&attachment_row.filename))
                .header(ContentTransferEncoding::Base64)
                .body(file_bytes);

            mixed = mixed.singlepart(attachment_part);
        }

        let msg = msg_builder.multipart(mixed)?;
        mailer.send(msg).await?;
    }

    info!(
        outbox_id = %record.id,
        account_id = %record.account_id,
        to_count = to_addresses.len(),
        cc_count = cc_addresses.len(),
        bcc_count = bcc_addresses.len(),
        has_attachments = record.has_attachments,
        "email sent via SMTP"
    );

    Ok(())
}

fn parse_optional_addresses(json: Option<&serde_json::Value>) -> Result<Vec<Mailbox>> {
    match json {
        Some(val) if !val.is_null() => parse_addresses(val),
        _ => Ok(vec![]),
    }
}

fn build_mailer(smtp: &SmtpRelayConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = lettre::transport::smtp::authentication::Credentials::new(
        smtp.username.clone(),
        smtp.password.clone(),
    );

    let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)?
        .port(smtp.port)
        .credentials(creds)
        .build();

    Ok(transport)
}

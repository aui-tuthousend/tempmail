use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use redis::aio::ConnectionManager;
use shared::models::{Envelope, RawEmail};
use shared::queue::QueueMessage;
use shared::redis_helper::push_queue_message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, info, warn};

use crate::config::SmtpReceiverConfig;

#[derive(Clone)]
pub struct AppState {
    config: Arc<SmtpReceiverConfig>,
    redis: ConnectionManager,
    tls_acceptor: Option<TlsAcceptor>,
}

impl AppState {
    pub fn new(
        config: SmtpReceiverConfig,
        redis: ConnectionManager,
        tls_acceptor: Option<TlsAcceptor>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            redis,
            tls_acceptor,
        }
    }
}

pub async fn run(state: AppState) -> Result<()> {
    let listener = TcpListener::bind(&state.config.listen_addr)
        .await
        .with_context(|| format!("failed to bind SMTP listener: {}", state.config.listen_addr))?;

    info!(listen_addr = %state.config.listen_addr, "smtp receiver listening");

    loop {
        let (stream, remote_addr) = listener.accept().await?;
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, remote_addr, state).await {
                warn!(%remote_addr, %error, "smtp connection failed");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    remote_addr: SocketAddr,
    state: AppState,
) -> Result<()> {
    let mut session = SmtpSession::new(stream, remote_addr, state);
    session.run().await
}

struct SmtpSession {
    stream: Option<SessionStream>,
    remote_addr: SocketAddr,
    state: AppState,
    helo_seen: bool,
    mail_from: Option<String>,
    rcpt_to: Vec<String>,
    accepted_messages: usize,
    tls_active: bool,
}

impl SmtpSession {
    fn new(stream: TcpStream, remote_addr: SocketAddr, state: AppState) -> Self {
        Self {
            stream: Some(SessionStream::Plain(stream)),
            remote_addr,
            state,
            helo_seen: false,
            mail_from: None,
            rcpt_to: Vec::new(),
            accepted_messages: 0,
            tls_active: false,
        }
    }

    async fn run(&mut self) -> Result<()> {
        self.write_response(
            220,
            &format!("{} ESMTP TempMail", self.state.config.hostname),
        )
        .await?;

        while let Some(line) = self.read_line().await? {
            let line = trim_smtp_line(&line);
            if line.is_empty() {
                self.write_response(500, "empty command").await?;
                continue;
            }

            debug!(%line, remote_addr = %self.remote_addr, "smtp command");

            let (command, args) = split_command(line);
            match command.as_str() {
                "HELO" => self.handle_helo(args).await?,
                "EHLO" => self.handle_ehlo(args).await?,
                "STARTTLS" => self.handle_starttls().await?,
                "MAIL" => self.handle_mail(args).await?,
                "RCPT" => self.handle_rcpt(args).await?,
                "DATA" => self.handle_data().await?,
                "RSET" => self.handle_rset().await?,
                "NOOP" => self.write_response(250, "ok").await?,
                "QUIT" => {
                    self.write_response(221, "bye").await?;
                    break;
                }
                _ => self.write_response(502, "command not implemented").await?,
            }
        }

        Ok(())
    }

    async fn handle_helo(&mut self, args: &str) -> Result<()> {
        if args.trim().is_empty() {
            self.write_response(501, "hostname required").await?;
            return Ok(());
        }

        self.helo_seen = true;
        self.write_response(250, &self.state.config.hostname.clone())
            .await
    }

    async fn handle_ehlo(&mut self, args: &str) -> Result<()> {
        if args.trim().is_empty() {
            self.write_response(501, "hostname required").await?;
            return Ok(());
        }

        self.helo_seen = true;
        self.write_line(&format!("250-{}", self.state.config.hostname))
            .await?;
        self.write_line(&format!("250-SIZE {}", self.state.config.max_message_bytes))
            .await?;

        if self.state.config.starttls_configured() && !self.tls_active {
            self.write_line("250-STARTTLS").await?;
        }

        self.write_line("250 HELP").await
    }

    async fn handle_starttls(&mut self) -> Result<()> {
        if self.tls_active {
            self.write_response(503, "TLS already active").await?;
            return Ok(());
        }

        let Some(acceptor) = self.state.tls_acceptor.clone() else {
            self.write_response(454, "TLS not available").await?;
            return Ok(());
        };

        self.write_response(220, "ready to start TLS").await?;
        let stream = self
            .stream
            .take()
            .ok_or_else(|| anyhow!("SMTP stream is closed"))?;
        let SessionStream::Plain(stream) = stream else {
            return Err(anyhow!("invalid STARTTLS state"));
        };

        let tls_stream = acceptor
            .accept(stream)
            .await
            .context("STARTTLS handshake failed")?;
        self.stream = Some(SessionStream::Tls(Box::new(tls_stream)));
        self.tls_active = true;
        self.helo_seen = false;
        self.reset_envelope();
        Ok(())
    }

    async fn handle_mail(&mut self, args: &str) -> Result<()> {
        if !self.helo_seen {
            self.write_response(503, "send HELO/EHLO first").await?;
            return Ok(());
        }

        if self.accepted_messages >= self.state.config.max_messages_per_connection {
            self.write_response(421, "message limit reached").await?;
            return Ok(());
        }

        let Some(address) = extract_path(args, "FROM:") else {
            self.write_response(501, "MAIL FROM address required")
                .await?;
            return Ok(());
        };

        self.mail_from = Some(address);
        self.rcpt_to.clear();
        self.write_response(250, "sender ok").await
    }

    async fn handle_rcpt(&mut self, args: &str) -> Result<()> {
        if self.mail_from.is_none() {
            self.write_response(503, "send MAIL first").await?;
            return Ok(());
        }

        let Some(address) = extract_path(args, "TO:") else {
            self.write_response(501, "RCPT TO address required").await?;
            return Ok(());
        };

        if !recipient_is_local(&address, &self.state.config.mailbox_domain) {
            self.write_response(550, "relay denied").await?;
            return Ok(());
        }

        self.rcpt_to.push(address);
        self.write_response(250, "recipient ok").await
    }

    async fn handle_data(&mut self) -> Result<()> {
        if self.mail_from.is_none() || self.rcpt_to.is_empty() {
            self.write_response(503, "sender and recipient required")
                .await?;
            return Ok(());
        }

        self.write_response(354, "end data with <CR><LF>.<CR><LF>")
            .await?;

        let data = match self.read_data().await? {
            DataRead::Complete(data) => data,
            DataRead::TooLarge => {
                self.write_response(552, "message too large").await?;
                self.reset_envelope();
                return Ok(());
            }
        };

        let envelope = Envelope {
            mail_from: self.mail_from.clone().unwrap_or_default(),
            rcpt_to: self.rcpt_to.clone(),
            remote_addr: Some(self.remote_addr.ip().to_string()),
        };
        let raw_email = RawEmail::new(envelope, data);
        let message = QueueMessage::from(raw_email);
        let mut redis = self.state.redis.clone();
        let stream_id = push_queue_message(
            &mut redis,
            &self.state.config.queue.raw_email_stream,
            &message,
        )
        .await?;

        self.accepted_messages += 1;
        self.reset_envelope();
        info!(%stream_id, remote_addr = %self.remote_addr, "raw email pushed to Redis Stream");
        self.write_response(250, "queued").await
    }

    async fn handle_rset(&mut self) -> Result<()> {
        self.reset_envelope();
        self.write_response(250, "reset ok").await
    }

    fn reset_envelope(&mut self) {
        self.mail_from = None;
        self.rcpt_to.clear();
    }

    async fn read_data(&mut self) -> Result<DataRead> {
        let mut data = Vec::new();

        loop {
            let Some(line) = self.read_line().await? else {
                return Ok(DataRead::Complete(data));
            };

            if line == ".\r\n" || line == ".\n" || line == "." {
                return Ok(DataRead::Complete(data));
            }

            let payload = if line.starts_with("..") {
                &line.as_bytes()[1..]
            } else {
                line.as_bytes()
            };

            if data.len() + payload.len() > self.state.config.max_message_bytes {
                return Ok(DataRead::TooLarge);
            }

            data.extend_from_slice(payload);
        }
    }

    async fn read_line(&mut self) -> Result<Option<String>> {
        let mut line = Vec::new();
        let mut byte = [0_u8; 1];

        loop {
            let read = self.stream_mut()?.read(&mut byte).await?;
            if read == 0 {
                if line.is_empty() {
                    return Ok(None);
                }
                break;
            }

            line.push(byte[0]);
            if byte[0] == b'\n' {
                break;
            }

            if line.len() > 8192 {
                return Err(anyhow!("SMTP command line too long"));
            }
        }

        Ok(Some(String::from_utf8_lossy(&line).into_owned()))
    }

    async fn write_response(&mut self, code: u16, message: &str) -> Result<()> {
        self.write_line(&format!("{code} {message}")).await
    }

    async fn write_line(&mut self, line: &str) -> Result<()> {
        self.stream_mut()?.write_all(line.as_bytes()).await?;
        self.stream_mut()?.write_all(b"\r\n").await?;
        self.stream_mut()?.flush().await?;
        Ok(())
    }

    fn stream_mut(&mut self) -> Result<&mut SessionStream> {
        self.stream
            .as_mut()
            .ok_or_else(|| anyhow!("SMTP stream is closed"))
    }
}

enum SessionStream {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

impl SessionStream {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(stream) => stream.read(buf).await,
            Self::Tls(stream) => stream.read(buf).await,
        }
    }

    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.write_all(buf).await,
            Self::Tls(stream) => stream.write_all(buf).await,
        }
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(stream) => stream.flush().await,
            Self::Tls(stream) => stream.flush().await,
        }
    }
}

enum DataRead {
    Complete(Vec<u8>),
    TooLarge,
}

fn split_command(line: &str) -> (String, &str) {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    let Some((command, args)) = trimmed.split_once(char::is_whitespace) else {
        return (trimmed.to_ascii_uppercase(), "");
    };

    (command.to_ascii_uppercase(), args.trim())
}

fn trim_smtp_line(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

fn extract_path(args: &str, prefix: &str) -> Option<String> {
    let trimmed = args.trim();
    if !trimmed.get(..prefix.len())?.eq_ignore_ascii_case(prefix) {
        return None;
    }

    let rest = trimmed[prefix.len()..].trim_start();
    let address = if let Some(rest) = rest.strip_prefix('<') {
        rest.split_once('>')?.0
    } else {
        rest.split_whitespace().next()?
    };

    let address = address.trim().to_ascii_lowercase();
    (!address.is_empty() && address.contains('@')).then_some(address)
}

fn recipient_is_local(address: &str, mailbox_domain: &str) -> bool {
    let Some((local_part, domain)) = address.rsplit_once('@') else {
        return false;
    };

    !local_part.is_empty() && domain.eq_ignore_ascii_case(mailbox_domain)
}

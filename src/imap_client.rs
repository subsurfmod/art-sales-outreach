//! Lightweight, zero-OpenSSL pure-Rust IMAP4rev1 over TLS (rustls + ring) client
//! supporting LOGIN, APPEND (for Drafts upload), SELECT, SEARCH SINCE, and FETCH RFC822.

use anyhow::{bail, Context, Result};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

pub struct ImapSession {
    reader: BufReader<StreamOwned<ClientConnection, TcpStream>>,
    tag_seq: u32,
}

impl ImapSession {
    pub fn connect(host: &str, port: u16, user: &str, password: &str) -> Result<Self> {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()?
        .with_root_certificates(root_store)
        .with_no_client_auth();

        let server_name = ServerName::try_from(host.to_string())
            .with_context(|| format!("Invalid IMAP server name: {host}"))?;
        let conn = ClientConnection::new(Arc::new(config), server_name)?;
        let tcp = TcpStream::connect((host, port))
            .with_context(|| format!("Failed to connect to IMAP {host}:{port}"))?;
        tcp.set_read_timeout(Some(Duration::from_secs(20)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(20)))?;

        let stream = StreamOwned::new(conn, tcp);
        let mut sess = Self {
            reader: BufReader::new(stream),
            tag_seq: 0,
        };

        // Read server greeting
        let mut greeting = String::new();
        sess.reader.read_line(&mut greeting)?;
        if !greeting.starts_with("* ") {
            bail!("Unexpected IMAP greeting: {}", greeting.trim());
        }

        // Authenticate
        let login_cmd = format!("LOGIN {} {}", imap_quote(user), imap_quote(password));
        sess.run_simple(&login_cmd)
            .context("IMAP LOGIN failed (check App Password / credentials)")?;
        Ok(sess)
    }

    fn next_tag(&mut self) -> String {
        self.tag_seq += 1;
        format!("A{:04}", self.tag_seq)
    }

    fn run_simple(&mut self, cmd: &str) -> Result<Vec<String>> {
        let tag = self.next_tag();
        let wire = format!("{tag} {cmd}\r\n");
        self.reader.get_mut().write_all(wire.as_bytes())?;
        self.reader.get_mut().flush()?;

        let mut lines = Vec::new();
        let mut buf = String::new();
        loop {
            buf.clear();
            let n = self.reader.read_line(&mut buf)?;
            if n == 0 {
                bail!("IMAP connection closed unexpectedly");
            }
            let trimmed = buf.trim_end_matches(&['\r', '\n'][..]).to_string();
            if trimmed.starts_with(&format!("{tag} ")) {
                if trimmed[tag.len() + 1..].to_uppercase().starts_with("OK") {
                    return Ok(lines);
                }
                bail!("IMAP command failed: {trimmed}");
            }
            lines.push(trimmed);
        }
    }

    pub fn append_draft(&mut self, folder: &str, raw_rfc822: &[u8]) -> Result<()> {
        let tag = self.next_tag();
        let cmd = format!(
            "{tag} APPEND {} (\\Draft) {{{}}}\r\n",
            imap_quote(folder),
            raw_rfc822.len()
        );
        self.reader.get_mut().write_all(cmd.as_bytes())?;
        self.reader.get_mut().flush()?;

        // Wait for continuation '+'
        let mut cont = String::new();
        self.reader.read_line(&mut cont)?;
        if !cont.starts_with('+') {
            bail!("IMAP APPEND rejected: {}", cont.trim());
        }

        self.reader.get_mut().write_all(raw_rfc822)?;
        self.reader.get_mut().write_all(b"\r\n")?;
        self.reader.get_mut().flush()?;

        let mut buf = String::new();
        loop {
            buf.clear();
            let n = self.reader.read_line(&mut buf)?;
            if n == 0 {
                bail!("IMAP connection closed during APPEND");
            }
            let trimmed = buf.trim_end_matches(&['\r', '\n'][..]);
            if trimmed.starts_with(&format!("{tag} ")) {
                if trimmed[tag.len() + 1..].to_uppercase().starts_with("OK") {
                    return Ok(());
                }
                bail!("IMAP APPEND failed: {trimmed}");
            }
        }
    }

    pub fn search_since(&mut self, mailbox: &str, since_imap_date: &str) -> Result<Vec<String>> {
        self.run_simple(&format!("EXAMINE {}", imap_quote(mailbox)))?;
        let lines = self.run_simple(&format!("SEARCH SINCE {since_imap_date}"))?;
        let mut ids = Vec::new();
        for l in lines {
            if let Some(rest) = l.strip_prefix("* SEARCH") {
                for tok in rest.split_whitespace() {
                    ids.push(tok.to_string());
                }
            }
        }
        Ok(ids)
    }

    pub fn fetch_raw_message(&mut self, seq_id: &str) -> Result<String> {
        let tag = self.next_tag();
        let cmd = format!("{tag} FETCH {seq_id} (BODY.PEEK[])\r\n");
        self.reader.get_mut().write_all(cmd.as_bytes())?;
        self.reader.get_mut().flush()?;

        let mut raw_body = String::new();
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.reader.read_line(&mut line)?;
            if n == 0 {
                break;
            }
            let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
            if trimmed.starts_with(&format!("{tag} ")) {
                break;
            }
            if let Some(brace_start) = trimmed.rfind('{') {
                if trimmed.ends_with('}') {
                    if let Ok(lit_len) = trimmed[brace_start + 1..trimmed.len() - 1].parse::<usize>()
                    {
                        let mut lit_buf = vec![0u8; lit_len];
                        self.reader.read_exact(&mut lit_buf)?;
                        raw_body = String::from_utf8_lossy(&lit_buf).into_owned();
                    }
                }
            }
        }
        Ok(raw_body)
    }

    pub fn logout(mut self) {
        let _ = self.run_simple("LOGOUT");
    }
}

fn imap_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

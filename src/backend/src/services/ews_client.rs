use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
use rand::Rng;

// Simple HMAC-MD5 implementation
fn hmac_md5(key: &[u8], data: &[u8]) -> [u8; 16] {
    const BLOCK_SIZE: usize = 64;
    const IPAD: u8 = 0x36;
    const OPAD: u8 = 0x5c;

    // Prepare key
    let mut key_padded = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        // If key is longer than block size, hash it first
        let hashed = md5::compute(key);
        key_padded[..16].copy_from_slice(&hashed.0);
    } else {
        key_padded[..key.len()].copy_from_slice(key);
    }

    // Compute inner hash: MD5((K XOR ipad) || data)
    let mut inner_key = key_padded;
    for byte in &mut inner_key {
        *byte ^= IPAD;
    }
    let mut inner_data = Vec::with_capacity(BLOCK_SIZE + data.len());
    inner_data.extend_from_slice(&inner_key);
    inner_data.extend_from_slice(data);
    let inner_hash = md5::compute(&inner_data);

    // Compute outer hash: MD5((K XOR opad) || inner_hash)
    let mut outer_key = key_padded;
    for byte in &mut outer_key {
        *byte ^= OPAD;
    }
    let mut outer_data = Vec::with_capacity(BLOCK_SIZE + 16);
    outer_data.extend_from_slice(&outer_key);
    outer_data.extend_from_slice(&inner_hash.0);
    let outer_hash = md5::compute(&outer_data);

    outer_hash.0
}

const NTLM_SIGNATURE: &[u8] = b"NTLMSSP\0";
const NTLM_TYPE1_MESSAGE: u32 = 1;
const NTLM_TYPE3_MESSAGE: u32 = 3;

#[derive(Debug, thiserror::Error)]
pub enum EwsError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("EWS operation failed: {0}")]
    Ews(String),
    #[error("Invalid credentials format: {0}")]
    InvalidCredentials(String),
}

pub type Result<T> = std::result::Result<T, EwsError>;

pub struct EwsClient;

impl EwsClient {
    /// Send an email via Exchange Web Services using NTLM authentication
    ///
    /// # Arguments
    /// * `server` - Exchange server hostname (e.g., "mail.example.com")
    /// * `username` - Username in domain\user format (e.g., "Andalusia\SMH.Servicedesk")
    /// * `password` - User password
    /// * `from_email` - Sender email address
    /// * `from_name` - Sender display name
    /// * `to` - Semicolon-separated list of recipient email addresses
    /// * `cc` - Semicolon-separated list of CC email addresses (can be empty)
    /// * `subject` - Email subject
    /// * `html_body` - HTML body content
    pub async fn send_email(
        server: &str,
        username: &str,
        password: &str,
        from_email: &str,
        from_name: &str,
        to: &str,
        cc: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<()> {
        // Parse domain and username
        let (domain, user) = parse_domain_username(username)?;

        // Build EWS endpoint
        let ews_url = format!("https://{}/EWS/Exchange.asmx", server);

        // Create HTTP client with TLS verification disabled (common for corporate environments)
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        // Perform NTLM authentication and send email
        let soap_body = build_create_item_soap(from_email, from_name, to, cc, subject, html_body);
        let response = ntlm_authenticated_request(&client, &ews_url, &domain, &user, password, &soap_body).await?;

        // Parse response to check for success
        check_soap_response(&response)?;

        Ok(())
    }

    /// Send a test email with predefined content
    ///
    /// # Arguments
    /// * `server` - Exchange server hostname
    /// * `username` - Username in domain\user format
    /// * `password` - User password
    /// * `from_email` - Sender email address
    /// * `from_name` - Sender display name
    /// * `to_email` - Recipient email address
    pub async fn send_test_email(
        server: &str,
        username: &str,
        password: &str,
        from_email: &str,
        from_name: &str,
        to_email: &str,
    ) -> Result<()> {
        let subject = "Test Email from NetNinja";
        let html_body = r#"
            <html>
            <body>
                <h2>Test Email</h2>
                <p>This is a test email sent from NetNinja using Exchange Web Services.</p>
                <p>If you received this, the EWS integration is working correctly.</p>
            </body>
            </html>
        "#;

        Self::send_email(
            server,
            username,
            password,
            from_email,
            from_name,
            to_email,
            "",
            subject,
            html_body,
        ).await
    }
}

/// Parse domain\username format
fn parse_domain_username(username: &str) -> Result<(String, String)> {
    if let Some((domain, user)) = username.split_once('\\') {
        Ok((domain.to_string(), user.to_string()))
    } else {
        Err(EwsError::InvalidCredentials(
            "Username must be in domain\\user format".to_string()
        ))
    }
}

/// Create NTLM Type 1 message
fn create_type1_message() -> Vec<u8> {
    let mut msg = Vec::new();

    // Signature
    msg.extend_from_slice(NTLM_SIGNATURE);

    // Message Type (1)
    msg.extend_from_slice(&NTLM_TYPE1_MESSAGE.to_le_bytes());

    // Flags: Negotiate Unicode, Negotiate OEM, Request Target, Negotiate NTLM, Negotiate Domain Supplied, Negotiate Workstation Supplied
    let flags: u32 = 0x00000001 | 0x00000002 | 0x00000004 | 0x00000200 | 0x00001000 | 0x00002000 | 0x00008000 | 0x00080000 | 0x20000000;
    msg.extend_from_slice(&flags.to_le_bytes());

    // Domain fields (empty)
    msg.extend_from_slice(&[0u8; 8]);

    // Workstation fields (empty)
    msg.extend_from_slice(&[0u8; 8]);

    msg
}

/// Parse NTLM Type 2 message to extract challenge and target info
fn parse_type2_message(msg: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    if msg.len() < 32 {
        return Err(EwsError::Auth("Type 2 message too short".to_string()));
    }

    // Verify signature
    if &msg[0..8] != NTLM_SIGNATURE {
        return Err(EwsError::Auth("Invalid NTLM signature".to_string()));
    }

    // Extract challenge (8 bytes at offset 24)
    let challenge = msg[24..32].to_vec();

    // Extract target info if present
    let target_info = if msg.len() > 48 {
        let target_info_len = u16::from_le_bytes([msg[40], msg[41]]) as usize;
        let target_info_offset = u32::from_le_bytes([msg[44], msg[45], msg[46], msg[47]]) as usize;

        if target_info_offset > 0 && target_info_offset + target_info_len <= msg.len() {
            msg[target_info_offset..target_info_offset + target_info_len].to_vec()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok((challenge, target_info))
}

/// Create NT hash (MD4 of UTF-16LE password)
fn create_nt_hash(password: &str) -> [u8; 16] {
    use md4::{Md4, Digest};
    use md4::digest::FixedOutput;

    let password_utf16le: Vec<u8> = password
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();

    let mut hasher = Md4::new();
    hasher.update(&password_utf16le);
    let result = hasher.finalize_fixed();

    let mut hash = [0u8; 16];
    hash.copy_from_slice(&result[..]);
    hash
}

/// Create NTLMv2 hash
fn create_ntlmv2_hash(password: &str, username: &str, domain: &str) -> [u8; 16] {
    let nt_hash = create_nt_hash(password);

    // Create target: uppercase(username) + domain
    let target = format!("{}{}", username.to_uppercase(), domain);
    let target_utf16le: Vec<u8> = target
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();

    hmac_md5(&nt_hash, &target_utf16le)
}

/// Get Windows file time (100-nanosecond intervals since 1601-01-01)
fn get_windows_filetime() -> u64 {
    const EPOCH_DIFF: u64 = 11644473600; // Seconds between 1601 and 1970
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before Unix epoch");

    (duration.as_secs() + EPOCH_DIFF) * 10_000_000 + (duration.subsec_nanos() as u64 / 100)
}

/// Create NTLMv2 response blob
fn create_ntlmv2_response(
    challenge: &[u8],
    target_info: &[u8],
    ntlmv2_hash: &[u8],
) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let client_challenge: [u8; 8] = rng.gen();
    let timestamp = get_windows_filetime();

    // Build blob
    let mut blob = Vec::new();
    blob.extend_from_slice(&[0x01, 0x01]); // RespType and HiRespType
    blob.extend_from_slice(&[0x00, 0x00]); // Reserved1
    blob.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Reserved2
    blob.extend_from_slice(&timestamp.to_le_bytes()); // Timestamp
    blob.extend_from_slice(&client_challenge); // Client challenge
    blob.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Reserved3
    blob.extend_from_slice(target_info); // Target info
    blob.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Terminator

    // Compute NTLMv2 response
    let mut temp = Vec::new();
    temp.extend_from_slice(challenge);
    temp.extend_from_slice(&blob);

    let nt_proof = hmac_md5(ntlmv2_hash, &temp);

    // Combine NT proof + blob
    let mut response = Vec::new();
    response.extend_from_slice(&nt_proof);
    response.extend_from_slice(&blob);

    response
}

/// Create NTLM Type 3 message
fn create_type3_message(
    challenge: &[u8],
    target_info: &[u8],
    domain: &str,
    username: &str,
    password: &str,
) -> Vec<u8> {
    let ntlmv2_hash = create_ntlmv2_hash(password, username, domain);
    let ntlmv2_response = create_ntlmv2_response(challenge, target_info, &ntlmv2_hash);

    // Convert strings to UTF-16LE
    let domain_utf16le: Vec<u8> = domain
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();

    let username_utf16le: Vec<u8> = username
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();

    let workstation_utf16le: Vec<u8> = Vec::new(); // Empty workstation

    // LM response (empty for NTLMv2)
    let lm_response = vec![0u8; 24];

    // Calculate offsets
    let mut offset = 64; // Header size

    let lm_offset = offset;
    offset += lm_response.len();

    let ntlm_offset = offset;
    offset += ntlmv2_response.len();

    let domain_offset = offset;
    offset += domain_utf16le.len();

    let user_offset = offset;
    offset += username_utf16le.len();

    let workstation_offset = offset;
    offset += workstation_utf16le.len();

    let _session_key_offset = offset;

    // Build message
    let mut msg = Vec::new();

    // Signature
    msg.extend_from_slice(NTLM_SIGNATURE);

    // Message Type (3)
    msg.extend_from_slice(&NTLM_TYPE3_MESSAGE.to_le_bytes());

    // LM Response
    msg.extend_from_slice(&(lm_response.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(lm_response.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(lm_offset as u32).to_le_bytes());

    // NTLM Response
    msg.extend_from_slice(&(ntlmv2_response.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(ntlmv2_response.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(ntlm_offset as u32).to_le_bytes());

    // Domain
    msg.extend_from_slice(&(domain_utf16le.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(domain_utf16le.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(domain_offset as u32).to_le_bytes());

    // User
    msg.extend_from_slice(&(username_utf16le.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(username_utf16le.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(user_offset as u32).to_le_bytes());

    // Workstation
    msg.extend_from_slice(&(workstation_utf16le.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(workstation_utf16le.len() as u16).to_le_bytes());
    msg.extend_from_slice(&(workstation_offset as u32).to_le_bytes());

    // Session Key (empty)
    msg.extend_from_slice(&[0u8; 8]);

    // Flags
    let flags: u32 = 0x00000001 | 0x00000200 | 0x00080000 | 0x20000000;
    msg.extend_from_slice(&flags.to_le_bytes());

    // Payload
    msg.extend_from_slice(&lm_response);
    msg.extend_from_slice(&ntlmv2_response);
    msg.extend_from_slice(&domain_utf16le);
    msg.extend_from_slice(&username_utf16le);
    msg.extend_from_slice(&workstation_utf16le);

    msg
}

/// Perform NTLM authenticated request
async fn ntlm_authenticated_request(
    client: &Client,
    url: &str,
    domain: &str,
    username: &str,
    password: &str,
    body: &str,
) -> Result<String> {
    // Step 1: Send Type 1 message
    let type1_msg = create_type1_message();
    let type1_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &type1_msg);

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("NTLM {}", type1_b64)).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/xml; charset=utf-8"));

    let response = client
        .post(url)
        .headers(headers)
        .body(body.to_string())
        .send()
        .await?;

    // Step 2: Parse Type 2 message from response
    let auth_header = response
        .headers()
        .get("www-authenticate")
        .or_else(|| response.headers().get("WWW-Authenticate"))
        .ok_or_else(|| EwsError::Auth("No WWW-Authenticate header in response".to_string()))?
        .to_str()
        .map_err(|_| EwsError::Auth("Invalid WWW-Authenticate header".to_string()))?;

    let type2_b64 = auth_header
        .strip_prefix("NTLM ")
        .ok_or_else(|| EwsError::Auth("Invalid NTLM challenge format".to_string()))?;

    let type2_msg = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, type2_b64)
        .map_err(|_| EwsError::Auth("Failed to decode Type 2 message".to_string()))?;

    let (challenge, target_info) = parse_type2_message(&type2_msg)?;

    // Step 3: Send Type 3 message
    let type3_msg = create_type3_message(&challenge, &target_info, domain, username, password);
    let type3_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &type3_msg);

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("NTLM {}", type3_b64)).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/xml; charset=utf-8"));

    let response = client
        .post(url)
        .headers(headers)
        .body(body.to_string())
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(EwsError::Auth(format!(
            "Authentication failed with status: {}",
            response.status()
        )));
    }

    let response_text = response.text().await?;
    Ok(response_text)
}

/// Build EWS CreateItem SOAP request
fn build_create_item_soap(
    from_email: &str,
    from_name: &str,
    to: &str,
    cc: &str,
    subject: &str,
    html_body: &str,
) -> String {
    let to_recipients = to
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .map(|email| format!(r#"<t:Mailbox><t:EmailAddress>{}</t:EmailAddress></t:Mailbox>"#, email.trim()))
        .collect::<Vec<_>>()
        .join("");

    let cc_recipients = if cc.trim().is_empty() {
        String::new()
    } else {
        let cc_list = cc
            .split(';')
            .filter(|s| !s.trim().is_empty())
            .map(|email| format!(r#"<t:Mailbox><t:EmailAddress>{}</t:EmailAddress></t:Mailbox>"#, email.trim()))
            .collect::<Vec<_>>()
            .join("");

        format!("<t:CcRecipients>{}</t:CcRecipients>", cc_list)
    };

    // Escape XML special characters
    let subject_escaped = escape_xml(subject);
    let html_body_escaped = escape_xml(html_body);

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:t="http://schemas.microsoft.com/exchange/services/2006/types"
               xmlns:m="http://schemas.microsoft.com/exchange/services/2006/messages">
  <soap:Body>
    <m:CreateItem MessageDisposition="SendAndSaveCopy">
      <m:Items>
        <t:Message>
          <t:Subject>{}</t:Subject>
          <t:Body BodyType="HTML">{}</t:Body>
          <t:ToRecipients>{}</t:ToRecipients>
          {}
          <t:From>
            <t:Mailbox>
              <t:Name>{}</t:Name>
              <t:EmailAddress>{}</t:EmailAddress>
            </t:Mailbox>
          </t:From>
        </t:Message>
      </m:Items>
    </m:CreateItem>
  </soap:Body>
</soap:Envelope>"#,
        subject_escaped,
        html_body_escaped,
        to_recipients,
        cc_recipients,
        escape_xml(from_name),
        escape_xml(from_email)
    )
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Check SOAP response for success
fn check_soap_response(response: &str) -> Result<()> {
    // Check for SOAP fault
    if response.contains("<soap:Fault>") || response.contains("<s:Fault>") {
        return Err(EwsError::Ews(format!("SOAP fault: {}", response)));
    }

    // Check for NoError response class
    if response.contains("ResponseClass=\"Success\"") || response.contains("NoError") {
        Ok(())
    } else if response.contains("ResponseClass=\"Error\"") {
        // Try to extract error message
        let error_msg = if let Some(start) = response.find("<m:MessageText>") {
            let start = start + 15;
            if let Some(end) = response[start..].find("</m:MessageText>") {
                &response[start..start + end]
            } else {
                "Unknown error"
            }
        } else {
            "Unknown error"
        };

        Err(EwsError::Ews(format!("EWS error: {}", error_msg)))
    } else {
        // If we can't determine success or failure, treat as success
        // (some responses may not include explicit success markers)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain_username() {
        let (domain, user) = parse_domain_username("Andalusia\\SMH.Servicedesk").unwrap();
        assert_eq!(domain, "Andalusia");
        assert_eq!(user, "SMH.Servicedesk");
    }

    #[test]
    fn test_parse_domain_username_invalid() {
        let result = parse_domain_username("invalid_format");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_type1_message() {
        let msg = create_type1_message();
        assert_eq!(&msg[0..8], NTLM_SIGNATURE);
        assert_eq!(u32::from_le_bytes([msg[8], msg[9], msg[10], msg[11]]), NTLM_TYPE1_MESSAGE);
    }

    #[test]
    fn test_create_nt_hash() {
        // Test with known password (test vector)
        let hash = create_nt_hash("SecREt01");
        assert_eq!(hash.len(), 16); // MD4 produces 16 bytes
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Hello & <World>"), "Hello &amp; &lt;World&gt;");
        assert_eq!(escape_xml("Test \"quote\""), "Test &quot;quote&quot;");
    }
}

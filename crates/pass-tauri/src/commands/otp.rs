//! OTP input preview for the entry form.
//!
//! The write commands normalize and validate the OTP themselves, so this exists
//! only to give the form live feedback while typing: what the input was
//! understood to be, or why it cannot be saved.

use serde::Serialize;

use crate::error::{CommandError, CommandResult};

/// What the backend made of the user's OTP input.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct OtpPreview {
    /// Canonical `otpauth://` URI, or `None` when the input is blank.
    pub uri: Option<String>,
    /// Human summary, e.g. `GitHub (alice) · 6 digits · 30s · SHA1`.
    pub summary: Option<String>,
}

pub fn otp_preview_impl(
    input: &str,
    path: &str,
    fields: &[(String, String)],
) -> CommandResult<OtpPreview> {
    let (issuer, account) = passcore::otp::label_from_entry(path, fields);
    let uri =
        passcore::otp::normalize_input(input, &issuer, &account).map_err(|e| CommandError {
            message: passcore::otp::error_message(&e),
        })?;
    let summary = match &uri {
        None => None,
        Some(u) => Some(
            passcore::OtpConfig::parse(u)
                .map_err(CommandError::from)?
                .summary(),
        ),
    };
    Ok(OtpPreview { uri, summary })
}

#[tauri::command]
pub fn otp_preview(
    input: String,
    path: String,
    fields: Vec<(String, String)>,
) -> CommandResult<OtpPreview> {
    otp_preview_impl(&input, &path, &fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "GEZDGNBVGY3TQOJQ";

    fn user_field() -> Vec<(String, String)> {
        vec![("user".to_string(), "alice".to_string())]
    }

    #[test]
    fn bare_secret_previews_as_a_labelled_uri() {
        let p = otp_preview_impl(SECRET, "web/github.com", &user_field()).unwrap();
        let uri = p.uri.expect("a secret produces a URI");
        assert!(uri.starts_with("otpauth://totp/github.com:alice?"), "{uri}");
        let summary = p.summary.unwrap();
        assert!(summary.contains("github.com"), "{summary}");
        assert!(summary.contains("6 digits"), "{summary}");
    }

    #[test]
    fn blank_input_previews_as_nothing() {
        let p = otp_preview_impl("  ", "web/x", &[]).unwrap();
        assert_eq!(
            p,
            OtpPreview {
                uri: None,
                summary: None
            }
        );
    }

    #[test]
    fn invalid_input_is_an_error_the_form_can_show() {
        let err = otp_preview_impl("nope!!", "web/x", &[]).unwrap_err();
        assert!(
            !err.message.is_empty(),
            "the form needs something to render"
        );
    }
}

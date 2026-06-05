//! Loose-schema entry templates. A template only *suggests* `key:` fields for a
//! NEW entry; nothing is validated and the user may add/remove anything.

use crate::config::Config;

/// A named set of suggested field keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub fields: Vec<String>,
}

impl Template {
    pub fn new(name: &str, fields: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            fields: fields.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// The built-in templates, in display order.
    pub fn builtins() -> Vec<Template> {
        vec![
            Template::new("Login", &["user", "url"]),
            Template::new("OAuth / API", &["client_id", "client_secret", "url"]),
            Template::new("Server / SSH", &["host", "user", "port"]),
            Template::new("Note", &[]),
            Template::new("Blank", &[]),
        ]
    }

    /// The default template offered first (Login).
    pub fn default_template() -> Template {
        Template::new("Login", &["user", "url"])
    }

    /// Build a starter entry body: an empty password line followed by a `key: `
    /// stub per suggested field. Always ends with a trailing newline so it
    /// round-trips through `Entry::parse` cleanly.
    pub fn starter_text(&self) -> String {
        let mut out = String::from("\n");
        for key in &self.fields {
            out.push_str(key);
            out.push_str(": \n");
        }
        out
    }

    /// Merge config-provided templates over the built-ins: any template whose
    /// `name` matches a built-in replaces it; new names are appended.
    pub fn resolve(config: &Config) -> Vec<Template> {
        let mut resolved = Template::builtins();
        for tc in &config.templates {
            let t = Template {
                name: tc.name.clone(),
                fields: tc.fields.clone(),
            };
            match resolved.iter_mut().find(|b| b.name == t.name) {
                Some(existing) => *existing = t,
                None => resolved.push(t),
            }
        }
        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_include_the_expected_set() {
        let builtins = Template::builtins();
        let names: Vec<&str> = builtins.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["Login", "OAuth / API", "Server / SSH", "Note", "Blank"]
        );
    }

    #[test]
    fn login_is_the_default_and_suggests_user_url() {
        let t = Template::default_template();
        assert_eq!(t.name, "Login");
        assert_eq!(t.fields, vec!["user".to_string(), "url".to_string()]);
    }

    #[test]
    fn starter_text_has_blank_password_then_suggested_keys() {
        let t = Template::new("OAuth / API", &["client_id", "client_secret", "url"]);
        // First line is the (empty) password line; then `key: ` stubs.
        assert_eq!(t.starter_text(), "\nclient_id: \nclient_secret: \nurl: \n");
    }

    #[test]
    fn blank_template_is_password_only() {
        let t = Template::new("Blank", &[]);
        assert_eq!(t.starter_text(), "\n");
    }

    #[test]
    fn config_templates_override_builtins_by_name() {
        let toml = r#"
            [[templates]]
            name = "Login"
            fields = ["user", "url", "note"]
        "#;
        let cfg = crate::config::Config::from_toml_str(toml).unwrap();
        let resolved = Template::resolve(&cfg);
        let login = resolved.iter().find(|t| t.name == "Login").unwrap();
        assert_eq!(login.fields, vec!["user", "url", "note"]);
        // builtins not overridden are still present
        assert!(resolved.iter().any(|t| t.name == "Blank"));
    }
}

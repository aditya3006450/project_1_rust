use serde_json::Value;

#[derive(Debug, Clone)]
pub struct MailData {
    pub to: String,
    pub cc: Vec<String>,
    pub subject: String,
    pub context: Value,
    pub template: Option<String>,
    pub raw_html: Option<String>,
}

impl MailData {
    pub fn with_template(to: String, subject: String, template: String, context: Value) -> Self {
        Self {
            to,
            cc: vec![],
            subject,
            context,
            template: Some(template),
            raw_html: None,
        }
    }

    pub fn with_html(to: String, subject: String, html: String) -> Self {
        Self {
            to,
            cc: vec![],
            subject,
            context: Value::Null,
            template: None,
            raw_html: Some(html),
        }
    }
}

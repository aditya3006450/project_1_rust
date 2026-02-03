use serde_json::Value;
use tera::{Context, Tera};

pub struct TeraRenderer {
    tera: Tera,
}

impl TeraRenderer {
    pub fn new() -> Self {
        let tera = Tera::new("templates/**/*").expect("Failed to load HTML templates");

        Self { tera }
    }

    pub fn render(&self, template_name: &str, context: Value) -> Result<String, tera::Error> {
        let ctx = Context::from_value(context).expect("Invalid JSON context for template");
        self.tera.render(template_name, &ctx)
    }
}

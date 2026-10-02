use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameKind {
    Text,
    Number,
    Any,
}

pub trait TemplateContext: Send {
    fn text(&self, template: &str) -> Result<String, String>;
    fn value(&self, template: &str) -> Result<Value, String>;
    fn set_item(&mut self, item: Option<(Value, usize)>);
}

pub trait WidgetTemplates: Send + Sync {
    fn problem(
        &self,
        template: &str,
        allows: &dyn Fn(&str) -> Result<NameKind, String>,
    ) -> Option<String>;
    fn open(
        &self,
        data: &Value,
        id: &str,
        title: Option<&str>,
        fetched_at: Option<String>,
    ) -> Box<dyn TemplateContext>;
}

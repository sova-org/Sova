use crate::lang::bali::bali_ast::expression::Expression;

#[derive(Debug, Clone)]
pub struct BaliContext {
    pub channel: Option<Expression>,
    pub device: Option<Expression>,
    pub velocity: Option<Expression>,
    pub duration: Option<Expression>,
}

impl Default for BaliContext {
    fn default() -> Self {
        Self::new()
    }
}

impl BaliContext {
    pub fn new() -> BaliContext {
        BaliContext {
            channel: None,
            device: None,
            velocity: None,
            duration: None,
        }
    }

    pub fn update(&self, above: &BaliContext) -> BaliContext {
        BaliContext {
            channel: self.channel.clone().or_else(|| above.channel.clone()),
            device: self.device.clone().or_else(|| above.device.clone()),
            velocity: self.velocity.clone().or_else(|| above.velocity.clone()),
            duration: self.duration.clone().or_else(|| above.duration.clone()),
        }
    }
}

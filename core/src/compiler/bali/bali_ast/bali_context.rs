use crate::compiler::bali::bali_ast::expression::Expression;

#[derive(Debug, Clone)]
pub struct BaliContext {
    pub channel: Option<Expression>,
    pub device: Option<Expression>,
    pub velocity: Option<Expression>,
    pub duration: Option<Expression>,
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

    pub fn update(self, above: BaliContext) -> BaliContext {
        let mut b = BaliContext::new();
        b.channel = match self.channel {
            Some(_) => self.channel,
            None => above.channel,
        };
        b.device = match self.device {
            Some(_) => self.device,
            None => above.device,
        };
        b.velocity = match self.velocity {
            Some(_) => self.velocity,
            None => above.velocity,
        };
        b.duration = match self.duration {
            Some(_) => self.duration,
            None => above.duration,
        };
        b
    }
}

use crate::runtime::context::Context;

pub struct Compartment {
    contexts: Vec<Context>
}

impl Compartment {
    pub fn new() -> Compartment {
        Compartment {
            contexts: Vec::new()
        }
    }

    // pub fn create_context(&self) -> Result<Context, String> {
    //     let ctx = Context;
    //     self.contexts.push(ctx);
    //     Ok(ctx)
    // }
}

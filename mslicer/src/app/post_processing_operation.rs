use crate::post_processing::Pass;
use std::collections::HashMap;

pub struct PostProcessingOperation {
    passes: HashMap<&'static str, Box<dyn Pass>>,
}

impl PostProcessingOperation {
    pub fn new(passes: impl Iterator<Item = Box<dyn Pass>>) -> Self {
        let passes: HashMap<&'static str, Box<dyn Pass>> = passes.map(|p| (p.name(), p)).collect();
        Self { passes }
    }

    #[allow(unused)]
    pub fn passes(&self) -> impl Iterator<Item = &Box<dyn Pass>> {
        self.passes.values()
    }

    pub fn passes_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Pass>> {
        self.passes.values_mut()
    }
}

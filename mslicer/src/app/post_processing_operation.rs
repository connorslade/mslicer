use crate::post_processing::Pass;

pub struct PostProcessingOperation {
    passes: Vec<Box<dyn Pass>>,
}

impl PostProcessingOperation {
    pub fn new(passes: Vec<Box<dyn Pass>>) -> Self {
        Self { passes }
    }

    #[allow(unused)]
    pub fn passes(&self) -> impl Iterator<Item = &Box<dyn Pass>> {
        self.passes.iter()
    }

    pub fn passes_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Pass>> {
        self.passes.iter_mut()
    }
}

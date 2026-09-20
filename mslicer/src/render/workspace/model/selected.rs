use std::range::Range;

use num_integer::Integer;

use crate::core::project::model::Model;

pub struct Selected {
    words: Vec<u32>,
}

impl Selected {
    pub fn new(faces: usize) -> Self {
        Selected {
            words: vec![0; faces.div_ceil(32)],
        }
    }

    pub fn for_model(model: &Model) -> Self {
        Self::new(model.mesh.face_count())
    }

    pub fn into_inner(self) -> Vec<u32> {
        self.words
    }

    pub fn set_selected(&mut self, idx: usize) {
        let (word, bit) = idx.div_rem(&32);
        self.words[word] |= 1 << bit;
    }

    pub fn set_selected_range(&mut self, range: Range<u32>) {
        // todo: optimize
        (range.into_iter()).for_each(|i| self.set_selected(i as usize));
    }
}

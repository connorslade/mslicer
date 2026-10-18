use std::range::Range;

#[derive(Default)]
pub struct Selected {
    ranges: Vec<[u32; 2]>,
}

impl Selected {
    pub fn into_inner(mut self) -> Vec<[u32; 2]> {
        self.ranges.sort_by_key(|x| x[0]);
        self.ranges
    }

    pub fn set_selected(&mut self, idx: u32) {
        self.ranges.push([idx, idx + 1]);
    }

    pub fn set_selected_range(&mut self, range: Range<u32>) {
        self.ranges.push([range.start, range.end]);
    }
}

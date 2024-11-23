// yes, i know this is jank as hell

use std::{
    mem,
    ops::{Deref, DerefMut},
};

use image::GrayImage;

use super::FormatSliceFile;

pub struct SliceLayerIterator<'a> {
    pub(crate) file: &'a mut FormatSliceFile,
    pub(crate) layer: usize,
    pub(crate) layers: usize,
}

pub struct SliceLayerElement {
    image: GrayImage,

    file: *mut FormatSliceFile,
    layer: usize,
}

impl<'a> Iterator for SliceLayerIterator<'a> {
    type Item = SliceLayerElement;

    fn next(&mut self) -> Option<Self::Item> {
        if self.layer >= self.layers {
            return None;
        }

        let image = self.file.read_layer(self.layer);
        self.layer += 1;

        Some(SliceLayerElement {
            image,
            file: self.file as *mut _,
            layer: self.layer - 1,
        })
    }
}

impl Drop for SliceLayerElement {
    fn drop(&mut self) {
        // SAFETY: it's not... But the idea is that each SliceLayerElement will
        // only be writing to one layer each, meaning the same memory will only
        // be mutably borrowed once.
        //
        // You could easily keep one of these objects alive after the slice
        // layer iter is dropped, but don't please.
        let file = unsafe { &mut *self.file };
        file.overwrite_layer(self.layer, mem::take(&mut self.image));
    }
}

impl Deref for SliceLayerElement {
    type Target = GrayImage;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl DerefMut for SliceLayerElement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image
    }
}

unsafe impl Send for SliceLayerElement {}

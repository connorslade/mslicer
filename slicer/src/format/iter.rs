// yes, i know this is jank as hell

use std::{
    marker::PhantomData,
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

pub struct SliceLayerElement<'a> {
    image: GrayImage,

    file: *mut FormatSliceFile,
    layer: usize,

    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Iterator for SliceLayerIterator<'a> {
    type Item = SliceLayerElement<'a>;

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
            _lifetime: PhantomData,
        })
    }
}

impl<'a> Drop for SliceLayerElement<'a> {
    fn drop(&mut self) {
        // SAFETY: it's not... But the idea is that each SliceLayerElement will
        // only be writing to one layer each, meaning the same memory will only
        // be mutably borrowed once.
        let file = unsafe { &mut *self.file };
        file.overwrite_layer(self.layer, mem::take(&mut self.image));
    }
}

impl<'a> Deref for SliceLayerElement<'a> {
    type Target = GrayImage;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl<'a> DerefMut for SliceLayerElement<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image
    }
}

unsafe impl<'a> Send for SliceLayerElement<'a> {}

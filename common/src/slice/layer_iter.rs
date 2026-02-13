// yes, i know this is jank as hell

use std::{
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

use image::GrayImage;

use crate::{container::Image, slice::DynSlicedFile};

pub struct SliceLayerIterator<'a> {
    pub(crate) file: &'a mut DynSlicedFile,
    pub(crate) layer: usize,
    pub(crate) layers: usize,
}

pub struct SliceLayerElement<'a> {
    image: Option<Image>,

    file: *mut DynSlicedFile,
    layer: usize,

    _lifetime: PhantomData<&'a ()>,
}

impl<'a> SliceLayerIterator<'a> {
    pub fn new(file: &'a mut DynSlicedFile) -> Self {
        Self {
            layer: 0,
            layers: file.info().layers as usize,
            file,
        }
    }
}

impl<'a> SliceLayerElement<'a> {
    pub fn gray_image(&mut self, callback: impl FnOnce(&mut GrayImage)) {
        let (width, height) = (self.size.x as u32, self.size.y as u32);
        let inner = self.image.take().unwrap().take();
        let mut image = GrayImage::from_raw(width, height, inner).unwrap();

        callback(&mut image);
        self.image = Some(Image::from_raw(self.size, image.into_raw()));
    }
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
            image: Some(image),
            file: self.file as *mut _,
            layer: self.layer - 1,
            _lifetime: PhantomData,
        })
    }
}

impl Drop for SliceLayerElement<'_> {
    fn drop(&mut self) {
        // SAFETY: it's not... But the idea is that each SliceLayerElement will
        // only be writing to one layer each, meaning the same memory will only
        // be mutably borrowed once.
        let file = unsafe { &mut *self.file };
        file.overwrite_layer(self.layer, mem::take(&mut self.image).unwrap());
    }
}

impl Deref for SliceLayerElement<'_> {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        self.image.as_ref().unwrap()
    }
}

impl DerefMut for SliceLayerElement<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.image.as_mut().unwrap()
    }
}

unsafe impl Send for SliceLayerElement<'_> {}

use std::sync::Arc;

use egui::{Color32, ColorImage, Context, ImageData, TextureId, TextureOptions, WidgetText};
use image::RgbaImage;

pub struct LazyTextureId {
    inner: Option<TextureId>,
}

pub struct LazyText {
    inner: Box<dyn FnOnce() -> WidgetText>,
}

impl LazyTextureId {
    pub fn empty() -> Self {
        Self { inner: None }
    }

    pub fn get(&mut self, ctx: &Context, image: &RgbaImage) -> TextureId {
        if let Some(id) = self.inner {
            return id;
        }

        let id = upload_texture_egui(ctx, image);
        self.inner = Some(id);
        id
    }
}

impl LazyText {
    pub fn new<T: Into<WidgetText>>(text: impl FnOnce() -> T + 'static) -> Self {
        Self {
            inner: Box::new(move || text().into()),
        }
    }
}

impl From<LazyText> for WidgetText {
    fn from(value: LazyText) -> Self {
        (value.inner)()
    }
}

// todo: dealloc when slice operation is overwritten!!
fn upload_texture_egui(ctx: &Context, image: &RgbaImage) -> TextureId {
    let image = ColorImage::new(
        [image.width(), image.height()].map(|x| x as usize),
        image
            .pixels()
            .map(|x| Color32::from_rgb(x.0[0], x.0[1], x.0[2]))
            .collect(),
    );
    ctx.tex_manager().write().alloc(
        "Preview Image".into(),
        ImageData::Color(Arc::new(image)),
        TextureOptions::NEAREST,
    )
}

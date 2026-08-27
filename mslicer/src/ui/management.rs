use std::sync::Arc;

use egui::{Color32, ColorImage, Context, ImageData, TextureId, TextureOptions, WidgetText};
use image::RgbaImage;

pub struct LazyTextureId {
    inner: Option<LazyTextureIdInner>,
}

struct LazyTextureIdInner {
    // egui's Context is internally reference counted
    ctx: Context,
    texture: TextureId,
}

pub struct LazyText {
    inner: Box<dyn FnOnce() -> WidgetText>,
}

impl LazyTextureId {
    pub fn empty() -> Self {
        Self { inner: None }
    }

    pub fn get(&mut self, ctx: &Context, image: &RgbaImage) -> TextureId {
        if let Some(LazyTextureIdInner { texture, .. }) = &self.inner {
            return *texture;
        }

        let id = upload_texture_egui(ctx, image);
        self.inner = Some(LazyTextureIdInner {
            ctx: ctx.clone(),
            texture: id,
        });
        id
    }
}

impl Drop for LazyTextureId {
    fn drop(&mut self) {
        if let Some(LazyTextureIdInner { ctx, texture }) = self.inner.take() {
            ctx.tex_manager().write().free(texture);
        }
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
        TextureOptions::LINEAR,
    )
}

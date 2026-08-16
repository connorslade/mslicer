use std::sync::Arc;

use egui::{Color32, ColorImage, Context, ImageData, TextureId, TextureOptions};
use image::RgbaImage;

pub struct LazyTextureId {
    inner: Option<TextureId>,
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
        TextureOptions::LINEAR,
    )
}

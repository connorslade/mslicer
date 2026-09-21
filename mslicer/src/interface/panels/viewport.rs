use egui::{
    Align2, Area, Color32, Context, Frame, Id, Order, Painter, Pos2, Rect, Sense, Stroke,
    StrokeKind, Theme, Ui, Vec2, pos2, vec2,
};
use egui_wgpu::Callback;
use nalgebra::Matrix4;

use crate::{
    core::{
        App,
        config::ui::HoverOverlay,
        state::{GeometryHit, WorkspaceHover},
    },
    interface::{components::grid, panels::supports::manual_support_placement},
    render::{interface::basis::BasisRenderCallback, workspace::WorkspaceRenderCallback},
};

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    app.camera.handle_movement(&response, ui);

    let focused = ui.input(|i| i.focused);
    let mut is_moving = app.spacenav().handle_movement(focused);
    if is_moving {
        app.state.move_timeout = ((0.025 / app.fps.frame_time()).round() as u32).min(60);
    } else if app.state.move_timeout > 0 {
        app.state.move_timeout -= 1;
        is_moving = true;
    }
    is_moving |= response.dragged();

    let aspect = rect.width() / rect.height();
    let px = response.hover_pos().unwrap_or_default();
    let uv = (px - rect.min) / rect.size();
    app.state.workspace = WorkspaceHover::new(is_moving, aspect, uv);

    if response.clicked() && !is_moving {
        if app.state.support_placement {
            manual_support_placement(app, true);
        } else if let Some(hover) = app.state.hovered_geometry {
            let shift = ui.input(|x| x.modifiers.shift);
            if hover.support {
                if let Some(model) = app.project.model(hover.model)
                    && let Some(support) = model.supports.support_for_face(hover.face)
                {
                    (app.state.selected_supports).support_clicked(model.id, support);
                }
            } else {
                app.state.selected.model_clicked(hover.model, shift);
            }
        }
    }

    let painter = ui.painter();
    let color = match app.config.ui.theme {
        Theme::Dark => Color32::from_rgb(9, 9, 9),
        Theme::Light => Color32::from_rgb(255, 255, 255),
    };
    painter.rect_filled(rect, 0.0, color);

    painter.add(Callback::new_paint_callback(
        rect,
        app.get_workspace_render_callback(),
    ));

    paint_basis_vectors(painter, app, &rect);

    if app.config.ui.hover_overlay != HoverOverlay::Off
        && let Some(hover) = app.state.hovered_geometry
        && response.contains_pointer()
    {
        paint_hover_overlay(ui, app, hover, px);
    }
}

fn paint_basis_vectors(painter: &Painter, app: &mut App, rect: &Rect) {
    let size = app.config.render.basis_size;
    if size == 0.0 {
        return;
    }

    let pad = 4.0;
    let color = |r, g, b, a| Color32::from_rgba_unmultiplied(r, g, b, a);
    let (color_bg, color_edge) = match app.config.ui.theme {
        Theme::Dark => (color(255, 255, 255, 40), color(255, 255, 255, 180)),
        Theme::Light => (color(0, 0, 0, 40), color(0, 0, 0, 180)),
    };
    let stroke = Stroke::new(1.5_f32, color_edge);

    let rect = Rect::from_min_size(
        pos2(rect.max.x - size - pad, rect.min.y + pad),
        vec2(size, size),
    );
    painter.rect_filled(rect, size / 2.0, color_bg);
    painter.rect_stroke(rect, size / 2.0, stroke, StrokeKind::Outside);

    painter.add(Callback::new_paint_callback(
        rect.expand(-pad),
        BasisRenderCallback {
            camera: app.camera.clone(),
        },
    ));
}

fn paint_hover_overlay(ui: &mut Ui, app: &mut App, hover: GeometryHit, px: Pos2) {
    let p = Vec2::splat(8.0);
    let detail = app.config.ui.hover_overlay == HoverOverlay::Detailed;

    Area::new(Id::new("hover_overlay"))
        .order(Order::Tooltip)
        .fixed_pos(px + vec2(p.x, -p.y))
        .pivot(Align2::LEFT_BOTTOM)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            Frame::new()
                .inner_margin(p)
                .corner_radius(ui.visuals().menu_corner_radius)
                .fill(Color32::BLACK.lerp_to_gamma(Color32::TRANSPARENT, 0.25))
                .show(ui, |ui| {
                    ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
                    ui.set_max_width(300.0);

                    let Some(model) = app.project.model(hover.model) else {
                        return;
                    };

                    grid("overlay").spacing([4.0, 4.0]).show(ui, |ui| {
                        ui.label("Model");
                        ui.label(&model.name);
                        ui.end_row();

                        if hover.support
                            && let Some(support) = model.supports.support_for_face(hover.face)
                        {
                            ui.label("Support");
                            ui.label(format!("#{}", support.raw()));
                            ui.end_row();
                        }

                        if detail {
                            ui.label("Face");
                            ui.label(hover.face.to_string());
                            ui.end_row();
                        }
                    });
                });
        });
}

impl App {
    pub fn get_workspace_render_callback(&mut self) -> WorkspaceRenderCallback {
        WorkspaceRenderCallback {
            app: self as *mut _,
        }
    }

    pub fn view_projection(&self) -> Matrix4<f32> {
        let aspect = self.state.workspace.aspect;
        self.camera
            .view_projection_matrix(self.config.render.projection, aspect)
    }
}

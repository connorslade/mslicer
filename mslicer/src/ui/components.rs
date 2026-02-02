use egui::{Align, Color32, DragValue, FontId, Layout, Response, Separator, Ui, emath::Numeric};
use egui_phosphor::regular::INFO;

pub fn labeled_separator(ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        let width = ui.fonts_mut(|f| {
            f.layout_no_wrap(text.into(), FontId::default(), Color32::default())
                .rect
                .width()
        });
        let spacing = ui.style().spacing.item_spacing.x;
        let bar = (ui.available_width() - width) / 2.0 - spacing;

        ui.add_sized([bar, 1.0], Separator::default().horizontal());
        ui.label(text);
        ui.add_sized([bar, 1.0], Separator::default().horizontal());
    });
}

pub fn dragger<Num: Numeric>(
    ui: &mut Ui,
    label: &str,
    value: &mut Num,
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(value)));
        ui.label(label);
    });
}

pub fn dragger_tip<Num: Numeric>(
    ui: &mut Ui,
    label: &str,
    tip: &str,
    value: &mut Num,
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(value)));
        ui.label(label);
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            ui.label(INFO).on_hover_text(tip);
            ui.add_space(ui.available_width());
        })
    });
}

pub fn vec2_dragger<Num: Numeric>(
    ui: &mut Ui,
    val: &mut [Num; 2],

    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(&mut val[0])));
        ui.label("×");
        ui.add(func(DragValue::new(&mut val[1])));
    });
}

/// Returns weather the widget is being edited.
pub fn vec3_dragger<Num: Numeric>(
    ui: &mut Ui,
    val: &mut [Num; 3],
    func: fn(DragValue) -> DragValue,
) -> bool {
    let mut edit = false;
    ui.horizontal(|ui| {
        edit |= being_edited(&ui.add(func(DragValue::new(&mut val[0]))));
        ui.label("×");
        edit |= being_edited(&ui.add(func(DragValue::new(&mut val[1]))));
        ui.label("×");
        edit |= being_edited(&ui.add(func(DragValue::new(&mut val[2]))));
    });
    edit
}

// Note: could have issues if more than one value is edited in a frame
pub fn vec3_dragger_proportional(
    ui: &mut Ui,
    val: &mut [f32; 3],
    func: fn(DragValue) -> DragValue,
) -> bool {
    let mut edit = false;
    ui.horizontal(|ui| {
        let (x, y, z) = (val[0], val[1], val[2]);

        edit |= being_edited(&ui.add(func(DragValue::new(&mut val[0]))));
        ui.label("×");
        edit |= being_edited(&ui.add(func(DragValue::new(&mut val[1]))));
        ui.label("×");
        edit |= being_edited(&ui.add(func(DragValue::new(&mut val[2]))));

        if x != val[0] {
            let diff = val[0] / x;
            val[1] *= diff;
            val[2] *= diff;
        } else if y != val[1] {
            let diff = val[1] / y;
            val[0] *= diff;
            val[2] *= diff;
        } else if z != val[2] {
            let diff = val[2] / z;
            val[0] *= diff;
            val[1] *= diff;
        }
    });
    edit
}

/// Returns if the supplied widget response is being dragged or has focus.
fn being_edited(response: &Response) -> bool {
    response.dragged() || response.has_focus()
}

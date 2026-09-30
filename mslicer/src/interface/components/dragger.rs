use common::{
    slice::Height,
    units::{Milimeter, Milimeters},
};
use egui::{DragValue, Ui, Widget, emath::Numeric};
use egui_phosphor::regular::SCALES;

use crate::interface::components::BeingEditedExt;

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

pub fn vec2<Num: Numeric>(
    ui: &mut Ui,
    val: &mut [Num; 2],

    func: fn(DragValue) -> DragValue,
) -> bool {
    let mut edit = false;
    ui.horizontal(|ui| {
        (func(DragValue::new(&mut val[0])).ui(ui)).being_edited(&mut edit);
        ui.label("×");
        (func(DragValue::new(&mut val[1])).ui(ui)).being_edited(&mut edit);
    });
    edit
}

/// Returns weather the widget is being edited.
pub fn vec3<Num: Numeric>(
    ui: &mut Ui,
    val: &mut [Num; 3],
    func: fn(DragValue) -> DragValue,
) -> bool {
    let mut edit = false;
    ui.horizontal(|ui| {
        (func(DragValue::new(&mut val[0])).ui(ui)).being_edited(&mut edit);
        ui.label("×");
        (func(DragValue::new(&mut val[1])).ui(ui)).being_edited(&mut edit);
        ui.label("×");
        (func(DragValue::new(&mut val[2])).ui(ui)).being_edited(&mut edit);
    });
    edit
}

// Note: could have issues if more than one value is edited in a frame
pub fn vec3_proportional(
    ui: &mut Ui,
    val: &mut [f32; 3],
    func: fn(DragValue) -> DragValue,
) -> bool {
    let mut edit = false;
    ui.horizontal(|ui| {
        let (x, y, z) = (val[0], val[1], val[2]);

        (func(DragValue::new(&mut val[0])).ui(ui)).being_edited(&mut edit);
        ui.label("×");
        (func(DragValue::new(&mut val[1])).ui(ui)).being_edited(&mut edit);
        ui.label("×");
        (func(DragValue::new(&mut val[2])).ui(ui)).being_edited(&mut edit);

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

pub fn height(ui: &mut Ui, slice_height: Milimeters, height: &mut Height) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= match height {
            Height::Layers(layers) => DragValue::new(layers).ui(ui).changed(),
            Height::Distance(length) => length.with::<Milimeter, _>(|x| {
                DragValue::new(x).speed(0.1).suffix(" mm").ui(ui).changed()
            }),
        };

        let tip = ["Specify height in layers", "Specify height in mm"]
            [matches!(height, Height::Layers(..)) as usize];
        if ui.button(SCALES).on_hover_text(tip).clicked() {
            *height = height.flip(slice_height);
            changed |= true;
        }
    });

    changed
}

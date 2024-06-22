use egui::{emath::Numeric, DragValue, Ui};

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

pub fn vec2_dragger<Num: Numeric>(ui: &mut Ui, val: &mut [Num; 2]) {
    ui.horizontal(|ui| {
        ui.add(DragValue::new(&mut val[0]));
        ui.label("x");
        ui.add(DragValue::new(&mut val[1]));
    });
}

pub fn vec3_dragger<Num: Numeric>(ui: &mut Ui, val: &mut [Num; 3]) {
    ui.horizontal(|ui| {
        ui.add(DragValue::new(&mut val[0]));
        ui.label("x");
        ui.add(DragValue::new(&mut val[1]));
        ui.label("x");
        ui.add(DragValue::new(&mut val[2]));
    });
}

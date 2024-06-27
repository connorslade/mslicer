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

pub fn vec2_dragger<Num: Numeric>(
    ui: &mut Ui,
    val: &mut [Num; 2],

    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(&mut val[0])));
        ui.label("x");
        ui.add(func(DragValue::new(&mut val[1])));
    });
}

pub fn vec3_dragger<Num: Numeric>(
    ui: &mut Ui,
    val: &mut [Num; 3],
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        ui.add(func(DragValue::new(&mut val[0])));
        ui.label("x");
        ui.add(func(DragValue::new(&mut val[1])));
        ui.label("x");
        ui.add(func(DragValue::new(&mut val[2])));
    });
}

pub fn vec3_dragger_proportional(
    ui: &mut Ui,
    val: &mut [f32; 3],
    func: fn(DragValue) -> DragValue,
) {
    ui.horizontal(|ui| {
        let (x, y, z) = (val[0], val[1], val[2]);

        ui.add(func(DragValue::new(&mut val[0])));
        ui.label("x");
        ui.add(func(DragValue::new(&mut val[1])));
        ui.label("x");
        ui.add(func(DragValue::new(&mut val[2])));

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
}

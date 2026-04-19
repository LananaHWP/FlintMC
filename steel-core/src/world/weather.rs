#[expect(
    clippy::struct_field_names,
    reason = "field names match vanilla weather state naming"
)]
#[derive(Debug, Default)]
pub struct Weather {
    pub rain_level: f32,
    pub previous_rain_level: f32,
    pub thunder_level: f32,
    pub previous_thunder_level: f32,
}

impl Weather {
    pub fn clear(&mut self) {
        self.rain_level = 0.0;
        self.thunder_level = 0.0;
    }

    pub fn start_rain(&mut self) {
        self.rain_level = 1.0;
    }

    pub fn start_thunder(&mut self) {
        self.thunder_level = 1.0;
    }
}

use serde::{Deserialize, Serialize};

const LITER_PER_GALLON: f32 = 3.78541;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Liters(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Gallons(pub f32);

impl Liters {
    pub fn gallons(self) -> Gallons {
        let g = self.0 / LITER_PER_GALLON;
        Gallons(g)
    }
}

impl From<Gallons> for Liters {
    fn from(value: Gallons) -> Self {
        value.liters()
    }
}

impl Gallons {
    pub fn liters(self) -> Liters {
        let l = self.0 * LITER_PER_GALLON;
        Liters(l)
    }
}

impl From<Liters> for Gallons {
    fn from(value: Liters) -> Self {
        value.gallons()
    }
}

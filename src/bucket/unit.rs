use serde::{Deserialize, Serialize};

// MARK: US units

// https://www.unitconverters.net/volume/gallons-to-liters.htm
#[allow(clippy::excessive_precision)]
const LITER_PER_GALLON: f32 = 3.785411784;

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

// MARK: UK units

// https://www.unitconverters.net/volume/pint-uk-to-liter.htm
const LITRE_PER_UK_PINT: f32 = 0.56826125;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Litres(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Pints(pub f32);

impl Litres {
    pub fn pints(self) -> Pints {
        let p = self.0 / LITRE_PER_UK_PINT;
        Pints(p)
    }
}

impl From<Pints> for Litres {
    fn from(value: Pints) -> Self {
        value.litres()
    }
}

impl Pints {
    pub fn litres(self) -> Litres {
        let l = self.0 * LITRE_PER_UK_PINT;
        Litres(l)
    }
}

impl From<Litres> for Pints {
    fn from(value: Litres) -> Self {
        value.pints()
    }
}

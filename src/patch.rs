#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PatchId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatchCoord {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Debug)]
pub struct PatchState {
    pub prey: f64,
    pub predators: f64,
    pub vegetation: f64,
    pub rainfall: f64,
    pub temperature: f64,
    pub disease_pressure: f64,
    pub carrying_capacity: f64,
}

impl PatchState {
    pub fn clamp_nonnegative(&mut self) {
        self.prey = clamp_small_negative(self.prey);
        self.predators = clamp_small_negative(self.predators);
        self.vegetation = clamp_small_negative(self.vegetation);
        self.rainfall = clamp_small_negative(self.rainfall);
        self.temperature = clamp_small_negative(self.temperature);
        self.disease_pressure = clamp_small_negative(self.disease_pressure);
    }
}

#[derive(Clone, Debug)]
pub struct Patch {
    pub id: PatchId,
    pub coord: PatchCoord,
    pub state: PatchState,
}

impl Patch {
    pub fn new(id: usize, row: usize, col: usize, state: PatchState) -> Self {
        Self {
            id: PatchId(id),
            coord: PatchCoord { row, col },
            state,
        }
    }
}

fn clamp_small_negative(value: f64) -> f64 {
    if value < 0.0 && value > -1.0e-9 {
        0.0
    } else {
        value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EcologicalEvent {
    Drought,
    Disease,
    TemperatureAnomaly,
    HabitatDisturbance,
}

impl EcologicalEvent {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Drought => "drought",
            Self::Disease => "disease",
            Self::TemperatureAnomaly => "temperature_anomaly",
            Self::HabitatDisturbance => "habitat_disturbance",
        }
    }
}

pub fn event_label(events: &[EcologicalEvent]) -> String {
    if events.is_empty() {
        return String::from("none");
    }

    events
        .iter()
        .map(EcologicalEvent::label)
        .collect::<Vec<_>>()
        .join("|")
}

#[derive(Clone, Debug)]
pub struct SmallRng {
    state: u64,
}

impl SmallRng {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        };
        Self { state }
    }

    pub fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        (value as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    pub fn chance(&mut self, probability: f64) -> bool {
        self.next_f64() < probability.clamp(0.0, 1.0)
    }

    pub fn centered(&mut self, width: f64) -> f64 {
        (self.next_f64() - 0.5) * 2.0 * width
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
}

#[cfg(test)]
mod tests {
    use super::SmallRng;

    #[test]
    fn seeded_rng_is_reproducible() {
        let mut a = SmallRng::new(42);
        let mut b = SmallRng::new(42);

        for _ in 0..100 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
    }
}

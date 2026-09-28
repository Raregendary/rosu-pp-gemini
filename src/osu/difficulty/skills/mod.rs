use crate::{any::difficulty::skills::StrainSkill, model::mods::GameMods};

use self::{aim::Aim, flashlight::Flashlight, reading::Reading, speed::Speed};

use super::{object::OsuDifficultyObject, scaling_factor::ScalingFactor};

pub mod aim;
pub mod flashlight;
pub mod reading;
pub mod speed;

pub struct OsuSkills {
    pub aim: Aim,
    pub aim_no_sliders: Aim,
    pub speed: Speed,
    pub reading: Reading,
    pub flashlight: Flashlight,
}

impl OsuSkills {
    pub fn new(
        mods: &GameMods,
        scaling_factor: &ScalingFactor,
        total_objects: usize,
    ) -> Self {
        let aim = Aim::with_capacity(mods, true, total_objects);
        let aim_no_sliders = Aim::with_capacity(mods, false, total_objects);
        let speed = Speed::with_capacity(mods, total_objects);
        let reading = Reading::with_capacity(mods, total_objects);
        let flashlight = Flashlight::new(mods, scaling_factor.radius, total_objects);

        Self {
            aim,
            aim_no_sliders,
            speed,
            reading,
            flashlight,
        }
    }

    pub fn process(&mut self, curr: &OsuDifficultyObject<'_>, objects: &[OsuDifficultyObject<'_>]) {
        self.aim.process(curr, objects);
        self.aim_no_sliders.process(curr, objects);
        self.speed.process(curr, objects);
        self.reading.process(curr, objects);
        self.flashlight.process(curr, objects);
    }
}

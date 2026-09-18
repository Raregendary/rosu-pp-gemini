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
    pub has_flashlight: bool,
}

impl OsuSkills {
    pub fn new(
        mods: &GameMods,
        scaling_factor: &ScalingFactor,
        total_objects: usize,
    ) -> Self {
        let has_flashlight = mods.fl();
        let aim = Aim::new(mods, true);
        let aim_no_sliders = Aim::new(mods, false);
        let speed = Speed::new(mods);
        let reading = Reading::new(mods);
        let flashlight = Flashlight::new(mods, scaling_factor.radius, total_objects);

        Self {
            aim,
            aim_no_sliders,
            speed,
            reading,
            flashlight,
            has_flashlight,
        }
    }

    pub fn process(&mut self, curr: &OsuDifficultyObject<'_>, objects: &[OsuDifficultyObject<'_>]) {
        self.aim.process(curr, objects);
        self.aim_no_sliders.process(curr, objects);
        self.speed.process(curr, objects);
        self.reading.process(curr, objects);
        if self.has_flashlight {
            self.flashlight.process(curr, objects);
        }
    }
}

use crate::{any::difficulty::skills::StrainSkill, model::mods::GameMods};

use self::{aim::Aim, flashlight::Flashlight, reading::Reading, speed::Speed};

use super::{context::OsuDifficultyContext, object::OsuDifficultyObject};

pub mod aim;
pub mod flashlight;
pub mod reading;
mod scratch;
pub mod speed;
mod weights;

pub struct OsuSkills {
    pub aim: Aim,
    pub aim_no_sliders: Aim,
    pub speed: Speed,
    pub reading: Reading,
    pub flashlight: Flashlight,
}

impl OsuSkills {
    pub fn new(mods: &GameMods, total_objects: usize, ctx: OsuDifficultyContext) -> Self {
        let aim = Aim::with_capacity(mods, true, total_objects, ctx);
        let aim_no_sliders = Aim::with_capacity(mods, false, total_objects, ctx);
        let speed = Speed::with_capacity(mods, total_objects);
        let reading = Reading::with_capacity(mods, total_objects, ctx);
        let flashlight = Flashlight::new(mods, total_objects, ctx);

        Self {
            aim,
            aim_no_sliders,
            speed,
            reading,
            flashlight,
        }
    }

    /// Runs every skill over a single hit object.
    ///
    /// The five skills are independent: each reads only `curr` and `objects` and
    /// writes only its own fields, so the results do not depend on the order they
    /// run in and the floating point output is identical either way.
    ///
    /// Deliberately **not** parallelised. The work per hit object is on the order
    /// of a microsecond, so splitting it across a thread pool costs far more in
    /// synchronisation than the imbalance between the skills can win back:
    /// measured on a Ryzen 7 7800X3D it was a 40% *regression*. Use
    /// [`Self::process_all`] for parallelism.
    pub fn process(&mut self, curr: &OsuDifficultyObject<'_>, objects: &[OsuDifficultyObject<'_>]) {
        self.aim.process(curr, objects);
        self.aim_no_sliders.process(curr, objects);
        self.speed.process(curr, objects);
        self.reading.process(curr, objects);
        self.flashlight.process(curr, objects);
    }

    /// Runs every skill over the first `take` objects.
    ///
    /// Each skill is inherently sequential in the objects - strain decay is a
    /// running maximum - so the only parallelism available is *across skills*.
    /// Doing that once per calculation rather than once per hit object is the
    /// difference between roughly five synchronisation points and thousands of
    /// them.
    ///
    /// This is what `DifficultyValues::calculate` uses; [`Self::process`] is the
    /// per-object entry point that gradual difficulty needs, where there is no
    /// work left to spread.
    pub fn process_all(&mut self, objects: &[OsuDifficultyObject<'_>], take: usize) {
        #[cfg(feature = "rayon")]
        {
            use rayon::join;

            // Borrow the fields individually: that is what makes the parallelism
            // safe, since each closure gets exclusive access to one skill.
            let aim = &mut self.aim;
            let aim_no_sliders = &mut self.aim_no_sliders;
            let speed = &mut self.speed;
            let reading = &mut self.reading;
            let flashlight = &mut self.flashlight;

            // Nested rather than flat so the two most expensive skills get a
            // whole thread each, the aim pair shares one, and the very cheap
            // flashlight skill is not worth a thread of its own.
            let _ = join(
                || {
                    join(
                        || aim.process_all(objects, take),
                        || aim_no_sliders.process_all(objects, take),
                    )
                },
                || {
                    join(
                        || speed.process_all(objects, take),
                        || {
                            join(
                                || reading.process_all(objects, take),
                                || flashlight.process_all(objects, take),
                            )
                        },
                    )
                },
            );
        }

        #[cfg(not(feature = "rayon"))]
        {
            self.aim.process_all(objects, take);
            self.aim_no_sliders.process_all(objects, take);
            self.speed.process_all(objects, take);
            self.reading.process_all(objects, take);
            self.flashlight.process_all(objects, take);
        }
    }
}

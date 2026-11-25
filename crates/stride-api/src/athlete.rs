//! The athlete's own facts: the race, the model and the resting heart rate.
//!
//! Three of the insight routes need them — the ladder, the cost curve and
//! the threshold — and they are read here, in one place, so that the
//! three cannot disagree about what the athlete's own facts are.
//!
//! ## What the facts are
//!
//! * the **race** — the athlete's own, the one the paces come from: the
//!   row the store holds as the anchor. A race is an observation the
//!   athlete typed; the app never infers one from the training and never
//!   invents one;
//! * the **goal distance** — the race's own distance. The goal race is
//!   the race the athlete pointed at, and the method's rules branch on
//!   its distance;
//! * the **zone model** — the athlete's own configuration, the one in
//!   force on the day the answer is about;
//! * the **resting heart rate** — the athlete's own measured reading, the
//!   most recent one on or before that day.
//!
//! ## No value
//!
//! Every one of them may be missing, and a missing one makes its own
//! answers no values — never a zero, and never a default. A store with
//! no race holds no paces, and the reason is the app's to state.

use stride_core::date::Date;
use stride_core::training_paces::{Ladder, LadderInput, LongRunKind, Race};

use crate::error::ApiResult;
use crate::state::Api;
use crate::zonemodel::ZoneModel;

/// The athlete's own facts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Facts {
    /// The race the paces come from, as the core reads it.
    pub race: Option<Race>,
    /// The same race, as the store holds it.
    pub stored_race: Option<stride_store::RaceObservation>,
    /// The goal distance, in metres: the race's own distance.
    pub goal_distance_m: Option<f64>,
    /// The zone model in force on the day the answer is about.
    pub model: Option<ZoneModel>,
    /// The resting heart rate: the athlete's own measured reading.
    pub resting_heart_rate: Option<f64>,
}

impl Facts {
    /// The athlete's own facts, read from the store.
    ///
    /// The race is the **anchor**: the row the athlete pointed at. A store
    /// with no race holds no paces, and the answers that need one say so
    /// with the core's own reason.
    pub fn read(api: &Api, today: Date) -> ApiResult<Facts> {
        let stored = api
            .store()
            .anchor_race()
            .map_err(|error| api.store_error("the race could not be read", &error))?;
        let race = stored.as_ref().and_then(|race| {
            let date = Date::parse(&race.race_date)?;
            Some(Race {
                distance_m: race.distance_m,
                time_s: race.time_s,
                date,
                name: race.name.clone(),
            })
        });

        // The goal distance is the race's own distance: the goal race is
        // the race the athlete pointed at.
        let goal_distance_m = race.as_ref().map(|race| race.distance_m);

        // The zone model in force today: the athlete's own configuration.
        //
        // **The watch's own model**, and not the bands from the heart rates
        // the athlete entered: the declaration does not move the
        // threshold, and the threshold's own watch candidate is the
        // watch's configuration and not the athlete's declaration.
        let models = crate::zonemodel::ZoneModels::read(api)?;
        let model = models.watch_in_force(today).cloned();

        // The resting heart rate: the athlete's own measured reading, the
        // most recent one on or before today.
        let wellness = api
            .store()
            .wellness_between("0000-01-01", &today.to_iso())
            .map_err(|error| api.store_error("the wellness could not be read", &error))?;
        let resting_heart_rate = wellness
            .iter()
            .rev()
            .find_map(|row| row.resting_hr)
            .filter(|bpm| bpm.is_finite() && *bpm > 0.0);

        Ok(Facts {
            race,
            stored_race: stored,
            goal_distance_m,
            model,
            resting_heart_rate,
        })
    }

    /// The maximum heart rate the athlete's own model carries.
    pub fn max_heart_rate(&self) -> Option<f64> {
        self.model
            .as_ref()
            .and_then(|model| model.max_heart_rate)
            .filter(|bpm| bpm.is_finite() && *bpm > 0.0)
    }

    /// The ladder, built from the athlete's own facts.
    ///
    /// The core owns the ladder: the rules, the anchors and the wording.
    /// This is the adapter, and it decides nothing.
    pub fn ladder(&self) -> Ladder {
        let input = LadderInput {
            race: self.race.clone(),
            goal_distance_m: self.goal_distance_m,
            max_heart_rate: self.max_heart_rate(),
            resting_heart_rate: self.resting_heart_rate,
            long_run: LongRunKind::Standard,
        };
        stride_core::training_paces::ladder(&input)
    }
}

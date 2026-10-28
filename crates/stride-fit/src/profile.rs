//! The slice of the FIT profile this importer needs.
//!
//! The FIT file itself does not carry units: a `distance` is a `uint32` and
//! only the *profile* says that it is centimetres, and an `altitude` is a
//! `uint16` scaled by five and offset by 500 metres. The reference decoder
//! (`fitparse`, the Python one) applies `value = raw / scale - offset` using
//! exactly the tables below, so they are transcribed from the same profile
//! the reference uses. Getting a scale wrong is a wrong number, not a
//! cosmetic difference, so the table is explicit rather than clever.

/// Global message numbers used by the importer.
pub(crate) const MESG_FILE_ID: u16 = 0;
pub(crate) const MESG_SESSION: u16 = 18;
pub(crate) const MESG_LAP: u16 = 19;
pub(crate) const MESG_RECORD: u16 = 20;
pub(crate) const MESG_DEVICE_INFO: u16 = 23;

use std::collections::HashMap;
use std::sync::OnceLock;

/// What a field holds, and therefore how it is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldKind {
    /// A number, scaled by `scale` and offset by `offset`.
    Number,
    /// A UTF-8 string.
    Text,
    /// An enumeration; the raw number is rendered as the profile's name.
    Enum,
}

/// One field of the FIT profile.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldSpec {
    /// The global message the field belongs to.
    pub mesg_num: u16,
    /// The field's definition number inside the message.
    pub def_num: u8,
    /// The profile's name for the field.
    pub name: &'static str,
    /// The divisor applied to the raw value.
    pub scale: f64,
    /// The value subtracted after the division.
    pub offset: f64,
    /// How the field is rendered.
    pub kind: FieldKind,
}

impl FieldSpec {
    /// Apply the profile's scale and offset to a raw value.
    ///
    /// This mirrors `fitparse`'s `_apply_scale_offset`: divide by the scale,
    /// then subtract the offset.
    pub fn apply(&self, raw: f64) -> f64 {
        let mut value = raw;
        if self.scale != 1.0 {
            value /= self.scale;
        }
        if self.offset != 0.0 {
            value -= self.offset;
        }
        value
    }
}

const fn num(mesg_num: u16, def_num: u8, name: &'static str, scale: f64, offset: f64) -> FieldSpec {
    FieldSpec {
        mesg_num,
        def_num,
        name,
        scale,
        offset,
        kind: FieldKind::Number,
    }
}

const fn text(mesg_num: u16, def_num: u8, name: &'static str) -> FieldSpec {
    FieldSpec {
        mesg_num,
        def_num,
        name,
        scale: 1.0,
        offset: 0.0,
        kind: FieldKind::Text,
    }
}

const fn enumeration(mesg_num: u16, def_num: u8, name: &'static str) -> FieldSpec {
    FieldSpec {
        mesg_num,
        def_num,
        name,
        scale: 1.0,
        offset: 0.0,
        kind: FieldKind::Enum,
    }
}

/// The profile fields the importer reads.
///
/// The reference decoder looks fields up by *name* (`value_of(message,
/// "total_distance")`), so this table is keyed by name too: the profile is
/// the contract, and a wrong scale is a wrong number.
pub(crate) const FIELDS: &[FieldSpec] = &[
    // file_id
    num(MESG_FILE_ID, 3, "serial_number", 1.0, 0.0),
    // device_info
    enumeration(MESG_DEVICE_INFO, 2, "manufacturer"),
    text(MESG_DEVICE_INFO, 27, "product_name"),
    // record
    num(MESG_RECORD, 0, "position_lat", 1.0, 0.0),
    num(MESG_RECORD, 1, "position_long", 1.0, 0.0),
    num(MESG_RECORD, 2, "altitude", 5.0, 500.0),
    num(MESG_RECORD, 3, "heart_rate", 1.0, 0.0),
    num(MESG_RECORD, 4, "cadence", 1.0, 0.0),
    num(MESG_RECORD, 5, "distance", 100.0, 0.0),
    num(MESG_RECORD, 6, "speed", 1000.0, 0.0),
    num(MESG_RECORD, 73, "enhanced_speed", 1000.0, 0.0),
    num(MESG_RECORD, 78, "enhanced_altitude", 5.0, 500.0),
    num(MESG_RECORD, 253, "timestamp", 1.0, 0.0),
    // lap
    num(MESG_LAP, 7, "total_elapsed_time", 1000.0, 0.0),
    num(MESG_LAP, 8, "total_timer_time", 1000.0, 0.0),
    num(MESG_LAP, 9, "total_distance", 100.0, 0.0),
    num(MESG_LAP, 13, "avg_speed", 1000.0, 0.0),
    num(MESG_LAP, 14, "max_speed", 1000.0, 0.0),
    num(MESG_LAP, 15, "avg_heart_rate", 1.0, 0.0),
    num(MESG_LAP, 16, "max_heart_rate", 1.0, 0.0),
    num(MESG_LAP, 17, "avg_cadence", 1.0, 0.0),
    num(MESG_LAP, 18, "max_cadence", 1.0, 0.0),
    num(MESG_LAP, 21, "total_ascent", 1.0, 0.0),
    num(MESG_LAP, 22, "total_descent", 1.0, 0.0),
    // session
    enumeration(MESG_SESSION, 5, "sport"),
    num(MESG_SESSION, 7, "total_elapsed_time", 1000.0, 0.0),
    num(MESG_SESSION, 8, "total_timer_time", 1000.0, 0.0),
    num(MESG_SESSION, 9, "total_distance", 100.0, 0.0),
    num(MESG_SESSION, 11, "total_calories", 1.0, 0.0),
    num(MESG_SESSION, 14, "avg_speed", 1000.0, 0.0),
    num(MESG_SESSION, 15, "max_speed", 1000.0, 0.0),
    num(MESG_SESSION, 16, "avg_heart_rate", 1.0, 0.0),
    num(MESG_SESSION, 17, "max_heart_rate", 1.0, 0.0),
    num(MESG_SESSION, 18, "avg_cadence", 1.0, 0.0),
    num(MESG_SESSION, 19, "max_cadence", 1.0, 0.0),
    num(MESG_SESSION, 22, "total_ascent", 1.0, 0.0),
];

/// The profile fields, indexed by message and name.
fn index() -> &'static HashMap<(u16, &'static str), &'static FieldSpec> {
    static INDEX: OnceLock<HashMap<(u16, &'static str), &'static FieldSpec>> = OnceLock::new();
    INDEX.get_or_init(|| {
        FIELDS
            .iter()
            .map(|spec| ((spec.mesg_num, spec.name), spec))
            .collect()
    })
}

/// Look a profile field up by message and name.
pub(crate) fn spec(mesg_num: u16, name: &str) -> Option<&'static FieldSpec> {
    index().get(&(mesg_num, name)).copied()
}

/// The FIT `sport` enumeration.
const SPORT: &[(u8, &str)] = &[
    (0, "generic"),
    (1, "running"),
    (2, "cycling"),
    (3, "transition"),
    (4, "fitness_equipment"),
    (5, "swimming"),
    (6, "basketball"),
    (7, "soccer"),
    (8, "tennis"),
    (9, "american_football"),
    (10, "training"),
    (11, "walking"),
    (12, "cross_country_skiing"),
    (13, "alpine_skiing"),
    (14, "snowboarding"),
    (15, "rowing"),
    (16, "mountaineering"),
    (17, "hiking"),
    (18, "multisport"),
    (19, "paddling"),
    (20, "flying"),
    (21, "e_biking"),
    (22, "motorcycling"),
    (23, "boating"),
    (24, "driving"),
    (25, "golf"),
    (26, "hang_gliding"),
    (27, "horseback_riding"),
    (28, "hunting"),
    (29, "fishing"),
    (30, "inline_skating"),
    (31, "rock_climbing"),
    (32, "sailing"),
    (33, "ice_skating"),
    (34, "sky_diving"),
    (35, "snowshoeing"),
    (36, "snowmobiling"),
    (37, "stand_up_paddleboarding"),
    (38, "surfing"),
    (39, "wakeboarding"),
    (40, "water_skiing"),
    (41, "kayaking"),
    (42, "rafting"),
    (43, "windsurfing"),
    (44, "kitesurfing"),
    (45, "tactical"),
    (46, "jumpmaster"),
    (47, "boxing"),
    (48, "floor_climbing"),
    (254, "all"),
];

/// The FIT `manufacturer` enumeration, truncated to the names that are worth
/// carrying on a watch.
const MANUFACTURER: &[(u8, &str)] = &[
    (1, "garmin"),
    (2, "garmin_fr405_antfs"),
    (3, "zephyr"),
    (4, "dayton"),
    (5, "idt"),
    (6, "srm"),
    (7, "quarq"),
    (8, "ibike"),
    (9, "saris"),
    (10, "spark_hk"),
    (11, "tanita"),
    (12, "echowell"),
    (13, "dynastream"),
    (14, "nautilus"),
    (15, "dynastream_oem"),
    (16, "north_pole_engineering"),
    (17, "btcadence"),
    (18, "campagnolo"),
    (19, "craft_cycling"),
    (20, "mio_technology_europe"),
    (21, "dexcom"),
    (22, "wahoo_fitness"),
    (23, "octane_fitness"),
    (24, "archinoetics"),
    (25, "the_hurt_box"),
    (26, "citizen_systems"),
    (27, "magellan"),
    (28, "osynce"),
    (29, "holux"),
    (30, "concept2"),
    (31, "one_giant_leap"),
    (32, "acerbs"),
    (33, "didoy"),
    (34, "samstrain"),
    (35, "icg"),
    (36, "mi_audios"),
    (37, "synthes"),
    (38, "lima_quarq"),
    (39, "on_your_mark"),
    (40, "saris_titanium"),
    (41, "fitness_equipment"),
    (42, "favero_electronics"),
    (43, "dynastream_oem"),
    (44, "nci_technology"),
    (45, "iqsquare"),
    (46, "leomo"),
    (47, "fitcare"),
    (48, "magnus_indoor"),
    (49, "atarion"),
    (50, "wearmetrics"),
    (51, "nucleus"),
    (52, "fit_life"),
    (53, "gps_pollution"),
    (54, "bikespark"),
    (55, "powerbahn"),
    (56, "tacx"),
    (57, "dirtain"),
    (58, "magtonic"),
    (59, "1partcarbon"),
    (60, "inside_ride_technologies"),
    (255, "development"),
];

fn lookup(table: &'static [(u8, &'static str)], value: u8) -> Option<&'static str> {
    table.iter().find(|(v, _)| *v == value).map(|(_, n)| *n)
}

/// The name of a FIT `sport` value.
///
/// An unknown value is rendered as its number, exactly like the reference
/// decoder does for an enumeration it does not know.
pub(crate) fn sport_name(value: u8) -> String {
    lookup(SPORT, value)
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

/// The name of a FIT `manufacturer` value.
pub(crate) fn manufacturer_name(value: u8) -> String {
    lookup(MANUFACTURER, value)
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

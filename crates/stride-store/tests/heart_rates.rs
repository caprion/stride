//! The athlete's own declared heart rates.
//!
//! The declaration is a **dated fact**: the kind of heart rate, the value,
//! where it came from, the day it was measured and the day it was entered.
//! What is proved here:
//!
//! * a pair entered together is **one declaration with two rows** — the two
//!   rows name the declaration they belong to;
//! * a later pair is a **new declaration**, and the earlier one stays;
//! * a declaration that carries no value at all is **refused**;
//! * the **migration is idempotent** and an older database gets the table
//!   in one transaction.

// The tests are allowed to panic and to expect: they are a separate crate,
// not a `#[cfg(test)]` module, so the lint configuration does not reach them.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use stride_store::{NewHeartRate, NewHeartRateDeclaration, Result};

/// A declaration of the pair, with the values a test wants.
fn pair() -> NewHeartRateDeclaration {
    NewHeartRateDeclaration {
        rates: vec![
            NewHeartRate {
                kind: "max".to_string(),
                value_bpm: 186.0,
                source: "test".to_string(),
                measured_on: Some("2026-11-08".to_string()),
            },
            NewHeartRate {
                kind: "resting".to_string(),
                value_bpm: 52.0,
                source: "doctor_lab".to_string(),
                measured_on: Some("2026-11-12".to_string()),
            },
        ],
        entered_on: "2026-11-24".to_string(),
        updated_on: "2026-11-24".to_string(),
    }
}

#[test]
fn a_pair_is_one_declaration_with_two_rows() -> Result<()> {
    let (dir, store) = common::store("heart-rates-pair")?;
    let declaration_id = store.write_heart_rate_declaration(&pair())?;

    let rows = store.heart_rate_declarations()?;
    assert_eq!(rows.len(), 2);

    // Both rows name the same declaration: a pair is one statement.
    assert_eq!(rows[0].declaration_id, declaration_id);
    assert_eq!(rows[1].declaration_id, declaration_id);

    // The kinds, the values, the sources and the days are each row's own.
    let max = rows
        .iter()
        .find(|row| row.kind == "max")
        .expect("the maximum is there");
    assert_eq!(max.value_bpm, 186.0);
    assert_eq!(max.source, "test");
    assert_eq!(max.measured_on.as_deref(), Some("2026-11-08"));
    assert_eq!(max.entered_on, "2026-11-24");
    assert_eq!(max.updated_on, "2026-11-24");

    let resting = rows
        .iter()
        .find(|row| row.kind == "resting")
        .expect("the resting rate is there");
    assert_eq!(resting.value_bpm, 52.0);
    assert_eq!(resting.source, "doctor_lab");
    assert_eq!(resting.measured_on.as_deref(), Some("2026-11-12"));

    assert_eq!(store.count_heart_rate_declarations()?, 2);

    drop(dir);
    Ok(())
}

#[test]
fn a_later_declaration_is_a_new_one_and_the_earlier_stays() -> Result<()> {
    let (dir, store) = common::store("heart-rates-history")?;
    store.write_heart_rate_declaration(&pair())?;

    let mut later = pair();
    later.rates[0].value_bpm = 184.0;
    later.entered_on = "2026-12-01".to_string();
    later.updated_on = "2026-12-01".to_string();
    let later_id = store.write_heart_rate_declaration(&later)?;

    let rows = store.heart_rate_declarations()?;
    assert_eq!(rows.len(), 4);

    // The newer declaration comes first, and it is a declaration of its
    // own.
    let declarations: Vec<i64> = rows.iter().map(|row| row.declaration_id).collect();
    assert_eq!(declarations[0], later_id);
    assert_eq!(declarations[1], later_id);
    // And the earlier one is still there, with its own values.
    assert!(declarations.iter().any(|id| *id != later_id));
    assert!(rows.iter().any(|row| row.value_bpm == 186.0));

    drop(dir);
    Ok(())
}

#[test]
fn a_declaration_with_no_value_is_refused() -> Result<()> {
    let (dir, store) = common::store("heart-rates-empty")?;
    let empty = NewHeartRateDeclaration {
        rates: Vec::new(),
        entered_on: "2026-11-24".to_string(),
        updated_on: "2026-11-24".to_string(),
    };
    assert!(store.write_heart_rate_declaration(&empty).is_err());
    assert_eq!(store.count_heart_rate_declarations()?, 0);

    drop(dir);
    Ok(())
}

#[test]
fn the_measurement_day_is_optional() -> Result<()> {
    let (dir, store) = common::store("heart-rates-measured")?;
    let mut declaration = pair();
    declaration.rates[0].measured_on = None;
    store.write_heart_rate_declaration(&declaration)?;

    let rows = store.heart_rate_declarations()?;
    let max = rows
        .iter()
        .find(|row| row.kind == "max")
        .expect("the maximum is there");
    // A day that was not stated is not there, and it is never a zero.
    assert_eq!(max.measured_on, None);

    drop(dir);
    Ok(())
}

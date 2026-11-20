//! The heatmap: a grid over the stored positions.
//!
//! The engine is a pure function of the positions handed to it: there is no
//! clock in these tests, no file is read and nothing is fetched. Every
//! expected number is worked out from the definition in the test itself.
//!
//! **The positions are invented.** They sit near the corpus's synthetic
//! anchor in the South Pacific, and they are not anybody's route.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use stride_core::heatmap::{
    bounds_of, cell_of, cell_size_m, heatmap, Bounds, Cell, GRID_CELLS_PER_AXIS, GRID_ZOOM,
};

/// A position, as the grid reads it: the latitude and the longitude.
type Position = [f64; 2];

/// The anchor of the corpus, in the South Pacific.
const ANCHOR: Position = [-35.0, -140.0];

/// A run's positions.
fn run(positions: &[Position]) -> Vec<Position> {
    positions.to_vec()
}

/// The runs, as the grid reads them.
fn runs(runs: &[Vec<Position>]) -> Vec<&[Position]> {
    runs.iter().map(|positions| positions.as_slice()).collect()
}

/// The cell a position is in.
fn cell_of_position(position: Position) -> Cell {
    cell_of(position[0], position[1]).expect("the position is a position")
}

#[test]
fn a_run_counts_once_in_a_cell_it_crosses_repeatedly() {
    // One run that goes round the same corner four times: the cell counts
    // the run, not the laps.
    let corner = ANCHOR;
    let run = run(&[corner, corner, corner, corner]);
    let grid = heatmap(&runs(&[run]));

    assert_eq!(grid.runs, 1, "one run was counted");
    assert_eq!(grid.cells.len(), 1, "the run touched one cell");
    assert_eq!(grid.cells[0].runs, 1, "the cell counts the run once");
}

#[test]
fn a_cell_counts_every_run_that_went_through_it() {
    // Two runs over the same corner and one run somewhere else.
    let here = ANCHOR;
    let there = [-35.5, -140.5];
    let grid = heatmap(&runs(&[run(&[here, here]), run(&[here]), run(&[there])]));

    assert_eq!(grid.runs, 3, "three runs were counted");
    assert_eq!(grid.cells.len(), 2, "two cells were touched");
    let busiest = grid.busiest();
    assert_eq!(busiest, 2, "the corner cell holds two runs");
    // And the cells are ordered north to south, west to east.
    assert!(grid.cells[0].cell.y <= grid.cells[1].cell.y);
}

#[test]
fn a_cell_that_no_run_went_through_is_not_in_the_grid() {
    let grid = heatmap(&runs(&[run(&[ANCHOR])]));

    let elsewhere = cell_of(-35.5, -140.5).expect("the position is a position");
    assert!(
        !grid.cells.iter().any(|cell| cell.cell == elsewhere),
        "a cell no run went through is not in the grid"
    );
}

#[test]
fn a_run_with_no_position_is_not_counted() {
    // A run whose file carries no position: there is nothing to place, so
    // the run is not in the grid and does not make it empty either.
    let grid = heatmap(&runs(&[run(&[]), run(&[ANCHOR])]));

    assert_eq!(grid.runs, 1, "the run with no position was not counted");
    assert_eq!(grid.cells.len(), 1);
}

#[test]
fn an_empty_history_has_no_grid() {
    let grid = heatmap(&runs(&[]));

    assert!(grid.is_empty(), "an empty history has no heatmap");
    assert_eq!(grid.runs, 0);
    assert!(grid.cells.is_empty(), "there are no cells to draw");
    assert!(grid.extent.is_none(), "there is no box to draw them in");
    assert_eq!(grid.busiest(), 0);
}

#[test]
fn the_grid_is_coarse_on_purpose() {
    // The resolution is the reference implementation's own: zoom 14 of the
    // slippy-tile grid, which is about two kilometres across at the
    // latitudes a runner lives at.
    assert_eq!(GRID_ZOOM, 14);
    assert_eq!(GRID_CELLS_PER_AXIS, 16_384);

    // The cell's own size on the ground, at the anchor's latitude.
    let size = cell_size_m(ANCHOR[0]);
    assert!(
        size > 1_500.0 && size < 2_500.0,
        "a cell is about two kilometres across, not {size}"
    );

    // And the grid is coarse in the world, too: two positions a few
    // hundred metres apart share a cell, and two positions a few
    // kilometres apart do not.
    let near = [-35.0, -139.995];
    let far = [-35.0, -139.97];
    assert_eq!(
        cell_of_position(near),
        cell_of_position(ANCHOR),
        "a few hundred metres apart is the same cell"
    );
    assert_ne!(
        cell_of_position(far),
        cell_of_position(ANCHOR),
        "a few kilometres apart is not the same cell"
    );
}

#[test]
fn a_position_that_is_not_a_position_is_in_no_cell() {
    assert!(cell_of(f64::NAN, -140.0).is_none());
    assert!(cell_of(-35.0, f64::INFINITY).is_none());
    assert!(cell_of(91.0, -140.0).is_none());
    assert!(cell_of(-35.0, 181.0).is_none());
}

#[test]
fn a_position_exactly_on_a_cell_edge_belongs_to_the_cell_it_opens() {
    let cell = cell_of_position(ANCHOR);
    let bounds = bounds_of(cell);

    // The western edge of the cell: the position on it is in the cell, and
    // a position just west of it is in the cell before.
    assert_eq!(
        cell_of(bounds.north, bounds.west),
        Some(cell),
        "the edge opens the cell to its east"
    );
    assert_eq!(
        cell_of(bounds.north, bounds.west - 0.000_001),
        Some(Cell::new(cell.x - 1, cell.y)),
        "the cell before the edge is the cell before it"
    );

    // The cell's own corners are inside it, and the cell is a rectangle
    // that the next one starts where this one ends.
    assert!(bounds.west < bounds.east);
    assert!(bounds.south < bounds.north);
    let next = bounds_of(Cell::new(cell.x + 1, cell.y));
    assert!((next.west - bounds.east).abs() < 1e-9);
}

#[test]
fn the_grid_carries_no_order_and_no_time() {
    // The answer is a set of rectangles with a number each: there is no
    // time on a cell, no run on a cell and no connection between cells.
    let grid = heatmap(&runs(&[run(&[ANCHOR, [-35.01, -140.01]])]));
    for cell in &grid.cells {
        assert!(cell.runs >= 1, "a cell counts at least one run");
        let bounds: Bounds = cell.bounds;
        assert!(bounds.south.is_finite() && bounds.east.is_finite());
    }
}

#[test]
fn the_extent_covers_every_cell() {
    let grid = heatmap(&runs(&[run(&[[-35.0, -140.0], [-35.6, -140.6]])]));
    let extent = grid.extent.expect("the grid has an extent");

    for cell in &grid.cells {
        assert!(cell.bounds.south >= extent.south);
        assert!(cell.bounds.north <= extent.north);
        assert!(cell.bounds.west >= extent.west);
        assert!(cell.bounds.east <= extent.east);
    }
}

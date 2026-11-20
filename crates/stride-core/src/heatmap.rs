//! The heatmap: a grid over the stored positions, computed on the device.
//!
//! **Nothing leaves the device.** There is no tile request here, no
//! geocoding, no reverse geocoding and no third-party service: the grid is
//! made of the positions the athlete's own runs carry, and the answer is a
//! number per cell. Like every other derived series in this crate, it is
//! **recomputed on read and never stored**: there is no grid, no tile and no
//! heatmap blob in the database, and a run's route is not written down
//! twice.
//!
//! ## The grid
//!
//! The grid is the slippy-tile grid of the reference implementation, at
//! **its own resolution** ([`GRID_ZOOM`]): the map tiles it counts a run
//! against. A cell is about two kilometres across at the latitudes a
//! runner lives at, and that coarseness is the point — a cell is a part of
//! town, not a street and not a house, and a coarse grid is both faster to
//! compute and less identifying than a fine one.
//!
//! The grid is in the Web Mercator projection, so a cell is a rectangle in
//! the athlete's own coordinates and a square on the ground.
//!
//! ## What a cell counts
//!
//! A cell's value is **the number of runs that went through it** — never a
//! route, never a path, never a time. Three properties follow, and they
//! are what makes the surface safe to show:
//!
//! * **A run counts once in a cell**, however many times it passed
//!   through it. A cell is a count of runs, not of positions.
//! * **No cell carries a time, an order or a connection to another cell.**
//!   The answer is a set of rectangles with a number each: there is no
//!   sequence in it to reconstruct a path from.
//! * **A cell is a place, not a person.** Nothing here names a street, an
//!   address or a neighbourhood, and nothing here is a conclusion about the
//!   athlete.
//!
//! ## The boundary
//!
//! A position that falls exactly on a cell edge belongs to the cell the
//! edge opens — the **higher** cell, east or north. The rule is the same
//! one the grid is built with: a cell covers the range from its own edge
//! (inclusive) to the next edge (exclusive), so a position is never in two
//! cells and never in none.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// The grid's resolution.
///
/// The reference implementation's own zoom level: the map tiles it counts
/// a run against. A cell is about two kilometres across at the latitudes a
/// runner lives at, which is coarse on purpose — a coarse grid is faster
/// and less identifying than a fine one, and this surface is about a habit.
pub const GRID_ZOOM: u32 = 14;

/// How many cells the grid has along one axis.
pub const GRID_CELLS_PER_AXIS: u32 = 1 << GRID_ZOOM;

/// The limit of the Web Mercator projection, where its scale stays finite.
const MERCATOR_LIMIT: f64 = 85.05112878;

/// The radius of the earth, in metres — the projection's own sphere.
const EARTH_RADIUS_M: f64 = 6_378_137.0;

/// One cell of the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Cell {
    /// The cell's column, counted from the west.
    pub x: u32,
    /// The cell's row, counted from the north.
    pub y: u32,
}

impl Cell {
    /// A cell of the grid.
    pub fn new(x: u32, y: u32) -> Cell {
        Cell { x, y }
    }
}

/// The corners of a cell, in the athlete's own coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    /// The southern edge.
    pub south: f64,
    /// The western edge.
    pub west: f64,
    /// The northern edge.
    pub north: f64,
    /// The eastern edge.
    pub east: f64,
}

impl Bounds {
    /// The box that holds both boxes.
    ///
    /// The union of two boxes is the box around them: the smallest box
    /// that holds both, and nothing else.
    pub fn union(self, other: Bounds) -> Bounds {
        Bounds {
            south: self.south.min(other.south),
            west: self.west.min(other.west),
            north: self.north.max(other.north),
            east: self.east.max(other.east),
        }
    }
}

/// A cell of the heatmap: where it is, and how many runs went through it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeatmapCell {
    /// The cell's place in the grid.
    pub cell: Cell,
    /// The cell's corners.
    pub bounds: Bounds,
    /// How many runs went through this cell.
    pub runs: u32,
}

/// The grid over a set of runs.
///
/// The answer is a set of rectangles with a number each. It carries no
/// order, no time and no connection between the cells, so it cannot be
/// read back as a path.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Heatmap {
    /// The runs that carried at least one position.
    pub runs: u32,
    /// The cells, north to south and west to east.
    pub cells: Vec<HeatmapCell>,
    /// The box the cells cover.
    pub extent: Option<Bounds>,
}

impl Heatmap {
    /// Whether there is nothing to draw.
    ///
    /// A history with no position in it has no heatmap: that is the
    /// absence of the whole result, never a grid of empty cells.
    pub fn is_empty(&self) -> bool {
        self.runs == 0
    }

    /// The busiest cell's count — the top of the heatmap's scale.
    pub fn busiest(&self) -> u32 {
        self.cells.iter().map(|cell| cell.runs).max().unwrap_or(0)
    }
}

/// The cell a position falls in.
///
/// `None` when the position is not a position: not a number, or outside
/// the world. A position that is not there is not in a cell.
///
/// **The boundary rule:** a position exactly on a cell edge belongs to the
/// cell the edge opens — the higher one, east or north.
pub fn cell_of(latitude: f64, longitude: f64) -> Option<Cell> {
    if !latitude.is_finite() || !longitude.is_finite() {
        return None;
    }
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        return None;
    }
    let axis = GRID_CELLS_PER_AXIS as f64;
    let clamped = latitude.clamp(-MERCATOR_LIMIT, MERCATOR_LIMIT);
    // The column is computed in this order on purpose: the scale is
    // applied before the division, so a position that is exactly on a
    // column's western edge lands exactly on the column it opens.
    let x = ((longitude + 180.0) * axis / 360.0).floor();
    let y = (1.0 - clamped.to_radians().tan().asinh() / std::f64::consts::PI) / 2.0 * axis;
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }
    // The last column and row: the world ends there, and the cell that
    // opens at the edge is the last one.
    let x = (x as u64).min(GRID_CELLS_PER_AXIS as u64 - 1) as u32;
    let y = (y as u64).min(GRID_CELLS_PER_AXIS as u64 - 1) as u32;
    Some(Cell::new(x, y))
}

/// The corners of a cell.
///
/// The cell covers the range from its own edge (inclusive) to the next
/// edge (exclusive), so the cells of a row tile the row without a gap and
/// without an overlap.
pub fn bounds_of(cell: Cell) -> Bounds {
    let axis = GRID_CELLS_PER_AXIS as f64;
    let west = cell.x as f64 / axis * 360.0 - 180.0;
    let east = (cell.x as f64 + 1.0) / axis * 360.0 - 180.0;
    let north = latitude_of_row(cell.y);
    let south = latitude_of_row(cell.y + 1);
    Bounds {
        south,
        west,
        north,
        east,
    }
}

/// The size of a cell on the ground, in metres, at a latitude.
///
/// The cells of the projection are squares on the ground, and they shrink
/// towards the poles: this is the size of one of them where the athlete
/// runs.
pub fn cell_size_m(latitude: f64) -> f64 {
    let side = 2.0 * std::f64::consts::PI / GRID_CELLS_PER_AXIS as f64;
    EARTH_RADIUS_M * side * latitude.to_radians().cos()
}

/// The grid over the positions of a set of runs.
///
/// Each run contributes **one** to every cell it touches, and nothing to
/// the cells it does not. A run with no position in it is not counted at
/// all: it is not a run that went nowhere, it is a run the answer does not
/// cover.
///
/// The cells come back north to south and west to east, so the answer is
/// the same on every read.
pub fn heatmap(runs: &[&[[f64; 2]]]) -> Heatmap {
    let mut counts: BTreeMap<Cell, u32> = BTreeMap::new();
    let mut counted: u32 = 0;
    for run in runs {
        // The cells this run touches, each of them once: a run that loops
        // around a block crosses the same cell many times, and the cell
        // counts the run, not the laps.
        let mut touched: BTreeSet<Cell> = BTreeSet::new();
        for position in run.iter() {
            if let Some(cell) = cell_of(position[0], position[1]) {
                touched.insert(cell);
            }
        }
        if touched.is_empty() {
            continue;
        }
        counted += 1;
        for cell in touched {
            *counts.entry(cell).or_insert(0) += 1;
        }
    }

    let mut cells: Vec<HeatmapCell> = counts
        .into_iter()
        .map(|(cell, runs)| HeatmapCell {
            cell,
            bounds: bounds_of(cell),
            runs,
        })
        .collect();
    // North to south, then west to east.
    cells.sort_by(|left, right| {
        left.cell
            .y
            .cmp(&right.cell.y)
            .then(left.cell.x.cmp(&right.cell.x))
    });

    let extent = cells.iter().map(|cell| cell.bounds).reduce(Bounds::union);

    Heatmap {
        runs: counted,
        cells,
        extent,
    }
}

/// The latitude a row of the grid opens at.
fn latitude_of_row(row: u32) -> f64 {
    let axis = GRID_CELLS_PER_AXIS as f64;
    let radians = (std::f64::consts::PI * (1.0 - 2.0 * row as f64 / axis))
        .sinh()
        .atan();
    radians.to_degrees()
}

use na::{Point2, Vector2};
use nalgebra as na;

use std::f64;

use crate::boid::Boid;

pub const DEFAULT_NR_OF_BOIDS: usize = 100;
pub const DEFAULT_MAX_SPEED: f64 = 300.0;
pub const DEFAULT_MAX_STEER: f64 = 30.0;
pub const DEFAULT_ALIGN_RADIUS: f64 = 50.0;
pub const DEFAULT_COHESION_RADIUS: f64 = 70.0;
pub const DEFAULT_SEPERATION_RADIUS: f64 = 15.0;
pub const DEFAULT_ANGST_RADIUS: f64 = 100.0;
pub const DEFAULT_ALIGN_FACTOR: f64 = 1.0 / 8.0;
pub const DEFAULT_COHESION_FACTOR: f64 = 1.0 / 100.0;
pub const DEFAULT_SEPERATION_FACTOR: f64 = 1.0;
pub const DEFAULT_ANGST_FACTOR: f64 = 2000.0;
pub const RETURN_STEER_VAL: f64 = 10.0;
pub const WALL_SIZE: f64 = 100.0;
pub const BUCKET_SIZE: usize = 50;

pub struct Boids {
    pub boids: Vec<Boid>,
    pub size: (f64, f64),
    pub align_radius_squared: f64,
    pub cohesion_radius_squared: f64,
    pub seperation_radius_squared: f64,
    pub angst_radius_squared: f64,
    pub debug_mode: bool,
    pub predator: Vector2<f64>,
    pub max_speed: f64,
    pub max_steer: f64,
    pub align_factor: f64,
    pub cohesion_factor: f64,
    pub seperation_factor: f64,
    pub angst_factor: f64,
}

impl Boids {
    pub fn new(width: f64, height: f64) -> Self {
        let boids = (0..DEFAULT_NR_OF_BOIDS)
            .map(|_| Boid::new(width, height))
            .collect();
        let size = (width, height);
        let align_radius_squared = DEFAULT_ALIGN_RADIUS.powf(2.0);
        let cohesion_radius_squared = DEFAULT_COHESION_RADIUS.powf(2.0);
        let seperation_radius_squared = DEFAULT_SEPERATION_RADIUS.powf(2.0);
        let angst_radius_squared = DEFAULT_ANGST_RADIUS.powf(2.0);
        let debug_mode = false;
        let predator = Vector2::new(width / 2.0, height / 2.0);
        let max_speed = DEFAULT_MAX_SPEED;
        let max_steer = DEFAULT_MAX_STEER;
        let align_factor = DEFAULT_ALIGN_FACTOR;
        let center_factor = DEFAULT_COHESION_FACTOR;
        let seperation_factor = DEFAULT_SEPERATION_FACTOR;
        let angst_factor = DEFAULT_ANGST_FACTOR;
        Boids {
            boids,
            size,
            align_radius_squared,
            cohesion_radius_squared,
            seperation_radius_squared,
            angst_radius_squared,
            predator,
            debug_mode,
            max_speed,
            max_steer,
            align_factor,
            cohesion_factor: center_factor,
            seperation_factor,
            angst_factor,
        }
    }
    pub fn scatter(&mut self) {
        for boid in &mut self.boids {
            *boid = Boid::new(self.size.0, self.size.1)
        }
    }
    /// Get an iterator of relevant neighbours and their distance to the current boid.
    /// A boid is not relevant if the distance to it is larger than the largest
    /// distance considered.
    pub fn get_weighted_others(&self, curr_idx: usize) -> impl Iterator<Item = (&Boid, f64)> {
        let max_radius_squared = self
            .align_radius_squared
            .max(self.cohesion_radius_squared)
            .max(self.seperation_radius_squared);
        let max_bucket_dist = 1 + max_radius_squared as usize / BUCKET_SIZE;
        // The current element and it's position
        let this = &self.boids[curr_idx];
        let this_pos = Point2::new(this.pos.x, this.pos.y);
        // Filter the rest and map add the distance to them
        self.boids
            .iter()
            .enumerate()
            .filter(move |(idx, _)| *idx != curr_idx)
            .filter(move |(_, boid)| {
                let diff = bucket_diff(&this.id, &boid.id);
                diff.0 < max_bucket_dist || diff.1 < max_bucket_dist
            })
            .filter_map(move |(_, other)| {
                let pos = Point2::new(other.pos.x, other.pos.y);
                let dist_squared = na::distance_squared(&this_pos, &pos);
                if dist_squared < max_radius_squared {
                    Some((other, dist_squared))
                } else {
                    None
                }
            })
    }

    pub fn get_return_steer(&self, curr_idx: usize) -> Vector2<f64> {
        let size = self.size;
        let curr = &self.boids[curr_idx];
        let mut steer: Vector2<_> = na::zero();
        if curr.pos.x < 0.0 + WALL_SIZE {
            steer.x += RETURN_STEER_VAL;
        } else if curr.pos.x > size.0 - WALL_SIZE {
            steer.x -= RETURN_STEER_VAL;
        }
        if curr.pos.y < 0.0 + WALL_SIZE {
            steer.y += RETURN_STEER_VAL;
        } else if curr.pos.y > size.1 - WALL_SIZE {
            steer.y -= RETURN_STEER_VAL;
        }
        steer
    }

    pub fn get_angst_steer(&self, curr_idx: usize) -> Vector2<f64> {
        let this_pos = Point2::origin() + self.boids[curr_idx].pos;
        let pred_pos = Point2::origin() + self.predator;
        let dist_sq = na::distance_squared(&this_pos, &pred_pos);
        if dist_sq > self.angst_radius_squared {
            na::zero()
        } else if (dist_sq).abs() > f64::EPSILON {
            (self.boids[curr_idx].pos - self.predator) / dist_sq
        } else {
            self.boids[curr_idx].pos - self.predator
        }
    }

    pub fn get_cohesion_steer(&self, relevant: &[(&Boid, f64)], curr_idx: usize) -> Vector2<f64> {
        let (sum, count): (Vector2<f64>, usize) = relevant
            .iter()
            .filter(|(_, dist)| *dist <= self.cohesion_radius_squared)
            .map(|(boid, _)| boid.pos)
            .fold((na::zero(), 0), |(sum, count), pos| (sum + pos, count + 1));
        if count != 0 {
            let target = sum / count as f64;
            target - self.boids[curr_idx].pos
        } else {
            na::zero()
        }
    }

    pub fn get_seperation_steer(&self, relevant: &[(&Boid, f64)], curr_idx: usize) -> Vector2<f64> {
        relevant
            .iter()
            .filter(|(_, dist)| *dist <= self.seperation_radius_squared)
            .map(|(boid, _)| boid.pos - self.boids[curr_idx].pos)
            .fold(na::zero(), |sum: Vector2<f64>, el| sum - el)
    }

    pub fn get_align_steer(&self, relevant: &[(&Boid, f64)]) -> Vector2<f64> {
        // We're not our friend, no filtering necessary
        let (sum, count) = relevant
            .iter()
            .filter(|(_, dist)| *dist <= self.align_radius_squared)
            .map(|(boid, _)| boid.vel)
            .fold((na::zero(), 0), |(sum, count), vel| (sum + vel, count + 1));
        if count != 0 {
            sum / count as f64
        } else {
            sum
        }
    }

    pub fn to_url_suffix(&self) -> String {
        let mut values = vec![
            maybe(
                "align-radius",
                self.align_radius_squared.sqrt(),
                DEFAULT_ALIGN_RADIUS,
            ),
            maybe(
                "cohesion-radius",
                self.cohesion_radius_squared.sqrt(),
                DEFAULT_COHESION_RADIUS,
            ),
            maybe(
                "seperation-radius",
                self.seperation_radius_squared.sqrt(),
                DEFAULT_SEPERATION_RADIUS,
            ),
            maybe(
                "angst-radius",
                self.angst_radius_squared.sqrt(),
                DEFAULT_ANGST_RADIUS,
            ),
            maybe(
                "nr-of-boids",
                self.boids.len() as f64,
                DEFAULT_NR_OF_BOIDS as f64,
            ),
            maybe("max-speed", self.max_speed, DEFAULT_MAX_SPEED),
            maybe("max-steer", self.max_steer, DEFAULT_MAX_STEER),
            maybe("align-factor", self.align_factor, DEFAULT_ALIGN_FACTOR),
            maybe(
                "cohesion-factor",
                self.cohesion_factor,
                DEFAULT_COHESION_FACTOR,
            ),
            maybe(
                "seperation-factor",
                self.seperation_factor,
                DEFAULT_SEPERATION_FACTOR,
            ),
            maybe("angst-factor", self.angst_factor, DEFAULT_ANGST_FACTOR),
        ];
        values
            .drain(..)
            .filter_map(|option| option)
            .map(|(name, val)| format!("{}={}", name, val))
            .fold(String::from("?"), |concat, elem| concat + &elem + "&")
    }
}

fn maybe(name: &str, val: f64, default: f64) -> Option<(&str, f64)> {
    if val != default {
        Some((name, val))
    } else {
        None
    }
}

fn bucket_diff(this: &(usize, usize), other: &(usize, usize)) -> (usize, usize) {
    let x = this.0.max(other.0) - this.0.min(other.0);
    let y = this.1.max(other.1) - this.1.min(other.1);
    (x, y)
}

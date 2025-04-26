use std::fmt::Display;
use std::ops::{Add, Div, Mul, Neg, Sub};

pub const PI: f64 = 3.14159265358979323846;
pub const INF: f64 = 1e128;
pub const EPS: f64 = 1e-6;

#[derive(Debug, Clone, Copy)]
pub struct Vec3(pub f64, pub f64, pub f64);

pub type Point3 = Vec3;
pub type Color = Vec3;

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Self::Output {
        Vec3(-self.0, -self.1, -self.2)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl Vec3 {
    pub fn new(c: f64) -> Self {
        Vec3(c, c, c)
    }

    pub fn length_sq(&self) -> f64 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    pub fn length(&self) -> f64 {
        self.length_sq().sqrt()
    }

    pub fn normalize(&self) -> Self {
        *self / self.length()
    }
}

pub fn multiply(v: Vec3, w: Vec3) -> Vec3 {
    Vec3(v.0 * w.0, v.1 * w.1, v.2 * w.2)
}

pub fn dot(v: Vec3, w: Vec3) -> f64 {
    v.0 * w.0 + v.1 * w.1 + v.2 * w.2
}

pub fn cross(v: Vec3, w: Vec3) -> Vec3 {
    Vec3(
        v.1 * w.2 - v.2 * w.1,
        v.2 * w.0 - v.0 * w.2,
        v.0 * w.1 - v.1 * w.0,
    )
}

pub fn max_elm(v: Vec3) -> f64 {
    let mut max = v.0;

    if max < v.1 {
        max = v.1;
    }
    if max < v.2 {
        max = v.2
    }
    max
}

pub fn min_elm(v: Vec3) -> f64 {
    let mut min = v.0;

    if min > v.1 {
        min = v.1;
    }
    if min > v.2 {
        min = v.2;
    }
    min
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

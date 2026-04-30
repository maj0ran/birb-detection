#[derive(Debug, Copy, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

#[allow(unused)]
impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2 { x, y }
    }

    pub fn perp(&self) -> Vec2<T>
    where
        T: std::ops::Neg<Output = T>,
        T: Copy,
    {
        Vec2 {
            x: self.y,
            y: -self.x,
        }
    }
}

#[allow(unused)]
impl Vec2<f32> {
    pub fn len(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y)
    }

    pub fn norm(&self) -> Vec2<f32> {
        *self / self.len()
    }

    pub fn abs(&self) -> Vec2<f32> {
        Vec2 {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }

    pub fn max(&self, other: Vec2<f32>) -> Vec2<f32> {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    pub fn min(&self, other: Vec2<f32>) -> Vec2<f32> {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }
}

impl From<Vec2<i16>> for Vec2<f32> {
    fn from(value: Vec2<i16>) -> Self {
        Vec2::<f32> {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}

impl From<Vec2<f32>> for Vec2<i16> {
    fn from(value: Vec2<f32>) -> Self {
        Vec2::<i16> {
            x: value.x as i16,
            y: value.y as i16,
        }
    }
}
impl From<UVec> for Vec2<f32> {
    fn from(value: UVec) -> Self {
        Vec2::<f32> {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}

impl From<UVec> for IVec {
    fn from(value: UVec) -> Self {
        IVec {
            x: value.x as i16,
            y: value.y as i16,
        }
    }
}

impl From<IVec> for UVec {
    fn from(value: IVec) -> Self {
        UVec {
            x: value.x as u16,
            y: value.y as u16,
        }
    }
}

impl<T> std::ops::Add for Vec2<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Vec2<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2::<T> {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> std::ops::Sub for Vec2<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Vec2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::<T> {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> std::ops::Mul<T> for Vec2<T>
where
    T: std::ops::Mul<Output = T>,
    T: Copy,
{
    type Output = Vec2<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vec2::<T> {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> std::ops::Div<T> for Vec2<T>
where
    T: std::ops::Div<Output = T>,
    T: Copy,
{
    type Output = Vec2<T>;

    fn div(self, rhs: T) -> Self::Output {
        Vec2::<T> {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> PartialEq for Vec2<T>
where
    T: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn ne(&self, other: &Self) -> bool {
        self.x != other.x || self.y != other.y
    }
}

pub struct Mat2<T> {
    m: [[T; 2]; 2],
}

#[allow(unused)]
impl<T> Mat2<T> {
    pub fn new(vals: (T, T, T, T)) -> Mat2<T> {
        Mat2 {
            m: [[vals.0, vals.2], [vals.1, vals.3]], // column-major
        }
    }
}

impl<T> std::ops::Mul<Vec2<T>> for Mat2<T>
where
    T: std::ops::Mul<Output = T>,
    T: std::ops::Add<Output = T>,
    T: Copy,
{
    type Output = Vec2<T>;

    fn mul(self, rhs: Vec2<T>) -> Self::Output {
        let x = self.m[0][0] * rhs.x + self.m[0][1] * rhs.y;
        let y = self.m[1][0] * rhs.x + self.m[1][1] * rhs.y;

        Vec2 { x, y }
    }
}

impl From<Vec2<f32>> for rusttype::Point<f32> {
    fn from(value: Vec2<f32>) -> Self {
        rusttype::Point {
            x: value.x,
            y: value.y,
        }
    }
}

pub type UVec = Vec2<u16>;
pub type IVec = Vec2<i16>;

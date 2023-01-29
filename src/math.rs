pub fn min(a:f64, b:f64, c:f64) -> f64 {
    if a < b && a < c {
        return a;
    }
    return if b < c { b } else { c };
}

pub fn max(a:f64, b:f64, c:f64) -> f64 {
    if a > b && a > c {
        return a;
    }
    return if b > c { b } else { c };
}

pub fn clamp(value:f64, min:f64, max:f64) -> f64 {
    if value > max { return max; }
    if value < min { return min; }
    return value;
}


#[derive(Clone)]
pub struct V3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub trait VectorMath {
    fn add(&self, other: &V3) -> V3;
    fn subtract(&self, other: &V3) -> V3;
    fn multiply(&self, amount: f64) -> V3;
    fn dot(&self, other: &V3) -> f64;
    fn cross(&self, other: &V3) -> V3;
    fn length(&self) -> f64;
    fn normalize(&self) -> V3;
    fn lerp(&self, other: &V3, percentage: f64) -> V3;
}

impl VectorMath for V3 {
    fn add(&self, other: &V3) -> V3 {
        return V3 { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z };
    }

    fn subtract(&self, other: &V3) -> V3 {
        return V3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z };
    }

    fn multiply(&self, amount: f64) -> V3 {
        return V3 { x: self.x * amount, y: self.y * amount, z: self.z * amount };
    }

    fn dot(&self, other: &V3) -> f64 {
        return self.x * other.x + self.y * other.y + self.z * other.z;
    }

    fn cross(&self, other: &V3) -> V3 {
        return V3 {
            x: (self.y * other.z) - (self.z * other.y),
            y: (self.z * other.x) - (self.x * other.z),
            z: (self.x * other.y) - (self.y * other.x),
        }
    }

    fn length(&self) -> f64 {
        let squared_length = self.x * self.x + self.y * self.y + self.z * self.z;
        return squared_length.sqrt();
    }

    fn normalize(&self) -> V3 {
        let length = self.length();
        return V3 { x: self.x / length,
                    y: self.y / length,
                    z: self.z / length };
    }

    fn lerp(&self, other: &V3, ratio: f64) -> V3 {
        return other.subtract(&self).multiply(ratio).add(&self);
    }
}

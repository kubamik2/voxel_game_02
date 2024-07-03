pub const LIGHT_LEVEL_BITS: u32 = 4;
pub const LIGHT_LEVEL_MAX_VALUE: u8 = 2_u8.pow(LIGHT_LEVEL_BITS);

#[derive(Clone, Copy)]
pub struct LightLevel(u8);

impl LightLevel {
    #[inline]
    pub fn new(level: u8) -> Option<Self> {
        if level > LIGHT_LEVEL_MAX_VALUE { return None; }
        Some(LightLevel(level))
    }

    #[inline]
    pub fn add(&self, level: u8) -> Option<Self> {
        if let Some(inner) = self.0.checked_add(level) {
            return Self::new(inner);
        }
        None
    }

    #[inline]
    pub fn sub(&self, level: u8) -> Option<Self> {
        if let Some(inner) = self.0.checked_sub(level) {
            return Self::new(inner);
        }
        None
    }

    #[inline]
    pub fn get(&self) -> u8 {
        self.0
    }
}
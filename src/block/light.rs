pub const LIGHT_LEVEL_BITS: u32 = 4;
pub const LIGHT_LEVEL_MAX_VALUE: u8 = (1 << LIGHT_LEVEL_BITS) - 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)] // TODO implement Debug manually
pub struct LightLevel(u8);

impl LightLevel {
    #[inline]
    const fn invariants_satisfied(level: u8) -> bool {
        level <= LIGHT_LEVEL_MAX_VALUE
    }

    #[inline]
    pub const fn new(block_level: u8, sky_level: u8) -> Option<Self> {
        if !Self::invariants_satisfied(block_level) || !Self::invariants_satisfied(sky_level) { return None; }
        Some(LightLevel(block_level | (sky_level << LIGHT_LEVEL_BITS)))
    }

    #[inline]
    pub fn set_block(&mut self, level: u8) {
        assert!(Self::invariants_satisfied(level));
        self.0 &= LIGHT_LEVEL_MAX_VALUE << LIGHT_LEVEL_BITS;
        self.0 |= level;
    }
    
    #[inline]
    pub fn set_block_saturate(&mut self, mut level: u8) {
        level = level.min(LIGHT_LEVEL_MAX_VALUE);
        self.0 &= LIGHT_LEVEL_MAX_VALUE << LIGHT_LEVEL_BITS;
        self.0 |= level;
    }

    #[inline]
    pub fn get_block(&self) -> u8 {
        self.0 & LIGHT_LEVEL_MAX_VALUE
    }

    #[inline]
    pub fn set_sky(&mut self, level: u8) {
        assert!(Self::invariants_satisfied(level));
        self.0 &= LIGHT_LEVEL_MAX_VALUE;
        self.0 |= level << LIGHT_LEVEL_BITS;
    }

    #[inline]
    pub fn set_sky_saturate(&mut self, mut level: u8) {
        level = level.min(LIGHT_LEVEL_MAX_VALUE);
        self.0 &= LIGHT_LEVEL_MAX_VALUE;
        self.0 |= level << LIGHT_LEVEL_BITS;
    }

    #[inline]
    pub fn get_sky(&self) -> u8 {
        self.0 >> LIGHT_LEVEL_BITS
    }

    #[inline]
    pub fn to_u8(&self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct LightNode {
    pub x: i8,
    pub y: i16,
    pub z: i8,
    pub level: u8,
}

impl LightNode {
    pub fn new(x: i8, y: i16, z: i8, level: u8) -> Self {
        Self { x, y, z, level }
    }
}
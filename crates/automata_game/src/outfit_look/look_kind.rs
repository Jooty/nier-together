pub const LOOK_KIND_COUNT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LookKind {
    Outfit,
    Accessory,
    Hair,
    PodSkin,
    GunSkin,
    ClothState,
}

impl LookKind {
    pub const ALL: [Self; LOOK_KIND_COUNT] = [
        Self::Outfit,
        Self::Accessory,
        Self::Hair,
        Self::PodSkin,
        Self::GunSkin,
        Self::ClothState,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Outfit => 0,
            Self::Accessory => 1,
            Self::Hair => 2,
            Self::PodSkin => 3,
            Self::GunSkin => 4,
            Self::ClothState => 5,
        }
    }

    pub const fn heading(self) -> &'static str {
        match self {
            Self::Outfit => "outfit",
            Self::Accessory => "head accessory",
            Self::Hair => "hair colour",
            Self::PodSkin => "pod skin",
            Self::GunSkin => "gun skin",
            Self::ClothState => "cloth state",
        }
    }
}

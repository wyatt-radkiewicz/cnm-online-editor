#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, num_derive::FromPrimitive, num_derive::ToPrimitive, PartialEq, Eq, strum::EnumIter, Default, strum::Display)]
pub enum ItemType {
    #[default]
    Shotgun,
    Knife,
    Apple,
    Cake,
    StrengthPotion,
    SpeedPotion,
    JumpPotion,
    Sword,
    HealthPotion,
    Sniper,
    Money50,
    Money100,
    Money500,
    Cheeseburger,
    GoldenAxe,
    UnboundWand,
    FireWand,
    IceWand,
    AirWand,
    LightningWand,
    GoldenShotgun,
    LaserRifle,
    RocketLauncher,
    FirePotion,
    Minigun,
    MegaPotion,
    UltraMegaPotion,
    Awp,
    Flamethrower,
    PoisionusStrengthPotion,
    PoisionusSpeedPotion,
    PoisionusJumpPotion,
    Beastchurger,
    UltraSword,
    HeavyHammer,
    FissionGun,
    KeyRed,
    KeyGreen,
    KeyBlue,
}

impl ItemType {
    pub fn from_item_id(id: u32) -> Option<Self> {
        if id == 0 {
            return None;
        } else {
            num_traits::FromPrimitive::from_u32(id - 1)
        }
    }

    pub fn get_item_id(&self) -> u32 {
        num_traits::ToPrimitive::to_u32(self).unwrap_or(0) + 1
    }
}

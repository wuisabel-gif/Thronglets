pub type Rgb = (u8, u8, u8);

#[derive(Clone, Copy)]
pub struct Palette {
    pub name: &'static str,
    pub grass: [Rgb; 4],
    pub grass_path: [Rgb; 2],
    pub water: Rgb,
    pub water_light: Rgb,
    pub rock: Rgb,
    pub rock_dark: Rgb,
    pub tree_canopy: Rgb,
    pub tree_light: Rgb,
    pub tree_trunk: Rgb,
    pub bush: Rgb,
    pub berry: Rgb,
    pub pellet: Rgb,
    pub body: Rgb,
    pub body_dark: Rgb,
    pub body_light: Rgb,
    pub face: Rgb,
    pub feet: Rgb,
    pub shadow: Rgb,
    pub egg: Rgb,
    pub egg_spot: Rgb,
    pub faded: Rgb,
    pub zzz: Rgb,
    pub chirp: Rgb,
    pub cursor: Rgb,
    pub panel_bg: Rgb,
    pub panel_bg_2: Rgb,
    pub badge_bg: Rgb,
}

pub const THEMES: &[Palette] = &[VERDANT, DUSK, TIDEPOOL, AMBER];

pub fn by_name(name: &str) -> Option<&'static Palette> {
    THEMES.iter().find(|theme| theme.name == name)
}

pub fn default() -> &'static Palette {
    &VERDANT
}

pub fn next(current: &Palette) -> &'static Palette {
    let idx = THEMES
        .iter()
        .position(|theme| theme.name == current.name)
        .unwrap_or(0);
    &THEMES[(idx + 1) % THEMES.len()]
}

pub fn names() -> String {
    THEMES
        .iter()
        .map(|theme| theme.name)
        .collect::<Vec<_>>()
        .join(", ")
}

const VERDANT: Palette = Palette {
    name: "verdant",
    grass: [(58, 154, 96), (66, 171, 107), (49, 137, 89), (76, 184, 116)],
    grass_path: [(71, 146, 96), (80, 159, 105)],
    water: (0, 89, 172),
    water_light: (20, 130, 211),
    rock: (105, 143, 142),
    rock_dark: (63, 95, 110),
    tree_canopy: (18, 119, 76),
    tree_light: (33, 160, 96),
    tree_trunk: (83, 92, 70),
    bush: (37, 137, 76),
    berry: (236, 80, 112),
    pellet: (250, 220, 104),
    body: (246, 226, 62),
    body_dark: (206, 178, 34),
    body_light: (255, 242, 106),
    face: (72, 95, 85),
    feet: (36, 142, 186),
    shadow: (35, 93, 70),
    egg: (246, 226, 150),
    egg_spot: (215, 176, 70),
    faded: (108, 112, 116),
    zzz: (220, 220, 235),
    chirp: (255, 214, 120),
    cursor: (255, 255, 255),
    panel_bg: (23, 45, 39),
    panel_bg_2: (31, 61, 50),
    badge_bg: (166, 194, 93),
};

const DUSK: Palette = Palette {
    name: "dusk",
    grass: [(73, 82, 118), (82, 92, 132), (59, 70, 103), (93, 104, 145)],
    grass_path: [(96, 94, 132), (111, 102, 145)],
    water: (36, 63, 146),
    water_light: (68, 91, 184),
    rock: (113, 119, 151),
    rock_dark: (76, 80, 111),
    tree_canopy: (42, 70, 94),
    tree_light: (62, 100, 124),
    tree_trunk: (83, 78, 93),
    bush: (54, 88, 100),
    berry: (236, 104, 150),
    pellet: (245, 198, 102),
    body: (255, 211, 94),
    body_dark: (207, 153, 66),
    body_light: (255, 235, 139),
    face: (72, 79, 99),
    feet: (73, 174, 198),
    shadow: (43, 55, 78),
    egg: (238, 215, 154),
    egg_spot: (185, 134, 82),
    faded: (111, 113, 129),
    zzz: (224, 220, 248),
    chirp: (255, 217, 128),
    cursor: (255, 255, 255),
    panel_bg: (33, 37, 61),
    panel_bg_2: (42, 48, 78),
    badge_bg: (184, 150, 102),
};

const TIDEPOOL: Palette = Palette {
    name: "tidepool",
    grass: [
        (42, 141, 132),
        (50, 161, 148),
        (34, 120, 117),
        (62, 176, 156),
    ],
    grass_path: [(53, 139, 130), (68, 153, 139)],
    water: (0, 109, 158),
    water_light: (32, 157, 204),
    rock: (111, 153, 154),
    rock_dark: (65, 105, 119),
    tree_canopy: (20, 105, 98),
    tree_light: (34, 143, 126),
    tree_trunk: (73, 93, 82),
    bush: (36, 124, 103),
    berry: (247, 98, 119),
    pellet: (248, 220, 117),
    body: (249, 231, 78),
    body_dark: (204, 178, 39),
    body_light: (255, 244, 125),
    face: (54, 91, 88),
    feet: (33, 128, 196),
    shadow: (30, 85, 86),
    egg: (239, 226, 159),
    egg_spot: (199, 169, 86),
    faded: (103, 120, 123),
    zzz: (214, 238, 244),
    chirp: (255, 225, 130),
    cursor: (255, 255, 255),
    panel_bg: (20, 52, 54),
    panel_bg_2: (25, 70, 68),
    badge_bg: (126, 193, 144),
};

const AMBER: Palette = Palette {
    name: "amber",
    grass: [
        (120, 132, 74),
        (138, 150, 82),
        (102, 113, 65),
        (156, 164, 91),
    ],
    grass_path: [(143, 131, 82), (163, 145, 92)],
    water: (30, 93, 144),
    water_light: (58, 132, 181),
    rock: (142, 130, 106),
    rock_dark: (103, 90, 78),
    tree_canopy: (80, 102, 54),
    tree_light: (111, 132, 64),
    tree_trunk: (103, 76, 55),
    bush: (91, 117, 55),
    berry: (208, 76, 86),
    pellet: (255, 222, 116),
    body: (255, 224, 70),
    body_dark: (203, 154, 37),
    body_light: (255, 243, 126),
    face: (82, 80, 61),
    feet: (38, 125, 176),
    shadow: (82, 91, 54),
    egg: (242, 220, 145),
    egg_spot: (190, 137, 70),
    faded: (119, 114, 101),
    zzz: (233, 228, 206),
    chirp: (255, 223, 116),
    cursor: (255, 255, 255),
    panel_bg: (57, 45, 30),
    panel_bg_2: (76, 61, 39),
    badge_bg: (195, 164, 78),
};

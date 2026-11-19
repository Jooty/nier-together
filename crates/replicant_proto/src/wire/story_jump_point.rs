#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryJumpPoint {
    VillageAfterPrologue,
    LibraryPopolaTalk,
    VillageBeforeNextDay,
    ShrineEntrance,
    ShrineTop,
    ShrineBoss2,
    AerieBoss3,
    JunkHeapB2,
    JunkHeapBoss5,
    AerieBoss6,
    AerieAfterBoss6,
    DesertAfterFacade,
    TempleEntrance,
    TempleBoss7,
    VillageAfterTemple,
    ForestArrival,
    SouthPlainsRedBag,
    ManorFront,
    ManorEmilFollows,
    ManorBoss8,
    VillageBoss9,
    LibraryBoss10,
    LibraryBoss11,
    Part2Start,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoryJumpRow {
    pub point: StoryJumpPoint,
    pub id: u16,
    pub code: &'static str,
    pub name: &'static str,
    pub former_codes: &'static [&'static str],
}

pub const STORY_JUMP_ROWS: [StoryJumpRow; 24] = [
    StoryJumpRow {
        point: StoryJumpPoint::VillageAfterPrologue,
        id: 30,
        code: "VillageAfterPrologue",
        name: "Village, after prologue",
        former_codes: &["AfterPrologue"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::LibraryPopolaTalk,
        id: 40,
        code: "LibraryPopolaTalk",
        name: "Library, Popola talk",
        former_codes: &["AtPopolaTalk"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::VillageBeforeNextDay,
        id: 60,
        code: "VillageBeforeNextDay",
        name: "Village, before next day",
        former_codes: &["BeforeNextDay"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::ShrineEntrance,
        id: 70,
        code: "ShrineEntrance",
        name: "Shrine, entrance",
        former_codes: &["AtShrineEntrance"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::ShrineTop,
        id: 80,
        code: "ShrineTop",
        name: "Shrine, top",
        former_codes: &["AtShrineTop"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::ShrineBoss2,
        id: 100,
        code: "ShrineBoss2",
        name: "Shrine, boss 2",
        former_codes: &["AtShrineBoss"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::AerieBoss3,
        id: 120,
        code: "AerieBoss3",
        name: "Aerie, boss 3",
        former_codes: &["BossFight3"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::JunkHeapB2,
        id: 180,
        code: "JunkHeapB2",
        name: "Junk Heap, floor B2",
        former_codes: &["JunkHeapFloorB2"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::JunkHeapBoss5,
        id: 190,
        code: "JunkHeapBoss5",
        name: "Junk Heap, boss 5",
        former_codes: &["JunkHeapBossFight5"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::AerieBoss6,
        id: 200,
        code: "AerieBoss6",
        name: "Aerie, boss 6",
        former_codes: &["BossFight6"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::AerieAfterBoss6,
        id: 210,
        code: "AerieAfterBoss6",
        name: "Aerie, after boss 6",
        former_codes: &["AfterBossFight6"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::DesertAfterFacade,
        id: 240,
        code: "DesertAfterFacade",
        name: "Desert, after Facade",
        former_codes: &["AfterFacade"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::TempleEntrance,
        id: 250,
        code: "TempleEntrance",
        name: "Temple, entrance",
        former_codes: &["TempleEntrance"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::TempleBoss7,
        id: 280,
        code: "TempleBoss7",
        name: "Temple, boss 7",
        former_codes: &["TempleBeforeBoss"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::VillageAfterTemple,
        id: 290,
        code: "VillageAfterTemple",
        name: "Village, after temple",
        former_codes: &["VillageBackFromTemple"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::ForestArrival,
        id: 310,
        code: "ForestArrival",
        name: "Forest, arrival",
        former_codes: &["ForestInside"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::SouthPlainsRedBag,
        id: 340,
        code: "SouthPlainsRedBag",
        name: "South plains, red bag",
        former_codes: &["SeafrontWithRedBag"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::ManorFront,
        id: 350,
        code: "ManorFront",
        name: "Manor, front",
        former_codes: &["ManorFront"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::ManorEmilFollows,
        id: 370,
        code: "ManorEmilFollows",
        name: "Manor, Emil follows",
        former_codes: &["ManorEmilFollows"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::ManorBoss8,
        id: 380,
        code: "ManorBoss8",
        name: "Manor, boss 8",
        former_codes: &["ManorBeforeBossFight8"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::VillageBoss9,
        id: 400,
        code: "VillageBoss9",
        name: "Village, boss 9",
        former_codes: &["BossFight9"],
    },
    StoryJumpRow {
        point: StoryJumpPoint::LibraryBoss10,
        id: 410,
        code: "LibraryBoss10",
        name: "Library, boss 10",
        former_codes: &[],
    },
    StoryJumpRow {
        point: StoryJumpPoint::LibraryBoss11,
        id: 420,
        code: "LibraryBoss11",
        name: "Library, boss 11",
        former_codes: &[],
    },
    StoryJumpRow {
        point: StoryJumpPoint::Part2Start,
        id: 430,
        code: "Part2Start",
        name: "Part 2, start",
        former_codes: &[],
    },
];

impl StoryJumpPoint {
    pub fn row(self) -> &'static StoryJumpRow {
        STORY_JUMP_ROWS
            .iter()
            .find(|row| row.point == self)
            .expect("every point has a row")
    }

    pub fn id(self) -> u16 {
        self.row().id
    }

    pub fn code(self) -> &'static str {
        self.row().code
    }

    pub fn name(self) -> &'static str {
        self.row().name
    }

    pub fn from_id(id: u16) -> Option<Self> {
        STORY_JUMP_ROWS
            .iter()
            .find(|row| row.id == id)
            .map(|row| row.point)
    }

    pub fn from_code(code: &str) -> Option<Self> {
        STORY_JUMP_ROWS
            .iter()
            .find(|row| row.code == code || row.former_codes.contains(&code))
            .map(|row| row.point)
    }

    pub fn in_id_order() -> impl Iterator<Item = Self> {
        STORY_JUMP_ROWS.iter().map(|row| row.point)
    }
}

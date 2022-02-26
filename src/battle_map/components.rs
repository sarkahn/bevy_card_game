use bevy::prelude::*;
use bevy_ascii_terminal::Point2d;

#[derive(Component)]
pub struct Selectable;

#[derive(Default, Copy, Clone, Debug, Component)]
pub struct MapPosition {
    pub xy: IVec2,
}

impl Point2d for MapPosition {
    fn x(&self) -> i32 {
        self.xy.x
    }

    fn y(&self) -> i32 {
        self.xy.y
    }

    fn xy(&self) -> IVec2 {
        self.xy
    }
}

impl From<IVec2> for MapPosition {
    fn from(p: IVec2) -> Self {
        Self { xy: p }
    }
}

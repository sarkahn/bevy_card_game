use bevy::{math::Vec3Swizzles, prelude::*};

pub trait GridPosition {
    fn grid_pos(&self) -> IVec2;
    fn from_grid_pos(&self, xy: impl GridPosition) -> Self;
}

impl GridPosition for Transform {
    fn grid_pos(&self) -> IVec2 {
        let p = self.translation.xy().floor() + Vec2::new(0.5, 0.5);
        p.as_ivec2()
    }

    fn from_grid_pos(&self, xy: impl GridPosition) -> Self {
        let xy = xy.grid_pos();
        let mut new = self.clone();
        new.translation.x = xy.x as f32;
        new.translation.y = xy.y as f32;
        new
    }
}

impl GridPosition for IVec2 {
    fn grid_pos(&self) -> IVec2 {
        *self
    }

    fn from_grid_pos(&self, xy: impl GridPosition) -> Self {
        xy.grid_pos()
    }
}

impl GridPosition for [i32; 2] {
    fn grid_pos(&self) -> IVec2 {
        IVec2::from(*self)
    }

    fn from_grid_pos(&self, xy: impl GridPosition) -> Self {
        xy.grid_pos().into()
    }
}

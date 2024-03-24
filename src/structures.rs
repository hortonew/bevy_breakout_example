use bevy::prelude::*;

#[derive(Component)]
pub struct Paddle;

#[derive(Component)]
pub struct Ball;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Collider;

#[derive(Event, Default)]
pub struct CollisionEvent;

#[derive(Component)]
pub struct Brick;

#[derive(Resource)]
pub struct CollisionSound(pub Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
pub struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    pub sprite_bundle: SpriteBundle,
    pub collider: Collider,
}

// This resource tracks the game's score
#[derive(Resource)]
pub struct Scoreboard {
    pub score: usize,
}

#[derive(Component)]
pub struct ScoreboardUi;

/// Which side of the arena is this wall located on?
pub enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

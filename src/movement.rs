use crate::settings::{
    BALL_DIAMETER, LEFT_WALL, PADDLE_PADDING, PADDLE_SIZE, PADDLE_SPEED, RIGHT_WALL, WALL_THICKNESS,
};
use crate::structures::{
    Ball, Brick, Collider, Collision, CollisionEvent, CollisionSound, Paddle, Scoreboard, Velocity,
};
use bevy::{
    input::touch::Touches,
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
};

pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

pub fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_seconds();

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
}

// function that moves paddle using touch input controls
pub fn move_paddle_with_touch(
    touches: Res<Touches>,
    mut paddle: Query<&mut Transform, With<Paddle>>,
) {
    let mut paddle_transform = paddle.single_mut();
    for touch in touches.iter() {
        paddle_transform.translation.x = touch.position().x - PADDLE_SIZE.x / 2.0;
    }
}

pub fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    // check collision with walls
    for (collider_entity, transform, maybe_brick) in &collider_query {
        let collision = collide_with_side(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.),
            Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.,
            ),
        );

        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();

            // Bricks should be despawned and increment the scoreboard on collision
            if maybe_brick.is_some() {
                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
            }

            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

pub fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !collision_events.is_empty() {
        // This prevents events staying active on the next frame.
        collision_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

// Returns `Some` if `ball` collides with `wall`. The returned `Collision` is the
// side of `wall` that `ball` hit.
pub fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

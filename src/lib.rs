use bevy::asset::AssetMetaCheck;
use bevy::input::common_conditions::input_toggle_active;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::WindowResized};
// use bevy_embedded_assets::EmbeddedAssetPlugin;
mod implementations;
mod movement;
mod settings;
mod stepping;
mod structures;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use settings::{
    BACKGROUND_COLOR, BALL_COLOR, BALL_DIAMETER, BALL_SPEED, BALL_STARTING_POSITION, BRICK_COLOR,
    GAP_BETWEEN_BRICKS, GAP_BETWEEN_BRICKS_AND_CEILING, GAP_BETWEEN_BRICKS_AND_SIDES,
    GAP_BETWEEN_PADDLE_AND_BRICKS, GAP_BETWEEN_PADDLE_AND_FLOOR, INITIAL_BALL_DIRECTION,
    PADDLE_COLOR, PADDLE_SIZE, SCOREBOARD_FONT_SIZE, SCOREBOARD_TEXT_PADDING, SCORE_COLOR,
    TEXT_COLOR,
};
use structures::{
    Ball, Brick, Collider, CollisionEvent, CollisionSound, Paddle, Scoreboard, ScoreboardUi,
    Velocity, WallBundle, WallLocation,
};

#[bevy_main]
fn main() {
    run_game();
}

pub fn run_game() {
    App::new()
        .insert_resource(AssetMetaCheck::Never) // set for not generating .meta files for itch.io
        .insert_resource(WallPositions {
            left_wall: -450.0,
            right_wall: 450.0,
            bottom_wall: -300.0,
            top_wall: 300.0,
        })
        // .add_plugins((EmbeddedAssetPlugin::default(), DefaultPlugins))
        .add_plugins(DefaultPlugins)
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     primary_window: Some(Window {
        //         window_level: bevy::window::WindowLevel::AlwaysOnTop,
        //         // make the window auto resize to fit the screen it's in
        //         mode: bevy::window::WindowMode::BorderlessFullscreen,
        //         ..default()
        //     }),
        //     ..default()
        // }))
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::F1)))
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        .add_systems(Update, update_wall_positions)
        .add_systems(
            FixedUpdate,
            (
                movement::apply_velocity,
                movement::move_paddle,
                movement::move_paddle_with_touch,
                movement::check_for_collisions,
                movement::play_collision_sound,
                //update_brick_size_based_on_window_size,
            )
                .chain(),
        )
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .run();
}

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    wall_positions: Res<WallPositions>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    let paddle_y = wall_positions.bottom_wall + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Collider,
    ));

    // Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(BALL_COLOR),
            transform: Transform::from_translation(BALL_STARTING_POSITION)
                .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // Scoreboard
    commands.spawn((
        ScoreboardUi,
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // Bricks
    let total_width_of_bricks =
        (wall_positions.right_wall - wall_positions.left_wall) - 2.0 * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks =
        wall_positions.top_wall - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // set a brick size that takes into account the size of the window
    let brick_size = Vec2::new(
        total_width_of_bricks / 10.0 - GAP_BETWEEN_BRICKS,
        total_height_of_bricks / 10.0 - GAP_BETWEEN_BRICKS,
    );

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_bricks / (brick_size.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (brick_size.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_bricks = (wall_positions.left_wall + wall_positions.right_wall) / 2.0;
    let left_edge_of_bricks = center_of_bricks
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * brick_size.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_bricks + brick_size.x / 2.;
    let offset_y = bottom_edge_of_bricks + brick_size.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (brick_size.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (brick_size.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: BRICK_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        translation: brick_position.extend(0.0),
                        scale: Vec3::new(brick_size.x, brick_size.y, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Brick,
                Collider,
            ));
        }
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreboardUi>>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

#[derive(Resource)]
struct WallPositions {
    left_wall: f32,
    right_wall: f32,
    bottom_wall: f32,
    top_wall: f32, // If you need a top wall position as well
}

fn update_wall_positions(
    mut resize_reader: EventReader<WindowResized>,
    mut wall_positions: ResMut<WallPositions>,
) {
    for e in resize_reader.read() {
        wall_positions.left_wall = -e.width / 2.0;
        wall_positions.right_wall = e.width / 2.0;
        wall_positions.bottom_wall = -e.height / 2.0;
        wall_positions.top_wall = e.height / 2.0;
    }
    // print WallPosition
    println!(
        "WallPositions: left_wall: {}, right_wall: {}, bottom_wall: {}, top_wall: {}",
        wall_positions.left_wall,
        wall_positions.right_wall,
        wall_positions.bottom_wall,
        wall_positions.top_wall
    );
}

fn update_brick_size_based_on_window_size(
    mut query: Query<(Entity, &mut Transform), With<Brick>>,
    wall_positions: Res<WallPositions>,
) {
    let total_width_of_bricks =
        (wall_positions.right_wall - wall_positions.left_wall) - 2.0 * GAP_BETWEEN_BRICKS_AND_SIDES;
    let total_height_of_bricks = wall_positions.top_wall
        - wall_positions.bottom_wall
        - GAP_BETWEEN_BRICKS_AND_CEILING
        - GAP_BETWEEN_PADDLE_AND_BRICKS;

    let brick_size = Vec2::new(
        total_width_of_bricks / 10.0 - GAP_BETWEEN_BRICKS,
        total_height_of_bricks / 10.0 - GAP_BETWEEN_BRICKS,
    );

    for (_, mut transform) in query.iter_mut() {
        transform.scale = brick_size.extend(1.0);
    }
}

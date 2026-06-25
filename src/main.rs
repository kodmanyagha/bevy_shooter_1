mod utils;

use bevy::{prelude::*, window::PrimaryWindow};
use rand::RngExt;

use crate::utils::logger::init_tracing_subscriber;

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

#[derive(Component)]
struct Cat;

#[derive(Component)]
struct Fading {
    timer: Timer,
}

#[derive(Component)]
struct Respawning {
    timer: Timer,
}

#[derive(Component)]
struct ScoreText;

#[derive(Resource)]
struct FireRate(Timer);

#[derive(Resource)]
struct Score(u32);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player { speed: 90_f32 },
        Sprite::from_image(asset_server.load("bevy_bird.png")),
    ));
}

fn random_cat_pos() -> Vec3 {
    let mut rng = rand::rng();
    Vec3::new(
        rng.random_range(-400.0_f32..400.0_f32),
        rng.random_range(-300.0_f32..300.0_f32),
        0.0,
    )
}

fn spawn_cat(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Cat,
        Sprite::from_image(asset_server.load("cat.png")),
        Transform::from_translation(random_cat_pos()),
    ));
}

fn spawn_score_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: FontSize::Px(32.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));
}

fn move_player(
    mut query: Query<(&mut Transform, &mut Player)>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, player) in &mut query {
        if input.pressed(KeyCode::KeyW) && input.pressed(KeyCode::KeyD) {
            transform.translation.y += player.speed * time.delta_secs();
            transform.translation.x += player.speed * time.delta_secs();
        } else if input.pressed(KeyCode::KeyW) && input.pressed(KeyCode::KeyA) {
            transform.translation.y += player.speed * time.delta_secs();
            transform.translation.x -= player.speed * time.delta_secs();
        } else if input.pressed(KeyCode::KeyS) && input.pressed(KeyCode::KeyD) {
            transform.translation.y -= player.speed * time.delta_secs();
            transform.translation.x += player.speed * time.delta_secs();
        } else if input.pressed(KeyCode::KeyS) && input.pressed(KeyCode::KeyA) {
            transform.translation.y -= player.speed * time.delta_secs();
            transform.translation.x -= player.speed * time.delta_secs();
        } else if input.pressed(KeyCode::KeyW) {
            transform.translation.y += player.speed * time.delta_secs();
        } else if input.pressed(KeyCode::KeyS) {
            transform.translation.y -= player.speed * time.delta_secs();
        } else if input.pressed(KeyCode::KeyA) {
            transform.translation.x -= player.speed * time.delta_secs();
        } else if input.pressed(KeyCode::KeyD) {
            transform.translation.x += player.speed * time.delta_secs();
        }
    }
}

fn aim(
    mut query: Query<(&mut Transform, &Player)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    for (mut player_transform, mut _player) in &mut query {
        let Ok(q_window) = q_window.single() else {
            break;
        };
        let Ok((camera, camera_transform)) = camera.single() else {
            break;
        };

        if let Some(cursor_position) = q_window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        {
            let player_direction = player_transform.local_x().truncate();
            let cursor_direction = cursor_position - player_transform.translation.truncate();
            let angle = player_direction.angle_to(cursor_direction);
            player_transform.rotate_z(angle);
        }
    }
}

fn shoot(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    asset_server: Res<AssetServer>,
    player: Query<&Transform, With<Player>>,
    time: Res<Time>,
    mut timer: ResMut<FireRate>,
) {
    let mut player_transform: &Transform = &Transform::default();
    for transform in &player {
        player_transform = transform;
    }

    let player_pos = player_transform.translation;

    timer.0.tick(time.delta());

    let just_clicked = buttons.just_pressed(MouseButton::Left);
    let auto_fire = buttons.pressed(MouseButton::Left) && timer.0.just_finished();

    if just_clicked {
        timer.0.reset();
    }

    if just_clicked || auto_fire {
        let mut target_transform = Transform::from_xyz(player_pos.x, player_pos.y, 0_f32);
        target_transform.rotation = player_transform.rotation;
        target_transform.scale = Vec3::new(0.5_f32, 0.5_f32, 0.5_f32);

        commands.spawn((
            target_transform,
            Bullet { speed: 1_000_f32 },
            Sprite::from_image(asset_server.load("bullet.png")),
        ));
    }
}

fn move_bullet(mut query: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in &mut query {
        let direction = transform.local_x();
        transform.translation += direction * bullet.speed * time.delta_secs();
    }
}

fn bullet_cat_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    mut cats: Query<(Entity, &Transform, &mut Sprite), (With<Cat>, Without<Fading>, Without<Respawning>)>,
    mut score: ResMut<Score>,
) {
    for (bullet_entity, bullet_transform) in &bullets {
        for (cat_entity, cat_transform, _sprite) in &mut cats {
            let dist = bullet_transform
                .translation
                .distance(cat_transform.translation);
            if dist < 48.0 {
                commands.entity(bullet_entity).despawn();
                score.0 += 1;
                commands.entity(cat_entity).insert(Fading {
                    timer: Timer::from_seconds(0.6, TimerMode::Once),
                });
            }
        }
    }
}

fn fade_cat(
    mut commands: Commands,
    time: Res<Time>,
    mut cats: Query<(Entity, &mut Sprite, &mut Fading), With<Cat>>,
) {
    for (entity, mut sprite, mut fading) in &mut cats {
        fading.timer.tick(time.delta());
        let alpha = 1.0 - fading.timer.fraction();
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);

        if fading.timer.just_finished() {
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.0);
            commands.entity(entity).remove::<Fading>();
            commands.entity(entity).insert(Respawning {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            });
        }
    }
}

fn respawn_cat(
    mut commands: Commands,
    time: Res<Time>,
    mut cats: Query<(Entity, &mut Sprite, &mut Transform, &mut Respawning), With<Cat>>,
) {
    for (entity, mut sprite, mut transform, mut respawning) in &mut cats {
        respawning.timer.tick(time.delta());
        if respawning.timer.just_finished() {
            transform.translation = random_cat_pos();
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 1.0);
            commands.entity(entity).remove::<Respawning>();
        }
    }
}

fn update_score_ui(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        for mut text in &mut query {
            **text = format!("Score: {}", score.0);
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    dotenvy::dotenv().expect(".env file not found.");
    init_tracing_subscriber();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .insert_resource(FireRate(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert_resource(Score(0))
        .add_systems(Startup, (setup, spawn_player, spawn_cat, spawn_score_ui))
        .add_systems(
            Update,
            (
                move_player,
                aim,
                shoot,
                move_bullet,
                bullet_cat_collision,
                fade_cat,
                respawn_cat,
                update_score_ui,
            ),
        )
        .run();
}

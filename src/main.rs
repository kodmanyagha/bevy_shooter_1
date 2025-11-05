use std::time::UNIX_EPOCH;

use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

#[derive(Resource)]
struct FireRate(Timer);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player { speed: 90_f32 },
        Sprite::from_image(asset_server.load("bevy_bird.png")),
    ));
}

fn move_player(
    mut query: Query<(&mut Transform, &mut Player)>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, player) in &mut query {
        println!(
            "Time: {} , x: {}",
            UNIX_EPOCH.elapsed().unwrap().as_secs(),
            transform.translation.x
        );

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

        // if transform.translation.x >= 500_f32 {
        // transform.translation.x = 0_f32;
        // } else {
        // transform.translation.x += player.speed * time.delta_secs();
        // }
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

        if let Some(cursor_pos) = q_window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        {
            println!("Cursor is in the game main window, pos: {:?}", cursor_pos);

            let player_dir = player_transform.local_x().truncate();
            let cursor_dir = cursor_pos - player_transform.translation.truncate();
            let angle = player_dir.angle_to(cursor_dir);
            player_transform.rotate_z(angle);
        } else {
            //println!("Cursor is not in the game main window.");
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

    let mut target_transform = Transform::from_xyz(player_pos.x, player_pos.y, 0_f32);
    target_transform.rotation = player_transform.rotation;
    target_transform.scale = Vec3::new(0.5_f32, 0.5_f32, 0.5_f32);

    // println!(
    //     "Shoot fn invoked, mouse: {:?}",
    //     buttons.pressed(MouseButton::Left)
    // );

    if buttons.pressed(MouseButton::Left) && timer.0.tick(time.delta()).is_finished() {
        commands.spawn((
            target_transform,
            Bullet { speed: 1000_f32 },
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.2)))
        .insert_resource(FireRate(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(Startup, (setup, spawn_player))
        .add_systems(Update, (move_player, aim, shoot, move_bullet))
        .run();
}

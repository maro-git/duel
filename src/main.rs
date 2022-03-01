use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use nalgebra::{center, Isometry2, UnitComplex};

use bevy_rapier2d::prelude::*;

use std::time::{Duration, Instant};

use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(setup_world.system())
        .add_system(update_player_movement)
        .add_system(update_ai_movement)
        .add_system(update_gloves_position)
        .add_system(update_gun_position)
        .add_system(update_shotgun_position)
        .add_system(update_bullets)
        .add_system(player_lifes_update_system)
        .add_system(enemy_lifes_update_system)
        .add_system(update_game_state)
        .run();
}

#[derive(Component)]
struct PlayerText;

#[derive(Component)]
struct AiText;

#[derive(Component)]
struct WinLoseText;

#[derive(Component)]
struct GameState {
    state: i32,
}

#[derive(Component)]
struct Player {
    is_jumping: bool,
    jump_duration: Duration,
    end_jump: bool,
    lives: u32,
    hit: bool,
    hit_duration: Duration,
    elapsed: Instant,
    elapsed_hit: Instant,
    can_jump: bool,
}

#[derive(Component)]
struct Ai {
    hit: bool,
    hit_duration: Duration,
    elapsed_hit: Instant,
    lives: u32,
    walk_x: i32,
    jump_y: i32,
    jump: bool,
    elapsed_jump: Instant,
    jump_duration: Duration,
    jump_end: bool,
}

#[derive(Component)]
enum Collider {
    Solid,
    Bullet,
    Gloves,
}

#[derive(Component)]
struct Gloves {
    is_shooting: bool,
    shot_duration: Duration,
    end_shot: bool,
    point_of_rotation: Point<f32>,
    offset: Vec2,
    elapsed: Instant,
}

#[derive(Component)]
struct Gun {
    point_of_rotation: Point<f32>,
    is_active: bool,
    do_shot: bool,
    shot_duration: Duration,
    elapsed: Instant,
}

#[derive(Component)]
struct Shotgun {
    do_shot: bool,
    shot_duration: Duration,
    elapsed: Instant,
    is_active: bool,
}

#[derive(Component)]
struct Bullet {
    lifetime: Duration,
    shoot_dir: Vec2,
    elapsed: Instant,
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    // images
    let boxing_gloves_image: Handle<Image> = asset_server.load("boxing_gloves.png");
    let boxer_image: Handle<Image> = asset_server.load("boxer.png");
    let gun_image: Handle<Image> = asset_server.load("gun.png");
    let shotgun_image: Handle<Image> = asset_server.load("shotgun.png");
    let weapon_specialist_image: Handle<Image> = asset_server.load("weapon_specialist.png");

    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let ground_shape = SpriteBundle {
        transform: Transform {
            scale: Vec3::new(800.0, 50.0, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgb(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    };

    let ground_rigid_body = RigidBodyBundle {
        body_type: RigidBodyTypeComponent(RigidBodyType::Static),
        position: Vec2::new(0.0, -200.0).into(),
        activation: RigidBodyActivation::cannot_sleep().into(),
        ccd: RigidBodyCcd {
            ccd_enabled: true,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };
    /* creating gloves */
    let boxing_gloves = SpriteBundle {
        texture: boxing_gloves_image,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            scale: Vec3::new(64.0, 64.0, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(0.5, 0.5)),
            ..Default::default()
        },
        ..Default::default()
    };

    /* boxing_gloves rigid body. */
    let boxing_gloves_rigid_body = RigidBodyBundle {
        body_type: RigidBodyTypeComponent(RigidBodyType::KinematicVelocityBased),
        position: Vec2::new(0.0, 200.0).into(),
        velocity: RigidBodyVelocity {
            linvel: Vec2::new(0.0, 0.0).into(),
            angvel: 0.0,
        }
        .into(),
        activation: RigidBodyActivation::cannot_sleep().into(),
        ccd: RigidBodyCcd {
            ccd_enabled: true,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    /* create player */
    let player = SpriteBundle {
        texture: boxer_image,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(64.0, 128.0, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(0.64, 1.28)),
            ..Default::default()
        },
        ..Default::default()
    };

    /* Create the bouncing ball. */
    let player_rigid_body = RigidBodyBundle {
        body_type: RigidBodyTypeComponent(RigidBodyType::KinematicVelocityBased),
        position: Vec2::new(0.0, 200.0).into(),
        velocity: RigidBodyVelocity {
            linvel: Vec2::new(0.0, 0.0).into(),
            angvel: 0.0,
        }
        .into(),
        activation: RigidBodyActivation::cannot_sleep().into(),
        ccd: RigidBodyCcd {
            ccd_enabled: true,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    /* create AI */
    let ai = SpriteBundle {
        texture: weapon_specialist_image,
        transform: Transform {
            scale: Vec3::new(64.0, 128.0, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(0.64, 1.28)),
            ..Default::default()
        },
        ..Default::default()
    };

    /* Create the bouncing ball. */
    let ai_rigid_body = RigidBodyBundle {
        body_type: RigidBodyTypeComponent(RigidBodyType::KinematicVelocityBased),
        position: Vec2::new(300.0, 200.0).into(),
        velocity: RigidBodyVelocity {
            linvel: Vec2::new(0.0, 5.0).into(),
            angvel: 0.0,
        }
        .into(),
        activation: RigidBodyActivation::cannot_sleep().into(),
        ccd: RigidBodyCcd {
            ccd_enabled: true,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    let gun = SpriteBundle {
        texture: gun_image,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            scale: Vec3::new(64.0, 64.0, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgba(1.0, 1.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(1.0, 1.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    /* boxing_gloves rigid body. */
    let gun_rigid_body = RigidBodyBundle {
        body_type: RigidBodyTypeComponent(RigidBodyType::KinematicVelocityBased),
        velocity: RigidBodyVelocity {
            linvel: Vec2::new(0.0, 0.0).into(),
            angvel: 0.0,
        }
        .into(),
        activation: RigidBodyActivation::cannot_sleep().into(),
        ccd: RigidBodyCcd {
            ccd_enabled: true,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    let shotgun = SpriteBundle {
        texture: shotgun_image,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            scale: Vec3::new(64.0, 64.0, 0.0),
            ..Default::default()
        },
        sprite: Sprite {
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(1.0, 1.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    /* boxing_gloves rigid body. */
    let shotgun_rigid_body = RigidBodyBundle {
        body_type: RigidBodyTypeComponent(RigidBodyType::KinematicVelocityBased),
        position: Vec2::new(300.0, 200.0).into(),
        velocity: RigidBodyVelocity {
            linvel: Vec2::new(0.0, 0.0).into(),
            angvel: 0.0,
        }
        .into(),
        activation: RigidBodyActivation::cannot_sleep().into(),
        ccd: RigidBodyCcd {
            ccd_enabled: true,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    commands
        .spawn_bundle(player)
        .insert_bundle(player_rigid_body)
        .insert(Collider::Solid)
        .insert(RigidBodyPositionSync::Discrete)
        .insert(Player {
            is_jumping: false,
            jump_duration: Duration::new(0, 0),
            end_jump: true,
            lives: 5,
            hit_duration: Duration::new(0, 0),
            hit: false,
            elapsed: Instant::now(),
            elapsed_hit: Instant::now(),
            can_jump: false,
        });

    commands
        .spawn_bundle(ai)
        .insert_bundle(ai_rigid_body)
        .insert(RigidBodyPositionSync::Discrete)
        .insert(Ai {
            hit: false,
            hit_duration: Duration::new(0, 0),
            lives: 10,
            elapsed_hit: Instant::now(),
            jump_y: 0,
            walk_x: 0,
            jump: false,
            elapsed_jump: Instant::now(),
            jump_duration: Duration::from_millis(300),
            jump_end: true,
        });
    commands
        .spawn_bundle(ground_shape)
        .insert_bundle(ground_rigid_body)
        .insert(Collider::Solid)
        .insert(RigidBodyPositionSync::Discrete);
    commands
        .spawn_bundle(boxing_gloves)
        .insert(Gloves {
            is_shooting: false,
            end_shot: true,
            shot_duration: Duration::new(0, 0),
            point_of_rotation: Point::new(0.0, 0.0),
            offset: Vec2::new(0.0, 0.0),
            elapsed: Instant::now(),
        })
        .insert_bundle(boxing_gloves_rigid_body)
        .insert(Collider::Gloves)
        .insert(RigidBodyPositionSync::Discrete);

    commands
        .spawn_bundle(gun)
        .insert_bundle(gun_rigid_body)
        .insert(Gun {
            point_of_rotation: Point::new(0.0, 0.0),
            is_active: true,
            elapsed: Instant::now(),
            do_shot: false,
            shot_duration: Duration::from_millis(256),
        })
        .insert(RigidBodyPositionSync::Discrete);

    commands
        .spawn_bundle(shotgun)
        .insert_bundle(shotgun_rigid_body)
        .insert(Shotgun {
            do_shot: false,
            shot_duration: Duration::new(5, 0),
            elapsed: Instant::now(),
            is_active: false,
        })
        .insert(RigidBodyPositionSync::Discrete);

    // Text
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "Player Lifes".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            font: asset_server.load("fonts/IceCaps.ttf"),
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            font: asset_server.load("fonts/IceCaps.ttf"),
                            color: Color::GOLD,
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PlayerText);

    // Text
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(20.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "Enemy Lifes".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            font: asset_server.load("fonts/IceCaps.ttf"),
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            font: asset_server.load("fonts/IceCaps.ttf"),
                            color: Color::GOLD,
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(AiText);

    // Text
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    top: Val::Px(20.0),
                    left: Val::Px(20.0),
                },
                ..Default::default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            font: asset_server.load("fonts/IceCaps.ttf"),
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            font: asset_server.load("fonts/IceCaps.ttf"),
                            color: Color::GOLD,
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(WinLoseText);

    commands
        .spawn_bundle(RigidBodyBundle {
            ..Default::default()
        })
        .insert(GameState { state: 0 });
}

fn update_player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut game_state_query: Query<&GameState>,
    mut player_query: Query<
        (
            &mut Player,
            &mut RigidBodyPositionComponent,
            &mut RigidBodyVelocityComponent,
            &mut Sprite,
        ),
        With<Player>,
    >,
    collider_query: Query<(Entity, &Collider, &RigidBodyPositionComponent), Without<Player>>,
    server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (mut player, mut position, mut velocities, mut sprite) in player_query.iter_mut() {
        for game_state in game_state_query.iter_mut() {
            let mut velocity = Vec2::new(velocities.0.linvel.x, velocities.0.linvel.y - 500.0);
            for (entity, collider, transform) in collider_query.iter() {
                if game_state.state == 1 {
                    if player.is_jumping {
                        player.can_jump = false;
                    }
                    if keyboard_input.pressed(KeyCode::A) {
                        velocity.x = -200.0;
                    }
                    if keyboard_input.pressed(KeyCode::D) {
                        velocity.x = 200.0;
                    }

                    if keyboard_input.just_pressed(KeyCode::W) {
                        if player.can_jump {
                            player.jump_duration = Duration::new(1, 0);
                            player.elapsed = Instant::now();

                            player.is_jumping = true;
                        }
                    }

                    let collision = collide(
                        Vec3::new(
                            position.0.position.translation.x,
                            position.0.position.translation.y - 15.0, // y_offset
                            0.0,
                        ),
                        Vec2::new(64.0, 128.0),
                        Vec3::new(
                            transform.0.position.translation.x,
                            transform.0.position.translation.y,
                            0.0,
                        ),
                        if matches!(collider, Collider::Solid) {
                            Vec2::new(700.0, 50.0)
                        } else {
                            Vec2::new(32.0, 32.0)
                        },
                    );

                    if let Some(collision) = collision {
                        if let Collider::Solid = *collider {
                            match collision {
                                Collision::Top => {
                                    velocity.y = 0.0;
                                    player.is_jumping = false;
                                    player.end_jump = false;
                                    player.can_jump = true;
                                }
                                _ => {}
                            }
                        }
                        if let Collider::Bullet = *collider {
                            match collision {
                                _ => {
                                    if !player.hit {
                                        let music: Handle<AudioSource> =
                                            server.load("audio/hit-someting-6037.ogg");
                                        audio.play(music);
                                        player.hit_duration = Duration::new(3, 0);
                                        player.elapsed_hit = Instant::now();
                                    }
                                    player.hit = true;
                                    sprite.color = Color::rgb(1.0, 0.0, 0.0);
                                }
                            }
                        }
                    }
                    if player.is_jumping && !player.end_jump {
                        velocity.y = 500.0;
                        if player.jump_duration.as_secs() <= player.elapsed.elapsed().as_secs() {
                            player.end_jump = true;
                        }
                    }

                    if player.hit {
                        if player.hit_duration.as_secs() <= player.elapsed_hit.elapsed().as_secs() {
                            sprite.color = Color::rgb(1.0, 1.0, 1.0);
                            player.lives -= 1;
                            player.hit = false;
                        }
                    }

                    if player.lives == 0 {
                        position.0.position.translation.x = 0.0;
                        position.0.position.translation.y = 0.0;
                    }

                    if position.0.position.translation.y < -200.0 {
                        position.0.position.translation.x = 0.0;
                        position.0.position.translation.y = 0.0;
                    }

                    velocities.0.linvel = Vec2::new(velocity.x, velocity.y).into();

                    velocities.0.angvel = 0.0;
                }
            }
        }
    }
}

fn update_gloves_position(
    mouse_button: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut gloves_query: Query<
        (
            &mut Gloves,
            &mut RigidBodyPositionComponent,
            &mut RigidBodyVelocityComponent,
            Without<Player>,
        ),
        With<Gloves>,
    >,
    mut camera_query: Query<(&mut Transform, With<Camera>), Without<Gloves>>,
    player_query: Query<&RigidBodyPositionComponent, With<Player>>,
    server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    let mut pos_world = Vec4::new(0.0, 0.0, 0.0, 0.0);
    for (mut gloves, mut position, mut velocities, _) in gloves_query.iter_mut() {
        for player_position in player_query.iter() {
            for mut camera_transform in camera_query.iter_mut() {
                let window = windows.get_primary().unwrap();
                if let Some(pos) = window.cursor_position() {
                    let size = Vec2::new(window.width() as f32, window.height() as f32);
                    let p = pos
                        - (size / 2.0)
                        - Vec2::new(
                            player_position.0.position.translation.x,
                            player_position.0.position.translation.y,
                        );

                    pos_world = camera_transform.0.compute_matrix() * p.extend(0.0).extend(1.0);
                    //println!("{:?}",pos_world);
                }
            }

            gloves.point_of_rotation = Point::new(
                player_position.0.position.translation.x,
                player_position.0.position.translation.y,
            );

            if gloves.is_shooting && !gloves.end_shot {
                velocities.0.linvel.y = pos_world.y;
                velocities.0.linvel.x = pos_world.x;
                if gloves.offset.x == 0.0 {
                    gloves.offset.x += pos_world.x / 100.0;
                };
                if gloves.offset.y == 0.0 {
                    gloves.offset.y += pos_world.y / 100.0;
                }
                if gloves.shot_duration.as_millis() <= gloves.elapsed.elapsed().as_millis() {
                    gloves.end_shot = true;
                }
                gloves.offset.x += gloves.offset.x * 0.07;
                gloves.offset.y += gloves.offset.y * 0.07;
            }

            if gloves.end_shot == true
                && gloves.shot_duration.as_millis() <= gloves.elapsed.elapsed().as_millis()
            {
                gloves.offset.x = 0.0;
                gloves.offset.y = 0.0;
            }

            if gloves.shot_duration.as_millis() <= gloves.elapsed.elapsed().as_millis() {
                position.0.position.translation.x = player_position.0.position.translation.x;
                gloves.is_shooting = false;
                gloves.elapsed = Instant::now();
            }

            position.0.position.translation.x =
                player_position.0.position.translation.x + gloves.offset.x;
            position.0.position.translation.y =
                player_position.0.position.translation.y + gloves.offset.y;

            if mouse_button.pressed(MouseButton::Left) {
                if !gloves.is_shooting {
                    let music: Handle<AudioSource> =
                        server.load("audio/fist-punch-or-kick-7171.ogg");
                    audio.play(music);
                    gloves.elapsed = Instant::now();
                    gloves.shot_duration = Duration::from_millis(400);
                    gloves.is_shooting = true;
                    gloves.end_shot = false;
                }
            }
        }
    }
}

fn update_gun_position(
    mut cmd: Commands,
    mut gloves_query: Query<
        (
            &mut Gun,
            &mut RigidBodyPositionComponent,
            &mut RigidBodyVelocityComponent,
            &mut Sprite,
        ),
        (Without<Ai>, With<Gun>, Without<Player>),
    >,
    game_state_query: Query<&GameState>,
    mut player_query: Query<&RigidBodyPositionComponent, (With<Player>, Without<Ai>)>,
    ai_query: Query<&RigidBodyPositionComponent, (With<Ai>, Without<Player>)>,
    server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (mut gun, mut position, mut velocities, mut sprite) in gloves_query.iter_mut() {
        for ai_position in ai_query.iter() {
            for player_position in player_query.iter() {
                for game_state in game_state_query.iter() {
                    if game_state.state == 1 {
                        let rotation_z = f32::atan2(
                            position.0.position.translation.y
                                - player_position.0.position.translation.y,
                            position.0.position.translation.x
                                - player_position.0.position.translation.x,
                        );
                        position.0.position = Isometry2::rotation(rotation_z);
                        position.0.position.translation.x = ai_position.0.position.translation.x;
                        position.0.position.translation.y =
                            ai_position.0.position.translation.y - 10.0;
                        gun.point_of_rotation = Point::new(0.0, 0.0);

                        if f32::abs(
                            position.0.position.translation.x
                                - player_position.0.position.translation.x,
                        ) >= 400.0
                        {
                            gun.is_active = true;
                        } else {
                            gun.is_active = false;
                        }

                        if gun.is_active {
                            if gun.shot_duration.as_millis() <= gun.elapsed.elapsed().as_millis() {
                                gun.shot_duration = Duration::from_millis(569);
                                gun.elapsed = Instant::now();
                                sprite.color = Color::rgba(1.0, 1.0, 1.0, 1.0);
                                gun.do_shot = true;
                            }
                        } else {
                            sprite.color = Color::rgba(1.0, 1.0, 1.0, 0.0);
                            gun.is_active = false;
                        }

                        if gun.do_shot {
                            let music: Handle<AudioSource> =
                                server.load("audio/9mm-pistol-shot-6349.ogg");
                            audio.play(music);
                            gun.do_shot = false;
                            let bullet = SpriteBundle {
                                texture: server.load("bullet.png"),
                                transform: Transform {
                                    translation: Vec3::new(0.0, 0.0, 3.0),
                                    scale: Vec3::new(32.0, 32.0, 0.0),
                                    ..Default::default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1.0, 1.0, 1.0),
                                    custom_size: Some(Vec2::new(0.5, 0.5)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            let bullet_rigid_body = RigidBodyBundle {
                                body_type: RigidBodyTypeComponent(
                                    RigidBodyType::KinematicVelocityBased,
                                ),
                                position: Vec2::new(
                                    position.0.position.translation.x,
                                    position.0.position.translation.y,
                                )
                                .into(),
                                velocity: RigidBodyVelocity {
                                    linvel: Vec2::new(0.0, 0.0).into(),
                                    angvel: 0.0,
                                }
                                .into(),
                                activation: RigidBodyActivation::cannot_sleep().into(),
                                ccd: RigidBodyCcd {
                                    ccd_enabled: true,
                                    ..Default::default()
                                }
                                .into(),
                                ..Default::default()
                            };

                            cmd.spawn_bundle(bullet)
                                .insert_bundle(bullet_rigid_body)
                                .insert(Collider::Bullet)
                                .insert(Bullet {
                                    lifetime: Duration::new(4, 0),
                                    shoot_dir: Vec2::new(
                                        position.0.position.translation.x
                                            - player_position.position.translation.x,
                                        position.0.position.translation.y
                                            - player_position.position.translation.y,
                                    ),
                                    elapsed: Instant::now(),
                                })
                                .insert(RigidBodyPositionSync::Discrete);
                        }
                    }
                }
            }
        }
    }
}

fn update_shotgun_position(
    mut cmd: Commands,
    mut gloves_query: Query<
        (
            &mut Shotgun,
            &mut RigidBodyPositionComponent,
            &mut RigidBodyVelocityComponent,
            &mut Sprite,
        ),
        (Without<Ai>, With<Shotgun>, Without<Player>),
    >,
    game_state_query: Query<&GameState>,
    player_query: Query<&RigidBodyPositionComponent, (With<Player>, Without<Ai>)>,
    ai_query: Query<&RigidBodyPositionComponent, (With<Ai>, Without<Player>)>,
    server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (mut gun, mut position, mut velocities, mut sprite) in gloves_query.iter_mut() {
        for ai_position in ai_query.iter() {
            for player_position in player_query.iter() {
                for game_state in game_state_query.iter() {
                    if game_state.state == 1 {
                        let rotation_z = f32::atan2(
                            position.0.position.translation.y
                                - player_position.0.position.translation.y,
                            position.0.position.translation.x
                                - player_position.0.position.translation.x,
                        );
                        position.0.position = Isometry2::rotation(rotation_z);
                        position.0.position.translation.x = ai_position.0.position.translation.x;
                        position.0.position.translation.y =
                            ai_position.0.position.translation.y - 10.0;

                        if f32::abs(
                            position.0.position.translation.x
                                - player_position.0.position.translation.x,
                        ) <= 400.0
                        {
                            gun.is_active = true;
                        } else {
                            gun.is_active = false;
                        }

                        if gun.is_active {
                            if gun.shot_duration.as_secs() <= gun.elapsed.elapsed().as_secs() {
                                gun.do_shot = true;
                                gun.shot_duration = Duration::new(5, 0);
                                gun.elapsed = Instant::now();
                            }
                            sprite.color = Color::rgba(1.0, 1.0, 1.0, 1.0);
                        } else {
                            sprite.color = Color::rgba(1.0, 1.0, 1.0, 0.0);
                        }
                        if gun.do_shot {
                            let music: Handle<AudioSource> =
                                server.load("audio/9mm-pistol-shot-6349.ogg");
                            audio.play(music);

                            /* creating gloves */
                            let bullet = SpriteBundle {
                                texture: server.load("bullet.png"),
                                transform: Transform {
                                    translation: Vec3::new(0.0, 0.0, 3.0),
                                    scale: Vec3::new(32.0, 32.0, 0.0),
                                    ..Default::default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1.0, 1.0, 1.0),
                                    custom_size: Some(Vec2::new(0.5, 0.5)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            let bullet_rigid_body = RigidBodyBundle {
                                body_type: RigidBodyTypeComponent(
                                    RigidBodyType::KinematicVelocityBased,
                                ),
                                position: Vec2::new(
                                    position.0.position.translation.x,
                                    position.0.position.translation.y,
                                )
                                .into(),
                                velocity: RigidBodyVelocity {
                                    linvel: Vec2::new(0.0, 0.0).into(),
                                    angvel: 0.0,
                                }
                                .into(),
                                activation: RigidBodyActivation::cannot_sleep().into(),
                                ccd: RigidBodyCcd {
                                    ccd_enabled: true,
                                    ..Default::default()
                                }
                                .into(),
                                ..Default::default()
                            };

                            let bullet2 = SpriteBundle {
                                texture: server.load("bullet.png"),
                                transform: Transform {
                                    translation: Vec3::new(0.0, 0.0, 3.0),
                                    scale: Vec3::new(32.0, 32.0, 0.0),
                                    ..Default::default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1.0, 1.0, 1.0),
                                    custom_size: Some(Vec2::new(0.5, 0.5)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            let bullet2_rigid_body = RigidBodyBundle {
                                body_type: RigidBodyTypeComponent(
                                    RigidBodyType::KinematicVelocityBased,
                                ),
                                position: Vec2::new(
                                    position.0.position.translation.x,
                                    position.0.position.translation.y,
                                )
                                .into(),
                                velocity: RigidBodyVelocity {
                                    linvel: Vec2::new(0.0, 0.0).into(),
                                    angvel: 0.0,
                                }
                                .into(),
                                activation: RigidBodyActivation::cannot_sleep().into(),
                                ccd: RigidBodyCcd {
                                    ccd_enabled: true,
                                    ..Default::default()
                                }
                                .into(),
                                ..Default::default()
                            };

                            let bullet3 = SpriteBundle {
                                texture: server.load("bullet.png"),
                                transform: Transform {
                                    translation: Vec3::new(0.0, 0.0, 3.0),
                                    scale: Vec3::new(32.0, 32.0, 0.0),
                                    ..Default::default()
                                },
                                sprite: Sprite {
                                    color: Color::rgb(1.0, 1.0, 1.0),
                                    custom_size: Some(Vec2::new(0.5, 0.5)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            };

                            let bullet3_rigid_body = RigidBodyBundle {
                                body_type: RigidBodyTypeComponent(
                                    RigidBodyType::KinematicVelocityBased,
                                ),
                                position: Vec2::new(
                                    position.0.position.translation.x,
                                    position.0.position.translation.y,
                                )
                                .into(),
                                velocity: RigidBodyVelocity {
                                    linvel: Vec2::new(0.0, 0.0).into(),
                                    angvel: 0.0,
                                }
                                .into(),
                                activation: RigidBodyActivation::cannot_sleep().into(),
                                ccd: RigidBodyCcd {
                                    ccd_enabled: true,
                                    ..Default::default()
                                }
                                .into(),
                                ..Default::default()
                            };
                            cmd.spawn_bundle(bullet)
                                .insert_bundle(bullet_rigid_body)
                                .insert(Collider::Bullet)
                                .insert(Bullet {
                                    lifetime: Duration::new(4, 0),
                                    shoot_dir: Vec2::new(
                                        position.0.position.translation.x
                                            - player_position.position.translation.x,
                                        position.0.position.translation.y
                                            - player_position.position.translation.y,
                                    ),
                                    elapsed: Instant::now(),
                                })
                                .insert(RigidBodyPositionSync::Discrete);

                            cmd.spawn_bundle(bullet2)
                                .insert_bundle(bullet2_rigid_body)
                                .insert(Collider::Bullet)
                                .insert(Bullet {
                                    lifetime: Duration::new(4, 0),
                                    shoot_dir: Vec2::new(
                                        position.0.position.translation.x
                                            - player_position.position.translation.x,
                                        position.0.position.translation.y
                                            - player_position.position.translation.y * 0.5,
                                    ),
                                    elapsed: Instant::now(),
                                })
                                .insert(RigidBodyPositionSync::Discrete);
                            cmd.spawn_bundle(bullet3)
                                .insert_bundle(bullet3_rigid_body)
                                .insert(Collider::Bullet)
                                .insert(Bullet {
                                    lifetime: Duration::new(4, 0),
                                    shoot_dir: Vec2::new(
                                        position.0.position.translation.x
                                            - player_position.position.translation.x,
                                        position.0.position.translation.y
                                            - player_position.position.translation.y * 2.0,
                                    ),
                                    elapsed: Instant::now(),
                                })
                                .insert(RigidBodyPositionSync::Discrete);
                            gun.do_shot = false;
                        }
                    }
                }
            }
        }
    }
}

fn update_bullets(
    mut cmd: Commands,
    player_query: Query<(&RigidBodyPositionComponent), (With<Player>, Without<Bullet>)>,
    mut bullet_query: Query<
        (Entity, &mut Bullet, &mut RigidBodyPositionComponent),
        (With<Bullet>, Without<Player>),
    >,
) {
    for (Entity, mut class, mut bullet) in bullet_query.iter_mut() {
        for player_position in player_query.iter() {
            bullet.0.position.translation.x += -class.shoot_dir.normalize().x * 3.0;
            bullet.0.position.translation.y += -class.shoot_dir.normalize().y * 3.0;
            if class.lifetime.as_secs() <= class.elapsed.elapsed().as_secs() {
                cmd.entity(Entity).despawn();
            }
        }
    }
}

fn update_ai_movement(
    mut player_query: Query<
        (
            &mut Ai,
            &RigidBodyPositionComponent,
            &mut RigidBodyVelocityComponent,
            &mut Sprite,
        ),
        With<Ai>,
    >,
    game_state_query: Query<&GameState>,
    collider_query: Query<(Entity, &Collider, &RigidBodyPositionComponent), Without<Ai>>,
    gloves_query: Query<
        (&RigidBodyPositionComponent),
        (With<Gloves>, Without<Ai>, Without<Collider>),
    >,
    server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for (mut player, position, mut velocities, mut sprite) in player_query.iter_mut() {
        let mut velocity = Vec2::new(velocities.0.linvel.x, velocities.0.linvel.y - 500.0);
        for (entity, collider, transform) in collider_query.iter() {
            for game_state in game_state_query.iter() {
                if game_state.state == 1 {
                    let action = rand::thread_rng().gen_range(0..2);
                    let action_jump = rand::thread_rng().gen_range(0..800);
                    let collision = collide(
                        Vec3::new(
                            position.0.position.translation.x,
                            position.0.position.translation.y - 15.0, // y_offset
                            0.0,
                        ),
                        Vec2::new(64.0, 128.0),
                        Vec3::new(
                            transform.0.position.translation.x,
                            transform.0.position.translation.y,
                            0.0,
                        ),
                        if matches!(collider, Collider::Solid) {
                            Vec2::new(700.0, 50.0)
                        } else {
                            Vec2::new(32.0, 32.0)
                        },
                    );

                    if let Some(collision) = collision {
                        if let Collider::Solid = *collider {
                            match collision {
                                Collision::Top => {
                                    velocity.y = 0.0;
                                    player.jump_end = false;
                                }
                                _ => {}
                            }
                        }
                        if let Collider::Gloves = *collider {
                            match collision {
                                _ => {
                                    if player.lives > 0 {
                                        if !player.hit {
                                            let music: Handle<AudioSource> =
                                                server.load("audio/hit-someting-6037.ogg");
                                            audio.play(music);
                                            player.hit_duration = Duration::new(3, 0);
                                            player.hit = true;
                                            sprite.color = Color::rgb(1.0, 0.0, 0.0);
                                            player.elapsed_hit = Instant::now();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    player.walk_x = action;
                    player.jump_y = action_jump;
                    if player.walk_x == 1 && position.0.position.translation.x <= 300.0 {
                        velocity.x = 200.0;
                    } else if (player.walk_x == 0 && position.0.position.translation.x >= -500.0) {
                        velocity.x = -200.0;
                    }

                    if player.jump_y == 1 {
                        player.jump = true;
                    }

                    if player.jump == true {
                        if player.jump_duration.as_millis()
                            > player.elapsed_jump.elapsed().as_millis()
                        {
                            if !player.jump_end {
                                velocity.y = 500.0;
                            }
                        } else {
                            player.jump_end = true;
                            player.jump = false;
                        }
                    }

                    if player.jump_end {
                        player.elapsed_jump = Instant::now();
                    }

                    if player.hit && player.lives > 0 {
                        if player.hit_duration.as_secs() <= player.elapsed_hit.elapsed().as_secs() {
                            player.hit = false;
                            sprite.color = Color::rgb(1.0, 1.0, 1.0);
                            player.lives -= 1;
                        }
                    }

                    if player.lives == 0 {
                        sprite.color = Color::rgba(0.0, 0.0, 0.0, 0.0)
                    }

                    velocities.0.linvel = Vec2::new(velocity.x, velocity.y).into();

                    velocities.0.angvel = 0.0;
                }
            }
        }
    }
}

fn player_lifes_update_system(
    mut query: Query<&mut Text, With<PlayerText>>,
    mut player_query: Query<&mut Player>,
) {
    for mut text in query.iter_mut() {
        for mut player in player_query.iter() {
            // Update the value of the second section
            text.sections[1].value = format!(" {}", player.lives);
        }
    }
}
fn enemy_lifes_update_system(
    mut query: Query<&mut Text, With<AiText>>,
    mut player_query: Query<&mut Ai>,
) {
    for mut text in query.iter_mut() {
        for mut player in player_query.iter() {
            // Update the value of the second section
            text.sections[1].value = format!(" {}", player.lives);
        }
    }
}

fn update_game_state(
    mouse_button: Res<Input<MouseButton>>,
    mut game_state_query: Query<&mut GameState>,
    mut player_query: Query<&mut Player>,
    mut Ai: Query<(&mut Ai, &mut Sprite)>,
    mut Winlose_query: Query<&mut Text, With<WinLoseText>>,
) {
    for mut game_state in game_state_query.iter_mut() {
        for mut player in player_query.iter_mut() {
            for (mut ai, mut ai_sprite) in Ai.iter_mut() {
                for mut win_lose_text in Winlose_query.iter_mut() {
                    if game_state.state == 0 {
                        win_lose_text.sections[0].value = format!(" Left click to start");
                        if mouse_button.just_pressed(MouseButton::Left) {
                            game_state.state = 1;
                            win_lose_text.sections[1].value = format!("");
                            win_lose_text.sections[0].value = format!("");
                        }
                    }
                    if player.lives == 0 {
                        game_state.state = 0;
                        player.lives = 3;
                        ai.lives = 10;
                        win_lose_text.sections[1].value = format!(" Enemy Wins!");
                    }
                    if ai.lives == 0 {
                        game_state.state = 0;
                        player.lives = 3;
                        ai.lives = 10;
                        win_lose_text.sections[1].value = format!(" Player Wins!");
                        ai_sprite.color = Color::rgba(1.0, 1.0, 1.0, 1.0)
                    }
                }
            }
        }
    }
}

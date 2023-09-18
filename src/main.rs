use bevy::prelude::{shape::CapsuleUvProfile, *};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_outline::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::{prelude::*, *};
use spooky_camera::{prelude::*, CameraFocus, PrimaryCamera};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_plugins(OutlinePlugin)
        .add_plugins(SpookyCameraPlugin)
        .add_plugins(WorldInspectorPlugin::default())
        .insert_resource(CameraTransform::default())
        .add_event::<NewWidgetEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (tilt_controls, rotate_camera, set_camera_target))
        .add_systems(
            Update,
            (detect_widget_sensors, update_current_widget).chain(),
        )
        .run();
}

#[derive(Component)]
pub struct Player {
    pub height: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player { height: 1.0 }
    }
}

#[derive(Component)]
pub struct Widget;

#[derive(Component)]
pub struct WidgetSensor(pub Entity);

#[derive(Event)]
pub struct NewWidgetEvent {
    pub old_widget: Entity,
    pub new_widget: Entity,
}

#[derive(Resource, Default)]
pub struct CameraTransform(pub Transform);

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Default, Reflect)]
pub enum PlayerAction {
    #[default]
    Tilt,
    CameraPan,
    Jump,
    Spin,
}

#[derive(Bundle)]
pub struct InputListenerBundle {
    input_manager: InputManagerBundle<PlayerAction>,
}

impl InputListenerBundle {
    pub fn input_map() -> InputListenerBundle {
        use PlayerAction::*;

        let input_map = input_map::InputMap::new([
            (DualAxis::right_stick(), CameraPan),
            (DualAxis::left_stick(), Tilt),
        ])
        .insert_multiple([
            (GamepadButtonType::South, Jump),
            (GamepadButtonType::West, Spin),
        ])
        .set_gamepad(Gamepad { id: 0 })
        .build();
        InputListenerBundle {
            input_manager: InputManagerBundle {
                input_map,
                ..Default::default()
            },
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentWidget(pub Option<Entity>);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibilty: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub collider: Collider,
    pub friction: Friction,
    pub gravity_scale: GravityScale,
    pub mass_properties: ColliderMassProperties,
}

impl PlayerBundle {
    pub fn with_height(mut self, height: f32) -> Self {
        self.player = Player { height };
        self.collider = Collider::cuboid(0.5, height * 0.5, 0.5);
        self
    }
}

impl Default for PlayerBundle {
    fn default() -> Self {
        PlayerBundle {
            player: Player::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            gravity_scale: GravityScale(5.0),
            velocity: Velocity::default(),
            mesh: Handle::default(),
            transform: Transform::default(),
            material: Handle::default(),
            global_transform: GlobalTransform::default(),
            visibilty: Visibility::default(),
            computed_visibility: ComputedVisibility::default(),
            mass_properties: ColliderMassProperties::Mass(100.0),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let height: f32 = 1.4;
    let butter_handle = meshes.add(Mesh::from(shape::Box::new(1.0, height, 1.0)));

    commands
        .spawn(Camera3dBundle::default())
        .insert(PrimaryCamera {
            offset: Vec3::new(0.0, 2.5, -10.0),
            ..default()
        });

    commands
        .spawn(
            PlayerBundle {
                mesh: butter_handle.clone(),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                ..default()
            }
            .with_height(height),
        )
        .insert(InputListenerBundle::input_map());

    let blue_mat = materials.add(Color::BLUE.into());

    let start_id = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(10.0, 0.5, 10.0))),
                material: blue_mat.clone(),
                transform: Transform::from_translation(Vec3::Y * -1.0),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            Collider::cuboid(5.0, 0.25, 5.0),
            Friction {
                coefficient: 0.2,
                combine_rule: CoefficientCombineRule::Min,
            },
            Widget,
            OutlineBundle {
                outline: OutlineVolume {
                    visible: true,
                    colour: Color::rgba(0.0, 1.0, 0.0, 1.0),
                    width: 15.0,
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TransformBundle::default(),
                Collider::ball(5.1),
                RigidBody::KinematicPositionBased,
                Sensor,
                WidgetSensor(parent.parent_entity()),
            ));
        })
        .id();

    commands.insert_resource(CurrentWidget(Some(start_id)));

    commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(10.0, 0.5, 10.0))),
                material: blue_mat.clone(),
                transform: Transform::from_translation(Vec3::new(0.0, -1.0, 12.0)),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            Collider::cuboid(5.0, 0.25, 5.0),
            Friction {
                coefficient: 0.2,
                combine_rule: CoefficientCombineRule::Min,
            },
            Widget,
        ))
        .with_children(|parent| {
            parent.spawn((
                TransformBundle::default(),
                Collider::ball(5.1),
                RigidBody::KinematicPositionBased,
                Sensor,
                WidgetSensor(parent.parent_entity()),
            ));
        });
}

fn shrink(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut player_query: Query<(Entity, &mut Player), (With<Collider>, With<Handle<Mesh>>)>,
) {
    if let Ok((entity, mut player)) = player_query.get_single_mut() {
        if player.height > 0.25 {
            let height = player.height - (time.delta_seconds() * 0.05);
            player.height = height;
            commands
                .entity(entity)
                .remove::<Collider>()
                .remove::<Handle<Mesh>>()
                .insert(Collider::cuboid(0.5, height * 0.5, 0.5))
                .insert(meshes.add(Mesh::from(shape::Box::new(1.0, height, 1.0))));
        }
    }
}

fn tilt_controls(
    camera_focus: Res<CameraFocus>,
    current_widget: Res<CurrentWidget>,
    player_query: Query<&ActionState<PlayerAction>, Without<Widget>>,
    mut level_query: Query<(Entity, &mut Transform), With<Widget>>,
) {
    if let Ok(action) = player_query.get_single() {
        if let Some(widget) = current_widget.0 {
            if action.pressed(PlayerAction::Tilt) {
                let axis_pair = action.clamped_axis_pair(PlayerAction::Tilt).unwrap();
                let z_rot = axis_pair.x();
                let x_rot = axis_pair.y();

                let max_rot: f32 = 7.0;

                let forward = camera_focus.forward_flat();
                let right = camera_focus.right_flat();

                let new_rotation = Quat::from_axis_angle(forward, (max_rot * z_rot).to_radians())
                    * Quat::from_axis_angle(right, (max_rot * -x_rot).to_radians());
                for (entity, mut transform) in &mut level_query {
                    if entity == widget {
                        transform.rotation = new_rotation;
                    }
                }
            }
        }
    }
}

fn rotate_camera(
    mut camera_query: Query<&mut PrimaryCamera>,
    player_query: Query<&ActionState<PlayerAction>>,
    time: Res<Time>,
) {
    if let Ok(mut camera) = camera_query.get_single_mut() {
        if let Ok(action) = player_query.get_single() {
            if action.pressed(PlayerAction::CameraPan) {
                let camera_pan_vector = action.axis_pair(PlayerAction::CameraPan).unwrap();

                let y_rot_change = if camera_pan_vector.x() != 0.0 {
                    180.0 * camera_pan_vector.x() * time.delta_seconds()
                } else {
                    0.0
                };
                let x_rot_change = if camera_pan_vector.y() != 0.0 {
                    90.0 * camera_pan_vector.y() * time.delta_seconds()
                } else {
                    0.0
                };
                if x_rot_change != 0.0 {
                    camera.adjust_x_angle(x_rot_change);
                }
                if y_rot_change != 0.0 {
                    camera.adjust_y_angle(-y_rot_change);
                }
            }
        }
    }
}

fn set_camera_target(
    time: Res<Time>,
    mut camera_query: Query<&mut PrimaryCamera>,
    player_query: Query<&Transform, With<Player>>,
) {
    if let Ok(mut camera) = camera_query.get_single_mut() {
        if let Ok(transform) = player_query.get_single() {
            camera.target = camera
                .target
                .lerp(transform.translation, time.delta_seconds() * 20.0);
        }
    }
}

fn detect_widget_sensors(
    current_widget: Res<CurrentWidget>,
    player_query: Query<Entity, With<Player>>,
    widget_query: Query<Entity, (Without<Player>, With<WidgetSensor>)>,
    mut new_widget_events: EventWriter<NewWidgetEvent>,
    rapier_context: Res<RapierContext>,
) {
    if let Ok(player_entity) = player_query.get_single() {
        for widget_sensor_entity in &widget_query {
            if rapier_context.intersection_pair(player_entity, widget_sensor_entity) == Some(true) {
                if let Some(old_entity) = current_widget.0 {
                    if old_entity != widget_sensor_entity {
                        new_widget_events.send(NewWidgetEvent {
                            old_widget: old_entity,
                            new_widget: widget_sensor_entity,
                        });
                    }
                }
            }
        }
    }
}

fn update_current_widget(
    mut commands: Commands,
    mut new_widget_events: EventReader<NewWidgetEvent>,
    mut current_widget: ResMut<CurrentWidget>,
) {
    for event in new_widget_events.iter() {
        commands.entity(event.old_widget).remove::<OutlineBundle>();
        commands.entity(event.new_widget).insert(OutlineBundle {
            outline: OutlineVolume {
                visible: true,
                colour: Color::rgba(0.0, 1.0, 0.0, 1.0),
                width: 10.0,
            },
            ..default()
        });
        current_widget.0 = Some(event.new_widget);
    }
}

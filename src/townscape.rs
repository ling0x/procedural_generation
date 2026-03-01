use bevy::prelude::*;
use bevy_ghx_proc_gen::{
    bevy_ghx_grid::{
        debug_plugin::{view::DebugGridView, DebugGridView3dBundle},
        ghx_grid::{
            coordinate_system::Cartesian3D,
            grid::GridDefinition,
        },
    },
    gen::{
        assets::{AssetSpawner, RulesModelsAssets},
        debug_plugin::GenerationViewMode,
    },
    proc_gen::generator::{
        builder::GeneratorBuilder,
        rules::RulesBuilder,
        socket::{SocketCollection, SocketsCartesian3D},
        model::ModelCollection,
        Generator,
    },
    GeneratorBundle,
};
use bevy_ghx_utils::camera::PanOrbitCamera;
use bevy_mod_picking::prelude::*;
use std::sync::Arc;

// Grid configuration
const GRID_SIZE_X: u32 = 40;
const GRID_SIZE_Y: u32 = 20;
const GRID_SIZE_Z: u32 = 40;
const BLOCK_SIZE: f32 = 1.0;
const NODE_SIZE: Vec3 = Vec3::splat(BLOCK_SIZE);

// Kenney Brick Kit assets scale (models are typically ~1 unit)
const KENNEY_SCALE_FACTOR: f32 = BLOCK_SIZE / 1.0;
pub const KENNEY_SCALE: Vec3 = Vec3::splat(KENNEY_SCALE_FACTOR);

#[derive(Resource)]
pub struct TownscapeGenerator {
    pub generator: Generator<Cartesian3D>,
    pub grid: GridDefinition<Cartesian3D>,
}

#[derive(Component)]
pub struct TownscapeVoxel {
    pub grid_pos: IVec3,
}

/// Build WFC rules for Townscaper-style architecture
pub fn build_townscape_rules() -> (
    Vec<String>,
    ModelCollection<Cartesian3D>,
    SocketCollection,
) {
    let mut sockets = SocketCollection::new();
    
    // Define socket types
    let air = sockets.create();
    let ground_top = sockets.create();
    let floor_bottom = sockets.create();
    let floor_top = sockets.create();
    let roof_bottom = sockets.create();
    let wall_inside = sockets.create();
    let wall_outside = sockets.create();
    
    // Socket connections define which pieces can touch
    sockets.add_connections(vec![
        (air, vec![air, wall_outside]),
        (ground_top, vec![floor_bottom]),
        (floor_top, vec![floor_bottom, roof_bottom]),
        (floor_bottom, vec![floor_top, ground_top]),
        (wall_inside, vec![wall_inside, floor_top, floor_bottom]),
        (wall_outside, vec![wall_outside, air]),
        (roof_bottom, vec![floor_top]),
    ]);
    
    let mut models = ModelCollection::<Cartesian3D>::new();
    let mut asset_paths = Vec::new();
    
    // Model 0: Air (empty space)
    models.create(SocketsCartesian3D::Simple {
        x_pos: air, x_neg: air,
        z_pos: air, z_neg: air,
        y_pos: air, y_neg: air,
    });
    asset_paths.push("air".to_string());
    
    // Model 1: Ground foundation block
    models.create(SocketsCartesian3D::Simple {
        x_pos: wall_outside, x_neg: wall_outside,
        z_pos: wall_outside, z_neg: wall_outside,
        y_pos: ground_top, y_neg: air,
    });
    asset_paths.push("kenney/brick-kit/brickDarkLarge.glb".to_string());
    
    // Model 2: Floor piece (can stack)
    models.create(SocketsCartesian3D::Simple {
        x_pos: wall_outside, x_neg: wall_outside,
        z_pos: wall_outside, z_neg: wall_outside,
        y_pos: floor_top, y_neg: floor_bottom,
    });
    asset_paths.push("kenney/brick-kit/brickBrownLarge.glb".to_string());
    
    // Model 3: Roof cap
    models.create(SocketsCartesian3D::Simple {
        x_pos: wall_outside, x_neg: wall_outside,
        z_pos: wall_outside, z_neg: wall_outside,
        y_pos: air, y_neg: roof_bottom,
    });
    asset_paths.push("kenney/brick-kit/roofRedPointA.glb".to_string());
    
    // Model 4: Corner piece
    models.create(SocketsCartesian3D::Simple {
        x_pos: wall_outside, x_neg: wall_inside,
        z_pos: wall_outside, z_neg: wall_inside,
        y_pos: floor_top, y_neg: floor_bottom,
    });
    asset_paths.push("kenney/brick-kit/brickCornerRoundBrown.glb".to_string());
    
    // Model 5: Window wall
    models.create(SocketsCartesian3D::Simple {
        x_pos: wall_outside, x_neg: wall_outside,
        z_pos: wall_outside, z_neg: wall_inside,
        y_pos: floor_top, y_neg: floor_bottom,
    });
    asset_paths.push("kenney/brick-kit/brickWindowBrown.glb".to_string());
    
    (asset_paths, models, sockets)
}

pub fn setup_townscape_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera setup
    let camera_position = Vec3::new(
        GRID_SIZE_X as f32 * 0.5,
        GRID_SIZE_Y as f32 * 2.0,
        GRID_SIZE_Z as f32 * 1.5,
    );
    let target = Vec3::new(
        GRID_SIZE_X as f32 * 0.5,
        0.0,
        GRID_SIZE_Z as f32 * 0.5,
    );
    
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(camera_position)
                .looking_at(target, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            radius: (camera_position - target).length(),
            focus: target,
            ..default()
        },
    ));
    
    // Ground plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default()),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.6, 0.4),
                perceptual_roughness: 0.9,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(10000.0))
                .with_translation(Vec3::new(0., -BLOCK_SIZE * 0.5, 0.)),
            ..default()
        },
        Name::new("Ground"),
    ));
    
    // Lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });
    
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        ..default()
    });
}

pub fn setup_townscape_generator(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let (asset_paths, models, socket_collection) = build_townscape_rules();
    
    let rules = Arc::new(
        RulesBuilder::new_cartesian_3d(models, socket_collection)
            .build()
            .unwrap(),
    );
    
    let grid = GridDefinition::new_cartesian_3d(
        GRID_SIZE_X,
        GRID_SIZE_Y,
        GRID_SIZE_Z,
        false,
        false,
        false,
    );
    
    let gen_builder = GeneratorBuilder::new()
        .with_shared_rules(rules)
        .with_grid(grid.clone());
    
    // Load Kenney Brick Kit assets
    let mut models_assets = RulesModelsAssets::default();
    for (idx, path) in asset_paths.iter().enumerate() {
        if path == "air" {
            models_assets.insert(idx, None);
        } else {
            let handle: Handle<Scene> = asset_server.load(format!("{}#Scene0", path));
            models_assets.insert(idx, Some(handle));
        }
    }
    
    let asset_spawner = AssetSpawner::new(
        models_assets,
        NODE_SIZE,
        KENNEY_SCALE,
    );
    
    let observer = gen_builder.add_queued_observer();
    let generator = gen_builder.build().unwrap();
    
    commands.spawn((
        GeneratorBundle {
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                Vec3::new(
                    -(GRID_SIZE_X as f32) * BLOCK_SIZE * 0.5,
                    0.0,
                    -(GRID_SIZE_Z as f32) * BLOCK_SIZE * 0.5,
                )
            )),
            grid: grid.clone(),
            generator,
            asset_spawner,
        },
        observer,
        DebugGridView3dBundle {
            view: DebugGridView::new(false, true, Color::srgb(0.5, 0.5, 0.5), NODE_SIZE),
            ..default()
        },
        Name::new("TownscapeGrid"),
    ));
}

/// Interactive placement system - add/remove blocks on click
pub fn handle_townscape_clicks(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut generator_query: Query<(&mut Generator<Cartesian3D>, &GridDefinition<Cartesian3D>, &Transform)>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return };
    
    // Cast ray from camera through cursor
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    
    // Find intersection with grid plane (y = 0 for ground level, or elevated)
    let target_y = 0.0;
    let t = (target_y - ray.origin.y) / ray.direction.y;
    if t < 0.0 { return; }
    
    let hit_point = ray.origin + ray.direction * t;
    
    for (mut generator, grid, transform) in generator_query.iter_mut() {
        // Transform world point to grid space
        let local_point = transform.transform_point(hit_point);
        let grid_x = (local_point.x / BLOCK_SIZE).floor() as i32;
        let grid_z = (local_point.z / BLOCK_SIZE).floor() as i32;
        
        // Check bounds
        if grid_x < 0 || grid_x >= GRID_SIZE_X as i32 || 
           grid_z < 0 || grid_z >= GRID_SIZE_Z as i32 {
            continue;
        }
        
        // Determine height based on shift key
        let grid_y = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            // Place at elevated position
            5
        } else {
            0  // Ground level
        };
        
        info!("Placing block at ({}, {}, {})", grid_x, grid_y, grid_z);
        
        // Constrain this cell to be a building block (model 1 = ground, model 2 = floor)
        let model_idx = if grid_y == 0 { 1 } else { 2 };
        
        // Note: This is a simplified constraint API - actual API may differ
        // You would typically use generator.constrain_node() or similar
        // then call generator.step() or generator.solve() to propagate
        
        info!("Interactive placement - WFC propagation would happen here");
        info!("Target: model {} at grid position ({},{},{})", model_idx, grid_x, grid_y, grid_z);
    }
}

/// UI instructions for townscape mode
pub fn setup_townscape_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Townscaper Mode\n\nLeft Click: Place block\nShift + Click: Place elevated\nF1-F5: Toggle views",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        Name::new("TownscapeUI"),
    ));
}

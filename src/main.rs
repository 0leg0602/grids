use bevy::prelude::*;

#[derive(Component)]
struct Rotator;

#[derive(Component)]
struct BoardPart;

#[derive(Component)]
struct ChessPieces;

#[derive(Component)]
struct Colored;

#[derive(Component)]
enum PieceColor{
    White,
    Black
}

#[derive(Resource)]
struct SelectedPiece(Option<Entity>);

#[derive(Resource)]
struct ChessMaterials{
    white: Handle<StandardMaterial>,
    black: Handle<StandardMaterial>,
}

struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedPiece(None));
        app.add_systems(Startup, (setup_materials, create).chain());
        app.add_systems(Update, (keyboard_update, update_textures));
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(MainPlugin)
    .run();
}

fn setup_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(ChessMaterials{
        white: materials.add(Color::WHITE),
        black: materials.add(Color::BLACK),
    });
}

fn update_textures(
    pieces_query: Query<(Entity, &PieceColor), (With<ChessPieces>, Without<Colored>)>,
    mut commands: Commands,
    mesh_query: Query<Entity, With<MeshMaterial3d<StandardMaterial>>>,
    children_query: Query<&Children>,
    chess_material: Res<ChessMaterials>,
){
    
    for (entity, color) in pieces_query{
        for child in children_query.iter_descendants(entity){
            if mesh_query.get(child).is_ok() {
                    match color {
                        PieceColor::White => {commands.entity(child).insert(MeshMaterial3d(chess_material.black.clone()));},
                        PieceColor::Black => {commands.entity(child).insert(MeshMaterial3d(chess_material.white.clone()));},
                    }

                    commands.entity(entity).insert(Colored);
            }
        }
    }
}

fn create(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, asset_server: Res<AssetServer>) {
    commands.spawn((
        Transform::default(),
        Rotator,
        children![(
            Camera3d::default(),
            Transform::from_xyz(0.0, 6.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y)
        )],

    ));

    let pawn_handle = asset_server.load("models/pawn.glb#Scene0");
    

    let board_size = 8;
    let offset_size = 0.1;

    for col in 0..board_size {
        for row in 0..board_size {
            let mut part_color = Color::srgb(0.0, 0.0, 0.0);
            if (row+col) % 2 != 0 {
                part_color = Color::srgb(1.0, 1.0, 1.0);
            }
            let board_part_x = (((board_size-1) as f32 + (board_size-1) as f32 * offset_size)/2.0) - (1.0 + offset_size) * row as f32;
            let board_part_y = (((board_size-1) as f32 + (board_size-1) as f32 * offset_size)/2.0) - (1.0 + offset_size) * col as f32;

            if row < 2 || row > 5 {
                    
                let mut chess_piece = commands.spawn((
                    SceneRoot(pawn_handle.clone()),
                    Transform::from_xyz(board_part_x, 0.55, board_part_y),
                    ChessPieces,
                ));

                chess_piece.observe(handle_click);

                if row < 2 {
                    chess_piece.insert(PieceColor::White);
                } else if row > 5 {
                    chess_piece.insert(PieceColor::Black);
                }


            }

            commands.spawn((
                BoardPart,
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(part_color)),
                Transform::from_xyz(board_part_x, 0.0, board_part_y)
            )).observe(handle_click);

        }
    }
    

    commands.spawn((
        PointLight{..Default::default()},
        Transform::from_xyz(2.0, 5.0, 2.0)
    ));

}

fn keyboard_update(
    mut rotator_query: Query<&mut Transform, (With<Rotator>, Without<Camera3d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Rotator>)>,
    time: Res<Time>, keyboard_input: Res<ButtonInput<KeyCode>>,
){
    if let Ok(mut rotator) = rotator_query.single_mut() && let Ok(mut camera) = camera_query.single_mut() {

        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            rotator.rotate_y(-2.0 * time.delta_secs());
            
        } else if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight){
            rotator.rotate_y(2.0 * time.delta_secs());

        } if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            camera.rotate_around(Vec3::ZERO, Quat::from_rotation_x(-2.0 * time.delta_secs()));

        } else if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown){
            camera.rotate_around(Vec3::ZERO, Quat::from_rotation_x(2.0 * time.delta_secs()));
        }
    }
}

fn handle_click(
    trigger: On<Pointer<Press>>, 
    mut transform_query: Query<&mut Transform>,
    chess_piece_query: Query<&ChessPieces>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut commands: Commands,
) {

    let hovered_entity = trigger.event_target();

    
    if let Some(selected_piece_enity) = selected_piece.0 {
        if chess_piece_query.contains(hovered_entity) && selected_piece_enity != hovered_entity {
            commands.entity(hovered_entity).despawn();
        }

        if let Ok([hover_transform, mut selected_transform]) = transform_query.get_many_mut([hovered_entity, selected_piece_enity]){
            selected_transform.translation.x = hover_transform.translation.x;
            selected_transform.translation.z = hover_transform.translation.z;
            selected_transform.translation.y = 0.55;

            selected_piece.0 = None;
        }

    } else {
        if let Ok(ChessPieces) = chess_piece_query.get(hovered_entity) {
            selected_piece.0 = Some(hovered_entity);
            if let Ok(mut hover_transform) = transform_query.get_mut(hovered_entity) {
                hover_transform.translation.y = 2.0;
            }
        }
    }
    
        
        

}






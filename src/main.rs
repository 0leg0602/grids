use std::f32::consts::PI;

use bevy::{input::mouse::MouseWheel, prelude::*};

#[derive(Component)]
struct RotatorHorizontal;

#[derive(Component)]
struct RotatorVertical;

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
struct Animation{
    target: Option<Entity>,
    final_location: Option<Vec3>,
    is_finished: bool,
}

#[derive(Resource)]
struct ChessMaterials{
    white: Handle<StandardMaterial>,
    black: Handle<StandardMaterial>,
}

struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedPiece(None));
        app.insert_resource(Animation{target: None, final_location: None, is_finished: true});
        app.add_systems(Startup, (setup_materials, init_scene, create_chess_pieces).chain());
        app.add_systems(Update, (input_update, update_textures, update_animation));
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

fn init_scene(mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        RotatorHorizontal,
        children![(
            Transform::default(),
            RotatorVertical,
            children![(
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y)
            )],
        )],

    ));

    commands.spawn((
        PointLight{..Default::default()},
        Transform::from_xyz(2.0, 5.0, 2.0)
    ));

    commands.spawn((
        PointLight{..Default::default()},
        Transform::from_xyz(-3.0, 5.0, -4.0)
    ));

}

fn create_chess_pieces(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, asset_server: Res<AssetServer>){
    let pawn_handle = asset_server.load("models/pawn.glb#Scene0");
    let rook_handle = asset_server.load("models/rook.glb#Scene0");
    let knight_handle = asset_server.load("models/knight.glb#Scene0");
    let bishop_handle = asset_server.load("models/bishop.glb#Scene0");
    let king_handle = asset_server.load("models/king.glb#Scene0");
    let queen_handle = asset_server.load("models/queen.glb#Scene0");
    

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

                let mut chess_piece_transform = Transform::from_xyz(board_part_x, 0.55, board_part_y);
                    
                let mut chess_piece = commands.spawn((
                    ChessPieces,
                ));

                if row == 1 || row == 6{
                    chess_piece.insert(SceneRoot(pawn_handle.clone()));
                } else if col == 0 || col == 7{
                    chess_piece.insert(SceneRoot(rook_handle.clone()));
                } else if col == 1 || col == 6 {
                    chess_piece.insert(SceneRoot(knight_handle.clone()));
                } else if col == 2 || col == 5 {
                    chess_piece.insert(SceneRoot(bishop_handle.clone()));
                } else if col == 3 {
                    chess_piece.insert(SceneRoot(king_handle.clone()));
                } else if col == 4 {
                    chess_piece.insert(SceneRoot(queen_handle.clone()));
                }

                
                if row < 2 {
                    chess_piece.insert(PieceColor::White);
                    if col == 1 || col == 6 {
                        chess_piece_transform.rotate_local_y(-PI/2.0);
                    }
                } else if row > 5 {
                    chess_piece.insert(PieceColor::Black);
                    if col == 1 || col == 6 {
                        chess_piece_transform.rotate_local_y(PI/2.0);
                    }
                }
                
                chess_piece.observe(handle_click);
                chess_piece.insert(chess_piece_transform);

            }

            commands.spawn((
                BoardPart,
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(part_color)),
                Transform::from_xyz(board_part_x, 0.0, board_part_y)
            )).observe(handle_click);

            

        }
    }
}

fn input_update(
    mut set: ParamSet<(
        Query<&mut Transform, With<RotatorHorizontal>>,
        Query<&mut Transform, With<RotatorVertical>>,
        Query<&mut Transform, With<Camera3d>>,
    )>,
    time: Res<Time>, 
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_events: MessageReader<MouseWheel>,
){


    for mut rotator_h in set.p0().iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        rotator_h.rotate_y(-2.0 * time.delta_secs());
        
        } else if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight){
            rotator_h.rotate_y(2.0 * time.delta_secs());

        }
    }

    for mut rotator_v in set.p1().iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        rotator_v.rotate_x(-2.0 * time.delta_secs());

        } else if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown){
            rotator_v.rotate_x(2.0 * time.delta_secs());
        }

        if keyboard_input.pressed(KeyCode::KeyF) || keyboard_input.pressed(KeyCode::KeyJ) {
            rotator_v.translation.x -= 6.0 * time.delta_secs();

        } else if keyboard_input.pressed(KeyCode::KeyH) || keyboard_input.pressed(KeyCode::KeyL) {
            rotator_v.translation.x += 6.0 * time.delta_secs();
        }

        if keyboard_input.pressed(KeyCode::KeyT) || keyboard_input.pressed(KeyCode::KeyI) {
            rotator_v.translation.z -= 6.0 * time.delta_secs();

        } else if keyboard_input.pressed(KeyCode::KeyG) || keyboard_input.pressed(KeyCode::KeyK) {
            rotator_v.translation.z += 6.0 * time.delta_secs();
        }
    }

    for mut camera in set.p2().iter_mut() {
        if keyboard_input.pressed(KeyCode::Equal){
            camera.translation.z -= 6.0 * time.delta_secs();

        } else if keyboard_input.pressed(KeyCode::Minus){
            camera.translation.z += 6.0 * time.delta_secs();

        }  

        for event in scroll_events.read() {
            camera.translation.z -= (20.0 * event.y) * time.delta_secs();
        }
    }

        

        
}

fn handle_click(

    trigger: On<Pointer<Press>>, 
    mut transform_query: Query<&mut Transform>,
    chess_piece_query: Query<&ChessPieces>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut animation: ResMut<Animation>,
    mut commands: Commands,
) {
    if !animation.is_finished {
        return;
    }


    let hovered_entity = trigger.event_target();

    
    if let Some(selected_piece_enity) = selected_piece.0 {
        if chess_piece_query.contains(hovered_entity) && selected_piece_enity != hovered_entity {
            commands.entity(hovered_entity).despawn();
        }

        if let Ok([hover_transform, mut selected_transform]) = transform_query.get_many_mut([hovered_entity, selected_piece_enity]){
            let mut final_vec = hover_transform.translation;
            final_vec.y = 0.55;

            animation.target = Some(selected_piece_enity);
            animation.final_location = Some(final_vec);
            animation.is_finished = false;


            selected_piece.0 = None;
        }

    } else {
        if let Ok(ChessPieces) = chess_piece_query.get(hovered_entity) {
            selected_piece.0 = Some(hovered_entity);
            
            if let Ok(mut hover_transform) = transform_query.get_mut(hovered_entity) {
                
                let mut final_vec = hover_transform.translation;
                final_vec.y = 2.0;

                animation.target = Some(hovered_entity);
                animation.final_location = Some(final_vec);
                animation.is_finished = false;
                // hover_transform.translation.y = 2.0;
            }
        }
    }
}

fn update_animation(
    mut animation: ResMut<Animation>,
    mut transform_query: Query<&mut Transform>,
    time: Res<Time>,
){

    let speed = 5.0;

    if let (Some(target), Some(final_location)) = (animation.target, animation.final_location){
        if let Ok(mut target_transform) = transform_query.get_mut(target) {
            target_transform.translation.y += 1.0 * time.delta_secs();
            let direction = target_transform.translation - final_location;
            let distance = direction.length();

            if distance > 0.1 {
                let step = speed * time.delta_secs();
                
                let move_amount = direction.normalize() * (step + (distance/10.0));
                target_transform.translation -= move_amount;
            } else {
                target_transform.translation = final_location;
                animation.target = None;
                animation.final_location = None;
                animation.is_finished = true;
            }

        }
        // 


    }
}
use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use bevy::text::DrawableText;
use rand::prelude::random;
use std::time::Duration;

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "snek".to_string(),
            width: 500,
            height: 500,
            ..Default::default()
        })
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(SnakeMoveTimer(Timer::new(
            Duration::from_millis(150 as u64),
            true,
        )))
        .add_resource(SnakeSegments::default())
        .add_resource(LastTailPosition::default())
        .add_resource(Score::default())
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", spawn_snake.system())
        .add_system(snake_timer.system())
        .add_system(snake_movement.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_system(food_spawner.system())
        .add_system(snake_eating.system())
        .add_system(snake_growth.system())
        .add_system(game_over.system())
        .add_system(score_text.system())
        .add_plugins(DefaultPlugins)
        .run();
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

struct SnakeHead {
    direction: Direction,
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

struct Food;

struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

struct SnakeMoveTimer(Timer);

struct SnakeSegment;

#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

struct GrowthEvent;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

struct GameOverEvent;

struct Score(u32);

impl Default for Score {
    fn default() -> Self {
        Score(0)
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>
) {
    commands.spawn(Camera2dComponents::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        segment_material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
    });
    commands
        .spawn(UiCameraComponents::default())
        .spawn(TextComponents {
            text: Text {
                value: "0".to_string(),
                font: asset_server.get_handle("fonts/DejaVuSansMono.ttf").clone(),
                style: TextStyle {
                    color: Color::WHITE,
                    font_size: 30.0,
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Score(0));
}

fn spawn_snake(
    mut commands: Commands,
    materials: Res<Materials>,
    mut segments: ResMut<SnakeSegments>,
) {
    segments.0 = vec![
        commands
            .spawn(SpriteComponents {
                material: materials.head_material.clone(),
                sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                ..Default::default()
            })
            .with(SnakeHead {
                direction: Direction::Up,
            })
            .with(SnakeSegment)
            .with(Position { x: 3, y: 3 })
            .with(Size::square(0.8))
            .current_entity()
            .unwrap(),
        spawn_segment(
            &mut commands,
            &materials.segment_material,
            Position { x: 3, y: 2 },
        )
    ];
}

fn spawn_segment(
    commands: &mut Commands,
    material: &Handle<ColorMaterial>,
    position: Position
) -> Entity {
    commands
        .spawn(SpriteComponents {
            material: material.clone(),
            ..SpriteComponents::default()
        })
        .with(SnakeSegment)
        .with(position)
        .with(Size::square(0.65))
        .current_entity()
        .unwrap()
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
        );
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + tile_size / 2.
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snake_timer: ResMut<SnakeMoveTimer>,
    segments: ResMut<SnakeSegments>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut heads: Query<(Entity, &mut SnakeHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();
        let mut head_pos = positions.get_mut(head_entity).unwrap();
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else {
            head.direction
        };

        if dir != head.direction.opposite() {
            head.direction = dir;
        }
        if !snake_timer.0.finished {
            return;
        }
        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        }
        if segment_positions.contains(&head_pos) {
            game_over_events.send(GameOverEvent);
        }
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT
        {
            game_over_events.send(GameOverEvent);
        }
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        last_tail_position.0 = Some(*segment_positions.last().unwrap());
    }
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    mut timer: Local<FoodSpawnTimer>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(Position {
                x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
                y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
            })
            .with(Size::square(0.8));
    }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeMoveTimer>) {
    snake_timer.0.tick(time.delta_seconds);
}

fn snake_eating(
    mut commands: Commands,
    snake_timer: ResMut<SnakeMoveTimer>,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<With<Food, (Entity, &Position)>>,
    head_positions: Query<With<SnakeHead, &Position>>,
    mut score_query: Query<&mut Score>,
) {
    if !snake_timer.0.finished {
        return;
    }
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.despawn(ent);
                growth_events.send(GrowthEvent);
                for mut score in score_query.iter_mut() {
                    score.0 = score.0 + 1;
                }
            }
        }
    }
}

fn score_text(
    mut score_query: Query<(&mut Text, &Score)>
) {
    for (mut text, score) in score_query.iter_mut() {
        text.value = score.0.to_string();
    }
}

fn snake_growth(
    mut commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GrowthEvent>>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    materials: Res<Materials>,
) {
    if growth_reader.iter(&growth_events).next().is_some() {
        segments.0.push(spawn_segment(
            &mut commands,
            &materials.segment_material,
            last_tail_position.0.unwrap(),
        ));
    }
}

fn game_over(
    mut commands: Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    materials: Res<Materials>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<With<Food, Entity>>,
    segments: Query<With<SnakeSegment, Entity>>,
) {
    if reader.iter(&game_over_events).next().is_some() {
        for ent in food.iter().chain(segments.iter()) {
            commands.despawn(ent);
        }
        spawn_snake(commands, materials, segments_res);
    }
}
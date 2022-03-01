extern crate derive_more;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use derive_more::{Add, AddAssign, SubAssign};
use macroquad::prelude::*;
use serde_derive::{Deserialize, Serialize};

const TILE_WIDTH:     f32 = 70.0;
const TILE_HEIGHT:    f32 = TILE_WIDTH;
const TILE_THICKNESS: f32 = 10.0;
const TILE_SPACE:     f32 = 5.0;

#[derive(Clone, Copy, Serialize, Deserialize)]
enum TileDirection {
    Up,
    Down,
    Left,
    Right,
}

const TILE_RANGE: f32 = 25.0;

impl TileDirection {
    pub fn get_angle(&self) -> Angle {
        match self {
            &TileDirection::Up    => Angle(180.0),
            &TileDirection::Down  => Angle(0.0),
            &TileDirection::Left  => Angle(270.0),
            &TileDirection::Right => Angle(90.0),
        }
    }

    pub fn get_range(&self) -> (f32, f32) {
        match self {
            &TileDirection::Up    => (180.0 - TILE_RANGE, 180.0 + TILE_RANGE),
            &TileDirection::Down  => (360.0 - TILE_RANGE, 0.0   + TILE_RANGE),
            &TileDirection::Left  => (270.0 - TILE_RANGE, 270.0 + TILE_RANGE),
            &TileDirection::Right => (90.0  - TILE_RANGE, 90.0  + TILE_RANGE),
        }
    }

    pub fn get_move_pos(&self) -> Position {
        match self {
            &TileDirection::Right => {
                Position{x: TILE_WIDTH + TILE_SPACE, y: 0.0}
            },
            &TileDirection::Left => {
                Position{x: -(TILE_WIDTH + TILE_SPACE), y: 0.0}
            },
            &TileDirection::Up => {
                Position{x: 0.0, y: -(TILE_HEIGHT + TILE_SPACE)}
            },
            &TileDirection::Down => {
                Position{x: 0.0, y: TILE_HEIGHT + TILE_SPACE}
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Tile {
    next_dir: TileDirection,
}

const LEVEL_COUNT: usize = 2;

struct Map {
    tiles: [Vec<Tile>; LEVEL_COUNT],
    current_level: usize,
}

impl Map {
    pub fn new() -> Self {
        Map {
            current_level: 0,
            tiles: Default::default(),
            //tiles: vec![
            //    Tile{next_dir: TileDirection::Right},
            //    Tile{next_dir: TileDirection::Right},
            //    Tile{next_dir: TileDirection::Up},
            //    Tile{next_dir: TileDirection::Up},
            //    Tile{next_dir: TileDirection::Up},
            //    Tile{next_dir: TileDirection::Left},
            //    Tile{next_dir: TileDirection::Left},
            //    Tile{next_dir: TileDirection::Down},
            //],
        }
    }

    pub async fn load_levels(&mut self) -> Result<(), String> {
        for i in 0..LEVEL_COUNT {
            self.tiles[i] = serde_json::from_str(&load_string(&format!("levels/{}.json", i)).await.map_err(|x| x.to_string())?).map_err(|x| x.to_string())?;
        }
        Ok(())
    }

    pub fn draw(&self, offset: &Position) {
        let mut last_pos =
            Position{
                x: screen_width()/2.0 - TILE_WIDTH/2.0,
                y: screen_height()/2.0 - TILE_HEIGHT/2.0
            };

        for tile in &self.tiles[self.current_level] {
            draw_rectangle_lines(
                last_pos.x + offset.x,
                last_pos. y + offset.y,
                TILE_WIDTH, TILE_HEIGHT, TILE_THICKNESS, RED);

            last_pos += tile.next_dir.get_move_pos();
        }
        draw_rectangle_lines(
            last_pos.x + offset.x,
            last_pos. y + offset.y,
            TILE_WIDTH, TILE_HEIGHT, TILE_THICKNESS, RED);
    }

    pub fn get_tiles(&self) -> &Vec<Tile> {
        &self.tiles[self.current_level]
    }

    pub fn get_current_level(&self) -> usize {
        self.current_level
    }

    pub fn next_level(&mut self) {
        self.current_level += 1;
    }
}

struct GameState {
    speed: f32,
    map:   Map,
    current_tile: usize,
}

enum MoveNextReturn {
    Move(TileDirection),
    Reset,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            speed: 5.0,
            map:   Map::new(),
            current_tile: 0,
        }
    }

    pub fn draw_map(&self, offset: &Position) {
        self.map.draw(&offset);
    }

    pub fn move_next(&mut self, facing: &Angle) -> Option<MoveNextReturn> {
        let angle_range = self.map.get_tiles()[self.current_tile].next_dir.get_range();

        let did_hit = |facing: &Angle, angle_range: &(f32, f32)| -> bool {
            if angle_range.0 > angle_range.1 {
                if facing.get() >= angle_range.0 && facing.get() <= 360.0 {
                    return true;
                } else if facing.get() >= 0.0 && facing.get() <= angle_range.1 {
                    return true;
                }
            }

            if facing.get() >= angle_range.0 && facing.get() <= angle_range.1 {
                return true;
            } else {
                return false;
            }
        };

        if did_hit(&facing, &angle_range) {
            self.current_tile += 1;
            debug!("Good facing: {} range: {:?}", facing.get(), angle_range);
            if self.current_tile < self.map.tiles[self.map.get_current_level()].len() {
                return Some(MoveNextReturn::Move(self.map.get_tiles()[self.current_tile-1].next_dir));
            } else {
                self.current_tile = 0;
                self.map.next_level();
                return Some(MoveNextReturn::Reset);
            }
        } else {
            debug!("Bad facing: {} range: {:?}", facing.get(), angle_range);
            return None;
        }
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }

    pub fn get_map_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    pub fn get_current_tile(&self) -> usize {
        self.current_tile
    }
}

#[derive(Default, Clone, Copy, Add, AddAssign, SubAssign)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default)]
struct Camera {
    pos: Position,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            ..Default::default()
        }
    }

    pub fn get_pos(&self) -> &Position {
        &self.pos
    }
}

#[derive(Clone, Copy, AddAssign, SubAssign)]
struct Angle(f32);

impl Angle {
    pub fn get(&self) -> f32 {
        let normalized = self.0 % 360.0;
        if normalized < 0.0 {
            return 360.0 + normalized;
        } else {
            return normalized;
        }
    }

    pub fn to_radians(&self) -> f32{
        self.get() * (std::f32::consts::PI / 180.0)
    }
}

impl Default for Angle {
    fn default() -> Self {
        Self(-90.0)
    }
}

const BALL_SIZE: f32 = 20.0;
const BALL_DISTANCE: f32 = TILE_WIDTH + TILE_SPACE;

#[derive(Default)]
struct Ball {
    pos:   Position,
    angle: Angle,
}

impl Ball {
    pub fn draw(&self, offset: &Position) {
        draw_circle(self.pos.x + offset.x, self.pos.y + offset.y, BALL_SIZE, BLACK);
    }

    pub fn update(&mut self, game_state: &GameState) {
        self.pos.x = BALL_DISTANCE * self.angle.to_radians().sin();
        self.pos.y = BALL_DISTANCE * self.angle.to_radians().cos();
        self.angle -= Angle(game_state.speed);
    }

    pub fn get_angle(&self) -> &Angle {
        &self.angle
    }

    pub fn get_angle_mut(&mut self) -> &mut Angle {
        &mut self.angle
    }
}

#[derive(Default)]
struct Head {
    pos: Position,
    ball: Ball,
}

impl Head {
    pub fn new() -> Self {
        let mut head = Head {
            ..Default::default()
        };
        head.reset();

        head
    }

    pub fn draw(&self, offset: &Position) {
        draw_circle(self.pos.x + offset.x, self.pos.y + offset.y, BALL_SIZE*0.75, BLACK);
        self.ball.draw(&(self.pos + *offset));
    }

    pub fn update(&mut self, game_state: &GameState) {
        self.ball.update(&game_state);
    }

    pub fn get_facing_angle(&self) -> &Angle {
        self.ball.get_angle()
    }

    pub fn move_pos(&mut self, game_state: &GameState, dir: TileDirection) {
        self.pos += dir.get_move_pos();

        self.ball.get_angle_mut().0 = game_state.get_map().get_tiles()[game_state.get_current_tile() - 1].next_dir.get_angle().0 + 180.0;
    }

    pub fn reset(&mut self) {
        self.pos = Position {x: screen_width()/2.0, y: screen_height()/2.0};
        self.ball = Default::default();
    }
}

#[macroquad::main("Rhythm")]
async fn main() {
    macroquad::file::set_pc_assets_folder("assets/");

    let mut head = Head::new();
    let camera = Camera::new();
    let mut game_state = GameState::new();

    game_state.get_map_mut().load_levels().await.unwrap();

    loop {
        clear_background(WHITE);

        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
            let out = game_state.move_next(head.get_facing_angle());
            if out.is_some() {
                match out.unwrap() {
                    MoveNextReturn::Move(pos) => head.move_pos(&game_state, pos),
                    MoveNextReturn::Reset => head.reset(),
                }
            }
        }

        head.update(&game_state);

        game_state.draw_map(&camera.pos);
        head.draw(camera.get_pos());

        next_frame().await
    }
}

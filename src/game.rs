use std::collections::HashMap;

use eframe::{egui, Storage};

use crate::assets::{AssetType, Assets};
use crate::grid::{Direction, Grid, Vec2};
use crate::modals::{PauseModal, PauseModalEvent, PuzzleSolvedModal, PuzzleSolvedModalEvent};
use crate::puzzle::{Feature, Kind, Alignment, Puzzle, Orientation, Tile};

const TILE_SIZE: f32 = 40.;

pub struct Game {
    assets: Assets,
    starting_position: Puzzle,
    puzzle: Puzzle,
    tile_widgets: Grid<TileSprite>,
    wall_sprites: Vec<WallSprite>,
    state: GameState,
    timer: Timer,
    move_counter: MoveCounter,
    settings: Settings,
}

impl Game {
    const INNER_MARGIN: f32 = 10.;

    /// Create a new game.
    pub fn new(puzzle: Puzzle, assets: Assets, settings: Settings) -> Self {
        let rows = puzzle.grid().rows();
        let cols = puzzle.grid().cols();
        let starting_position = puzzle.clone();

        let wall_sprites = Self::create_wall_sprites(&puzzle, &assets);

        Self {
            assets,
            starting_position,
            puzzle,
            tile_widgets: Grid::<TileSprite>::with_size(rows, cols, TileSprite::default()),
            wall_sprites,
            state: GameState::BeforeStart,
            timer: Timer::default(),
            move_counter: MoveCounter::default(),
            settings,
        }
    }

    /// Create wall sprite from the puzzle's wall objects. If playing on a torus, create the
    /// wall sprites along the seam twice (left and right, top and bottom).
    fn create_wall_sprites(puzzle: &Puzzle, assets: &Assets) -> Vec<WallSprite> {
        puzzle.walls().iter().flat_map(|wall| {
            let basic_sprite = WallSprite::new(wall.position(), wall.orientation(), assets);

            if puzzle.options().wrapping {
                if wall.position().x == 0 && wall.orientation() == Alignment::Vertical {
                    let pos = wall.position() + Vec2::new(puzzle.options().board_size as i32, 0);
                    let dual_sprite = WallSprite::new(pos, wall.orientation(), assets);
                    return vec![basic_sprite, dual_sprite];
                }
                if wall.position().y == 0 && wall.orientation() == Alignment::Horizontal {
                    let pos = wall.position() + Vec2::new(0, puzzle.options().board_size as i32);
                    let dual_sprite = WallSprite::new(pos, wall.orientation(), assets);
                    return vec![basic_sprite, dual_sprite];
                }
            }

            vec![basic_sprite]
        }).collect()
    }

    /// Restart the puzzle.
    pub fn restart(&mut self) {
        self.puzzle = self.starting_position.clone();
        let size = self.puzzle.grid().rows();
        self.tile_widgets = Grid::<TileSprite>::with_size(size, size, TileSprite::default());
        self.state = GameState::BeforeStart;
        self.timer = Timer::default();
        self.move_counter = MoveCounter::default();
    }

    /// Calculate the score.
    pub fn calc_score(&self) -> u32 {
        let weights = HashMap::from([
            (Kind::DeadEnd, 4),
            (Kind::Corner, 4),
            (Kind::Straight, 2),
            (Kind::TIntersection, 4),
            (Kind::CrossIntersection, 0),
        ]);
        let mut score: usize = self.puzzle.grid().iter()
            .map(|tile| {
                weights
                    .get(&tile.kind())
                    .expect("link type must be in map of weights")
            })
            .sum();
        score -= 2 * self.puzzle.walls().len();
        if !self.puzzle.options().wrapping {
            score -= self.puzzle.grid().rows() + self.puzzle.grid().cols();
        }
        let mut score = score as f32
            * (self.puzzle.expected_moves() as f32
                / (self.puzzle.grid().rows() * self.puzzle.grid().cols()) as f32);
        score = score * score / self.timer.duration().as_secs() as f32;

        score.round() as u32
    }

    pub fn update(&mut self, ui: &mut egui::Ui) -> Vec<GameEvent> {
        if self.state == GameState::Running {
            self.timer.update(ui.input(|i| i.time));
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }

        let mut events = ui
            .vertical_centered(|ui| {
                self.update_game_board(ui);
                ui.add_space(15.);
                let events = self.update_status_bar(ui);
                for event in &events {
                    if event == &GameEvent::Pause {
                        self.state = GameState::Paused {
                            game_was_started: self.state != GameState::BeforeStart,
                        };
                        self.timer.stop();
                    }
                }
                events
            })
            .inner;

        if let GameState::Paused { game_was_started } = self.state {
            let response = PauseModal::new().update(ui);
            match response {
                None => {}
                Some(PauseModalEvent::Continue) => {
                    if game_was_started {
                        self.state = GameState::Running;
                        self.timer.start();
                    } else {
                        self.state = GameState::BeforeStart;
                    }
                }
                Some(PauseModalEvent::NewGame) => events.push(GameEvent::NewGame),
                Some(PauseModalEvent::Restart) => {
                    self.restart();
                }
            }
        } else if let GameState::Ended { score } = self.state {
            let response = PuzzleSolvedModal::new(
                self.timer.duration(),
                self.move_counter.get(),
                self.puzzle.expected_moves(),
                score,
            )
            .update(ui);
            if let Some(PuzzleSolvedModalEvent::NewGame) = response {
                events.push(GameEvent::NewGame);
            }
        }

        events
    }

    fn update_game_board(&mut self, ui: &mut egui::Ui) {
        let board_size = self.puzzle.size();
        let desired_size =
            egui::Vec2::splat(board_size as f32 * TILE_SIZE + Self::INNER_MARGIN);
        ui.allocate_ui(desired_size, |ui| {
            let top_left =
                ui.max_rect().left_top().to_vec2() + egui::Vec2::splat(Self::INNER_MARGIN / 2.);

            // Manipulate top_left to ensure there are only integer values in x or y (no half pixels).
            // Rendering half-pixels does not work / does not play well with alpha blending texture
            // (which is not necessary, but seems to happen). Unclear if this is a bug in egui or
            // wgpu.
            let top_left = egui::Vec2::new(top_left.x.floor(), top_left.y.floor());

            let (hovered_tile, modified_tile) = self.draw_tiles(top_left, ui);

            for wall in &self.wall_sprites {
                wall.draw(top_left, ui);
            }

            if self.puzzle.options().wrapping && self.settings.show_wrap_marker &&
                let Some(hovered_tile) = hovered_tile {
                self.apply_wrap_markers(hovered_tile, top_left, ui);
            }

            // Run updates
            if let Some(updated_tile) = modified_tile {
                if self.state == GameState::BeforeStart {
                    self.timer.start();
                    self.state = GameState::Running;
                }

                self.move_counter.update(updated_tile);
                self.puzzle.calc_energy();

                if self.puzzle.solved() {
                    let score = self.calc_score();
                    self.state = GameState::Ended { score }
                }
            }
        });
    }

    fn draw_tiles(&mut self, top_left: egui::Vec2, ui: &mut egui::Ui) -> (Option<Vec2>, Option<Vec2>) {
        let mut hovered_tile = None;
        let mut modified_tile = None;

        for row in 0..self.puzzle.grid().rows() {
            for col in 0..self.puzzle.grid().cols() {
                let index = Vec2::new(row as i32, col as i32);
                let tile = self
                    .puzzle
                    .grid_mut()
                    .get_mut(index)
                    .expect("(row, col) must be on the grid");
                let widget = self
                    .tile_widgets
                    .get_mut(index)
                    .expect("(row, col) must be on the grid");
                let pos =
                    egui::Pos2::new(index.x as f32 * 40., index.y as f32 * 40.) + top_left;
                let response = widget.update(tile, index, pos, &self.assets, ui);
                if response.modified {
                    modified_tile = Some(index);
                }
                if response.hovered {
                    hovered_tile = Some(index);
                }
            }
        }

        (hovered_tile, modified_tile)
    }

    fn update_status_bar(&mut self, ui: &mut egui::Ui) -> Vec<GameEvent> {
        let mut events = vec![];
        let board_size = self.puzzle.size();
        let desired_size =
            egui::Vec2::splat(board_size as f32 * TILE_SIZE + Self::INNER_MARGIN);
        ui.allocate_ui(desired_size, |ui| {
            ui.vertical(|ui| {

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.;
                    if ui.button(egui::RichText::new(
                        egui_phosphor::regular::DOTS_THREE_VERTICAL.to_string()).size(12.)).clicked()
                    {
                        events.push(GameEvent::Pause)
                    }
                    ui.label(format!("{}/{}", self.move_counter.get(), self.puzzle.expected_moves()));
                    ui.label(format!("{}", self.timer));
                });
                if self.puzzle.options().wrapping &&
                    ui.checkbox(&mut self.settings.show_wrap_marker, "Show wrap marker").clicked() {
                        events.push(GameEvent::SettingsChanged(self.settings));
                }
            })
        });

        events
    }

    fn apply_wrap_markers(&mut self, hovered_tile: Vec2, top_left: egui::Vec2, ui: &mut egui::Ui) {
        let x = hovered_tile.x;
        let y = hovered_tile.y;

        if x <= 0 {
            let opposite_x = self.puzzle.grid().cols() as i32 - 1;
            self.draw_wrap_marker(Vec2::new(opposite_x, y), Direction::Left, top_left, ui);
        }
        if x >= self.puzzle.grid().cols() as i32 - 1 {
            let opposite_x = 0;
            self.draw_wrap_marker(Vec2::new(opposite_x, y), Direction::Right, top_left, ui);
        }
        if y <= 0 {
            let opposite_y = self.puzzle.grid().rows() as i32 - 1;
            self.draw_wrap_marker(Vec2::new(x, opposite_y), Direction::Up, top_left, ui);
        }
        if y >= self.puzzle.grid().rows() as i32 - 1 {
            let opposite_y = 0;
            self.draw_wrap_marker(Vec2::new(x, opposite_y), Direction::Down, top_left, ui);
        }
    }

    fn draw_wrap_marker(&self, coord: Vec2, direction: Direction, top_left: egui::Vec2, ui: &mut egui::Ui) {
        // Direction "Up" here mean along the upper edge of the tile, etc.
        let tile_size = TILE_SIZE;
        let tile_size_2 = tile_size / 2.;
        let triangle_size = 8.0f32;
        let triangle_offset = triangle_size * 1.5;

        let points = Self::triangle_path(triangle_size, direction);
        let offset = match direction {
            Direction::Up => egui::Vec2::new(tile_size_2, tile_size + triangle_offset),
            Direction::Down => egui::Vec2::new(tile_size_2, -triangle_offset),
            Direction::Left => egui::Vec2::new(tile_size + triangle_offset, tile_size_2),
            Direction::Right => egui::Vec2::new(-triangle_offset, tile_size_2),
        };

        let points_on_screen = points.iter()
            .map(|&p| p + top_left + offset
                + egui::Vec2::new(coord.x as f32 * tile_size, coord.y as f32 * tile_size))
            .collect::<Vec<_>>();

        let fill = egui::Color32::GRAY;
        let stroke = egui::epaint::PathStroke::new(1.0, fill);
        ui.painter().add(egui::epaint::PathShape::convex_polygon(points_on_screen, fill, stroke));
    }

    fn triangle_path(size: f32, direction: Direction) -> Vec<egui::Pos2> {
        // size is the length of the sides of an equilateral triangle
        let size2 = size / 2.;
        let h = size2 * 3.0f32.sqrt();

        match direction {
            Direction::Up => {
                vec![egui::pos2(-size2, 0.), egui::pos2(0., -h), egui::pos2(size2, 0.)]
            }
            Direction::Down => {
                vec![egui::pos2(-size2, 0.), egui::pos2(size2, 0.), egui::pos2(0., h)]
            }
            Direction::Left => {
                vec![egui::pos2(0., -size2), egui::pos2(0., size2), egui::pos2(-h, 0.)]
            }
            Direction::Right => {
                vec![egui::pos2(0., -size2), egui::pos2(h, 0.), egui::pos2(0., size2)]
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Settings {
    show_wrap_marker: bool,
}

impl Settings {
    pub fn read(storage: &dyn Storage) -> Self {
        let mut settings = Self::default();

        if let Some(s) = storage.get_string("show_wrap_marker") &&
            let Ok(value) = s.parse::<bool>() { settings.show_wrap_marker = value };

        settings
    }

    pub fn write(&self, storage: &mut dyn Storage) {
        storage.set_string("show_wrap_marker", self.show_wrap_marker.to_string());
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum GameState {
    BeforeStart,
    Running,
    Paused { game_was_started: bool },
    Ended { score: u32 },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GameEvent {
    Close,
    Pause,
    NewGame,
    Restart,
    SettingsChanged(Settings),
}

#[derive(Copy, Clone, Debug)]
struct Animation {
    angle: f32,
    time_per_quarter: std::time::Duration,
    target_quarters: u32,
    running: bool,
}

impl Animation {
    const SECONDS_PER_FRAME: f64 = 1. / 60.;

    fn new(time_per_quarter: std::time::Duration) -> Self {
        Animation {
            angle: 0.,
            time_per_quarter,
            target_quarters: 1,
            running: true,
        }
    }

    fn angle(&self) -> f32 {
        self.angle
    }

    fn running(&self) -> bool {
        self.running
    }

    fn target_quarters(&self) -> u32 {
        self.target_quarters
    }

    fn add_quarter(&mut self) {
        self.target_quarters += 1;
    }

    fn update(&mut self, ui: &mut egui::Ui) {
        let dt = ui.input(|i| i.stable_dt);
        let speed = std::f32::consts::PI / 2.0 / self.time_per_quarter.as_secs_f32();
        self.angle += speed * dt;
        let target_angle = self.target_quarters as f32 * std::f32::consts::PI / 2.0;
        if self.angle > target_angle {
            self.running = false;
        }
    }

    fn request_repaint(&mut self, ui: &mut egui::Ui) {
        if self.running {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(Self::SECONDS_PER_FRAME));
        }
    }
}

impl Default for Animation {
    fn default() -> Self {
        Animation::new(std::time::Duration::from_millis(150))
    }
}

// Stores only the animation state, the rest is stored in game.puzzle
#[derive(Copy, Clone, Debug, Default)]
struct TileSprite {
    animation: Option<Animation>,
    locked: bool,
}

impl TileSprite {
    const TILE_SIZE: f32 = 40.;
    // Maximum speed should be circa 75 milliseconds per 90 degrees (circa 4-5 animation frames)
    // Minimum speed should be circa 250 milliseconds per 90 degrees
    const ANIMATION_TIME_PER_QUARTER_ROTATION: std::time::Duration =
        std::time::Duration::from_millis(75);

    fn update(
        &mut self,
        tile: &mut Tile,
        index: Vec2,
        location: egui::Pos2,
        assets: &Assets,
        ui: &mut egui::Ui,
    ) -> TileResponse {
        let mut modified = false;
        if let Some(animation) = self.animation.as_mut() {
            animation.update(ui);
            if !animation.running() {
                for _ in 0..animation.target_quarters() {
                    tile.rotate();
                }
                self.animation = None;
                modified = true;
            }
        };

        let rect = egui::Rect::from_min_size(location, egui::Vec2::splat(Self::TILE_SIZE));
        let link_texture = self.select_link_texture(tile, assets);
        let angle = tile.orientation().to_angle() + self.animation.map(|a| a.angle()).unwrap_or(0.);
        ui.put(
            rect,
            egui::Image::from_texture(&link_texture).rotate(-angle, egui::Vec2::splat(0.5)),
        );
        if tile.feature() != Feature::None {
            let feature_texture = self
                .select_feature_texture(tile, assets)
                .expect("texture not found");
            ui.put(rect, egui::Image::from_texture(&feature_texture));
        }

        if self.locked {
            let painter = ui.painter();
            painter.rect_filled(rect, 0., egui::Rgba::from_black_alpha(0.5));
        }
        let id = format!("tile-{}-{}", index.x, index.y);
        let response = ui.interact(rect, egui::Id::from(id), egui::Sense::click());
        if response.secondary_clicked() && response.interact_pointer_pos().is_some() {
            self.locked = !self.locked;
        }
        if response.clicked() && response.interact_pointer_pos().is_some() && !self.locked {
            // To be super-precise, we would need to distinguish between just clicked
            // (first click starts the timer) and modified (after rotation finished)
            if let Some(animation) = self.animation.as_mut() {
                animation.add_quarter();
            } else {
                self.animation = Some(Animation::new(Self::ANIMATION_TIME_PER_QUARTER_ROTATION));
            }
        }

        if let Some(animation) = self.animation.as_mut() {
            animation.request_repaint(ui);
        }

        let hovered = response.hovered();

        TileResponse {
            hovered,
            modified,
        }
    }

    fn select_link_texture(&self, tile: &Tile, assets: &Assets) -> egui::TextureHandle {
        let link = tile.kind();
        let powered = tile.powered();

        let asset_type = if powered {
            match link {
                Kind::DeadEnd => AssetType::DeadEndPowered,
                Kind::Straight => AssetType::StraightPowered,
                Kind::Corner => AssetType::CornerPowered,
                Kind::TIntersection => AssetType::TIntersectionPowered,
                Kind::CrossIntersection => AssetType::CrossIntersectionPowered,
            }
        } else {
            match link {
                Kind::DeadEnd => AssetType::DeadEnd,
                Kind::Straight => AssetType::Straight,
                Kind::Corner => AssetType::Corner,
                Kind::TIntersection => AssetType::TIntersection,
                Kind::CrossIntersection => AssetType::CrossIntersection,
            }
        };

        assets
            .get_rotated(asset_type, Orientation::Basic)
            .expect("texture not found")
            .clone()
    }

    fn select_feature_texture(&self, tile: &Tile, assets: &Assets) -> Option<egui::TextureHandle> {
        let drain = if tile.powered() { AssetType::DrainPowered } else { AssetType::Drain };
        let source = if tile.powered() { AssetType::SourcePowered } else { AssetType::Source };

        match tile.feature() {
            Feature::None => None,
            Feature::Drain => Some(
                assets
                    .get_rotated(drain, Orientation::Basic)
                    .expect("texture not found"),
            ),
            Feature::Source => Some(
                assets
                    .get_rotated(source, Orientation::Basic)
                    .expect("texture not found"),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TileResponse {
    hovered: bool,
    modified: bool,
}

#[derive(Clone, Eq, PartialEq)]
struct WallSprite {
    position: egui::Pos2,
    texture: egui::TextureHandle,
}

impl WallSprite {
    fn new(position: Vec2, orientation: Alignment, assets: &Assets) -> Self {
        let (offset, rotation) = match orientation {
            Alignment::Horizontal => {
                (-egui::Vec2::new(0.0, TILE_SIZE / 2.), Orientation::Ccw90)
            }
            Alignment::Vertical => {
                (-egui::Vec2::new(TILE_SIZE / 2., 0.), Orientation::Basic)
            }
        };

        let position = egui::pos2(position.x as f32, position.y as f32)
            * TILE_SIZE + offset;
        let texture = assets
            .get_rotated(AssetType::Wall, rotation)
            .expect("texture not found");

        Self { position, texture }
    }

    fn draw(&self, top_left: egui::Vec2, ui: &mut egui::Ui) {
        let rect = egui::Rect::from_min_size(self.position + top_left, egui::Vec2::splat(TILE_SIZE));
        ui.put(rect, egui::Image::from_texture(&self.texture));
    }
}

#[derive(Default)]
struct Timer {
    total: std::time::Duration,
    last_value: Option<f64>,
    running: bool,
}

impl Timer {
    fn duration(&self) -> std::time::Duration {
        self.total
    }

    /// Does nothing if the timer is already running.
    fn start(&mut self) {
        self.running = true;
    }

    fn stop(&mut self) {
        self.running = false;
        self.last_value = None;
    }

    fn update(&mut self, secs_since_unknown: f64) {
        if self.running {
            if let Some(last_value) = self.last_value {
                let diff = secs_since_unknown - last_value;
                self.total += std::time::Duration::from_secs_f64(diff);
            }
            self.last_value = Some(secs_since_unknown);
        }
    }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seconds = self.total.as_secs();
        let minutes = seconds / 60;
        let rem_secs = seconds - minutes * 60;
        write!(f, "{minutes:02}:{rem_secs:02}")
    }
}

// A move consists of rotating a tile once. This can be done by multiple mouse button clicks.
// But rotating another tile in-between counts separately, i.e., rotating tile A, then tile B,
// then tile A again counts as rotating tile A twice (2 moves), or three moves in total.

#[derive(Default)]
struct MoveCounter {
    move_count: u32,
    last_rotated_tile: Option<Vec2>,
}

impl MoveCounter {
    fn get(&self) -> u32 {
        self.move_count
    }

    fn update(&mut self, updated_tile: Vec2) {
        if self.last_rotated_tile.is_some_and(|t| t == updated_tile) {
            return;
        }
        self.move_count += 1;
        self.last_rotated_tile = Some(updated_tile);
    }
}

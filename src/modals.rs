use eframe::egui;

use crate::puzzle::{Difficulty, Options};

pub struct NewGameModal {
    options: Options,
}

impl NewGameModal {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) -> Option<NewGameModalEvent> {
        egui::Modal::new(egui::Id::new("Modal New Game"))
            .show(ui.ctx(), |ui| {
                ui.set_width(300.0);
                ui.vertical_centered(|ui| {
                    ui.heading("New Game");
                    ui.separator();
                    ui.add_space(32.0);
                    egui::Grid::new("Options")
                        .num_columns(2)
                        .spacing([20.0, 20.0])
                        .show(ui, |ui| {
                            ui.label("Size");
                            ui.add(egui::Slider::new(&mut self.options.board_size, 3..=20));
                            ui.end_row();

                            ui.label("Difficulty");
                            egui::ComboBox::from_id_salt("Difficulty")
                                .selected_text(self.options.difficulty.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.options.difficulty,
                                        Difficulty::Easy,
                                        Difficulty::Easy.to_string(),
                                    );
                                    ui.selectable_value(
                                        &mut self.options.difficulty,
                                        Difficulty::Medium,
                                        Difficulty::Medium.to_string(),
                                    );
                                    ui.selectable_value(
                                        &mut self.options.difficulty,
                                        Difficulty::Hard,
                                        Difficulty::Hard.to_string(),
                                    );
                                });
                            ui.end_row();

                            ui.label("No boundary");
                            ui.add(egui::Checkbox::without_text(&mut self.options.wrapping));
                            ui.end_row();
                        });
                });

                ui.add_space(20.0);

                ui.with_layout(egui::Layout::right_to_left(Default::default()), |ui| {
                    if ui
                        .add_sized([80., 30.], egui::Button::new("Start"))
                        .clicked()
                    {
                        // Close the modal dialog and start a new game with the given options
                        Some(NewGameModalEvent::StartNewGame(self.options))
                    } else {
                        None
                    }
                })
                .inner
            })
            .inner
    }
}

pub enum NewGameModalEvent {
    StartNewGame(Options),
}

#[derive(Default)]
pub struct PauseModal {}

impl PauseModal {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ui: &mut egui::Ui) -> Option<PauseModalEvent> {
        egui::Modal::new(egui::Id::new("Game Paused"))
            .show(ui.ctx(), |ui| {
                ui.set_width(200.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Game Paused");
                    ui.separator();
                    ui.add_space(15.0);
                    ui.vertical_centered(|ui| {
                        if ui
                            .add_sized([80., 30.], egui::Button::new("Restart"))
                            .clicked()
                        {
                            return Some(PauseModalEvent::Restart);
                        }
                        if ui
                            .add_sized([80., 30.], egui::Button::new("New Game"))
                            .clicked()
                        {
                            return Some(PauseModalEvent::NewGame);
                        }
                        if ui
                            .add_sized([80., 30.], egui::Button::new("Continue"))
                            .clicked()
                        {
                            return Some(PauseModalEvent::Continue);
                        }
                        ui.add_space(15.0);
                        None
                    })
                    .inner
                })
                .inner
            })
            .inner
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PauseModalEvent {
    Continue,
    NewGame,
    Restart,
}

pub struct PuzzleSolvedModal {
    time: std::time::Duration,
    moves: u32,
    expected_moves: u32,
    score: u32,
}

impl PuzzleSolvedModal {
    pub fn new(time: std::time::Duration, moves: u32, expected_moves: u32, score: u32) -> Self {
        PuzzleSolvedModal {
            time,
            moves,
            expected_moves,
            score,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) -> Option<PuzzleSolvedModalEvent> {
        egui::Modal::new(egui::Id::new("Puzzle Solved"))
            .show(ui.ctx(), |ui| {
                ui.set_width(200.0);
                ui.vertical_centered(|ui| {
                    ui.heading("Puzzle Solved");
                    ui.separator();
                    ui.add_space(15.0);
                    ui.vertical_centered(|ui| {
                        ui.style_mut().spacing.item_spacing.y = 10.0;
                        let seconds = self.time.as_secs();
                        let minutes = seconds / 60;
                        let rem_secs = seconds - minutes * 60;
                        ui.label(format!("Time {minutes:02}:{rem_secs:02}"));
                        ui.label(format!("Moves {}/{}", self.moves, self.expected_moves));
                        ui.label(format!("Score {}", self.score));
                    });
                    ui.add_space(15.0);
                    if ui
                        .add_sized([80., 30.], egui::Button::new("New Game"))
                        .clicked()
                    {
                        Some(PuzzleSolvedModalEvent::NewGame)
                    } else {
                        None
                    }
                })
                .inner
            })
            .inner
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PuzzleSolvedModalEvent {
    NewGame,
}

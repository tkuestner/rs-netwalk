use eframe::egui;

use netwalk::assets::Assets;
use netwalk::game::{Game, GameEvent};
use netwalk::modals::{NewGameModal, NewGameModalEvent};
use netwalk::puzzle::{self, Options};


fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Netwalk",
        native_options,
        Box::new(|cc| Ok(Box::new(Application::new(cc)))),
    )
}

struct Application {
    assets: Assets,
    state: ApplicationState,
    new_game_modal: NewGameModal,
}

impl Application {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        // Increasing the pixel per point results in a larger font, but also larger game objects.
        // cc.egui_ctx.set_pixels_per_point(1.25);

        // Increase the maximum click distance to enable faster play. The default value is 6.0.
        cc.egui_ctx
            .options_mut(|opts| opts.input_options.max_click_dist = 18.);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        let mut assets = Assets::new();
        assets.load_all(&cc.egui_ctx);

        Application {
            assets,
            state: ApplicationState::ShowingNewGameModal,
            new_game_modal: NewGameModal::new(Options::default()),
        }
    }

    fn start_new_game(&mut self, options: Options) {
        let puzzle = puzzle::Builder::new().with_options(options).build();
        let game = Game::new(puzzle, self.assets.clone());
        self.state = ApplicationState::RunningGame(Box::new(game));
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &mut self.state {
                ApplicationState::ShowingNewGameModal => {
                    if let Some(NewGameModalEvent::StartNewGame(options)) =
                        self.new_game_modal.update(ui)
                    {
                        self.start_new_game(options);
                    }
                }
                ApplicationState::RunningGame(game) => {
                    if let Some(event) = game.update(ui) {
                        match event {
                            GameEvent::Close => self.state = ApplicationState::ShowingNewGameModal,
                            GameEvent::NewGame => {
                                self.state = ApplicationState::ShowingNewGameModal
                            }
                            _ => (),
                        }
                    }
                }
            };
        });
    }
}

enum ApplicationState {
    ShowingNewGameModal,
    RunningGame(Box<Game>),
}

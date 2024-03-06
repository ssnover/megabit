use conway::BoardState;
use extism_pdk::*;
use megabit_app_sdk::{
    display::{self, Color},
    log::*,
    megabit_wasm_app, MegabitApp,
};

mod conway;

#[megabit_wasm_app]
struct GameOfLifeApp {
    state_a: BoardState,
    state_b: BoardState,
    show_state_a: bool,
    steps_without_change: u32,
}

impl MegabitApp for GameOfLifeApp {
    fn setup(display_cfg: display::DisplayConfiguration) -> FnResult<Self> {
        if display_cfg.is_rgb {
            display::set_monocolor_palette(Color::GREEN, Color::BLACK).unwrap();
        }

        let mut state_a = BoardState::new(display_cfg.width, display_cfg.height);
        let state_b = state_a.clone();

        state_a.randomize();

        Ok(Self {
            state_a,
            state_b,
            show_state_a: true,
            steps_without_change: 0,
        })
    }

    fn run(&mut self) -> FnResult<()> {
        // Show the last state calculated
        let (shown_state, working_state) = if self.show_state_a {
            (&self.state_a, &mut self.state_b)
        } else {
            (&self.state_b, &mut self.state_a)
        };
        display::write_region(
            (0, 0),
            (shown_state.width as u32, shown_state.height as u32),
            shown_state.state.clone(),
        )?;
        display::render(0..shown_state.height as u8)?;

        // Calculate the next state
        if self.steps_without_change == 8 {
            self.steps_without_change = 0;
            working_state.randomize();
            log(Level::Info, "Regenerating");
        } else {
            working_state.step(shown_state);
            let count_a = self.state_a.count_live_population();
            let count_b = self.state_b.count_live_population();
            log(Level::Debug, format!("Counts: {count_a}/{count_b}"));

            if count_a == count_b {
                self.steps_without_change += 1;
            } else {
                self.steps_without_change = 0;
            }
        }

        self.show_state_a = !self.show_state_a;

        Ok(())
    }
}

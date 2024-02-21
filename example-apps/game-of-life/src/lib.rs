use conway::BoardState;
use extism_pdk::*;
use megabit_app_sdk::{
    display::{self, Color},
    kv_store,
    log::*,
};

mod conway;

mod vars {
    pub const STATE_A: &str = "state_a";
    pub const STATE_B: &str = "state_b";
    pub const SHOW_STATE_A: &str = "show_state_a";
    pub const STEPS_WITHOUT_CHANGE: &str = "steps_without_change";
}

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    let display_cfg = display::get_display_info()?;

    if display_cfg.is_rgb {
        display::set_monocolor_palette(Color::GREEN, Color::BLACK).unwrap();
    }

    let mut state_a = BoardState::new(display_cfg.width, display_cfg.height);
    let state_b = state_a.clone();

    state_a.randomize();

    kv_store::write(vars::STATE_A, state_a)?;
    kv_store::write(vars::STATE_B, state_b)?;
    kv_store::write(vars::SHOW_STATE_A, true)?;
    kv_store::write(vars::STEPS_WITHOUT_CHANGE, 0u32)?;

    Ok(())
}

#[plugin_fn]
pub fn run() -> FnResult<()> {
    let mut state_a: BoardState = kv_store::read(vars::STATE_A)?.unwrap();
    let mut state_b: BoardState = kv_store::read(vars::STATE_B)?.unwrap();
    let show_state_a: bool = kv_store::read(vars::SHOW_STATE_A)?.unwrap();
    let mut steps_without_change: u32 = kv_store::read(vars::STEPS_WITHOUT_CHANGE)?.unwrap();

    // Show the last state calculated
    let (shown_state, working_state) = if show_state_a {
        (&state_a, &mut state_b)
    } else {
        (&state_b, &mut state_a)
    };
    display::write_region(
        (0, 0),
        (shown_state.width as u32, shown_state.height as u32),
        shown_state.state.clone(),
    )?;
    display::render(0..shown_state.height as u8)?;

    // Calculate the next state
    if steps_without_change == 8 {
        steps_without_change = 0;
        working_state.randomize();
        log(Level::Info, "Regenerating");
    } else {
        working_state.step(shown_state);
        let count_a = state_a.count_live_population();
        let count_b = state_b.count_live_population();
        log(Level::Debug, format!("Counts: {count_a}/{count_b}"));

        if count_a == count_b {
            steps_without_change += 1;
        } else {
            steps_without_change = 0;
        }
    }

    kv_store::write(vars::STATE_A, state_a)?;
    kv_store::write(vars::STATE_B, state_b)?;
    kv_store::write(vars::SHOW_STATE_A, !show_state_a)?;
    kv_store::write(vars::STEPS_WITHOUT_CHANGE, steps_without_change)?;

    Ok(())
}

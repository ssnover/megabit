use extism_pdk::*;
use megabit_app_sdk::{
    display::{self, simple::*},
    kv_store,
    log::*,
};
use rand::prelude::*;

type BoardState = [[u8; SCREEN_WIDTH / 8]; SCREEN_HEIGHT];

#[plugin_fn]
pub fn setup() -> FnResult<()> {
    let mut state_a = [[0u8; SCREEN_WIDTH / 8]; SCREEN_HEIGHT];
    let state_b = [[0u8; SCREEN_WIDTH / 8]; SCREEN_HEIGHT];

    randomize_state(&mut state_a);

    kv_store::write("state_a", state_a)?;
    kv_store::write("state_b", state_b)?;
    kv_store::write("show_state_a", true)?;
    kv_store::write("steps_without_change", 0u32)?;

    Ok(())
}

#[plugin_fn]
pub fn run() -> FnResult<()> {
    let mut state_a: BoardState = kv_store::read("state_a")?.unwrap();
    let mut state_b: BoardState = kv_store::read("state_b")?.unwrap();
    let show_state_a: bool = kv_store::read("show_state_a")?.unwrap();
    let mut steps_without_change: u32 = kv_store::read("steps_without_change")?.unwrap();

    // Show the last state calculated
    let (shown_state, working_state) = if show_state_a {
        (&state_a, &mut state_b)
    } else {
        (&state_b, &mut state_a)
    };
    let shown_state_data = shown_state
        .iter()
        .cloned()
        .map(|row| row.into_iter())
        .flatten()
        .collect::<Vec<u8>>();
    display::write_region(
        (0, 0),
        (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),
        shown_state_data,
    )?;
    display::render((0..SCREEN_HEIGHT as u8).into_iter().collect())?;

    // Calculate the next state
    if steps_without_change == 8 {
        steps_without_change = 0;
        randomize_state(working_state);
        log(Level::Info, "Regenerating");
    } else {
        step(working_state, shown_state);
        let count_a = count_live_population(&state_a);
        let count_b = count_live_population(&state_b);
        log(Level::Info, format!("Counts: {count_a}/{count_b}"));

        if count_a == count_b {
            steps_without_change += 1;
        } else {
            steps_without_change = 0;
        }
    }

    kv_store::write("state_a", state_a)?;
    kv_store::write("state_b", state_b)?;
    kv_store::write("show_state_a", !show_state_a)?;
    kv_store::write("steps_without_change", steps_without_change)?;

    Ok(())
}

fn get_cell(state: &BoardState, col: usize, row: usize) -> bool {
    (state[row][col / 8] & (1 << (col % 8))) != 0
}

fn set_cell(state: &mut BoardState, col: usize, row: usize, val: bool) {
    if val {
        state[row][col / 8] |= 1 << (col % 8);
    } else {
        state[row][col / 8] &= !(1 << (col % 8));
    }
}

fn step(next: &mut BoardState, prev: &BoardState) {
    for col in 0..SCREEN_WIDTH {
        for row in 0..SCREEN_HEIGHT {
            match (get_cell(prev, col, row), neighbors_alive(prev, col, row)) {
                (true, 2..=3) => set_cell(next, col, row, true),
                (false, 3) => set_cell(next, col, row, true),
                (true, _) => set_cell(next, col, row, false),
                (false, _) => set_cell(next, col, row, false),
            }
        }
    }
}

fn neighbors_alive(state: &BoardState, col: usize, row: usize) -> u8 {
    let lower_x = col.saturating_sub(1);
    let upper_x = core::cmp::min(col + 1, SCREEN_WIDTH - 1);
    let lower_y = row.saturating_sub(1);
    let upper_y = core::cmp::min(row + 1, SCREEN_HEIGHT - 1);

    let mut count = 0;
    for x in lower_x..=upper_x {
        for y in lower_y..=upper_y {
            if col == x && row == y {
                continue;
            }
            if get_cell(state, x, y) {
                count += 1;
            }
        }
    }
    count
}

fn count_live_population(state: &BoardState) -> u32 {
    state
        .iter()
        .map(|row| {
            row.iter()
                .map(|elem| {
                    (0..8)
                        .into_iter()
                        .map(|bit| if (*elem & (1 << bit)) != 0 { 1 } else { 0 })
                        .sum::<u32>()
                })
                .sum::<u32>()
        })
        .sum()
}

fn randomize_state(state: &mut BoardState) {
    let mut rng = rand::thread_rng();
    for row in state.iter_mut() {
        for elem in row {
            for bit in 0..8 {
                let val = rng.gen_range(0..4);
                if val == 3 {
                    *elem |= 1 << bit;
                }
            }
        }
    }
}

use matrix_display::MatrixDisplay;
use runner_ui::RunnerUi;
//use simulator_ui::SimulatorUi;
use yew::{function_component, html, Html, NodeRef};

mod matrix_display;
mod runner_ui;
mod simulator_ui;

#[function_component(ControlPage)]
pub fn control_page() -> Html {
    let node_ref = NodeRef::default();
    let div_ref = node_ref.clone();
    html! {
        <div class="container bg-dark" style="padding-top: 75px; padding-bottom: 20px; height: 100%">
            <div class="row align-items-center">
                <RunnerUi/>
            </div>
            <div class="row">
                <div class="col">
                </div>
                <div class="col-10">
                    <div class="row">
                        //<SimulatorUi/>
                    </div>
                    <div ref={node_ref} class="row" style="padding-top: 20px">
                        <MatrixDisplay {div_ref} />
                    </div>
                </div>
                <div class="col">
                </div>
            </div>
        </div>
    }
}

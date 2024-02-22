# megabit
A retro-style display which renders pixels at the behest of apps powered by WebAssembly!

### Hello World from Megabit (Simulator)
![Scrolling text saying "Hello world from Megabit" in red on black background](docs/assets/hello-megabit.gif)

### Hello World from Megabit (Monocolor Screen Hardware)
![Image of electronics and screen displaying message "Hello world from Megabit" in red pixels](docs/assets/hello-megabit.webp)

### More Examples
See the respective `README.md` of each Cargo workspace under `example-apps`.

## Components
For a more detailed write up of the design of Megabit and its components, see [architecture.md](docs/architecture.md)

* `app-sdk` - Rust library for developing Wasm apps for the Megabit display.
* `coproc-embassy` - Implementation of a coprocessor firmware for the nRF52840 using `embassy`.
* `example-apps` - A collection of example Wasm apps implemented in Rust.
* `runner` - The main Linux application responsible for executing the WebAssembly applications and sending commands to the coprocessor.
* `serial-protocol` - Common crate containing structures and code for the protocol used for intercommunication by the runner and the coprocessor.
* `simulator` - A web server and frontend application which emulate the display and a serial device using pseudoterminals.

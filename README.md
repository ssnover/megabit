# megabit
A retro-style display which renders pixels at the behest of apps powered by WebAssembly!

## Components
* `app-sdk` - Rust library for developing Wasm apps for the Megabit display.
* `coproc-embassy` - Implementation of a coprocessor firmware for the nRF52840 using `embassy`.
* `example-apps` - A collection of example Wasm apps implemented in Rust.
* `runner` - The main Linux application responsible for executing the WebAssembly applications and sending commands to the coprocessor.
* `serial-protocol` - Common crate containing structures and code for the protocol used for intercommunication by the runner and the coprocessor.
* `simulator` - A web server and frontend application which emulate the display and a serial device using pseudoterminals.

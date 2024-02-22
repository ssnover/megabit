## Megabit Architecture

Megabit is an experiment in creating a WebAssembly application sandbox for controlling a very specific function of hardware. I was initially inspired by the [Tidbyt](https://tidbyt.com), but I was miffed that it required a connection to the internet because the rendering happens in the cloud. To that end, I wanted to build a similar device where apps were executed locally on an embedded Linux device.

- [Hardware Components](#hardware-components)
- [Software Components](#software-components)
  - [Host](#host)
  - [Coprocessor](#coprocessor)
  - [Apps](#apps)
  - [Simulator](#simulator)

### Hardware Components

The chief hardware components are a single board computer (SBC) running Linux, a coprocessor running a bare metal firmware, and 64x32 RGB matrix display. The SBC will likely be a Raspberry Pi, but so far has just been my laptop. For a coprocessor, I'm using an Arduino Nano 33 BLE, which has an nRF52840 onboard. Finally, the screen is not a specific variety, these screens are commonly available from a number of distributors, including Adafruit.

The choice of SBC is primarily one of convenience, the only real constraint is that the instruction set of the hardware is supported by `wasmtime`/`cranelift`. For the time being that does not include AArch32 which unfortunately rules out the BeagleBone Black I was hoping to use. WebAssembly has truly been so performant that I don't think it'll be especially constrained on any modern Linux SBC. One bonus of using a Raspberry Pi over the Beaglebone Black is that it comes with WiFi and Bluetooth which means no wired connection is necessary and I don't need to figure out pushing Linux image updates over Bluetooth + USB.

The coprocessor is an nRF52840 which is also Bluetooth capable, but I'm just using it for convenience because I had a couple lying around and their footprint is relatively small. It's an ARM Cortex-M and a popular board so I thought originally that a firmware written with ZephyrRTOS in C++ would be a good fit, but that turned out to be a mess. I switched back to Rust with `embassy` and that's been very productive. The board is connected to the Linux system over USB as a CDC ACM which allows it to act like a serial device. It receives commands to update the display, executes them, then responds.

Finally there's the screen. I was originally using dot matrix displays for testing. These displays are little 8x8 pixel segments with a single state: on or off. This allowed writing some basic applications like a "Hello world" or Conway's game of life, but the truly interesting applications must run on an RGB screen. The screen I bought supports 4 channels of R, G, and B which is slightly less than the 5 that I'd like, but should still be sufficient. Driving these displays seems to be fairly complicated as the protocol requires a lot of pins (8), but I'm currently confident that I can get it working reliably at a decent frame rate if Adafruit can get this thing running with an Arduino Uno at all.

### Software Components
There's a number of pieces of software I'm writing and maintaining in this repo. Some of them are for running the core application, some are for users who want to write apps for Megabit, and some are testing tools which can be useful for both myself and users.

#### Host
The primary host process is the `megabit-runner`. It's currently responsible for finding the manifests for each installed app, creating and linking a sandbox for them, and then executing one app at a time. It maintains a serial connection to the coprocessor to allow apps to render through the APIs made available inside of the sandbox.

An application manifest is pretty small and straightforward right now, pretty much just pointing the runner in the direction of the WebAssembly binary and dictating the frequency that the app is executed. In the future I'd like to explore leverage the manifest to describe fine-grained permissions that are available similar to the permissions that a mobile app would have on a smartphone. A user could use a smartphone app or a web app to grant specific permissions to individual apps and those permissions would show up in the manifest. Currently, manifests are expected to be in directories under `$HOME/.megabit/<app>/manifest.json`.

The runner uses `extism` in order to build the sandbox and link host functionality including rendering, access to a semi-persistent data storage, and WASI APIs. Under the hood this is simplifying usage of the `wasmtime` API and makes it easier to write an SDK in multiple languages for building apps. It does hide a lot of details however and it may be necessary in the future to find ways to circumvent it (for fine-grained permissions to WASI APIs for example).

Some extensions to the execution sandbox I'm thinking about are the aforementioned fine-grained permissions which make Wasm so exciting, but also some safeguards in the process against eating tons of memory in the KV-store, restricting the filesystem access to a smaller subdirectory tree, and preventing the app from just executing forever (which currently would make it impossible to switch). It also needs to be able to dynamically reload the apps that are currently running on it and to be able to preview apps which haven't been installed yet. These latter features will require building out a means of listening and executing commands from a client, be it a mobile app or another external API mechanism.

#### Coprocessor
The coprocessor firmware is fairly straightforward. It's driven completely through a USB-Serial connection via a protocol defined in `serial-protocol`. Requests come in, they are routed and processed, and responses go back. It can also report events like a press of the hardware button which can be used for cycling through apps or the like.

I've been using `embassy` and it's async HALs and it's proven to be very productive compared to ZephyrRTOS and C++. USB serial just completely worked without complaint, though I'd still like to try to get `defmt` log messages available on the hardware UART pins as the board is currently annoying to program via JTAG/SWD. I'm also still getting a feel for how much headroom I have while programming in Rust in terms of stack memory space and code space. This microcontroller should be plenty capable.

#### Apps
There are a number of example applications which I've written and are each using an `app-sdk` library which define declarations of all of the host functions, but also nicer wrapper APIs and convenient structs for using `embedded-graphics` or using `serde` compatible types with the KV-store.

Apps currently have to use `extism` in order to bind their entrypoints, but I think this is an opportunity to define a trait which should be implemented and a proc-macro which generates the necessary bindings and then instantiates and calls the type that implements the trait via those entrypoints that the sandbox expects. Those entrypoints are a `setup` function for one-time initialization and a `run` function which is called periodically.

Some limitations that I'm not sure about related to the above and more generally are to what degree an app can persist memory between calls from the sandbox. It should be possible to define a global variable with `MaybeUninit` or something like that, but I don't know how much memory the application has and whether it would persist between calls.

#### Simulator
For my development, but also potentially in general, for convenience in testing apps without hardware, there is a simulator. The simulator's role is to run a web server on `localhost:8000` with `axum`, create a TCP listener at port `9009`, and talk to a frontend application over WebSockets. 

The web server loads a little frontend application which shows a simulated state for the display, some LEDs like those that exist on the hardware, and some buttons. One of these buttons reflects a hardware button, but the other two allow control of recording a GIF of what's currently on the screen.

At runtime, after running the simulator I use `socat` in order to create a pseudoterminal which is linked to that TCP listener. That allows running `megabit-runner` just the same as when it talks to the real coprocessor, with the only difference being the path to chardev.
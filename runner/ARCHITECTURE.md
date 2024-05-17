# megabit-runner Architecture
The nominal task of the megabit runner is to load an app into a WebAssembly sandbox and run it. It will continue to do this endlessly in a loop unless interrupted by an outside event.

This document attempts to capture my stream of consciousness on how the runner codebase needs to be molded in order to scalably support more features and prevent sliding into a ball-of-mud architecture. It does not yet reflect the actual architecture of the codebase today.

## Event Streams
There are currently three sources of events:
* WebSocket clients (the contents of these are encoded in the `runner_msgs` sub-crate)
* USB coproc messages (see `serial-protocol`)
* Filesystem events in the manifest directory tree from inotify

The goal is to create a command queue which combines these event sources such that they can each be handled one at a time and any responses can be routed back to their requester's origin.

### Sending Outgoing Events
Handling an event may result in a response that should go out of the same stream that a request was received on. Each stream will have a stream-specific data type along with information for routing a message (like a `client_id` for a websockets message).

## Command Queue
This will consist of a queue which accepts commands that are of a single enum type which can be stored in the queue and then handled in a synchronous context. The command queue will own a router which takes each command object and routes it to an appropriate handler.

### Command Router
The command router is owned by the command queue and it's primary role is to accept a command and send it to an appropriate handler. These handlers will be a collection of receiver handles which accept every command type via a handle and are handled by an actor task.

### Command Receivers
Command receivers will be specific to a specific subset of the functionality that can be requested or handled by a command. They accept commands through a channel and process them with the actor pattern. Handling commands may be handled in a handful of general ways, including one or a combination of the following:
* Altering the state of the runner or the related filesystem (application logic execution).
* Queueing a new command with more specific data.
* Sending a response on the origin stream.

Some example receivers:
* `CoprocEventReceiver`
  * Handles button presses from the coprocessor.
* `UiControlReceiver`
  * Handles requests to change app to next or previous, play or pause, etc.
* `AppLibraryReceiver`
  * Handles requests for information about applicatioons on the system or inotify events indicating a refresh is necessary.

## App Runner
The app runner is quite incongruous with this general architecture; it's running synchronous code and frequently blocks it's thread of execution. In order to accept requests to play/pause, go to next app, etc, it must periodically pause it's execution and check. This execution model requires spinning up a thread dedicated to the runner and linking it to a receiver via a channel which can accept events asynchronously and receives events synchronously.
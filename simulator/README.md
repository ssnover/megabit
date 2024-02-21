# Running the Simulator

The simulator can run as either a monocolor display or an RGB555 display (the default). Run the simulator simply with: `cargo run --bin megabit-coproc-simulator -- --is-monocolor <true|false>`.

Once the simulator is running it will listen on a TCP port. Next, to make it possible for the `megabit-runner` to communicate, we use `socat` to pipe a pseudoterminal to that TCP listener:

```sh
$ sudo socat -dd pty,link=/dev/ttyVA00,echo=0,perm=0777 TCP:0.0.0.0:9009,nodelay
```

This creates a pseudoterminal at `/dev/ttyVA00` which is connected full-duplex to the TCP listener at port 9009.
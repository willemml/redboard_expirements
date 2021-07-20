# Redboard RED-V Expirements

Currently just displays some text on an E-Ink display (waveshare 2.9" v2).

Requires the [JLinkGDBServer](https://www.segger.com/products/debug-probes/j-link/tools/j-link-gdb-server/about-j-link-gdb-server) and the `riscv-toolchain` (available for macOS [on homebrew](https://github.com/riscv/homebrew-riscv)).

You will also need to install  the correct rust build target by running `rustup target add riscv32imac-unknown-none-elf`.

To run/flash the program, run `JLinkGDBServer -device FE310 -if JTAG -speed 4000 -port 3333 -nogui` in one terminal window and then run `cargo run` in another.

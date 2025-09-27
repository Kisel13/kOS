# k/OS

> Languages: English | [Russian](readme.ru.md)



###### Fork of the Blog OS, operating system from [Writing an OS in Rust](https://os.phil-opp.com) series. See [LICENSE].

[LICENSE]: LICENSE
[LICENSE-GPL]: LICENSE-GPL

## Building

This project requires a nightly version of Rust because it uses some unstable features. Run `rustup update nightly --force` to update to the latest nightly.

To build project just run `cargo build`.

To create a bootable disk image from the compiled kernel, you need to install the [`bootimage`] tool:

[`bootimage`]: https://github.com/rust-osdev/bootimage

```
cargo install bootimage
```

After installing, you can create the bootable disk image by running:

```
cargo bootimage
```

This creates a bootable disk image in the `target/x86_64-kos/debug/bootimage-kos.bin`.

## Running

You can run the disk image in almost any x86_64 emulator.

`cargo run` runs [QEMU] VM.

[QEMU] and the [`bootimage`] tool need to be installed for this.

[QEMU]: https://www.qemu.org/

You can also write the image to an USB stick for booting it on a real machine. On Linux, the command for this is:

```
sudo dd if=target/x86_64-kos/debug/bootimage-kos.bin of=/dev/ conv=fsync status=progress
```

Where `sdX` is the device name of your USB stick. 

> **Everything on that device is overwritten!**

## Testing

To run the unit and integration tests, use `cargo test`.

## License

This fork is distributed under the terms of the
GNU General Public License, version 3.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

Read [LICENSE] and [LICENSE-GPL]

## Current state

- Kernel
  - [X] VGA text buffer driver
  - [X] Catching CPU exceptions
  - [X] Memory management, allocator
  - [X] Serial driver
  - [X] Keyboard
  - [ ] Mutitasking
    - [X] Async/Await implementation

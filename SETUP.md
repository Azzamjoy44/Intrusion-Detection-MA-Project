# Intrusion-Detection - SETUP
Follow these steps for setting up the project (this guide is for windows only)


## RP Pico W Firmware

1. Make sure you have installed the Rust [toolchain manager](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe),
if everything went well you should get the version of the toolchain in the terminal after running:

```powershell
rustup --version
```

2. Install the elf2uf2-rs tool to be able to flash the firmware to the RP Pico W, run this command in the terminal:

```powershell
cargo install elf2uf2-rs
```

3. Once you have cloned this repo, navigate to the code folder by running in the terminal

```powershell
cd path/to/Intrusion-Detection-MA-Project/code
```

4. Build the MCU's firmware by running

```powershell
cargo build --release --target thumbv6m-none-eabi
```

5. The KiCAD schematic provides all the necessary information on how to connect the hardware components.

6. Make sure that you have connected your Pico W to your PC via USB

7. Flash the program by running this command

```powershell
elf2uf2-rs -ds .\target\thumbv6m-none-eabi\release\pico_firmware
```

8. Now you can use the intrusion detection system.



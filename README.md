# rust_chip8
Another Chip 8 emulator in Rust

<img width="1279" alt="Screenshot 2024-06-04 at 16 15 28" 
src="https://github.com/lmarzocchetti/rust_chip8/assets/61746163/a9f3bfdf-dbb6-49b8-b127-cd45f70cac7f">

### Run
```
$ cargo run --release <path-to-rom>
```

### Informations
This Chip8 emulator is following strictly the original Cosmac VIP specification, so no support for SuperCHIP.

#### Keyboard
Uses the classical keyboard layout for emulation so:
| 1 | 2 | 3 | 4 |
|---|---|---|---|
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

#### Instruction/sec
I have set to 720 instruction/sec so the delay timer can decrease by one every 12 instruction

### Images and Videos

<img width="1279" alt="Screenshot 2024-06-04 at 16 15 54" src="https://github.com/lmarzocchetti/rust_chip8/assets/61746163/66b61af9-b02a-4288-90bc-412074824807">

https://github.com/lmarzocchetti/rust_chip8/assets/61746163/6cd5bd4e-7c36-4b83-9b1a-a3a6cd058c75

https://github.com/lmarzocchetti/rust_chip8/assets/61746163/cb914235-1a94-424d-9c23-d354ef64cfce

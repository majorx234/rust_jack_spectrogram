# Info
- a egui based dynamic spectrogram
- plotting real time frequency data
- ![Alt text](documentation/screenshot.png?raw=true "rust_jack_spectrogram with patchage in background")
- need jack server running (QtJackCtl,...)

# build
- run `cargo build`

# ToDo
- improve fft call (less copy)
- test performance: RustFFT vs RealFFT
- interacting in GUI
- stereo processing

# History
- 2023-03-03 plotting spectrogram
- 2023-01-16 plotting spectrum
- 2023-01-16 ringbuffer is transporting data between threads

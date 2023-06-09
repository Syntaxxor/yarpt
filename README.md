# yarpt
Yet Another Rust Path Tracer

This is a path tracing program I wrote in Rust. I've written quite a few before, but with this one, I focused on more proper program structure and flow, and avoided any necessity for unsafe code. I'm pretty satisfied with the results so far, and I'll probably keep working on it for a bit.

Features:
- Basic primitive rendering.
- Transforms to allow full translation, rotation, and scale.
- Human-readable scene representation and loading.
- Light transport via path tracing for robust shadows, reflections, and global illumination.
- Depth of Field.
- Multithreading.
- Denoising via OpenImageDenoise.
- BVH Acceleration structures.

To-do:
- Triangle meshes.

## Building
- Edit `.cargo/config.toml` to point to a local installation of OpenImageDenoise.
- In a console, use the command `cargo build`
  - `--release` flag recommended.
- Copy `OpenImageDenoise.dll` and `tbb12.dll` from the OpenImageDenoise `bin` folder into the folder with your built executable.
- Play with the settings, run the renderer, and save rendered images!

## Example Images
![rotated_cube_denoised](https://user-images.githubusercontent.com/25652538/232280373-174c7968-61c6-420c-992a-e164d573f50f.png)

![room_test](https://user-images.githubusercontent.com/25652538/233825017-2bf12ad8-769c-4dd5-adcf-7ac0cb37e6cc.png)

![cornell_box](https://user-images.githubusercontent.com/25652538/233825020-56cb4504-9943-48df-83d0-075175c7faa1.png)

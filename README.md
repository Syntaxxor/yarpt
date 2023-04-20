# yarpt
Yet Another Rust Path Tracer

This is a path tracing program I wrote in Rust. I've written quite a few before, but with this one, I focused on more proper program structure and flow, and avoided any necessity for unsafe code. I'm pretty satisfied with the results so far, and I'll probably keep working on it for a bit.

Features:
- Basic primitive rendering.
- Transforms to allow full translation, rotation, and scale.
- Light transport via path tracing for robust shadows, reflections, and global illumination.
- Depth of Field.
- Multithreading.
- Denoising via OpenImageDenoise.

To-do:
- Acceleration structures.
- Triangle meshes.
- Scene saving/loading (likely using an external program such as Blender).

## Building
- Edit `.cargo/config.toml` to point to a local installation of OpenImageDenoise.
- In a console, use the command `cargo build`
  - `--release` flag recommended.
- Copy `OpenImageDenoise.dll` and `tbb12.dll` from the OpenImageDenoise `bin` folder into the folder with your built executable.
- Play with the settings, run the renderer, and save rendered images!

## Example Images
![rotated_cube_denoised](https://user-images.githubusercontent.com/25652538/232280373-174c7968-61c6-420c-992a-e164d573f50f.png)

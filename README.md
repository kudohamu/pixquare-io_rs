# pixquare-io

A Crate for reading/writing [Pixquare](https://www.pixquare.art/) files.  
[Pixquare](https://www.pixquare.art/) is the awesome and feature-rich pixel art editor.

## Supported files

- pixquare file(.px)

```toml
# Cargo.toml
[dependencies]
pixquare-io = "0.1.0"
```

## Reading from a file

```rust,ignore
use pixquare::Artwork;

let data = std::fs::read("sprite.px")?;
let file = Artwork::read(&data)?;

println!("{:?}", file);
// => Artwork { id: "5A8E2912-414D-49F0-B1FA-4C27ABEAB5C2", canvas_size: Size { width: 64, height: 64 }, ... }
```

## Writing data to a file

⚠️ **CAUTION** ⚠️

pixquare-io **DOES NOT** automatically manage data consistency across fields.  
(For example, if you directly change a Layer's ID but do not update the corresponding Entry's ID.)  
Be sure to thoroughly familiarize yourself with [the official binary specs](https://docs.pixquare.art/pixquare-file/binary-specs) before using this crate, such as when overwriting existing .px files.  
I assume no responsibility for any issues that may arise from using this crate.

```rust,ignore
use pixquare::Artwork;

let artwork = Artwork { .. };

let mut buf = std::fs::File::create("artwork.px")?;
artwork.write(&mut buf)?;
```

## Frame image

You can get binary data of specified frame.
Note: Fx and post-processors are reproduced as closely as possible, and the result may differ from what is viewed in Pixquare or exported image directly from it.
Also, this method currently supports 1x scale output only, so Round Pixel and CRT effects are not supported.

```rust,ignore
let frame_index = 0;
let image_buf = artwork.get_frame_image(frame_index, LayerVisibility::Visible)?;
```

## Compatible Pixquare versions

| Pixquare version | `pixquare-io` version |
| :-- | :-- |
| ~2.72.0 | 0.1.0 |

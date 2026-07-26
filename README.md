# pixquare-io

A Crate for reading/writing pixquare files.

## Supported files

- pixquare file(.px)

```toml
# Cargo.toml
[dependencies]
pixquare = "0.1.0"
```

## Reading from a file

```rust
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

```rust
use pixquare::Artwork;

let artwork = Artwork { .. };

let mut buf = std::fs::File::create("foo.txt")?;
artwork.write(&mut buf)?;
```

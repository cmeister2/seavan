# seavan

seavan is a crate which wraps files in a container layer for later composition.

## Examples

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
  use seavan::Seavan;
  let wrap = Seavan::new("README.md")?
    .with_registry("acr.azurecr.io")?
    .with_tag("readme")?;

  /// This creates the image using Docker. The user must be able to run
  /// Docker commands.
  let repo_name_and_tag = wrap.create_image()?;
  Ok(())
}
```

## Design

seavan uses a temporary Dockerfile:
```Dockerfile
FROM scratch
COPY <file> /
```
and builds that Dockerfile using `docker build` with a derived image name, and
specified tag.

## But OCI artifacts!

At time of writing you can't mount an OCI artifact while building a Docker image, whereas you _can_ do:

```Dockerfile

# syntax=docker/dockerfile:1.2

FROM someimage

RUN --mount=type=bind target=/mnt/imagemount,from=seavanpkg/myrandomfile:1.2.3 rpm -ivh /mnt/imagemount

```

using Docker Buildkit to mount image layers at a specific directory, without having to copy those files into the image itself.

## Why seavan?

"seavan" is apparently another name for a shipping container - seemed like an apt choice!
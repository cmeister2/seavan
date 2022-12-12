# seavan

seavan is a crate which wraps files in a container layer for later composition.

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
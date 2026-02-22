# mslicer [![Build][actions-badge]][actions] ![][download-badge]

An experimental open-source slicer for masked stereolithography (resin) printers.
Supports the following output formats: Chitu (.ctb), Elegoo (.goo), NanoDLP (.nanodlp), and Vector (.svg).
You can read more about the development of this project on its [project page].
Often 20× to 120× faster than competing slicers, see the [benchmark results].

![][hero-image]

## Installation

You can download stable builds for Linux or Windows from the GitHub [Releases] page.
Stable Linux builds are also available on [Flathub][flathub] and [Nixpkgs][nixpkgs].
You can find the latest development builds for Windows, Linux, and Mac OS on [Github Actions][actions-success], just open the latest workflow run and download the correct artifact for your system.

[![][flathub-badge]][flathub]
[![][nixpkgs-badge]][nixpkgs]

If you would rather build from source, just have the latest stable version of the [Rust toolchain][rust] installed and build the binaries you want (mslicer, slicer) as shown below.

```sh
git clone https://github.com/connorslade/mslicer
cd mslicer
cargo b -r -p mslicer
```

## Demo Video

Here is a demo video showing mslicer being used to slice and print [Treefrog] by [Morena Protti].
The video is also hosted on YouTube ([here][demo-yt]) if the one below doesn't play.

<https://github.com/user-attachments/assets/3dfae18c-ffa2-4cc8-a322-ad2d5b38d31f>

## Related Projects

- [msla-thumbs] &mdash; Adds thumbnail support to KDE for sliced .goo, .ctb, and .nanodlp files
- [UVtools] &mdash; A great tool for inspecting and post processing sliced msla files of all kinds
- [Runebrace] &mdash; Closed-source support placement software. The recommended way to get support in mslicer for now

<!-- Links -->

[actions-success]: https://github.com/connorslade/mslicer/actions/workflows/build.yml?query=branch%3Amain%20is%3Asuccess
[actions]: https://github.com/connorslade/mslicer/actions/workflows/build.yml
[releases]: https://github.com/connorslade/mslicer/releases

[actions-badge]: https://github.com/connorslade/mslicer/actions/workflows/build.yml/badge.svg
[download-badge]: https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fconnorcode.com%2Fapi%2Fdownloads%3Fgithub%3Dconnorslade%252Fmslicer%26flathub%3Dcom.connorcode.mslicer&query=%24%5B%27total-human%27%5D&label=downloads&color=limegreen;

[hero-image]: https://github.com/user-attachments/assets/77c9263f-c0f8-445a-80b4-6f95d7532282
[demo-yt]: https://youtu.be/_Xu0jFAEYLc
[project page]: https://connorcode.com/projects/mslicer
[benchmark results]: https://files.connorcode.com/Documents/mslicer/speed-chart.pdf

[flathub-badge]: https://flathub.org/api/badge?svg&locale=en
[nixpkgs-badge]: https://raw.githubusercontent.com/dch82/Nixpkgs-Badges/main/nixpkgs-badge-dark.svg
[flathub]: https://flathub.org/apps/com.connorcode.mslicer
[nixpkgs]: https://search.nixos.org/packages?channel=unstable&show=mslicer&size=1&query=mslicer

[treefrog]: https://www.thingiverse.com/thing:18479
[Morena Protti]: https://www.thingiverse.com/morenap/designs

[rust]: https://rustup.rs
[msla-thumbs]: https://github.com/connorslade/msla-thumbs
[runebrace]: https://www.tarabella.it/Runebrace
[uvtools]: https://github.com/sn4k3/UVtools

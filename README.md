# mslicer [![Build][actions-badge]][actions] ![][download-badge] [![][release-badge]][releases]

A high-performance, open-source slicer for MSLA resin printers.
Supports the following output formats: Chitu (.ctb), Elegoo (.goo), NanoDLP (.nanodlp), and Vector (.svg).
Often 20× to 120× faster than competing slicers; see the [benchmark results].

![][hero-image]

## Installation

You can download stable builds for Linux, Windows, or macOS from the GitHub [Releases] page.
Stable Linux builds are also available on [Flathub][flathub] and [Nixpkgs][nixpkgs].
You can find the latest development builds on [GitHub Actions][actions-success]. Just open the latest workflow run and download the correct artifact for your system.

[![][github-badge]][releases]
[![][flathub-badge]][flathub]
[![][nixpkgs-badge]][nixpkgs]

If you would rather build from source, just install the latest stable version of the [Rust toolchain][rust] and build the binaries you want (e.g., mslicer, slicer) as shown below.

```sh
git clone https://github.com/connorslade/mslicer --branch v0.9.2
cd mslicer
cargo build --release --package mslicer
```

## Support

If you run into any issues, have any questions, or just want to suggest a feature, don't hesitate to reach out on [Zulip](https://mslicer.zulipchat.com) or create an [issue on GitHub](https://github.com/connorslade/mslicer/issues/new).

## Documentation

Below are a few of the most important documents from the [mslicer website](https://mslicer.com).
It might be a good idea to look through the getting started guide at least!

- [Getting Started][getting-started] &mdash; Everything you need to know about mslicer (start here)
- [Printer Compatibility][printer-compatibility] &mdash; Printers that are known to work with mslicer
- [Support Placement][support-placement] &mdash; Recommended method for adding supports to models before printing
- [Repairing Non-Manifold Meshes][non-manifold-meshes] &mdash; What to do if you get the non-manifold mesh warning
- [Release Notes][release-notes] &mdash; Overview of the biggest changes in a release

And here are some documents on some more niche use cases.

- [PCB Photolithography][pcb-photolighography] &mdash; Exposing photoresist and solder mask with an MSLA printer
- [3D Printing Phonograph Records][phonograph-record] &mdash; Using the power of modern technology to make really bad phonograph records

## Related Projects

- [msla-thumbs] &mdash; Adds thumbnail support to KDE for sliced .goo, .ctb, and .nanodlp files
- [UVtools] &mdash; A great tool for inspecting and post-processing sliced MSLA files of all kinds
- [Runebrace] &mdash; Closed-source support placement software. The recommended way to get supports into mslicer for now

<!-- Links -->

[actions-success]: https://github.com/connorslade/mslicer/actions/workflows/build.yml?query=branch%3Amain%20is%3Asuccess
[actions]: https://github.com/connorslade/mslicer/actions/workflows/build.yml
[releases]: https://github.com/connorslade/mslicer/releases

[actions-badge]: https://github.com/connorslade/mslicer/actions/workflows/build.yml/badge.svg
[download-badge]: https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fconnorcode.com%2Fapi%2Fdownloads%3Fgithub%3Dconnorslade%252Fmslicer%26flathub%3Dcom.connorcode.mslicer&query=%24%5B%27total-human%27%5D&label=downloads&color=limegreen;
[release-badge]: https://img.shields.io/github/v/release/connorslade/mslicer

[hero-image]: https://github.com/user-attachments/assets/c862edbe-1e08-49e9-b0e2-6e21488977d5
[benchmark results]: https://mslicer.com/docs/benchmark

[github-badge]: https://github.com/user-attachments/assets/3546dfe9-323b-4986-a9bb-c6af04792305
[flathub-badge]: https://flathub.org/api/badge?svg&locale=en
[nixpkgs-badge]: https://raw.githubusercontent.com/dch82/Nixpkgs-Badges/main/nixpkgs-badge-dark.svg
[flathub]: https://flathub.org/apps/com.connorcode.mslicer
[nixpkgs]: https://search.nixos.org/packages?channel=unstable&show=mslicer&size=1&query=mslicer

[getting-started]: https://mslicer.com/docs/getting-started
[printer-compatibility]: https://mslicer.com/docs/printer-compatibility
[support-placement]: https://mslicer.com/docs/support-placement
[non-manifold-meshes]: https://mslicer.com/docs/non-manifold-meshes
[release-notes]: https://mslicer.com/docs/release-notes
[pcb-photolighography]: https://mslicer.com/docs/pcb-photolighography
[phonograph-record]: https://mslicer.com/docs/phonograph-record

[rust]: https://rustup.rs
[msla-thumbs]: https://github.com/connorslade/msla-thumbs
[runebrace]: https://www.tarabella.it/Runebrace
[uvtools]: https://github.com/sn4k3/UVtools

# Keyboard Data Forge

This repository automates the generation of Japanese NLP resources for smartphone keyboard applications.

## Components

1.  **Mozc Dictionary for Vibrato**: Downloads the Google Mozc dictionary and formats it for use with the Vibrato tokenizer.
2.  **Wikipedia N-gram FST**: Downloads the Japanese Wikipedia dump, generates n-grams, and builds a Rust-compatible FST.

## Usage

Resources are built automatically via GitHub Actions and available as artifacts.

## License

### Code
The scripts and source code in this repository are licensed under the [MIT License](LICENSE).

### Data Artifacts
The generated data artifacts follow the licenses of their respective sources:

- **Mozc Dictionary**: Derived from [Google Mozc](https://github.com/google/mozc), licensed under **BSD-3-Clause**.
- **Wikipedia N-grams**: Derived from [Japanese Wikipedia](https://ja.wikipedia.org/), licensed under **CC BY-SA 3.0** (or later).

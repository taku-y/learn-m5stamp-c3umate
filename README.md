# learn-m5stamp-c3umate

## Project Setup

Each project in this repository is created using the [esp-idf-template](https://github.com/esp-rs/esp-idf-template).

### Environment Setup

Before building the projects, you need to configure the Rust toolchain.

```bash
rustup override set nightly
rustup toolchain install nightly --component rust-src
```

### Example of Creating a New Project

The following is the command and the options selected when creating the `wifi` project. Please refer to this when creating a new project.

```console
% cargo generate esp-rs/esp-idf-template cargo
✔ 🤷   Which MCU to target? · esp32c3
✔ 🤷   Configure advanced template options? · true
✔ 🤷   ESP-IDF version (master = UNSTABLE) · v5.3
✔ 🤷   Configure project to use Dev Containers (VS Code and GitHub Codespaces)? · false
✔ 🤷   Configure project to support Wokwi simulation with Wokwi VS Code extension? · false
✔ 🤷   Add CI files for GitHub Action? · false
```

### References

* [The Rust on ESP Book](https://docs.esp-rs.org/book/introduction.html)
* [Generating Projects from Templates](https://docs.esp-rs.org/book/writing-your-own-application/generate-project/index.html#generating-projects-from-templates)

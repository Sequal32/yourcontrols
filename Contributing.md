Currently, the codebase is very messy. I plan to refactor most of it as we make the switch to Tauri (a new UI framework).

# Setting up the Dev Enviornment

## Rust
Rust is the main language used in this program. Before contributing, make sure you have a solid understanding of Rust, which you can learn how to set up and use in the [Rust Book](https://doc.rust-lang.org/book/title-page.html).

Once you've cloned the repository, simply open a terminal and type
`cargo run` to compile and run the program.

I recommend using VSCode with the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) and [Crates](https://marketplace.visualstudio.com/items?itemName=serayuzgur.crates) extension, but you can use whatever you prefer.

## Build Dependencies
Clang is required to build the bindings to SimConnect. You can grab the latest LLVM from [here](https://releases.llvm.org/download.html), and add the bin folder to your `PATH` enviornmental variable.

SimConnect.dll is included in this repository.

# Pull Request Workflow
* Create your own fork of the repository.
* Commit regularly with small changes to your fork.
* When finished, open a [pull request](https://github.com/Sequal32/yourcontrols/compare) *clearly* describing every change you've made with a descriptive title.
* Wait for your PR to get reviewed and be prepared to answer any questions or make changes.
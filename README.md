# p-next

A pure Rust implementation of the [P](https://github.com/p-org/P) modeling language
toolchain: parser, type checker and model checker. No .NET, no code generation; P
models are checked directly by an interpreter driven by a systematic scheduler.

Crates:

* `pnext-syntax` – lexer, parser and AST.
* `pnext-sema` – name resolution, type checking, module system; produces a typed program.
* `pnext-checker` – values, interpreter, scheduling strategies, bug detection and traces.
* `pnext-compat` – drop-in compatibility with the upstream `p` tool: command line
  options, project files, output layout and file formats.
* `pnext-cli` – the `p` binary.

The core crates avoid file system, process and threading APIs so they can be built for
WebAssembly and used in a browser.

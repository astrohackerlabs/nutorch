---
title: "Programming in Nu"
description: "Programming in Nu"
order: 1027
section: "Book · Programming in Nu"
---

This chapter goes into more detail of Nushell as a programming language.
Each major language feature has its own section.

Just like most programming languages allow you to define functions, Nushell uses [custom commands](/docs/shell/book/custom_commands/) for this purpose.

From other shells you might be used to [aliases](/docs/shell/book/aliases/).
Nushell's aliases work in a similar way and are a part of the programming language, not just a shell feature.

Common operations, such as addition or regex search, can be done with [operators](/docs/shell/book/operators/).
Not all operations are supported for all data types, and Nushell will make sure to let you know when there is a mismatch.

You can store intermediate results to [variables](/docs/shell/book/variables/).
Variables can be immutable, mutable, or a parse-time constant.

The last three sections are aimed at organizing your code:

[Scripts](/docs/shell/book/scripts/) are the simplest form of code organization: You just put the code into a file and source it.
However, you can also run scripts as standalone programs with command line signatures using the "special" `main` command.

With [modules](/docs/shell/book/modules/), just like in many other programming languages, it is possible to compose your code from smaller pieces.
Modules let you define a public interface vs. private commands and you can import custom commands, aliases, and environment variables from them.

[Overlays](/docs/shell/book/overlays/) build on top of modules.
By defining an overlay, you bring in module's definitions into its own swappable "layer" that gets applied on top of other overlays.
This enables features like activating virtual environments or overriding sets of default commands with custom variants.

The standard library also has a [testing framework](/docs/shell/book/testing/) if you want to prove your reusable code works perfectly.

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

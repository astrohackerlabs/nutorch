---
title: "(Not so) Advanced"
description: "(Not so) Advanced"
order: 1057
section: "Book · (Not So) Advanced"
---

While the "Advanced" title might sound daunting and you might be tempted to skip this chapter, in fact, some of the most interesting and powerful features can be found here.

Besides the built-in commands, Nushell has a [standard library](/docs/shell/book/standard_library/).

Nushell operates on _structured data_.
You could say that Nushell is a "data-first" shell and a programming language.
To further explore the data-centric direction, Nushell includes a full-featured dataframe processing engine using [Polars](https://github.com/pola-rs/polars) as the backend.
Make sure to check the [Dataframes documentation](/docs/shell/book/dataframes/) if you want to process large data efficiently directly in your shell.

Values in Nushell contain some extra [metadata](/docs/shell/book/metadata/).
This metadata can be used, for example, to [create custom errors](/docs/shell/book/creating_errors/).

Thanks to Nushell's strict scoping rules, it is very easy to [iterate over collections in parallel](/docs/shell/book/parallelism/) which can help you speed up long-running scripts by just typing a few characters.

You can [interactively explore data](/docs/shell/book/explore/) with the [`explore`](/docs/shell/commands/explore/) command.

Finally, you can extend Nushell's functionality with [plugins](/docs/shell/book/plugins/).
Almost anything can be a plugin as long as it communicates with Nushell in a protocol that Nushell understands.

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

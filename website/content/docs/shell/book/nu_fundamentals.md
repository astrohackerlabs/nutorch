---
title: "Nu Fundamentals"
description: "Nu Fundamentals"
order: 1011
section: "Book · Nu Fundamentals"
---

This chapter explains some of the fundamentals of the Nushell programming language.
After going through it, you should have an idea how to write simple Nushell programs.

Nushell has a rich type system.
You will find typical data types such as strings or integers and less typical data types, such as cell paths.
Furthermore, one of the defining features of Nushell is the notion of _structured data_ which means that you can organize types into collections: lists, records, or tables.
Contrary to the traditional Unix approach where commands communicate via plain text, Nushell commands communicate via these data types.
All of the above is explained in [Types of Data](/docs/shell/book/types_of_data/).

[Loading Data](/docs/shell/book/loading_data/) explains how to read common data formats, such as JSON, into _structured data_. This includes our own "NUON" data format.

Just like Unix shells, Nushell commands can be composed into [pipelines](/docs/shell/book/pipelines/) to pass and modify a stream of data.

Some data types have interesting features that deserve their own sections: [strings](/docs/shell/book/working_with_strings/), [lists](/docs/shell/book/working_with_lists/), and [tables](/docs/shell/book/working_with_tables/).
Apart from explaining the features, these sections also show how to do some common operations, such as composing strings or updating values in a list.

Finally, [Command Reference](/docs/shell/commands/readme/) lists all the built-in commands with brief descriptions.
Note that you can also access this info from within Nushell using the [`help`](/docs/shell/commands/help/) command.

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

---
title: "Commands that interact with types"
description: "Commands that interact with types"
order: 1114
section: "Language"
---

The main type inspector in Nu is the `describe` command that
takes any data type on input and reports its type signature.

E.g.

```nu
[foo bar baz] | describe
# => list<string>
```

## Commands

- `describe`
- `inspect`
- `help`
- `into (subcommands)`
  - The into commands are used to cast one type into another.
- `ast`
  - In the branches of abstract syntax tree that describe the type of some element

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

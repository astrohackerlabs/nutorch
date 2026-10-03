---
title: "Understanding the difference between get and select"
description: "Understanding the difference between get and select"
order: 1069
section: "Language"
---

`get` extracts the value and returns a single value or list if multiple keys have been specified.
```nu
~> {a: 1, b: 2, c: 3} | get b
2
~> {a: 1, b: 2, c: 3} | get b c
╭───┬───╮
│ 0 │ 2 │
│ 1 │ 3 │
╰───┴───╯
```

`select` maintains the nushell values and returns the records selected by column name or key.
```nu
~> {a: 1, b: 2, c: 3} | select b
╭───┬───╮
│ b │ 2 │
╰───┴───╯
~> {a: 1, b: 2, c: 3} | select b c
╭───┬───╮
│ b │ 2 │
│ c │ 3 │
╰───┴───╯
```

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

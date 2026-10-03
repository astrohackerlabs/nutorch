---
title: "Flow Control"
description: "Flow Control"
order: 1072
section: "Language"
---


Nushell includes a number of flow control statements and expressions similar to other languages.

However, keep in mind that many Nushell operations will be performed using structured data as input and/or output. While structured data can be created using flow control statements in conjunction with mutable variables, a better solution in these cases is to use Filters.

See:

```nu
help commands | where category == filters
```

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

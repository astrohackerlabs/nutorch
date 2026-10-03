---
title: "Direnv"
description: "Direnv"
order: 1119
section: "Cookbook"
---


Many people use [direnv](https://direnv.net) to load an environment when entering a directory and unload it when exiting the directory.

## How direnv works

From [direnv.net](https://direnv.net):

> Before each prompt, direnv checks for the existence of a `.envrc` file (and optionally a `.env` file) in the current and parent directories. If the file exists (and is authorized), it is loaded into a bash sub-shell and all exported variables are then captured by direnv and then made available to the current shell.

## Configuring direnv in Nushell

In Nushell, it's possible to run direnv each time the prompt displays (using a [`pre_prompt` hook](/docs/shell/book/hooks/)). However, it's more efficient to update only when the directory is changed using an `env_change` hook along with:

- [`from json`](/docs/shell/commands/from_json/) to convert the direnv output to structured data
- [`load-env`](/docs/shell/commands/load-env/)
- An `env-conversion` helper for the `PATH` from the [Standard Library](/docs/shell/book/standard_library/)

> **note**
> 
> The direnv configuration below requires Nushell 0.104 or later.
> 


```nu
use std/config *

# Initialize the PWD hook as an empty list if it doesn't exist
$env.config.hooks.env_change.PWD = $env.config.hooks.env_change.PWD? | default []

$env.config.hooks.env_change.PWD ++= [{||
  if (which direnv | is-empty) {
    # If direnv isn't installed, do nothing
    return
  }

  direnv export json | from json | default {} | update cells --columns [ PATH ] {
    # If direnv changes the PATH, it will become a string and we need to re-convert it to a list
    do (env-conversions).path.from_string $in
  } | load-env
}]
```

As with other configuration changes, this can be made permanent by adding it to your startup [configuration](/docs/shell/book/configuration/).

With this in place, direnv will now add/remove environment variables when entering/leaving a directory with an `.envrc` or `.env` file.

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

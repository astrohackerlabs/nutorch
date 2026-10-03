---
title: "Nu Map from Functional Languages"
description: "Nu Map from Functional Languages"
order: 1052
section: "Book · Coming to Nu"
---

The idea behind this table is to help you understand how Nu builtins and plugins relate to functional languages. We've tried to produce a map of relevant Nu commands and what their equivalents are in other languages. Contributions are welcome.

Note: this table assumes Nu 0.43 or later.

| Nushell                                        | Clojure                      | Tablecloth (Ocaml / Elm)        | Haskell                  |
|------------------------------------------------|------------------------------|---------------------------------|--------------------------|
| [append](/docs/shell/commands/append/)           | conj, into, concat           | append, (++), concat, concatMap | (++)                     |
| [into binary](/docs/shell/commands/into_binary/) | Integer/toHexString          |                                 | showHex                  |
| [length](/docs/shell/commands/length/)           | count                        | length, size                    | length, size             |
| [date](/docs/shell/commands/date/)               | java.time.LocalDate/now      |                                 |                          |
| [each](/docs/shell/commands/each/)               | map, mapv, iterate           | map, forEach                    | map, mapM                |
| [exit](/docs/shell/commands/each/)               | System/exit                  |                                 |                          |
| [first](/docs/shell/commands/first/)             | first                        | head                            | head                     |
| [format](/docs/shell/commands/format/)           | format                       |                                 | Text.Printf.printf       |
| [group-by](/docs/shell/commands/group-by/)       | group-by                     |                                 | group, groupBy           |
| [help](/docs/shell/cookbook/help/)                    | doc                          |                                 |                          |
| [is-empty](/docs/shell/commands/is-empty/)       | empty?                       | isEmpty                         |                          |
| [last](/docs/shell/commands/last/)               | last, peek, take-last        | last                            | last                     |
| [lines](/docs/shell/commands/lines/)             |                              |                                 | lines, words, split-with |
| [match](/docs/shell/commands/match/)             |                              | match (Ocaml), case (Elm)       | case                     |
| [select](/docs/shell/commands/select/)           | nth                          | Array.get                       | lookup                   |
| [open](/docs/shell/commands/open/)               | with-open                    |                                 |                          |
| [transpose](/docs/shell/commands/transpose/)     | (apply mapv vector matrix)   |                                 | transpose                |
| [prepend](/docs/shell/commands/prepend/)         | cons                         | cons, ::                        | ::                       |
| [print](/docs/shell/commands/print/)             | println                      |                                 | putStrLn, print          |
| [slice](/docs/shell/commands/slice/), 1..10      | range                        | range                           | 1..10, 'a'..'f'          |
| [reduce](/docs/shell/commands/reduce/)           | reduce, reduce-kv            | foldr                           | foldr                    |
| [reverse](/docs/shell/commands/reverse/)         | reverse, rseq                | reverse, reverseInPlace         | reverse                  |
| [select](/docs/shell/commands/select/)           | select-keys                  |                                 |                          |
| [shuffle](/docs/shell/commands/shuffle/)         | shuffle                      |                                 |                          |
| [length](/docs/shell/commands/length/)           | count                        |                                 | size, length             |
| [skip](/docs/shell/commands/skip/)               | rest                         | tail                            | tail                     |
| [skip until](/docs/shell/commands/skip_until/)   | drop-while                   |                                 |                          |
| [skip while](/docs/shell/commands/skip_while/)   | drop-while                   | dropWhile                       | dropWhile, dropWhileEnd  |
| [sort-by](/docs/shell/commands/sort-by/)         | sort, sort-by, sorted-set-by | sort, sortBy, sortWith          | sort, sortBy             |
| [split row](/docs/shell/commands/split_row/)     | split, split-{at,with,lines} | split, words, lines             | split, words, lines      |
| [str](/docs/shell/commands/str/)                 | clojure.string functions     | String functions                |                          |
| [str join](/docs/shell/commands/str_join/)       | join                         | concat                          | intercalate              |
| [str trim](/docs/shell/commands/str_trim/)       | trim, triml, trimr           | trim, trimLeft, trimRight       | strip                    |
| [math sum](/docs/shell/commands/math_sum/)       | apply +                      | sum                             | sum                      |
| [take](/docs/shell/commands/take/)               | take, drop-last, pop         | take, init                      | take, init               |
| [take until](/docs/shell/commands/take_until/)   | take-while                   | takeWhile                       | takeWhile                |
| [take while](/docs/shell/commands/take_while/)   | take-while                   | takeWhile                       | takeWhile                |
| [uniq](/docs/shell/commands/uniq/)               | set                          | Set.empty                       | Data.Set                 |
| [where](/docs/shell/commands/where/)             | filter, filterv, select      | filter, filterMap               | filter                   |

Ported from [nushell/nushell.github.io](https://github.com/nushell/nushell.github.io). Copyright (c) 2021 Nushell Project. MIT License.

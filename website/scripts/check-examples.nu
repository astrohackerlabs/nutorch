# Execute the website's exact native teaching examples with independent oracles.
# Reference signatures and host setup are checked separately, never executed here.
use std/assert

def main [] {
  let website = ($env.FILE_PWD | path dirname)
  let binary = ($website | path join ../../rs/target/release/nutorch | path expand)
  let scratch = (mktemp --directory)
  mkdir ($scratch | path join nushell)
  let cases = [
    {page: getting-started, expected: [[1.0 2.0 3.0] [2.0 4.0 6.0] [2.0 4.0 6.0]]}
    {page: tensors, expected: [[2 2] [[2.0 4.0] [6.0 8.0]] [9007199254740993 -7]]}
    {page: autograd, expected: [[2.0 4.0 6.0]]}
    {page: ops, expected: [[1.0 2.0 3.0 4.0 5.0 6.0] [2.0 2.0 3.0] [[1.0 2.0] [3.0]]]}
    {page: neural-networks, expected: [null [0 1] null]}
    {page: nushell, expected: [[[7.0 10.0] [15.0 22.0]]]}
  ]
  let prelude = 'use std/assert'
  let covered = (glob ($website | path join content/docs/*.md) | where {|file| open --raw $file | str contains ("```nu" + (char nl) + "use torch")} | path basename | str replace '.md' '' | sort)
  assert equal $covered ($cases.page | sort) 'Unaccounted narrative page'
  try {
    for case in $cases {
      let markdown = (open --raw ($website | path join content/docs $"($case.page).md"))
      let blocks = ($markdown | parse --regex '(?s)```nu\n(?P<code>.*?)\n```' | get code | where {|code| $code | str starts-with 'use torch'})
      assert equal ($blocks | length) ($case.expected | length) $"Unaccounted teaching blocks: ($case.page)"
      mut program = $prelude
      for entry in ($blocks | enumerate) {
        let lines = ($entry.item | lines)
        let body = ($lines | drop 1 | str join (char nl))
        let final = ($lines | last)
        let expected = ($case.expected | get $entry.index)
        $program += $"(char nl)($body)(char nl)let observed_($entry.index) = \(($final)\)(char nl)"
        if $expected != null {
          $program += $"let data_($entry.index) = \(if \($observed_($entry.index) | describe\) == tensor { $observed_($entry.index) | torch value } else { $observed_($entry.index) }\)(char nl)assert equal $data_($entry.index) ($expected | to nuon)(char nl)"
        }
      }
      if $case.page == neural-networks {
        $program += 'assert equal ($parameters | each {torch shape $in}) [[8 2] [8]]'
      }
      if $case.page == autograd {
        $program = ($program | str replace 'torch zero_grad $x' ('assert equal ($x | torch grad | torch value) [4.0 8.0 12.0]' + (char nl) + 'torch zero_grad $x'))
        $program += ((char nl) + 'assert equal ($x | torch grad | torch value) [0.0 0.0 0.0]')
      }
      if $case.page == tensors {
        $program += ((char nl) + 'assert equal ($flags | torch value --meta) {dtype: bool, shape: [2], data: [true false]}' + (char nl) + 'assert equal $exported {dtype: int64, shape: [2], data: [9007199254740993 -7]}')
      }
      let script = ($scratch | path join example.nu)
      $program | save --force $script
      let result = (with-env {XDG_CONFIG_HOME: $scratch, TMPDIR: $scratch} { cd $scratch; ^$binary --no-config-file --no-history $script } | complete)
      assert equal $result.exit_code 0 $"($case.page): ($result.stderr)"
      if $case.page == neural-networks { assert equal ($result.stdout | str trim) 'true' 'Checkpoint predictions differ' }
      print $"Pass: ($case.page) — ($blocks | length) native examples"
    }
    let home = (open --raw ($website | path join app/routes/home.tsx))
    let expected = {heroDemo: [5.0 7.0 9.0], nuDemo: [[7.0 10.0] [15.0 22.0]], trainDemo: [2.0 4.0 6.0]}
    let demos = ($home | parse --regex '(?s)const (?P<name>\w+) = `(?P<code>.*?)`;' | where {|demo| $demo.code | str starts-with 'use torch'})
    assert equal ($demos.name | sort) ($expected | columns | sort) 'Unaccounted homepage example'
    for demo in $demos {
      let program = $"($prelude)(char nl)let actual = \(do { ($demo.code)(char nl) }\)(char nl)assert equal $actual ($expected | get $demo.name | to nuon)"
      let script = ($scratch | path join home.nu)
      $program | save --force $script
      let result = (with-env {XDG_CONFIG_HOME: $scratch} { ^$binary --no-config-file --no-history $script } | complete)
      assert equal $result.exit_code 0 $"($demo.name): ($result.stderr)"
      print $"Pass: homepage ($demo.name)"
    }
    # The setup guide also launches these complete programs. Each checks its
    # own loss and learned parameters or predictions against independent targets.
    for name in [train-regression train-classify] {
      let script = ($website | path join ../../examples $"($name).nu" | path expand)
      let result = (with-env {XDG_CONFIG_HOME: $scratch, TMPDIR: $scratch} { cd $scratch; ^$binary --no-config-file --no-history $script } | complete)
      assert equal $result.exit_code 0 $"($name): ($result.stderr)"
      assert ($result.stdout | str contains 'PASS:') $"Missing training assertions: ($name)"
      print ($result.stdout | str trim)
    }
  } catch {|err|
    rm --recursive $scratch
    error make {msg: $err.msg}
  }
  rm --recursive $scratch
}

# Frozen accepted source contract, independent of the candidate under test.
def main [] {
  let baseline = (open scripts/accepted-site.json)
  let root = ($env.PWD | path expand)
  let files = ([...(glob content/docs/**/*.md) ...(glob public/**/*) ('app/styles/global.css' | path expand)] | where {|p| ($p | path type) == file} | each {|p| $p | path expand | path relative-to $root} | sort)
  if $files != ($baseline.sources | columns | sort) { error make {msg: 'Accepted source file set differs'} }
  for file in $files {
    # The source export relocates only this scan directive. Normalize that
    # explicit path, not arbitrary CSS, so the same style oracle works there.
    let bytes = if $file == 'app/styles/global.css' {
      open --raw $file | str replace '@source "../../vendor/astrohacker-ui/src";' '@source "../../../../../astrohacker/ts/ui/src";'
    } else { open --raw $file }
    if ($bytes | hash sha256) != ($baseline.sources | get $file) { error make {msg: $"Accepted content differs: ($file)"} }
  }
  print $"Pass: ($files | length) source files match accepted baseline ($baseline.commit)"
}

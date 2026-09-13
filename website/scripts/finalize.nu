# Only generated local build artifacts are written here.
def main [] {
  let root = 'build/client'
  open --raw ($root | path join '404/index.html') | save --force ($root | path join '404.html')
  let docs = (glob content/docs/**/*.md | each {|p| $"https://nutorch.com/docs/($p | path expand | path relative-to ('content/docs' | path expand) | str replace -r '\.md$' '')/"})
  let urls = ([https://nutorch.com/] | append $docs | sort | each {|url| $"<url><loc>($url)</loc></url>"} | str join '')
  $'<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">($urls)</urlset>' | save --force ($root | path join sitemap-0.xml)
  '<?xml version="1.0" encoding="UTF-8"?><sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><sitemap><loc>https://nutorch.com/sitemap-0.xml</loc></sitemap></sitemapindex>' | save --force ($root | path join sitemap-index.xml)
}

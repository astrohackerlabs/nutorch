import type { MetaDescriptor } from "react-router";

export function metadata(
  title: string,
  description: string,
  path: string,
  article = false,
): MetaDescriptor[] {
  const url = `https://nutorch.com${path}`;
  return [
    { title },
    { name: "description", content: description },
    ...(article ? [{ tagName: "link", rel: "canonical", href: url }] : []),
    { property: "og:title", content: title },
    { property: "og:description", content: description },
    { property: "og:type", content: article ? "article" : "website" },
    { property: "og:url", content: url },
    { property: "og:site_name", content: "NuTorch" },
    {
      property: "og:image",
      content: "https://nutorch.com/images/og-nutorch.png",
    },
    { name: "twitter:card", content: "summary_large_image" },
  ];
}

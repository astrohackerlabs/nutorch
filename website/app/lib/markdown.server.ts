import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from "remark-gfm";
import remarkSmartypants from "remark-smartypants";
import remarkRehype from "remark-rehype";
import rehypeSlug from "rehype-slug";
import rehypeStringify from "rehype-stringify";
import { codeToHast, codeToHtml } from "shiki";
import type { Root, Element } from "hast";

const themes = { light: "tokyo-night", dark: "tokyo-night" };
export async function highlight(code: string, lang: string): Promise<string> {
  return (await codeToHtml(code, { lang, themes }))
    .replace('class="shiki ', 'class="astro-code ')
    .replace("<pre ", `<pre data-language="${lang}" `);
}
function syntax(): (tree: Root) => Promise<void> {
  return async (tree: Root): Promise<void> => {
    async function walk(node: Root | Element): Promise<void> {
      for (let i = 0; i < node.children.length; i++) {
        const child = node.children[i];
        if (child?.type !== "element") continue;
        const code = child.children[0];
        if (
          child.tagName === "pre" &&
          code?.type === "element" &&
          code.tagName === "code"
        ) {
          const lang = (
            code.properties.className?.find((c) => c.startsWith("language-")) ??
            "language-text"
          ).slice(9);
          const text = code.children
            .map((c) => (c.type === "text" ? c.value : ""))
            .join("")
            .replace(/\n$/, "");
          const rendered = await codeToHast(text, { lang, themes });
          const pre = rendered.children[0];
          if (pre?.type !== "element" || pre.tagName !== "pre") {
            throw new Error("Syntax highlighter did not return a pre element");
          }
          delete pre.properties.class;
          pre.properties.className = [
            "astro-code",
            "shiki-themes",
            "tokyo-night",
          ];
          pre.properties.dataLanguage = lang;
          node.children[i] = pre;
        } else await walk(child);
      }
    }
    await walk(tree);
  };
}
export async function renderMarkdown(content: string): Promise<string> {
  return String(
    await unified()
      .use(remarkParse)
      .use(remarkGfm)
      .use(remarkSmartypants)
      .use(remarkRehype)
      .use(rehypeSlug)
      .use(syntax)
      .use(rehypeStringify)
      .process(content),
  );
}

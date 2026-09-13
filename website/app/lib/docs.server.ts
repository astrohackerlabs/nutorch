import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import matter from "gray-matter";

const directory = fileURLToPath(
  new URL("../../content/docs/", import.meta.url),
);
export const documents = readdirSync(directory, { recursive: true })
  .map(String)
  .filter((name) => name.endsWith(".md"))
  .map((name) => {
    const { data, content } = matter(
      readFileSync(`${directory}/${name}`, "utf8"),
    );
    if (
      typeof data.title !== "string" ||
      typeof data.description !== "string" ||
      typeof data.order !== "number"
    ) {
      throw new Error(`Invalid document frontmatter: ${name}`);
    }
    return {
      slug: name.replace(/\.md$/, ""),
      title: data.title,
      description: data.description,
      order: data.order,
      section: (data.section ?? "") as string,
      content,
    };
  })
  .sort((a, b) => a.order - b.order);

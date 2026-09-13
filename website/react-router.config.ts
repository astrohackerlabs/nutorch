import type { Config } from "@react-router/dev/config";
import { documents } from "./app/lib/docs.server";

export default {
  ssr: false,
  prerender: [
    "/",
    "/docs/",
    ...documents.map((doc) => `/docs/${doc.slug}/`),
    "/404",
  ],
} satisfies Config;

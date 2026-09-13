import { type RouteConfig, index, route } from "@react-router/dev/routes";
export default [
  index("routes/home.tsx"),
  route("docs/*", "routes/docs.tsx"),
  route("*", "routes/not-found.tsx"),
] satisfies RouteConfig;

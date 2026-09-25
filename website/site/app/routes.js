import { index, layout, route } from "@react-router/dev/routes";

export default [
  // Everything with the floating tab bar.
  layout("routes/tabs.jsx", [
    index("routes/home.jsx"),
    route("learn", "routes/learn.jsx"),
    route("examples", "routes/examples.jsx", [
      // An example opens as a bottom sheet over the grid, at its own URL.
      route(":id", "routes/example.jsx"),
    ]),
    route("more", "routes/more.jsx"),
    route("download", "routes/download.jsx"),
    route("licence", "routes/licence.jsx"),
  ]),
  route("*", "routes/not-found.jsx"),
];

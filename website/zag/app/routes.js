import { index, route } from "@react-router/dev/routes";

export default [index("routes/home.jsx"), route("more", "routes/more.jsx"), route("examples/:id?", "routes/examples.jsx"), route("*", "routes/not-found.jsx")];

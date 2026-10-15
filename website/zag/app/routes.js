import { index, route } from "@react-router/dev/routes";

export default [index("routes/home.jsx"), route("more", "routes/more.jsx"), route("*", "routes/not-found.jsx")];

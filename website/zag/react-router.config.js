// Every page is built to HTML ahead of time; there is no server.
export default {
  ssr: false,
  async prerender() {
    return ["/"];
  },
};

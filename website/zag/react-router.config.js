// Every page is built to HTML ahead of time; there is no server. /404 becomes the page the host
// shows for any address it has nothing for (scripts/post-build.mjs).
export default {
  ssr: false,
  async prerender() {
    return ["/", "/404"];
  },
};

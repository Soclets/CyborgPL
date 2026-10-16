// Every request, to the same path and query on CyborgPL's home. 301: permanent, so browsers and
// search engines move to cyborgpl.dev.
export default {
  fetch(request) {
    const url = new URL(request.url);
    url.protocol = "https:";
    url.hostname = "cyborgpl.dev";
    url.port = "";
    return Response.redirect(url.toString(), 301);
  },
};

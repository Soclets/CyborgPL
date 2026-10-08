export default {
  fetch(request) {
    const url = new URL(request.url);
    url.hostname = 'nerve-lang.com';
    url.protocol = 'https:';
    return Response.redirect(url.toString(), 301);
  },
};

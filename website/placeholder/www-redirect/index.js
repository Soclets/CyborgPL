export default {
  fetch(request) {
    const url = new URL(request.url);
    url.hostname = 'cyborgpl.dev';
    url.protocol = 'https:';
    return Response.redirect(url.toString(), 301);
  },
};

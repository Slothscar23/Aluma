// Very small service worker that caches core APP files for offline use and responds with cached versions
const CACHE_NAME = 'aluma-app-v1';
const FILES_TO_CACHE = [
  '/APP/index.html',
  '/APP/app.js',
  '/APP/style.css',
  '/APP/manifest.json'
];

self.addEventListener('install', (evt) => {
  evt.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(FILES_TO_CACHE))
  );
  self.skipWaiting();
});

self.addEventListener('activate', (evt) => {
  evt.waitUntil(self.clients.claim());
});

self.addEventListener('fetch', (evt) => {
  evt.respondWith(
    caches.match(evt.request).then((resp) => resp || fetch(evt.request))
  );
});

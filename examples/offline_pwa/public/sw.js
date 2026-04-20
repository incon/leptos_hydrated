const VERSION = '{{VERSION}}';
const CACHE_NAME = `{{OUTPUT_NAME}}-${VERSION}`;
const ASSETS = [
  '/',
  '/manifest.json',
  '/pkg/{{OUTPUT_NAME}}.js',
  '/pkg/{{OUTPUT_NAME}}.wasm',
  '/pkg/{{OUTPUT_NAME}}.css',
  '/icon.svg?v={{VERSION}}',
];

self.addEventListener('install', (event) => {
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.addAll(ASSETS);
    })
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME) {
            console.log('SW: Cleaning up old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    (async () => {
      // 1. Try network first
      try {
        const response = await fetch(event.request);
        if (response && response.ok) {
          return response;
        }
      } catch (e) {
        // Network failure
      }

      // 2. Asset/Navigation fallback
      const cachedResponse = await caches.match(event.request);
      if (cachedResponse) {
        return cachedResponse;
      }

      // 3. For navigation requests, fallback to the shell (root)
      if (event.request.mode === 'navigate') {
        const rootCached = await caches.match('/');
        if (rootCached) {
          return rootCached;
        }
      }

      return new Response('Offline and not cached', { status: 503 });
    })()
  );
});

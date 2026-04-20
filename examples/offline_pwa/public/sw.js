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
  // Force the waiting service worker to become the active service worker.
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
      // 1. Try network first for development or to get the latest version
      try {
        const networkResponse = await fetch(event.request);
        if (networkResponse && networkResponse.status === 200) {
          return networkResponse;
        }
      } catch (e) {
        // Network failure, move to cache
      }

      // 2. Try to match the request in the cache
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

      // 4. Final fallback for other assets
      return new Response('Offline and not in cache', {
        status: 503,
        statusText: 'Service Unavailable',
        headers: { 'Content-Type': 'text/plain' }
      });
    })()
  );
});

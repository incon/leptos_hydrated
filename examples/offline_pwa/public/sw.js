const VERSION = '{{VERSION}}';
const CACHE_NAME = `{{OUTPUT_NAME}}-${VERSION}`;
const OFFLINE_URL = '/offline.html';

const ASSETS = [
  // '/',             // We remove this to force fallback to OFFLINE_URL for navigations
  OFFLINE_URL,        // This is your CSR shell
  '/manifest.json',
  '/pkg/{{OUTPUT_NAME}}.js?v={{VERSION}}',
  '/pkg/{{OUTPUT_NAME}}.wasm?v={{VERSION}}',
  '/pkg/{{OUTPUT_NAME}}.css?v={{VERSION}}',
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
      // 1. Try network first (Network-First Strategy)
      try {
        const response = await fetch(event.request);
        // Only return if it's a valid successful response
        if (response && response.status === 200) {
          return response;
        }
      } catch (e) {
        // Network failure or offline
      }

      // 2. Exact match check (for JS, WASM, CSS, Icons)
      const cachedResponse = await caches.match(event.request);
      if (cachedResponse) {
        return cachedResponse;
      }

      // 3. Navigation Fallback (THE FIX)
      // If the user is navigating to a page (e.g., /settings) and we are offline,
      // serve the clean CSR shell so Leptos can mount without hydration errors.
      if (event.request.mode === 'navigate') {
        const offlineShell = await caches.match(OFFLINE_URL);
        if (offlineShell) return offlineShell;
      }

      return new Response('Offline and not cached', { status: 503 });
    })()
  );
});

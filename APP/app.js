// Simple IndexedDB helper for storing a document under profiles/default/data
(function(){
  const DB_NAME = 'aluma-db';
  const STORE = 'kv';
  let dbPromise = null;

  function openDB() {
    if (dbPromise) return dbPromise;
    dbPromise = new Promise((resolve, reject) => {
      const req = indexedDB.open(DB_NAME, 1);
      req.onupgradeneeded = (e) => {
        const db = e.target.result;
        if (!db.objectStoreNames.contains(STORE)) {
          db.createObjectStore(STORE);
        }
      };
      req.onsuccess = (e) => resolve(e.target.result);
      req.onerror = (e) => reject(e.target.error);
    });
    return dbPromise;
  }

  async function put(key, value) {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction([STORE], 'readwrite');
      const s = tx.objectStore(STORE);
      const req = s.put(value, key);
      req.onsuccess = () => resolve(true);
      req.onerror = () => reject(req.error);
    });
  }

  async function get(key) {
    const db = await openDB();
    return new Promise((resolve, reject) => {
      const tx = db.transaction([STORE], 'readonly');
      const s = tx.objectStore(STORE);
      const req = s.get(key);
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
  }

  async function init() {
    document.getElementById('save-local').addEventListener('click', async () => {
      const content = document.getElementById('editor').value;
      try {
        await put('profiles/default/data', content);
        document.getElementById('status').innerText = 'Saved locally to IndexedDB.';
      } catch (e) {
        document.getElementById('status').innerText = 'Local save error: ' + e;
      }
    });

    document.getElementById('save-remote').addEventListener('click', async () => {
      const content = document.getElementById('editor').value;
      const payload = {
        path: 'APP/data.json',
        content: content,
        message: 'Save from Aluma PWA (requested)'
      };
      try {
        const res = await fetch('http://127.0.0.1:7878/save', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
        const text = await res.text();
        document.getElementById('status').innerText = 'Remote save response: ' + text + ' (status ' + res.status + ')';
      } catch (e) {
        document.getElementById('status').innerText = 'Remote save error (no bootstrap?): ' + e;
      }
    });

    // try to load existing content
    try {
      const existing = await get('profiles/default/data');
      if (existing) document.getElementById('editor').value = existing;
    } catch(e) {
      console.warn('Could not read existing data', e);
    }

    // Register service worker for offline caching
    if ('serviceWorker' in navigator) {
      try {
        await navigator.serviceWorker.register('/APP/service-worker.js');
        console.log('Service worker registered');
      } catch (e) {
        console.warn('Service worker registration failed', e);
      }
    }
  }

  window.addEventListener('load', init);
})();

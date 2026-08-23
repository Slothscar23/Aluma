document.getElementById('save-local').addEventListener('click', async () => {
  const content = document.getElementById('editor').value;
  const payload = {
    path: 'profiles/default/data.json',
    content: content,
    message: 'Local save from Aluma'
  };
  try {
    const res = await fetch('http://127.0.0.1:7878/save-local', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    const text = await res.text();
    document.getElementById('status').innerText = 'Local save response: ' + text + ' (status ' + res.status + ')';
  } catch (e) {
    document.getElementById('status').innerText = 'Local save error: ' + e;
  }
});

// Optional: allow saving to GitHub if the bootstrap has a token configured
document.getElementById('save-remote').addEventListener('click', async () => {
  const content = document.getElementById('editor').value;
  const payload = {
    path: 'web/data.json',
    content: content,
    message: 'Save from Aluma prototype'
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
    document.getElementById('status').innerText = 'Remote save error: ' + e;
  }
});

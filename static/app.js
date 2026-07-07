// ru-share Media Frontend

class MediaServerAPI {
  constructor(baseURL = '') {
    this.baseURL = baseURL;
    this.session = null;
  }

  async login(username, password) {
    const res = await fetch(`${this.baseURL}/api/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    if (res.ok) {
      const data = await res.json();
      this.session = data;
      localStorage.setItem('session', JSON.stringify(data));
      return data;
    }
    throw new Error('Login failed');
  }

  async getLibrary() {
    const res = await fetch(`${this.baseURL}/api/library`);
    if (res.ok) {
      return res.json();
    }
    throw new Error('Failed to fetch library');
  }

  async scan(path) {
    const res = await fetch(`${this.baseURL}/api/scan`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path })
    });
    return res.ok;
  }
}

class MediaApp {
  constructor() {
    this.api = new MediaServerAPI('');
    this.init();
  }

  init() {
    this.setupEventListeners();
    this.loadLibrary();
    this.checkSession();
  }

  setupEventListeners() {
    const loginBtn = document.getElementById('login-btn');
    if (loginBtn) {
      loginBtn.addEventListener('click', () => this.showLoginModal());
    }
  }

  checkSession() {
    const session = localStorage.getItem('session');
    if (session) {
      this.api.session = JSON.parse(session);
    }
  }

  async loadLibrary() {
    try {
      const items = await this.api.getLibrary();
      this.renderLibrary(items);
    } catch (err) {
      console.error('Failed to load library:', err);
    }
  }

  renderLibrary(items) {
    const popularList = document.getElementById('popular-list');
    const recentList = document.getElementById('recent-list');
    
    if (!popularList || !recentList) return;

    popularList.innerHTML = items.map(item => `
      <div class="movie-card rounded-md overflow-hidden cursor-pointer" onclick="app.playMedia('${item.id}')">
        <img src="${item.poster_url || 'https://placehold.co/300x450/netflix-red/white?text=' + encodeURIComponent(item.title || item.filename)}" 
             alt="${item.title || item.filename}" class="w-full h-64 object-cover">
        <div class="p-2">
          <p class="font-medium text-sm truncate">${item.title || item.filename}</p>
        </div>
      </div>
    `).join('');
  }

  showLoginModal() {
    const username = prompt('Username:');
    const password = prompt('Password:');
    if (username && password) {
      this.api.login(username, password).then(() => {
        alert('Logged in successfully');
      }).catch(() => {
        alert('Login failed');
      });
    }
  }

  playMedia(id) {
    const modal = document.getElementById('modal');
    if (modal) {
      modal.classList.remove('hidden');
      modal.classList.add('flex');
    }
  }
}

const app = new MediaApp();
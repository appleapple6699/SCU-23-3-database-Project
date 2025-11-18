const api = {
  async request(path, method = 'GET', body) {
    const headers = { 'Content-Type': 'application/json' };
    const token = localStorage.getItem('token');
    if (token) headers['Authorization'] = `Bearer ${token}`;
    const res = await fetch(path, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });
    return res.json();
  },
};

async function showStatus() {
  const el = document.getElementById('status');
  if (!el) return;
  const token = localStorage.getItem('token');
  if (!token) { el.textContent = '未登录'; return; }
  const data = await api.request('/api/auth/me');
  if (data.error_code === 0) {
    const { userId, nickname, isAdmin } = data.data;
    el.textContent = `已登录：${nickname} (ID=${userId}) ${isAdmin ? '[管理员]' : ''}`;
  } else {
    el.textContent = '未登录';
  }
}

async function logout() {
  await api.request('/api/logout', 'POST', {});
  localStorage.removeItem('token');
  localStorage.removeItem('user_id');
  showStatus();
}

document.addEventListener('DOMContentLoaded', showStatus);

async function register() {
  const nickname = document.getElementById('reg-nickname').value;
  const phone = document.getElementById('reg-phone').value;
  const password = document.getElementById('reg-password').value;
  const data = await api.request('/api/users/register', 'POST', { nickname, phone, password });
  document.getElementById('reg-result').textContent = JSON.stringify(data, null, 2);
}

async function login() {
  const phone = document.getElementById('login-phone').value;
  const password = document.getElementById('login-password').value;
  const data = await api.request('/api/users/login', 'POST', { phone, password });
  if (data.error_code === 0 && data.data) {
    localStorage.setItem('token', data.data.token);
    localStorage.setItem('user_id', data.data.user_id);
  }
  document.getElementById('login-result').textContent = JSON.stringify(data, null, 2);
}

async function createGroup() {
  const name = document.getElementById('group-name').value;
  const description = document.getElementById('group-desc').value;
  const creatorUserId = parseInt(document.getElementById('group-creator').value, 10) || 0;
  const data = await api.request('/api/groups', 'POST', { name, description, creatorUserId });
  document.getElementById('group-result').textContent = JSON.stringify(data, null, 2);
}

async function listGroups() {
  const data = await api.request('/api/groups');
  document.getElementById('group-result').textContent = JSON.stringify(data, null, 2);
}

async function createTask() {
  const groupId = parseInt(document.getElementById('task-group').value, 10) || 0;
  const publisherId = parseInt(document.getElementById('task-pub').value, 10) || 0;
  const title = document.getElementById('task-title').value;
  const content = document.getElementById('task-content').value;
  const data = await api.request('/api/tasks', 'POST', { groupId, publisherId, title, content });
  document.getElementById('task-result').textContent = JSON.stringify(data, null, 2);
}

async function listTasks() {
  const data = await api.request('/api/tasks');
  document.getElementById('task-result').textContent = JSON.stringify(data, null, 2);
}

async function applyUserGroup() {
  const userId = parseInt(document.getElementById('apply-user').value, 10) || 0;
  const groupId = parseInt(document.getElementById('apply-group').value, 10) || 0;
  const data = await api.request('/api/usergroups/apply', 'POST', { userId, groupId });
  document.getElementById('apply-result').textContent = JSON.stringify(data, null, 2);
}
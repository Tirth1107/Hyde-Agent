const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

// Navigation
document.querySelectorAll('.nav-item').forEach(item => {
  item.addEventListener('click', (e) => {
    e.preventDefault();
    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
    e.target.classList.add('active');
    
    const targetId = e.target.getAttribute('data-target');
    document.querySelectorAll('main section').forEach(sec => sec.classList.remove('active'));
    document.getElementById(targetId).classList.add('active');
  });
});

// Logs Button
document.getElementById('open-logs-btn').addEventListener('click', async () => {
  try {
    await invoke('open_logs_folder');
  } catch (err) {
    console.error("Failed to open logs folder", err);
  }
});

const downloadBtn = document.getElementById('download-vosk-btn');
const statusText = document.getElementById('vosk-status-text');

if (downloadBtn) {
  downloadBtn.addEventListener('click', async () => {
    try {
      downloadBtn.disabled = true;
      downloadBtn.innerText = "Downloading Model... (Please wait)";
      statusText.style.display = 'block';
      statusText.style.color = '#eab308';
      statusText.innerText = "Downloading and extracting Vosk Native Engine...";
      
      const res = await invoke('download_vosk_model');
      
      statusText.style.color = '#4ade80';
      statusText.innerText = res;
      downloadBtn.innerText = "Engine Installed ✅";
    } catch (err) {
      statusText.style.color = '#ef4444';
      statusText.innerText = "Error: " + err;
      downloadBtn.disabled = false;
      downloadBtn.innerText = "Retry Download";
    }
  });
}

// Load Custom Commands
async function loadCustomCommands() {
  try {
    const jsonStr = await invoke('get_custom_commands_json');
    const cmds = JSON.parse(jsonStr);
    
    const container = document.getElementById('custom-list');
    container.innerHTML = '';
    
    if (cmds.length === 0) {
      container.innerHTML = '<p class="desc">No custom commands defined yet.</p>';
    }

    cmds.forEach((cmd, idx) => {
      const div = document.createElement('div');
      div.className = 'command-item';
      div.style.padding = '10px';
      div.style.border = '1px solid #333';
      div.style.marginBottom = '10px';
      div.style.borderRadius = '6px';
      div.innerHTML = `
        <strong>${cmd.keyword}</strong> - <em>${cmd.description}</em>
        <button class="secondary-btn" style="float:right; padding: 5px 10px;" onclick="deleteCustomCmd(${idx})">Delete</button>
      `;
      container.appendChild(div);
    });
  } catch(e) {
    console.error("Error loading custom commands", e);
  }
}

// Load Built-in Commands
async function loadBuiltinCommands() {
  try {
    const jsonStr = await invoke('get_builtin_commands_json');
    const cmds = JSON.parse(jsonStr);
    
    const container = document.getElementById('builtin-list');
    container.innerHTML = '';
    
    if (cmds.length === 0) {
      container.innerHTML = '<p class="desc">No built-in commands found.</p>';
    }

    cmds.forEach((cmd) => {
      const div = document.createElement('div');
      div.className = 'command-item';
      div.style.padding = '10px';
      div.style.border = '1px solid #333';
      div.style.marginBottom = '10px';
      div.style.borderRadius = '6px';
      div.innerHTML = `
        <strong>${cmd.keyword}</strong> - <em>${cmd.description}</em>
        <span style="float:right; font-size: 0.8em; color: #888;">${cmd.action_type}</span>
      `;
      container.appendChild(div);
    });
  } catch(e) {
    console.error("Error loading built-in commands", e);
  }
}

// Global func for deletion
window.deleteCustomCmd = async (idx) => {
  try {
    const jsonStr = await invoke('get_custom_commands_json');
    const cmds = JSON.parse(jsonStr);
    cmds.splice(idx, 1);
    await invoke('save_custom_commands_json', { json: JSON.stringify(cmds) });
    loadCustomCommands();
  } catch(e) {
    console.error(e);
  }
}

document.getElementById('add-custom-btn').addEventListener('click', async () => {
  const keyword = prompt("Enter command keyword:");
  if (!keyword) return;
  const desc = prompt("Enter description:");
  const url = prompt("Enter URL to open (or leave blank for app):");
  
  const newCmd = {
    keyword,
    aliases: [],
    action_type: url ? "open_url" : "open_app",
    parameters: url ? { url } : { app_name: keyword },
    description: desc || "Custom command"
  };

  try {
    const jsonStr = await invoke('get_custom_commands_json');
    const cmds = JSON.parse(jsonStr);
    cmds.push(newCmd);
    await invoke('save_custom_commands_json', { json: JSON.stringify(cmds) });
    loadCustomCommands();
  } catch(e) {
    console.error(e);
  }
});

// Init
loadBuiltinCommands();
loadCustomCommands();

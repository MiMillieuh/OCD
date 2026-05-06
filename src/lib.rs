use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, RunEvent, WebviewUrl};
use tauri::webview::{PageLoadEvent, WebviewWindowBuilder};

mod config;
use config::Config;

pub struct ServerProcess(pub Mutex<Option<Child>>);

fn start_server(config: &Config) -> Result<Child, String> {
    let mut cmd = Command::new("opencode");
    cmd.arg("serve").arg("--port").arg(config.port.to_string());

    if config.expose_network && !config.hostname.is_empty() {
        cmd.arg("--hostname").arg(&config.hostname);
    }

    cmd.env("OPENCODE_SERVER_USERNAME", &config.username);
    cmd.env("OPENCODE_SERVER_PASSWORD", &config.password);

    cmd.spawn().map_err(|e| e.to_string())
}

fn stop_server(process_state: &ServerProcess) {
    if let Ok(mut child_opt) = process_state.0.lock() {
        if let Some(mut child) = child_opt.take() {
            let _ = child.kill();
            let _ = child.wait();
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}

fn wait_for_server(port: u16) {
    let max_wait = Duration::from_secs(30);
    let poll_interval = Duration::from_millis(500);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > max_wait {
            eprintln!("Warning: server didn't become ready within 30s");
            break;
        }

        if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            println!("opencode serve is ready on port {}", port);
            break;
        }

        std::thread::sleep(poll_interval);
    }
}

fn validate_config(config: &Config) -> Result<(), String> {
    if config.expose_network {
        if config.username.is_empty() {
            return Err("Username is required when exposing on local network".to_string());
        }
        if config.password.is_empty() {
            return Err("Password is required when exposing on local network".to_string());
        }
    }
    Ok(())
}

fn restart_server_internal(app_handle: &AppHandle) -> Result<(), String> {
    let config = Config::load(app_handle);
    validate_config(&config)?;

    if let Some(process_state) = app_handle.try_state::<ServerProcess>() {
        stop_server(&process_state);
    }

    let child = start_server(&config)?;

    if let Some(process_state) = app_handle.try_state::<ServerProcess>() {
        if let Ok(mut child_opt) = process_state.0.lock() {
            *child_opt = Some(child);
        }
    }

    wait_for_server(config.port);

    for window in app_handle.webview_windows().values() {
        let _ = window.eval(&format!(
            "window.location.href = 'http://localhost:{}';",
            config.port
        ));
    }

    Ok(())
}

#[tauri::command]
fn get_config(app_handle: AppHandle) -> Config {
    Config::load(&app_handle)
}

#[tauri::command]
fn save_config(app_handle: AppHandle, config: Config) -> Result<(), String> {
    validate_config(&config)?;
    config.save(&app_handle)?;
    Ok(())
}

#[tauri::command]
fn restart_server(app_handle: AppHandle) -> Result<(), String> {
    restart_server_internal(&app_handle)
}

const INJECT_SCRIPT: &str = r#"
(function() {
    if (document.getElementById('ocd-settings')) return;

    const style = document.createElement('style');
    style.textContent = `
        #ocd-settings {
            position: fixed;
            bottom: 16px;
            right: 16px;
            z-index: 999999;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }
        #ocd-btn {
            width: 36px;
            height: 36px;
            border-radius: 8px;
            background: #151515;
            color: #fff;
            border: 1px solid #333;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 18px;
            transition: background 0.2s;
        }
        #ocd-btn:hover { background: #333; }
        #ocd-panel {
            display: none;
            position: fixed;
            bottom: 60px;
            right: 16px;
            width: 320px;
            background: #151515;
            border: 1px solid #374151;
            border-radius: 12px;
            padding: 20px;
            color: #f9fafb;
            box-shadow: 0 20px 25px -5px rgba(0,0,0,0.5);
        }
        #ocd-panel.open { display: block; }
        #ocd-panel h2 {
            margin: 0 0 16px 0;
            font-size: 16px;
            font-weight: 600;
        }
        #ocd-panel label {
            display: block;
            font-size: 12px;
            color: #9ca3af;
            margin-bottom: 4px;
            margin-top: 12px;
        }
        #ocd-panel input[type="text"],
        #ocd-panel input[type="number"],
        #ocd-panel input[type="password"] {
            width: 100%;
            box-sizing: border-box;
            padding: 8px 10px;
            border-radius: 6px;
            border: 1px solid #333;
            background: #1a1a1a;
            color: #f9fafb;
            font-size: 13px;
        }
        #ocd-panel input:focus {
            outline: none;
            border-color: #6366f1;
        }
        #ocd-panel .checkbox-row {
            display: flex;
            align-items: center;
            gap: 8px;
            margin-top: 12px;
            cursor: pointer;
        }
        #ocd-panel .checkbox-row input[type="checkbox"] {
            width: 16px;
            height: 16px;
            cursor: pointer;
        }
        #ocd-panel .checkbox-row label {
            margin: 0;
            cursor: pointer;
            font-size: 13px;
            color: #f9fafb;
        }
        #ocd-panel .hidden { display: none !important; }
        #ocd-panel .actions {
            display: flex;
            gap: 8px;
            margin-top: 16px;
        }
        #ocd-panel button {
            flex: 1;
            padding: 8px;
            border-radius: 6px;
            border: none;
            cursor: pointer;
            font-size: 13px;
            font-weight: 500;
        }
        #ocd-panel .save-btn {
            background: #6366f1;
            color: #fff;
        }
        #ocd-panel .save-btn:hover { background: #4f46e5; }
        #ocd-panel .restart-btn {
            background: #059669;
            color: #fff;
        }
        #ocd-panel .restart-btn:hover { background: #047857; }
        #ocd-panel .status {
            margin-top: 10px;
            font-size: 12px;
            min-height: 16px;
        }
        #ocd-panel .status.success { color: #34d399; }
        #ocd-panel .status.error { color: #f87171; }
    `;
    document.head.appendChild(style);

    const container = document.createElement('div');
    container.id = 'ocd-settings';
    container.innerHTML = `
        <button id="ocd-btn" title="OCD settings">&#9881;</button>
        <div id="ocd-panel">
            <h2>OCD Settings</h2>
            <label>Port</label>
            <input type="number" id="ods-port" placeholder="4096">
            <div class="checkbox-row">
                <input type="checkbox" id="ods-expose">
                <label for="ods-expose">Expose on local network</label>
            </div>
            <div id="ods-network-fields" class="hidden">
                <label>Hostname</label>
                <input type="text" id="ods-hostname" placeholder="0.0.0.0">
                <label>Username <span style="color:#f87171">*</span></label>
                <input type="text" id="ods-username" placeholder="">
                <label>Password <span style="color:#f87171">*</span></label>
                <input type="password" id="ods-password" placeholder="">
            </div>
            <div class="actions">
                <button class="save-btn" id="ods-save">Save</button>
                <button class="restart-btn" id="ods-restart">Restart</button>
            </div>
            <div class="status" id="ods-status"></div>
        </div>
    `;
    document.body.appendChild(container);

    const btn = document.getElementById('ocd-btn');
    const panel = document.getElementById('ocd-panel');
    const statusEl = document.getElementById('ods-status');
    const exposeCheckbox = document.getElementById('ods-expose');
    const networkFields = document.getElementById('ods-network-fields');

    exposeCheckbox.addEventListener('change', () => {
        networkFields.classList.toggle('hidden', !exposeCheckbox.checked);
    });

    btn.addEventListener('click', () => {
        panel.classList.toggle('open');
    });

    async function loadConfig() {
        try {
            const cfg = await window.__TAURI__.core.invoke('get_config');
            document.getElementById('ods-port').value = cfg.port || '';
            exposeCheckbox.checked = cfg.expose_network || false;
            document.getElementById('ods-hostname').value = cfg.hostname || '';
            document.getElementById('ods-username').value = cfg.username || '';
            document.getElementById('ods-password').value = cfg.password || '';
            networkFields.classList.toggle('hidden', !exposeCheckbox.checked);
        } catch(e) {
            statusEl.textContent = 'Failed to load config';
            statusEl.className = 'status error';
        }
    }

    function collectConfig() {
        return {
            port: parseInt(document.getElementById('ods-port').value) || 4096,
            expose_network: exposeCheckbox.checked,
            hostname: document.getElementById('ods-hostname').value,
            username: document.getElementById('ods-username').value,
            password: document.getElementById('ods-password').value,
        };
    }

    async function handleAction(action) {
        const cfg = collectConfig();
        if (cfg.expose_network && (!cfg.username || !cfg.password)) {
            statusEl.textContent = 'Username and password are required when exposing on local network';
            statusEl.className = 'status error';
            return;
        }

        try {
            await window.__TAURI__.core.invoke('save_config', { config: cfg });

            if (action === 'restart') {
                statusEl.textContent = 'Restarting...';
                statusEl.className = 'status';
                await window.__TAURI__.core.invoke('restart_server');
                statusEl.textContent = 'Server restarted';
            } else {
                statusEl.textContent = 'Settings saved';
            }
            statusEl.className = 'status success';
        } catch(e) {
            statusEl.textContent = (action === 'restart' ? 'Restart' : 'Save') + ' failed: ' + e;
            statusEl.className = 'status error';
        }
    }

    document.getElementById('ods-save').addEventListener('click', () => handleAction('save'));
    document.getElementById('ods-restart').addEventListener('click', () => handleAction('restart'));

    loadConfig();
})();
"#;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config = Config::load(app);
            let child = start_server(&config).map_err(|e| {
                eprintln!("Failed to start opencode serve: {}", e);
                e
            })?;

            app.manage(ServerProcess(Mutex::new(Some(child))));
            wait_for_server(config.port);

            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("OCD")
                .inner_size(1400.0, 900.0)
                .on_page_load(move |window, payload| {
                    if payload.event() == PageLoadEvent::Finished {
                        let url = payload.url().to_string();
                        if url.starts_with("http://localhost:") || url.starts_with("http://127.0.0.1:") {
                            let _ = window.eval(INJECT_SCRIPT);
                        }
                    }
                })
                .build()?;

            let _ = window.eval(&format!(
                "window.location.href = 'http://localhost:{}';",
                config.port
            ));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, save_config, restart_server])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Exit = event {
                if let Some(process_state) = app_handle.try_state::<ServerProcess>() {
                    stop_server(&process_state);
                    println!("opencode serve server stopped");
                }
            }
        });
}

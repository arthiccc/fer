import init, { TelcoSimulator, init_panic_hook } from "./pkg/telco_core.js";

async function run() {
    await init();
    init_panic_hook();

    const canvas = document.getElementById('bubbleCanvas');
    const ctx = canvas.getContext('2d');
    const balanceText = document.getElementById('balanceText');
    
    // Scale for high DPI
    const dpr = window.devicePixelRatio || 1;
    canvas.width = 300 * dpr;
    canvas.height = 300 * dpr;
    ctx.scale(dpr, dpr);

    let currentAccount = null;
    let maxBalance = 1000 * 1024 * 1024; // 1GB default scale

    // We implement the callback interface manually as a JS object
    // UniFFI/wasm-bindgen will map this to the TelcoLiveUpdateHandler trait
    const handler = {
        on_account_updated: (account) => {
            currentAccount = account;
            balanceText.innerText = (account.data_balance_bytes / (1024 * 1024)).toFixed(2);
            if (account.data_balance_bytes > maxBalance) {
                maxBalance = account.data_balance_bytes;
            }
        }
    };

    const simulator = TelcoSimulator.new("web_user", "memory.db");
    simulator.set_update_handler(handler);

    // Initial Topping for visibility
    simulator.handle_command("General 500MB");

    document.getElementById('simulateBtn').onclick = () => {
        try {
            simulator.simulate_usage(10 * 1024 * 1024, "General");
        } catch (e) {
            alert("Insufficient Balance!");
        }
    };

    document.getElementById('buyBtn').onclick = () => {
        simulator.handle_command("General 1GB");
    };

    function animate(time) {
        ctx.clearRect(0, 0, 300, 300);
        
        if (currentAccount) {
            const balance = currentAccount.data_balance_bytes;
            const ratio = Math.min(1.0, balance / maxBalance);
            const radius = 100 * ratio;
            const centerX = 150;
            const centerY = 150;

            ctx.beginPath();
            for (let angle = 0; angle <= 360; angle += 2) {
                const radian = angle * Math.PI / 180;
                const offset = 5 * Math.sin(radian * 4 + time * 0.005);
                const r = radius + offset;
                const x = centerX + r * Math.cos(radian);
                const y = centerY + r * Math.sin(radian);
                
                if (angle === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
            }
            ctx.closePath();

            const gradient = ctx.createLinearGradient(0, 0, 300, 300);
            gradient.addColorStop(0, '#6200ee');
            gradient.addColorStop(1, '#03dac6');
            
            ctx.fillStyle = gradient;
            ctx.fill();
            
            // Outer glow
            ctx.shadowBlur = 20;
            ctx.shadowColor = '#6200ee';
        }

        requestAnimationFrame(animate);
    }

    requestAnimationFrame(animate);
}

run();

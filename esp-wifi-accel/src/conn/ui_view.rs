
// The browser dashboard polls /accel every 500 ms.
pub const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>ESP32-S3 Accelerometer</title>
    <style>
        body {
            font-family: system-ui, sans-serif;
            max-width: 600px;
            margin: 40px auto;
            padding: 0 20px;
        }
        .status {
            font-weight: bold;
        }
        .reading {
            padding: 14px;
            margin: 10px 0;
            background: #f1f5f9;
            border-radius: 8px;
        }
    </style>
</head>
<body>
    <h1>ESP32-S3 Accelerometer</h1>

    <p>Status: <span id="status" class="status">Connecting...</span></p>

    <div class="reading">X: <span id="x">--</span> g</div>
    <div class="reading">Y: <span id="y">--</span> g</div>
    <div class="reading">Z: <span id="z">--</span> g</div>
    <div class="reading">
        Magnitude: <span id="magnitude">--</span> g
    </div>

    <script>
        async function updateReadings() {
            try {
                const response = await fetch('/accel', {
                    cache: 'no-store'
                });

                if (!response.ok) {
                    throw new Error('HTTP request failed');
                }

                const data = await response.json();

                if (!data.connected) {
                    document.getElementById('status').textContent =
                        'Accelerometer disconnected';

                    for (const id of ['x', 'y', 'z', 'magnitude']) {
                        document.getElementById(id).textContent = '--';
                    }
                    return;
                }

                document.getElementById('status').textContent = 'Connected';

                for (const id of ['x', 'y', 'z', 'magnitude']) {
                    document.getElementById(id).textContent =
                        Number(data[id]).toFixed(2);
                }
            } catch (_) {
                document.getElementById('status').textContent =
                    'ESP32 server unavailable';

                for (const id of ['x', 'y', 'z', 'magnitude']) {
                    document.getElementById(id).textContent = '--';
                }
            }
        }

        updateReadings();
        setInterval(updateReadings, 500);
    </script>
</body>
</html>
"#;
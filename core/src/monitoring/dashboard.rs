// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Real-time HTML dashboard with WebSocket integration.

use anyhow::Result;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::task::JoinHandle;
use axum::{
    routing::get,
    Router,
    response::{Html, IntoResponse},
};

use crate::monitoring::collector::MetricCollector;

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Port to serve dashboard on
    pub port: u16,
    /// Enable the dashboard
    pub enabled: bool,
    /// WebSocket server URL
    pub websocket_url: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            enabled: true,
            websocket_url: "ws://localhost:8080/ws".to_string(),
        }
    }
}

/// Real-time monitoring dashboard
pub struct Dashboard {
    config: DashboardConfig,
    collector: Arc<MetricCollector>,
    server_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl Dashboard {
    /// Create a new dashboard
    pub fn new(config: DashboardConfig, collector: Arc<MetricCollector>) -> Self {
        Self {
            config,
            collector,
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the dashboard server
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::debug!("Dashboard is disabled");
            return Ok(());
        }

        let websocket_url = self.config.websocket_url.clone();

        let app = Router::new()
            .route("/", get(move || dashboard_handler(websocket_url.clone())));

        let addr = format!("0.0.0.0:{}", self.config.port).parse()?;
        tracing::info!("Starting dashboard on http://{}", addr);

        let server = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .expect("Failed to bind dashboard server");

            axum::serve(listener, app)
                .await
                .expect("Dashboard server error");
        });

        let mut handle = self.server_handle.write();
        *handle = Some(server);

        Ok(())
    }

    /// Stop the dashboard server
    pub async fn stop(&self) -> Result<()> {
        let mut handle = self.server_handle.write();
        if let Some(h) = handle.take() {
            h.abort();
        }
        Ok(())
    }
}

/// Dashboard HTML handler
async fn dashboard_handler(websocket_url: String) -> impl IntoResponse {
    Html(generate_dashboard_html(&websocket_url))
}

/// Generate the dashboard HTML
fn generate_dashboard_html(websocket_url: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LLM Test Bench - Real-time Monitoring</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background: #0f172a;
            color: #e2e8f0;
            padding: 20px;
        }}

        .header {{
            text-align: center;
            padding: 30px 0;
            border-bottom: 2px solid #1e293b;
            margin-bottom: 30px;
        }}

        .header h1 {{
            font-size: 2.5em;
            color: #60a5fa;
            margin-bottom: 10px;
        }}

        .status {{
            display: inline-block;
            padding: 8px 16px;
            border-radius: 20px;
            font-size: 0.9em;
            font-weight: 600;
        }}

        .status.connected {{
            background: #10b981;
            color: #fff;
        }}

        .status.disconnected {{
            background: #ef4444;
            color: #fff;
        }}

        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}

        .stat-card {{
            background: #1e293b;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }}

        .stat-card h3 {{
            font-size: 0.9em;
            color: #94a3b8;
            margin-bottom: 10px;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}

        .stat-value {{
            font-size: 2.5em;
            font-weight: 700;
            color: #60a5fa;
        }}

        .stat-change {{
            font-size: 0.9em;
            margin-top: 8px;
        }}

        .stat-change.positive {{
            color: #10b981;
        }}

        .stat-change.negative {{
            color: #ef4444;
        }}

        .charts-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}

        .chart-card {{
            background: #1e293b;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }}

        .chart-card h2 {{
            font-size: 1.2em;
            margin-bottom: 20px;
            color: #cbd5e1;
        }}

        .providers-list {{
            background: #1e293b;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }}

        .provider-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 15px;
            margin-bottom: 10px;
            background: #0f172a;
            border-radius: 8px;
            border-left: 4px solid #60a5fa;
        }}

        .provider-name {{
            font-weight: 600;
            font-size: 1.1em;
        }}

        .provider-stats {{
            display: flex;
            gap: 20px;
            font-size: 0.9em;
            color: #94a3b8;
        }}

        .events-log {{
            background: #1e293b;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
            max-height: 400px;
            overflow-y: auto;
        }}

        .event-item {{
            padding: 10px;
            margin-bottom: 8px;
            background: #0f172a;
            border-radius: 6px;
            border-left: 3px solid #3b82f6;
            font-size: 0.9em;
        }}

        .event-time {{
            color: #64748b;
            font-size: 0.85em;
        }}

        .event-type {{
            display: inline-block;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 0.8em;
            font-weight: 600;
            margin-right: 8px;
        }}

        .event-type.request {{
            background: #3b82f6;
            color: #fff;
        }}

        .event-type.benchmark {{
            background: #8b5cf6;
            color: #fff;
        }}

        .event-type.error {{
            background: #ef4444;
            color: #fff;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 LLM Test Bench</h1>
        <p style="color: #94a3b8; margin: 10px 0;">Real-time Monitoring Dashboard</p>
        <span id="status" class="status disconnected">Disconnected</span>
    </div>

    <div class="stats-grid">
        <div class="stat-card">
            <h3>Total Requests</h3>
            <div class="stat-value" id="total-requests">0</div>
            <div class="stat-change positive" id="requests-change">+0 today</div>
        </div>
        <div class="stat-card">
            <h3>Avg Latency</h3>
            <div class="stat-value" id="avg-latency">0.0s</div>
            <div class="stat-change" id="latency-change">-</div>
        </div>
        <div class="stat-card">
            <h3>Total Tokens</h3>
            <div class="stat-value" id="total-tokens">0</div>
            <div class="stat-change positive" id="tokens-change">+0 today</div>
        </div>
        <div class="stat-card">
            <h3>Total Cost</h3>
            <div class="stat-value" id="total-cost">$0.00</div>
            <div class="stat-change positive" id="cost-change">+$0.00 today</div>
        </div>
    </div>

    <div class="charts-grid">
        <div class="chart-card">
            <h2>Requests per Second</h2>
            <canvas id="requests-chart"></canvas>
        </div>
        <div class="chart-card">
            <h2>Latency Distribution</h2>
            <canvas id="latency-chart"></canvas>
        </div>
    </div>

    <div class="charts-grid">
        <div class="chart-card">
            <h2>Token Usage</h2>
            <canvas id="tokens-chart"></canvas>
        </div>
        <div class="chart-card">
            <h2>Cost Over Time</h2>
            <canvas id="cost-chart"></canvas>
        </div>
    </div>

    <div class="providers-list">
        <h2 style="margin-bottom: 20px;">Provider Status</h2>
        <div id="providers-container">
            <p style="color: #64748b;">No active providers</p>
        </div>
    </div>

    <div class="events-log" style="margin-top: 20px;">
        <h2 style="margin-bottom: 15px;">Live Event Stream</h2>
        <div id="events-container"></div>
    </div>

    <script>
        const WS_URL = '{websocket_url}';
        let ws = null;
        let reconnectInterval = null;

        // Chart configurations
        const chartConfig = {{
            responsive: true,
            maintainAspectRatio: true,
            plugins: {{
                legend: {{
                    labels: {{
                        color: '#cbd5e1'
                    }}
                }}
            }},
            scales: {{
                y: {{
                    ticks: {{ color: '#94a3b8' }},
                    grid: {{ color: '#334155' }}
                }},
                x: {{
                    ticks: {{ color: '#94a3b8' }},
                    grid: {{ color: '#334155' }}
                }}
            }}
        }};

        // Initialize charts
        const requestsChart = new Chart(
            document.getElementById('requests-chart'),
            {{
                type: 'line',
                data: {{
                    labels: [],
                    datasets: [{{
                        label: 'Requests/sec',
                        data: [],
                        borderColor: '#60a5fa',
                        backgroundColor: 'rgba(96, 165, 250, 0.1)',
                        tension: 0.4
                    }}]
                }},
                options: chartConfig
            }}
        );

        const latencyChart = new Chart(
            document.getElementById('latency-chart'),
            {{
                type: 'bar',
                data: {{
                    labels: [],
                    datasets: [{{
                        label: 'Latency (ms)',
                        data: [],
                        backgroundColor: '#8b5cf6'
                    }}]
                }},
                options: chartConfig
            }}
        );

        const tokensChart = new Chart(
            document.getElementById('tokens-chart'),
            {{
                type: 'line',
                data: {{
                    labels: [],
                    datasets: [
                        {{
                            label: 'Input Tokens',
                            data: [],
                            borderColor: '#10b981',
                            backgroundColor: 'rgba(16, 185, 129, 0.1)'
                        }},
                        {{
                            label: 'Output Tokens',
                            data: [],
                            borderColor: '#f59e0b',
                            backgroundColor: 'rgba(245, 158, 11, 0.1)'
                        }}
                    ]
                }},
                options: chartConfig
            }}
        );

        const costChart = new Chart(
            document.getElementById('cost-chart'),
            {{
                type: 'line',
                data: {{
                    labels: [],
                    datasets: [{{
                        label: 'Cost ($)',
                        data: [],
                        borderColor: '#ec4899',
                        backgroundColor: 'rgba(236, 72, 153, 0.1)',
                        tension: 0.4
                    }}]
                }},
                options: chartConfig
            }}
        );

        // State
        let stats = {{
            totalRequests: 0,
            avgLatency: 0,
            totalTokens: 0,
            totalCost: 0
        }};

        function connectWebSocket() {{
            ws = new WebSocket(WS_URL);

            ws.onopen = () => {{
                console.log('WebSocket connected');
                document.getElementById('status').textContent = 'Connected';
                document.getElementById('status').className = 'status connected';
                if (reconnectInterval) {{
                    clearInterval(reconnectInterval);
                    reconnectInterval = null;
                }}
            }};

            ws.onmessage = (event) => {{
                try {{
                    const message = JSON.parse(event.data);
                    handleMessage(message);
                }} catch (e) {{
                    console.error('Failed to parse message:', e);
                }}
            }};

            ws.onclose = () => {{
                console.log('WebSocket disconnected');
                document.getElementById('status').textContent = 'Disconnected';
                document.getElementById('status').className = 'status disconnected';

                if (!reconnectInterval) {{
                    reconnectInterval = setInterval(() => {{
                        console.log('Attempting to reconnect...');
                        connectWebSocket();
                    }}, 5000);
                }}
            }};

            ws.onerror = (error) => {{
                console.error('WebSocket error:', error);
            }};
        }}

        function handleMessage(message) {{
            if (message.type === 'Event') {{
                handleEvent(message.data);
            }} else if (message.type === 'Connected') {{
                console.log('Connected with client ID:', message.data.client_id);
            }}
        }}

        function handleEvent(event) {{
            // Update stats
            updateStats(event);

            // Update charts
            updateCharts(event);

            // Add to event log
            addEventToLog(event);
        }}

        function updateStats(event) {{
            if (event.payload.type === 'Request') {{
                stats.totalRequests++;
                document.getElementById('total-requests').textContent = stats.totalRequests;

                if (event.payload.data.latency) {{
                    stats.avgLatency = (stats.avgLatency * (stats.totalRequests - 1) + event.payload.data.latency) / stats.totalRequests;
                    document.getElementById('avg-latency').textContent = stats.avgLatency.toFixed(2) + 's';
                }}

                if (event.payload.data.tokens) {{
                    stats.totalTokens += event.payload.data.tokens.total_tokens;
                    document.getElementById('total-tokens').textContent = stats.totalTokens.toLocaleString();
                }}

                if (event.payload.data.cost) {{
                    stats.totalCost += event.payload.data.cost;
                    document.getElementById('total-cost').textContent = '$' + stats.totalCost.toFixed(2);
                }}
            }}
        }}

        function updateCharts(event) {{
            const now = new Date().toLocaleTimeString();

            // Update requests chart
            if (requestsChart.data.labels.length > 20) {{
                requestsChart.data.labels.shift();
                requestsChart.data.datasets[0].data.shift();
            }}
            requestsChart.data.labels.push(now);
            requestsChart.data.datasets[0].data.push(Math.random() * 10); // Placeholder
            requestsChart.update('none');
        }}

        function addEventToLog(event) {{
            const container = document.getElementById('events-container');
            const eventDiv = document.createElement('div');
            eventDiv.className = 'event-item';

            const time = new Date(event.timestamp).toLocaleTimeString();
            const eventType = event.event_type.replace(/([A-Z])/g, ' $1').trim();

            eventDiv.innerHTML = `
                <span class="event-type ${{event.event_type.toLowerCase()}}">${{eventType}}</span>
                <span class="event-time">${{time}}</span>
            `;

            container.insertBefore(eventDiv, container.firstChild);

            // Keep only last 50 events
            while (container.children.length > 50) {{
                container.removeChild(container.lastChild);
            }}
        }}

        // Initialize
        connectWebSocket();
    </script>
</body>
</html>"#,
        websocket_url = websocket_url
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_config() {
        let config = DashboardConfig {
            port: 3001,
            enabled: true,
            websocket_url: "ws://localhost:8080/ws".to_string(),
        };
        assert_eq!(config.port, 3001);
        assert!(config.enabled);
    }

    #[test]
    fn test_generate_dashboard_html() {
        let html = generate_dashboard_html("ws://localhost:8080/ws");
        assert!(html.contains("LLM Test Bench"));
        assert!(html.contains("Real-time Monitoring"));
        assert!(html.contains("ws://localhost:8080/ws"));
    }
}

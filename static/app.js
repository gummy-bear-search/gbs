// Alpine.js Dashboard Component
function dashboard() {
    return {
        loading: false,
        showCreateModal: false,
        showIndexDetail: false,
        showSearchModal: false,
        showDocumentModal: false,
        currentView: 'overview', // overview, index-detail, search
        currentIndex: null,
        currentDocument: null,
        newIndexName: '',
        searchQuery: '',
        searchIndex: '',
        searchResults: [],
        indexDetails: null,
        wsConnected: false,
        ws: null,
        stats: {
            indices: '-',
            documents: '-',
            nodes: '-'
        },
        toasts: [],

        async init() {
            // Load initial stats
            await this.loadStats();

            // Connect to WebSocket
            this.connectWebSocket();

            // Set up auto-refresh fallback (if WebSocket fails)
            setInterval(() => {
                if (!this.wsConnected) {
                    this.refresh();
                }
            }, 30000);
        },

        connectWebSocket() {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/_ws`;

            try {
                this.ws = new WebSocket(wsUrl);

                this.ws.onopen = () => {
                    this.wsConnected = true;
                    this.showToast('Connected to real-time updates', 'success');
                };

                this.ws.onmessage = (event) => {
                    try {
                        const message = JSON.parse(event.data);
                        this.handleWebSocketMessage(message);
                    } catch (e) {
                        console.error('Error parsing WebSocket message:', e);
                    }
                };

                this.ws.onerror = (error) => {
                    console.error('WebSocket error:', error);
                    this.wsConnected = false;
                };

                this.ws.onclose = () => {
                    this.wsConnected = false;
                    // Attempt to reconnect after 5 seconds
                    setTimeout(() => this.connectWebSocket(), 5000);
                };
            } catch (error) {
                console.error('Failed to connect WebSocket:', error);
                this.wsConnected = false;
            }
        },

        handleWebSocketMessage(message) {
            switch (message.type) {
                case 'cluster_health':
                    this.updateClusterHealth(message.data);
                    break;
                case 'cluster_stats':
                    this.updateClusterStats(message.data);
                    break;
                case 'indices':
                    this.updateIndices(message.data);
                    break;
            }
        },

        updateClusterHealth(data) {
            const healthEl = document.getElementById('clusterHealth');
            if (healthEl) {
                const status = data.status || 'unknown';
                const statusColors = {
                    'green': 'bg-green-500',
                    'yellow': 'bg-yellow-500',
                    'red': 'bg-red-500'
                };
                const color = statusColors[status] || 'bg-gray-500';
                healthEl.innerHTML = `
                    <div class="flex items-center space-x-4">
                        <div class="${color} rounded-full h-12 w-12 flex items-center justify-center">
                            <span class="text-white text-xl font-bold">${status.charAt(0).toUpperCase()}</span>
                        </div>
                        <div>
                            <p class="text-lg font-semibold text-gray-800">Status: <span class="capitalize">${status}</span></p>
                            <p class="text-sm text-gray-600">Nodes: ${data.number_of_nodes || 0} | Shards: ${data.active_shards || 0}</p>
                        </div>
                    </div>
                `;
            }
        },

        updateClusterStats(data) {
            const indices = data.indices || {};
            const indicesCount = Object.keys(indices).length;
            let totalDocs = 0;
            Object.values(indices).forEach(index => {
                totalDocs += index.total?.docs?.count || 0;
            });
            this.stats = {
                indices: indicesCount,
                documents: totalDocs.toLocaleString(),
                nodes: data.nodes?.count?.total || 1
            };
        },

        updateIndices(data) {
            // Trigger htmx refresh for indices table
            htmx.trigger(document.body, 'refresh');
        },

        async refresh() {
            this.loading = true;
            // Trigger htmx refresh for all components
            htmx.trigger(document.body, 'refresh');
            await this.loadStats();
            this.loading = false;
        },

        async loadStats() {
            try {
                const response = await fetch('/_cluster/stats');
                const stats = await response.json();
                const indices = stats.indices || {};
                const indicesCount = Object.keys(indices).length;
                let totalDocs = 0;

                Object.values(indices).forEach(index => {
                    totalDocs += index.total?.docs?.count || 0;
                });

                this.stats = {
                    indices: indicesCount,
                    documents: totalDocs.toLocaleString(),
                    nodes: stats.nodes?.count?.total || 1
                };
            } catch (error) {
                console.error('Error loading stats:', error);
            }
        },

        async createIndex() {
            if (!this.newIndexName) return;

            this.loading = true;
            try {
                const response = await fetch(`/${this.newIndexName}`, {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({})
                });

                if (response.ok) {
                    this.showCreateModal = false;
                    this.newIndexName = '';
                    this.refresh();
                    this.showToast('Index created successfully!', 'success');
                } else {
                    const error = await response.json();
                    this.showToast(
                        error.error?.reason || 'Failed to create index',
                        'error'
                    );
                }
            } catch (error) {
                this.showToast(`Error: ${error.message}`, 'error');
            } finally {
                this.loading = false;
            }
        },

        showToast(message, type = 'info') {
            const id = Date.now();
            const toast = { id, message, type };
            this.toasts.push(toast);

            // Auto-remove after 5 seconds
            setTimeout(() => {
                this.toasts = this.toasts.filter(t => t.id !== id);
            }, 5000);
        },

        removeToast(id) {
            this.toasts = this.toasts.filter(t => t.id !== id);
        },

        async viewIndex(indexName) {
            this.currentIndex = indexName;
            this.currentView = 'index-detail';
            this.showIndexDetail = true;

            try {
                const response = await fetch(`/${indexName}`);
                if (response.ok) {
                    this.indexDetails = await response.json();
                } else {
                    this.showToast('Failed to load index details', 'error');
                }
            } catch (error) {
                this.showToast(`Error: ${error.message}`, 'error');
            }
        },

        backToOverview() {
            this.currentView = 'overview';
            this.showIndexDetail = false;
            this.currentIndex = null;
            this.indexDetails = null;
        },

        async search() {
            if (!this.searchIndex || !this.searchQuery) {
                this.showToast('Please select an index and enter a search query', 'error');
                return;
            }

            this.loading = true;
            try {
                // Try to parse as JSON, if it fails, treat as simple text query
                let queryBody;
                try {
                    queryBody = JSON.parse(this.searchQuery);
                } catch (e) {
                    // Not JSON, create a simple match query
                    queryBody = {
                        query: {
                            match: {
                                _all: this.searchQuery
                            }
                        },
                        highlight: {
                            fields: {
                                '*': {}
                            }
                        }
                    };
                }

                const response = await fetch(`/${this.searchIndex}/_search`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(queryBody)
                });

                if (response.ok) {
                    const result = await response.json();
                    this.searchResults = result.hits?.hits || [];
                    const total = result.hits?.total?.value || result.hits?.total || this.searchResults.length;
                    this.showToast(`Found ${total} result(s)`, 'success');
                } else {
                    const error = await response.json();
                    this.showToast(error.error?.reason || 'Search failed', 'error');
                }
            } catch (error) {
                this.showToast(`Error: ${error.message}`, 'error');
            } finally {
                this.loading = false;
            }
        },

        async viewDocument(indexName, docId) {
            try {
                const response = await fetch(`/${indexName}/_doc/${docId}`);
                if (response.ok) {
                    this.currentDocument = await response.json();
                    this.showDocumentModal = true;
                } else {
                    this.showToast('Failed to load document', 'error');
                }
            } catch (error) {
                this.showToast(`Error: ${error.message}`, 'error');
            }
        },

        closeDocumentModal() {
            this.showDocumentModal = false;
            this.currentDocument = null;
        }
    }
}

// htmx response processors for custom rendering
document.body.addEventListener('htmx:afterSwap', function(evt) {
    // Process cluster health response
    if (evt.detail.target.id === 'clusterHealth') {
        const health = JSON.parse(evt.detail.xhr.responseText);
        const status = health.status || 'unknown';
        const statusColors = {
            'green': 'bg-green-500',
            'yellow': 'bg-yellow-500',
            'red': 'bg-red-500'
        };
        const color = statusColors[status] || 'bg-gray-500';

        evt.detail.target.innerHTML = `
            <div class="flex items-center space-x-4">
                <div class="${color} rounded-full h-12 w-12 flex items-center justify-center">
                    <span class="text-white text-xl font-bold">${status.charAt(0).toUpperCase()}</span>
                </div>
                <div>
                    <p class="text-lg font-semibold text-gray-800">Status: <span class="capitalize">${status}</span></p>
                    <p class="text-sm text-gray-600">Nodes: ${health.number_of_nodes || 0} | Shards: ${health.active_shards || 0}</p>
                </div>
            </div>
        `;
    }

    // Process indices table response
    if (evt.detail.target.id === 'indicesTable') {
        const text = evt.detail.xhr.responseText;
        const lines = text.trim().split('\n');

        if (lines.length <= 1) {
            evt.detail.target.innerHTML = `
                <p class="text-gray-600 py-4">No indices found. Create your first index to get started.</p>
            `;
            return;
        }

        // Parse table
        const rows = lines.slice(1).map(line => {
            const values = line.split(/\s+/);
            return {
                health: values[0],
                status: values[1],
                index: values[2],
                uuid: values[3],
                pri: values[4],
                rep: values[5],
                docs: values[6],
                store: values[7]
            };
        });

        const tableHtml = `
            <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Index</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Health</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Documents</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                    </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                    ${rows.map(row => `
                        <tr>
                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">${row.index}</td>
                            <td class="px-6 py-4 whitespace-nowrap">
                                <span class="px-2 py-1 text-xs font-semibold rounded-full ${
                                    row.health === 'green' ? 'bg-green-100 text-green-800' :
                                    row.health === 'yellow' ? 'bg-yellow-100 text-yellow-800' :
                                    'bg-red-100 text-red-800'
                                }">${row.health}</span>
                            </td>
                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">${row.docs || '0'}</td>
                            <td class="px-6 py-4 whitespace-nowrap text-sm">
                                <button
                                    onclick="window.viewIndexFromTable('${row.index}')"
                                    class="text-primary hover:text-primary/80 mr-2"
                                >
                                    View
                                </button>
                                <button
                                    onclick="deleteIndex('${row.index}')"
                                    class="text-red-600 hover:text-red-800"
                                >
                                    Delete
                                </button>
                            </td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;
        evt.detail.target.innerHTML = tableHtml;
    }

    // Process cluster stats response
    if (evt.detail.target.id === 'clusterStats') {
        let stats;
        try {
            stats = JSON.parse(evt.detail.xhr.responseText);
            evt.detail.target.innerHTML = `
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <p class="text-sm text-gray-600">Cluster Name</p>
                        <p class="font-semibold">${stats.cluster_name || 'gummy-search'}</p>
                    </div>
                    <div>
                        <p class="text-sm text-gray-600">Version</p>
                        <p class="font-semibold">${stats.version || '6.4.0'}</p>
                    </div>
                </div>
            `;
        } catch (e) {
            console.error('Error parsing cluster stats:', e);
            evt.detail.target.innerHTML = `
                <div class="text-red-600">Error loading cluster stats</div>
            `;
        }
    }
});

// Global function for viewing index (called from htmx-rendered table)
window.viewIndexFromTable = function(indexName) {
    const dashboard = Alpine.$data(document.querySelector('[x-data]'));
    if (dashboard && dashboard.viewIndex) {
        dashboard.viewIndex(indexName);
    }
};

// Global function for deleting index (called from htmx-rendered table)
async function deleteIndex(indexName) {
    if (!confirm(`Are you sure you want to delete index "${indexName}"? This action cannot be undone.`)) {
        return;
    }

    try {
        const response = await fetch(`/${indexName}`, { method: 'DELETE' });
        if (response.ok) {
            // Trigger refresh
            htmx.trigger(document.body, 'refresh');
            // Show toast notification
            const dashboard = Alpine.$data(document.querySelector('[x-data]'));
            if (dashboard && dashboard.showToast) {
                dashboard.showToast('Index deleted successfully!', 'success');
            } else {
                alert('Index deleted successfully!');
            }
        } else {
            const error = await response.json();
            const dashboard = Alpine.$data(document.querySelector('[x-data]'));
            if (dashboard && dashboard.showToast) {
                dashboard.showToast(error.error?.reason || 'Failed to delete index', 'error');
            } else {
                alert('Failed to delete index');
            }
        }
    } catch (error) {
        const dashboard = Alpine.$data(document.querySelector('[x-data]'));
        if (dashboard && dashboard.showToast) {
            dashboard.showToast(`Error: ${error.message}`, 'error');
        } else {
            alert(`Error deleting index: ${error.message}`);
        }
    }
}

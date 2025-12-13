// Alpine.js Dashboard Component
function dashboard() {
    return {
        loading: false,
        showCreateModal: false,
        newIndexName: '',
        stats: {
            indices: '-',
            documents: '-',
            nodes: '-'
        },

        async init() {
            // Load initial stats
            await this.loadStats();

            // Set up auto-refresh every 30 seconds
            setInterval(() => {
                this.refresh();
            }, 30000);
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
                    this.showNotification('Index created successfully!', 'success');
                } else {
                    const error = await response.json();
                    this.showNotification(
                        error.error?.reason || 'Failed to create index',
                        'error'
                    );
                }
            } catch (error) {
                this.showNotification(`Error: ${error.message}`, 'error');
            } finally {
                this.loading = false;
            }
        },

        showNotification(message, type = 'info') {
            // Simple alert for now, can be enhanced with a toast component
            alert(message);
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
                                    onclick="viewIndex('${row.index}')"
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

// Global functions for index actions
function viewIndex(indexName) {
    window.location.href = `#index/${indexName}`;
    // TODO: Implement index detail view
    alert(`View index: ${indexName}`);
}

async function deleteIndex(indexName) {
    if (!confirm(`Are you sure you want to delete index "${indexName}"? This action cannot be undone.`)) {
        return;
    }

    try {
        const response = await fetch(`/${indexName}`, { method: 'DELETE' });
        if (response.ok) {
            // Trigger refresh
            htmx.trigger(document.body, 'refresh');
            alert('Index deleted successfully!');
        } else {
            alert('Failed to delete index');
        }
    } catch (error) {
        alert(`Error deleting index: ${error.message}`);
    }
}

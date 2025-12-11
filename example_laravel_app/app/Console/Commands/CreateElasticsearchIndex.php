<?php

namespace App\Console\Commands;

use App\Models\Content;
use App\Models\Dictionary;
use App\Models\Entity;
use App\Models\File;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Http;

class CreateElasticsearchIndex extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'elastic:create-index {model? : The model class name}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Create Elasticsearch indices for Scout models';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $host = config('scout.elasticsearch.host', env('ELASTICSEARCH_HOST', 'http://localhost:9200'));
        $model = $this->argument('model');

        $models = $model
            ? [app($model)]
            : [
                new Content(),
                new Entity(),
                new File(),
                new Dictionary(),
            ];

        foreach ($models as $modelInstance) {
            $index = $modelInstance->searchableAs();
            $this->info("Creating index: {$index}");

            // Check if index exists
            $existsResponse = Http::head("{$host}/{$index}");
            if ($existsResponse->successful()) {
                $this->warn("Index {$index} already exists. Skipping...");
                continue;
            }

            // Create index with mappings
            $mappings = $this->buildMappings($modelInstance->toSearchableArray());
            $settings = [
                'number_of_shards' => 1,
                'number_of_replicas' => 0,
            ];

            $response = Http::put("{$host}/{$index}", [
                'settings' => $settings,
                'mappings' => [
                    'properties' => $mappings,
                ],
            ]);

            if ($response->successful()) {
                $this->info("✓ Index {$index} created successfully");
            } else {
                $this->error("✗ Failed to create index {$index}: " . $response->body());
                return self::FAILURE;
            }
        }

        $this->info('All indices created successfully!');
        return self::SUCCESS;
    }

    /**
     * Build mappings from searchable array.
     *
     * @param array $data
     * @return array
     */
    protected function buildMappings(array $data): array
    {
        $mappings = [];

        foreach ($data as $key => $value) {
            if ($key === 'id') {
                $mappings[$key] = ['type' => 'keyword'];
            } elseif (is_string($value) && str_contains($key, '_at')) {
                // Date fields
                $mappings[$key] = ['type' => 'date'];
            } elseif (is_string($value)) {
                // Text fields
                $mappings[$key] = ['type' => 'text'];
            } elseif (is_int($value)) {
                // Integer fields
                $mappings[$key] = ['type' => 'integer'];
            } else {
                // Default to text
                $mappings[$key] = ['type' => 'text'];
            }
        }

        return $mappings;
    }
}
